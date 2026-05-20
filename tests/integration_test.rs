use std::io::Write;
use tempfile::tempdir;
use tempfile::NamedTempFile;
use vigil::models::HealthScore;
use vigil::scanners::{cargo::CargoLockScanner, EcosystemScanner};
use vigil::sources::cache::{Cache, CacheEntry, CacheManager};

#[tokio::test]
async fn test_cargo_scanner_integration() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let lock_path = dir.path().join("Cargo.lock");
    let mut file = std::fs::File::create(lock_path)?;

    writeln!(
        file,
        r#"
[[package]]
name = "integration-crate"
version = "2.0.0"
dependencies = [
 "dep-a",
]

[[package]]
name = "dep-a"
version = "1.0.0"
"#
    )?;

    let scanner = CargoLockScanner;
    assert!(scanner.can_scan(dir.path()));

    let deps = scanner.scan(dir.path()).await?;
    assert_eq!(deps.len(), 2);

    let main_crate = deps.iter().find(|d| d.name == "integration-crate").unwrap();
    assert_eq!(main_crate.version, "2.0.0");
    assert_eq!(main_crate.direct_dependencies, vec!["dep-a"]);

    Ok(())
}

#[test]
fn test_cache_integration() {
    let temp_file = NamedTempFile::new().unwrap();
    let cache_path = temp_file.path().to_path_buf();
    let manager = CacheManager::new_with_path(cache_path);

    let mut cache = Cache::default();
    let score = HealthScore {
        maintenance_score: 85,
        security_score: 95,
        composite_score: 92,
        maintenance_details: vec!["Healthy".to_string()],
        bloat_index: 1,
        license: Some("Apache-2.0".to_string()),
    };

    cache.entries.insert(
        "Npm:express@4.18.2".to_string(),
        CacheEntry {
            score,
            timestamp: chrono::Utc::now(),
        },
    );

    manager.save(&cache).unwrap();

    let loaded = manager.load();
    assert_eq!(loaded.entries.len(), 1);
    let entry = loaded.entries.get("Npm:express@4.18.2").unwrap();
    assert_eq!(entry.score.license, Some("Apache-2.0".to_string()));
}
