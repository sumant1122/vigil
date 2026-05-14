use super::EcosystemScanner;
use crate::models::{Dependency, Ecosystem};
use async_trait::async_trait;
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
struct CargoLock {
    package: Vec<Package>,
}

#[derive(Deserialize)]
struct Package {
    name: String,
    version: String,
    dependencies: Option<Vec<String>>,
}

pub struct CargoLockScanner;

#[async_trait]
impl EcosystemScanner for CargoLockScanner {
    fn name(&self) -> &'static str {
        "Cargo"
    }

    fn can_scan(&self, path: &Path) -> bool {
        path.join("Cargo.lock").exists()
    }

    async fn scan(&self, path: &Path) -> anyhow::Result<Vec<Dependency>> {
        let lock_path = path.join("Cargo.lock");
        let content = tokio::fs::read_to_string(lock_path).await?;
        let lock: CargoLock = toml::from_str(&content)?;

        Ok(lock
            .package
            .into_iter()
            .map(|p| {
                // Cargo dependencies in lockfile often have version info like "name version"
                let deps = p
                    .dependencies
                    .unwrap_or_default()
                    .into_iter()
                    .map(|d| d.split_whitespace().next().unwrap_or(&d).to_string())
                    .collect();

                Dependency {
                    name: p.name,
                    version: p.version,
                    ecosystem: Ecosystem::Cargo,
                    advisories: Vec::new(),
                    direct_dependencies: deps,
                    license: Some("MIT".to_string()), // Placeholder for now
                }
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_cargo_scanner() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let lock_path = dir.path().join("Cargo.lock");
        let mut file = std::fs::File::create(lock_path)?;

        writeln!(
            file,
            r#"
[[package]]
name = "test-crate"
version = "1.0.0"
dependencies = [
 "dep-a",
 "dep-b 0.1.0",
]

[[package]]
name = "dep-a"
version = "0.5.0"
"#
        )?;

        let scanner = CargoLockScanner;
        assert!(scanner.can_scan(dir.path()));

        let deps = scanner.scan(dir.path()).await?;
        assert_eq!(deps.len(), 2);

        let test_crate = deps.iter().find(|d| d.name == "test-crate").unwrap();
        assert_eq!(test_crate.version, "1.0.0");
        assert_eq!(test_crate.direct_dependencies, vec!["dep-a", "dep-b"]);

        Ok(())
    }
}
