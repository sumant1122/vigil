use super::EcosystemScanner;
use crate::models::{Dependency, Ecosystem};
use async_trait::async_trait;
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
struct UvLock {
    package: Vec<Package>,
}

#[derive(Deserialize)]
struct Package {
    name: String,
    version: String,
    dependencies: Option<Vec<DependencyEntry>>,
}

#[derive(Deserialize)]
struct DependencyEntry {
    name: String,
}

pub struct UvLockScanner;

#[async_trait]
impl EcosystemScanner for UvLockScanner {
    fn name(&self) -> &'static str {
        "uv"
    }

    fn can_scan(&self, path: &Path) -> bool {
        path.join("uv.lock").exists()
    }

    async fn scan(&self, path: &Path) -> anyhow::Result<Vec<Dependency>> {
        let lock_path = path.join("uv.lock");
        let content = tokio::fs::read_to_string(lock_path).await?;
        let lock: UvLock = toml::from_str(&content)?;

        Ok(lock
            .package
            .into_iter()
            .map(|p| {
                let deps = p
                    .dependencies
                    .unwrap_or_default()
                    .into_iter()
                    .map(|d| d.name)
                    .collect();

                Dependency {
                    name: p.name,
                    version: p.version,
                    ecosystem: Ecosystem::Pip, // uv is Python, so use Pip ecosystem for OSV/Maintenance
                    advisories: Vec::new(),
                    direct_dependencies: deps,
                    license: Some("Unknown".to_string()),
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
    async fn test_uv_scanner() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let lock_path = dir.path().join("uv.lock");
        let mut file = std::fs::File::create(lock_path)?;

        writeln!(
            file,
            r#"
version = 1
revision = 0
requires-python = ">=3.12"

[[package]]
name = "fastapi"
version = "0.111.0"
dependencies = [
    {{ name = "pydantic" }},
    {{ name = "starlette" }},
]

[[package]]
name = "pydantic"
version = "2.7.0"
"#
        )?;

        let scanner = UvLockScanner;
        assert!(scanner.can_scan(dir.path()));

        let deps = scanner.scan(dir.path()).await?;
        assert_eq!(deps.len(), 2);

        let fastapi = deps.iter().find(|d| d.name == "fastapi").unwrap();
        assert_eq!(fastapi.version, "0.111.0");
        assert_eq!(fastapi.direct_dependencies, vec!["pydantic", "starlette"]);

        Ok(())
    }
}
