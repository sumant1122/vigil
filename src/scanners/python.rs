use super::EcosystemScanner;
use crate::models::{Dependency, Ecosystem};
use async_trait::async_trait;
use std::path::Path;

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
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Basic parsing for "package==version" or "package>=version"
            let parts: Vec<&str> = if line.contains("==") {
                line.split("==").collect()
            } else if line.contains(">=") {
                line.split(">=").collect()
            } else {
                vec![line, "unknown"]
            };

            if let Some(name) = parts.first() {
                deps.push(Dependency {
                    name: name.trim().to_string(),
                    version: parts.get(1).unwrap_or(&"unknown").trim().to_string(),
                    ecosystem: Ecosystem::Pip,
                    advisories: Vec::new(),
                    direct_dependencies: Vec::new(),
                    license: Some("Unknown".to_string()),
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
    async fn test_python_scanner() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let req_path = dir.path().join("requirements.txt");
        let mut file = std::fs::File::create(req_path)?;

        writeln!(file, "requests==2.31.0\n# comment\nflask>=3.0.0\nnumpy")?;

        let scanner = RequirementsTxtScanner;
        assert!(scanner.can_scan(dir.path()));

        let deps = scanner.scan(dir.path()).await?;
        assert_eq!(deps.len(), 3);

        let requests = deps.iter().find(|d| d.name == "requests").unwrap();
        assert_eq!(requests.version, "2.31.0");

        let flask = deps.iter().find(|d| d.name == "flask").unwrap();
        assert_eq!(flask.version, "3.0.0");

        let numpy = deps.iter().find(|d| d.name == "numpy").unwrap();
        assert_eq!(numpy.version, "unknown");

        Ok(())
    }
}
