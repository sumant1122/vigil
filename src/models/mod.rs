use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Ecosystem {
    Cargo,
    Npm,
    Pip,
    Go,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    pub ecosystem: Ecosystem,
    pub advisories: Vec<Advisory>,
    pub direct_dependencies: Vec<String>,
    pub license: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct HealthScore {
    pub maintenance_score: u8, // 0-100
    pub security_score: u8,    // 0-100
    pub composite_score: u8,   // 0-100
    pub maintenance_details: Vec<String>,
    pub bloat_index: usize,    // Transitive dependency count
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Advisory {
    pub id: String,
    pub summary: String,
    pub severity: String,
}
