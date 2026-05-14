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
    dependencies: Option<HashMap<String, String>>,
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
                let direct_deps = package.dependencies.unwrap_or_default().keys().cloned().collect();
                
                deps.push(Dependency {
                    name: clean_name.to_string(),
                    version,
                    ecosystem: Ecosystem::Npm,
                    advisories: Vec::new(),
                    direct_dependencies: direct_deps,
                    license: Some("Apache-2.0".to_string()), // Placeholder
                });
            }
        }

        Ok(deps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_npm_scanner() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let lock_path = dir.path().join("package-lock.json");
        let mut file = std::fs::File::create(lock_path)?;
        
        writeln!(file, r#"{{
  "name": "test-project",
  "version": "1.0.0",
  "lockfileVersion": 2,
  "requires": true,
  "packages": {{
    "": {{
      "name": "test-project",
      "version": "1.0.0"
    }},
    "node_modules/express": {{
      "version": "4.18.2",
      "dependencies": {{
        "cookie": "0.5.0"
      }}
    }},
    "node_modules/cookie": {{
      "version": "0.5.0"
    }}
  }}
}}"#)?;

        let scanner = NpmLockScanner;
        assert!(scanner.can_scan(dir.path()));
        
        let deps = scanner.scan(dir.path()).await?;
        assert_eq!(deps.len(), 2);
        
        let express = deps.iter().find(|d| d.name == "express").unwrap();
        assert_eq!(express.version, "4.18.2");
        assert_eq!(express.direct_dependencies, vec!["cookie"]);
        
        Ok(())
    }
}
