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
        println!("Total unique dependencies: {}", all_deps.len());
    }
    
    Ok(())
}
