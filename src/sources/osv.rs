use serde::{Serialize, Deserialize};
use crate::models::{Dependency, Ecosystem, Advisory};

#[derive(Serialize)]
struct OsvQuery {
    version: String,
    package: OsvPackage,
}

#[derive(Serialize)]
struct OsvPackage {
    name: String,
    ecosystem: String,
}

#[derive(Deserialize)]
struct OsvResponse {
    vulns: Option<Vec<OsvVulnerability>>,
}

#[derive(Deserialize)]
struct OsvVulnerability {
    id: String,
    summary: Option<String>,
    details: Option<String>,
    database_specific: Option<OsvDatabaseSpecific>,
}

#[derive(Deserialize)]
struct OsvDatabaseSpecific {
    severity: Option<String>,
}

pub struct OsvClient {
    http: reqwest::Client,
}

impl OsvClient {
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::new(),
        }
    }

    pub async fn query(&self, dep: &Dependency) -> anyhow::Result<Vec<Advisory>> {
        let ecosystem = match dep.ecosystem {
            Ecosystem::Cargo => "Cargo",
            Ecosystem::Npm => "npm",
            Ecosystem::Pip => "PyPI",
            Ecosystem::Go => "Go",
        };

        let query = OsvQuery {
            version: dep.version.clone(),
            package: OsvPackage {
                name: dep.name.clone(),
                ecosystem: ecosystem.to_string(),
            },
        };

        let res = self.http.post("https://api.osv.dev/v1/query")
            .json(&query)
            .send()
            .await?
            .json::<OsvResponse>()
            .await?;

        let mut advisories = Vec::new();
        if let Some(vulns) = res.vulns {
            for vuln in vulns {
                advisories.push(Advisory {
                    id: vuln.id,
                    summary: vuln.summary.unwrap_or_else(|| "No summary provided".to_string()),
                    severity: vuln.database_specific.and_then(|s| s.severity).unwrap_or_else(|| "Unknown".to_string()),
                });
            }
        }

        Ok(advisories)
    }
}
