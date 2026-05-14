use std::path::Path;
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use crate::models::{Dependency, Ecosystem};
use super::EcosystemScanner;

#[derive(Deserialize)]
struct NpmLockV2 {
    packages: HashMap<String, NpmPackage>,
}

#[derive(Deserialize)]
struct NpmPackage {
    version: Option<String>,
}

pub struct NpmLockScanner;

#[async_trait]
impl EcosystemScanner for NpmLockScanner {
    fn name(&self) -> &'static str {
        "NPM"
    }

    fn can_scan(&self, path: &Path) -> bool {
        path.join("package-lock.json").exists()
    }

    async fn scan(&self, path: &Path) -> anyhow::Result<Vec<Dependency>> {
        let lock_path = path.join("package-lock.json");
        let content = tokio::fs::read_to_string(lock_path).await?;
        let lock: NpmLockV2 = serde_json::from_str(&content)?;

        let mut deps = Vec::new();
        for (name, package) in lock.packages {
            if name.is_empty() { continue; } // Root package
            
            // package-lock.json v2+ uses paths like "node_modules/express"
            let clean_name = name.strip_prefix("node_modules/").unwrap_or(&name);
            
            if let Some(version) = package.version {
                deps.push(Dependency {
                    name: clean_name.to_string(),
                    version,
                    ecosystem: Ecosystem::Npm,
                });
            }
        }

        Ok(deps)
    }
}
