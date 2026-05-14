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

#[derive(Serialize)]
struct OsvBatchQuery {
    queries: Vec<OsvQuery>,
}

#[derive(Deserialize)]
struct OsvBatchResponse {
    results: Vec<OsvResponse>,
}

impl OsvClient {
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::new(),
        }
    }

    pub async fn query_batch(&self, deps: &[Dependency]) -> anyhow::Result<Vec<Vec<Advisory>>> {
        let mut queries = Vec::new();
        for dep in deps {
            let ecosystem = match dep.ecosystem {
                Ecosystem::Cargo => "Cargo",
                Ecosystem::Npm => "npm",
                Ecosystem::Pip => "PyPI",
                Ecosystem::Go => "Go",
            };

            queries.push(OsvQuery {
                version: dep.version.clone(),
                package: OsvPackage {
                    name: dep.name.clone(),
                    ecosystem: ecosystem.to_string(),
                },
            });
        }

        let batch_query = OsvBatchQuery { queries };
        
        // OSV batch limit is 1000 per request, we are well under that for most projects
        let res = self.http.post("https://api.osv.dev/v1/querybatch")
            .json(&batch_query)
            .send()
            .await?
            .json::<OsvBatchResponse>()
            .await?;

        let mut all_advisories = Vec::new();
        for result in res.results {
            let mut advisories = Vec::new();
            if let Some(vulns) = result.vulns {
                for vuln in vulns {
                    advisories.push(Advisory {
                        id: vuln.id,
                        summary: vuln.summary.unwrap_or_else(|| "No summary provided".to_string()),
                        severity: vuln.database_specific.and_then(|s| s.severity).unwrap_or_else(|| "Unknown".to_string()),
                    });
                }
            }
            all_advisories.push(advisories);
        }

        Ok(all_advisories)
    }

    pub async fn query(&self, dep: &Dependency) -> anyhow::Result<Vec<Advisory>> {
        // Keeping individual query for fallback/convenience
        let advisories = self.query_batch(std::slice::from_ref(dep)).await?;
        Ok(advisories.into_iter().next().unwrap_or_default())
    }
}
