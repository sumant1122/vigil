use crate::models::Dependency;
use async_trait::async_trait;
use std::path::Path;

pub mod cargo;
pub mod go;
pub mod npm;
pub mod python;
pub mod uv;

#[async_trait]
pub trait EcosystemScanner {
    fn name(&self) -> &'static str;
    fn can_scan(&self, path: &Path) -> bool;
    async fn scan(&self, path: &Path) -> anyhow::Result<Vec<Dependency>>;
}
