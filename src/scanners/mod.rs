use std::path::Path;
use async_trait::async_trait;
use crate::models::Dependency;

pub mod cargo;
pub mod npm;
pub mod python;
pub mod go;

#[async_trait]
pub trait EcosystemScanner {
    fn name(&self) -> &'static str;
    fn can_scan(&self, path: &Path) -> bool;
    async fn scan(&self, path: &Path) -> anyhow::Result<Vec<Dependency>>;
}
