use std::path::Path;
use async_trait::async_trait;
use crate::models::{Dependency, Ecosystem};
use super::EcosystemScanner;

pub struct RequirementsTxtScanner;

#[async_trait]
impl EcosystemScanner for RequirementsTxtScanner {
    fn name(&self) -> &'static str {
        "Python"
    }

    fn can_scan(&self, path: &Path) -> bool {
        path.join("requirements.txt").exists()
    }

    async fn scan(&self, path: &Path) -> anyhow::Result<Vec<Dependency>> {
        let req_path = path.join("requirements.txt");
        let content = tokio::fs::read_to_string(req_path).await?;
        
        let mut deps = Vec::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') { continue; }
            
            // Basic parsing for "package==version" or "package>=version"
            let parts: Vec<&str> = if line.contains("==") {
                line.split("==").collect()
            } else if line.contains(">=") {
                line.split(">=").collect()
            } else {
                vec![line, "unknown"]
            };

            if let Some(name) = parts.get(0) {
                deps.push(Dependency {
                    name: name.trim().to_string(),
                    version: parts.get(1).unwrap_or(&"unknown").trim().to_string(),
                    ecosystem: Ecosystem::Pip,
                    advisories: Vec::new(),
                    direct_dependencies: Vec::new(),
                    license: Some("BSD-3-Clause".to_string()),
                });
            }
        }

        Ok(deps)
    }
}
