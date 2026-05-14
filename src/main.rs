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
        
        let maintenance = sources::maintenance::MaintenanceClient::new();
        // let _osv = sources::osv::OsvClient::new(); // Ready for use

        let mut enriched_deps = Vec::new();
        for dep in all_deps {
            let score = maintenance.get_health(&dep).await.unwrap_or_default();
            enriched_deps.push((dep, score));
        }

        ui::tui::run_tui(enriched_deps)?;
    }
    
    Ok(())
}
