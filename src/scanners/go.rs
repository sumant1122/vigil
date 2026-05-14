use std::path::Path;
use async_trait::async_trait;
use crate::models::{Dependency, Ecosystem};
use super::EcosystemScanner;

pub struct GoModScanner;

#[async_trait]
impl EcosystemScanner for GoModScanner {
    fn name(&self) -> &'static str {
        "Go"
    }

    fn can_scan(&self, path: &Path) -> bool {
        path.join("go.mod").exists()
    }

    async fn scan(&self, path: &Path) -> anyhow::Result<Vec<Dependency>> {
        let mod_path = path.join("go.mod");
        let content = tokio::fs::read_to_string(mod_path).await?;
        
        let mut deps = Vec::new();
        let mut in_require = false;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("//") { continue; }

            if line.starts_with("require (") {
                in_require = true;
                continue;
            }

            if in_require && line == ")" {
                in_require = false;
                continue;
            }

            if line.starts_with("require ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    deps.push(Dependency {
                        name: parts[1].to_string(),
                        version: parts[2].to_string(),
                        ecosystem: Ecosystem::Go,
                    });
                }
            } else if in_require {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    deps.push(Dependency {
                        name: parts[0].to_string(),
                        version: parts[1].to_string(),
                        ecosystem: Ecosystem::Go,
                    });
                }
            }
        }

        Ok(deps)
    }
}
