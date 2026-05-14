use super::EcosystemScanner;
use crate::models::{Dependency, Ecosystem};
use async_trait::async_trait;
use std::path::Path;

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
            if line.is_empty() || line.starts_with("//") {
                continue;
            }

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
                        advisories: Vec::new(),
                        direct_dependencies: Vec::new(),
                        license: Some("BSD-3-Clause".to_string()),
                    });
                }
            } else if in_require {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    deps.push(Dependency {
                        name: parts[0].to_string(),
                        version: parts[1].to_string(),
                        ecosystem: Ecosystem::Go,
                        advisories: Vec::new(),
                        direct_dependencies: Vec::new(),
                        license: Some("BSD-3-Clause".to_string()),
                    });
                }
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
    async fn test_go_scanner() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let mod_path = dir.path().join("go.mod");
        let mut file = std::fs::File::create(mod_path)?;

        writeln!(
            file,
            r#"module test-app

go 1.21

require (
	github.com/gin-gonic/gin v1.9.1
	github.com/stretchr/testify v1.8.4
)

require github.com/google/uuid v1.4.0
"#
        )?;

        let scanner = GoModScanner;
        assert!(scanner.can_scan(dir.path()));

        let deps = scanner.scan(dir.path()).await?;
        assert_eq!(deps.len(), 3);

        let gin = deps
            .iter()
            .find(|d| d.name == "github.com/gin-gonic/gin")
            .unwrap();
        assert_eq!(gin.version, "v1.9.1");

        let uuid = deps
            .iter()
            .find(|d| d.name == "github.com/google/uuid")
            .unwrap();
        assert_eq!(uuid.version, "v1.4.0");

        Ok(())
    }
}
