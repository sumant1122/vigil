use crate::models::{Dependency, HealthScore};

pub struct MaintenanceClient;

impl MaintenanceClient {
    pub fn new() -> Self {
        Self
    }

    pub async fn get_health(&self, dep: &Dependency) -> anyhow::Result<HealthScore> {
        // In a real implementation, we would query GitHub/Crates.io
        // For the MVP, we generate a deterministic "health" based on the name
        // to show variety in the UI.
        let name_hash = dep.name.chars().map(|c| c as u32).sum::<u32>();
        let maintenance_score = (name_hash % 40) + 60; // 60-100
        
        let last_commit_days = (name_hash % 300) + 1;
        let maintainers = (name_hash % 10) + 1;
        let stars = (name_hash % 5000) + 10;

        Ok(HealthScore {
            maintenance_score: maintenance_score as u8,
            security_score: 100,
            composite_score: maintenance_score as u8,
            maintenance_details: vec![
                format!("Last commit: {} days ago", last_commit_days),
                format!("Maintainers: {} active", maintainers),
                format!("Stars: {}", stars),
            ],
            bloat_index: 0,
        })
    }
}
