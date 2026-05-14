mod models;
mod scanners;
mod sources;
mod ui;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the project to analyze
    #[arg(short, long, default_value = ".")]
    path: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    println!("👁️ Vigil - Universal Supply Chain Health Dashboard");
    println!("Scanning project at {}...", args.path);

    let path = std::path::Path::new(&args.path);
    let scanners: Vec<Box<dyn scanners::EcosystemScanner>> = vec![
        Box::new(scanners::cargo::CargoLockScanner),
        Box::new(scanners::npm::NpmLockScanner),
        Box::new(scanners::python::RequirementsTxtScanner),
        Box::new(scanners::go::GoModScanner),
    ];

    let mut all_deps = Vec::new();

    for scanner in scanners {
        if scanner.can_scan(path) {
            println!("Found {} project...", scanner.name());
            match scanner.scan(path).await {
                Ok(deps) => {
                    println!("Parsed {} dependencies.", deps.len());
                    all_deps.extend(deps);
                }
                Err(e) => eprintln!("Error scanning {}: {}", scanner.name(), e),
            }
        }
    }

    if all_deps.is_empty() {
        println!("No supported projects found.");
    } else {
        println!("Analyzing {} dependencies...", all_deps.len());

        let maintenance = std::sync::Arc::new(sources::maintenance::MaintenanceClient::new());
        let osv = std::sync::Arc::new(sources::osv::OsvClient::new());
        let cache_mgr = sources::cache::CacheManager::new();
        let mut cache = cache_mgr.load();

        // 1. Batch OSV Queries
        println!("Checking OSV database...");
        let all_advisories = osv.query_batch(&all_deps).await.unwrap_or_default();
        for (i, advs) in all_advisories.into_iter().enumerate() {
            if i < all_deps.len() {
                all_deps[i].advisories = advs;
            }
        }

        // 2. Process Maintenance with Cache
        // Pre-calculate Bloat Index (Transitive counts)
        let dep_map: std::collections::HashMap<String, Vec<String>> = all_deps
            .iter()
            .map(|d| (d.name.clone(), d.direct_dependencies.clone()))
            .collect();

        let mut bloat_indices = std::collections::HashMap::new();
        for dep in &all_deps {
            let mut transitive_set = std::collections::HashSet::new();
            let mut stack = dep.direct_dependencies.clone();
            while let Some(child_name) = stack.pop() {
                if transitive_set.insert(child_name.clone()) {
                    if let Some(children) = dep_map.get(&child_name) {
                        stack.extend(children.clone());
                    }
                }
            }
            bloat_indices.insert(dep.name.clone(), transitive_set.len());
        }

        use futures::StreamExt;
        let bloat_indices = std::sync::Arc::new(bloat_indices);

        // Split deps into those we have in cache and those we don't
        let mut to_fetch = Vec::new();
        let mut cached_results = Vec::new();

        for dep in all_deps {
            let cache_key = format!("{:?}:{}@{}", dep.ecosystem, dep.name, dep.version);
            if let Some(entry) = cache.entries.get(&cache_key) {
                // If entry is less than 24h old, use it
                if (chrono::Utc::now() - entry.timestamp).num_hours() < 24 {
                    let mut score = entry.score.clone();
                    score.bloat_index = *bloat_indices.get(&dep.name).unwrap_or(&0);

                    if !dep.advisories.is_empty() {
                        score.security_score = 0;
                        score.composite_score = (score.maintenance_score as u16 / 2) as u8;
                    }
                    cached_results.push((dep, score));
                    continue;
                }
            }
            to_fetch.push(dep);
        }

        println!(
            "Fetching fresh data for {} dependencies ({} from cache)...",
            to_fetch.len(),
            cached_results.len()
        );

        let mut stream = futures::stream::iter(to_fetch)
            .map(|dep| {
                let maintenance = maintenance.clone();
                let bloat_indices = bloat_indices.clone();
                async move {
                    let mut score = maintenance.get_health(&dep).await.unwrap_or_else(|_| {
                        futures::executor::block_on(maintenance.get_fallback_health(&dep))
                    });
                    score.bloat_index = *bloat_indices.get(&dep.name).unwrap_or(&0);

                    if !dep.advisories.is_empty() {
                        score.security_score = 0;
                        score.composite_score = (score.maintenance_score as u16 / 2) as u8;
                    }
                    (dep, score)
                }
            })
            .buffer_unordered(10);

        let mut enriched_deps = cached_results;
        while let Some((dep, score)) = stream.next().await {
            // Update cache
            let cache_key = format!("{:?}:{}@{}", dep.ecosystem, dep.name, dep.version);
            cache.entries.insert(
                cache_key,
                sources::cache::CacheEntry {
                    score: score.clone(),
                    timestamp: chrono::Utc::now(),
                },
            );
            enriched_deps.push((dep, score));
        }

        // Save cache for next time
        let _ = cache_mgr.save(&cache);

        // Sort by composite score (lowest first to highlight risks)
        enriched_deps.sort_by_key(|(_, score)| score.composite_score);

        if enriched_deps.is_empty() {
            println!("No data to display.");
        } else {
            ui::tui::run_tui(enriched_deps)?;
        }
    }

    Ok(())
}
