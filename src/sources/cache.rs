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
