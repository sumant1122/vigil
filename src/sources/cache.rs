use crate::models::HealthScore;
use chrono::{DateTime, Utc};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct CacheEntry {
    pub score: HealthScore,
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Cache {
    pub entries: HashMap<String, CacheEntry>,
}

pub struct CacheManager {
    cache_path: PathBuf,
}

impl CacheManager {
    pub fn new() -> Self {
        let cache_dir = ProjectDirs::from("com", "vigil", "vigil")
            .map(|d| d.cache_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from(".vigil_cache"));

        std::fs::create_dir_all(&cache_dir).ok();
        Self {
            cache_path: cache_dir.join("data.json"),
        }
    }

    pub fn new_with_path(path: PathBuf) -> Self {
        Self { cache_path: path }
    }

    pub fn load(&self) -> Cache {
        if let Ok(content) = std::fs::read_to_string(&self.cache_path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Cache::default()
        }
    }

    pub fn save(&self, cache: &Cache) -> anyhow::Result<()> {
        let content = serde_json::to_string(cache)?;
        std::fs::write(&self.cache_path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::HealthScore;
    use tempfile::NamedTempFile;

    #[test]
    fn test_cache_save_and_load() {
        let temp_file = NamedTempFile::new().unwrap();
        let cache_path = temp_file.path().to_path_buf();
        let manager = CacheManager::new_with_path(cache_path);

        // Load empty cache
        let cache = manager.load();
        assert!(cache.entries.is_empty());

        // Save an entry
        let score = HealthScore {
            maintenance_score: 95,
            security_score: 90,
            composite_score: 93,
            maintenance_details: vec!["Updated".to_string()],
            bloat_index: 2,
            license: Some("MIT".to_string()),
        };

        let mut cache = Cache::default();
        cache.entries.insert(
            "Cargo:test@1.0.0".to_string(),
            CacheEntry {
                score: score.clone(),
                timestamp: chrono::Utc::now(),
            },
        );

        manager.save(&cache).unwrap();

        // Load and assert
        let loaded = manager.load();
        assert_eq!(loaded.entries.len(), 1);
        let loaded_entry = loaded.entries.get("Cargo:test@1.0.0").unwrap();
        assert_eq!(loaded_entry.score.maintenance_score, 95);
        assert_eq!(loaded_entry.score.security_score, 90);
        assert_eq!(loaded_entry.score.composite_score, 93);
        assert_eq!(loaded_entry.score.bloat_index, 2);
        assert_eq!(loaded_entry.score.license, Some("MIT".to_string()));
    }
}
