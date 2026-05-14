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

        // Pre-calculate Bloat Index (Transitive counts)
        let dep_map: std::collections::HashMap<String, Vec<String>> = all_deps.iter()
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
        let mut stream = futures::stream::iter(all_deps)
            .map(|mut dep| {
                let osv = osv.clone();
                let maintenance = maintenance.clone();
                let bloat_indices = bloat_indices.clone();
                async move {
                    let mut score = maintenance.get_health(&dep).await.unwrap_or_default();
                    score.bloat_index = *bloat_indices.get(&dep.name).unwrap_or(&0);
                    
                    // Query OSV for security advisories
                    if let Ok(advisories) = osv.query(&dep).await {
                        if !advisories.is_empty() {
                            score.security_score = 0; // Found vulnerabilities
                            score.composite_score = (score.maintenance_score as u16 / 2) as u8; // Heavily penalize
                            dep.advisories = advisories;
                        }
                    }
                    (dep, score)
                }
            })
            .buffer_unordered(20); // Increase parallelism to 20

        let mut enriched_deps = Vec::new();
        while let Some(item) = stream.next().await {
            enriched_deps.push(item);
        }

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
