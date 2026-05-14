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
    let _args = Args::parse();
    
    println!("👁️ Vigil - Universal Supply Chain Health Dashboard");
    println!("Scanning project...");
    
    // Placeholder for Phase 2 implementation
    
    Ok(())
}
