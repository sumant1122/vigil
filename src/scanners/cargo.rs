use std::path::Path;
use async_trait::async_trait;
use serde::Deserialize;
use crate::models::{Dependency, Ecosystem};
use super::EcosystemScanner;

#[derive(Deserialize)]
struct CargoLock {
    package: Vec<Package>,
}

#[derive(Deserialize)]
struct Package {
    name: String,
    version: String,
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

        Ok(lock.package.into_iter().map(|p| Dependency {
            name: p.name,
            version: p.version,
            ecosystem: Ecosystem::Cargo,
        }).collect())
    }
}
