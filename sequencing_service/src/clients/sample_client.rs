use reqwest::Client;
use crate::error::{Result, SequencingError};

#[derive(Debug, Clone)]
pub struct SampleClient {
    client: Client,
    base_url: String,
}

impl SampleClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }

    pub async fn health_check(&self) -> Result<()> {
        let url = format!("{}/health", self.base_url);
        
        match self.client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    Ok(())
                } else {
                    Err(SequencingError::ExternalService { 
                        service: "sample".to_string(),
                        message: format!("Health check failed: {}", response.status())
                    })
                }
            }
            Err(e) => {
                Err(SequencingError::ExternalService {
                    service: "sample".to_string(),
                    message: format!("Health check request failed: {}", e)
                })
            }
        }
    }

    pub async fn get_sample(&self, sample_id: uuid::Uuid) -> Result<serde_json::Value> {
        let url = format!("{}/samples/{}", self.base_url, sample_id);
        
        match self.client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let sample_data = response.json::<serde_json::Value>().await
                        .map_err(|e| SequencingError::ExternalService {
                            service: "sample".to_string(),
                            message: format!("Failed to parse sample data: {}", e)
                        })?;
                    Ok(sample_data)
                } else {
                    Err(SequencingError::ExternalService { 
                        service: "sample".to_string(),
                        message: format!("Get sample failed: {}", response.status())
                    })
                }
            }
            Err(e) => {
                Err(SequencingError::ExternalService {
                    service: "sample".to_string(),
                    message: format!("Get sample request failed: {}", e)
                })
            }
        }
    }

    // Add more sample-specific methods as needed
    // pub async fn get_samples_for_job(&self, job_id: Uuid) -> Result<Vec<Sample>> { ... }
    // pub async fn validate_samples(&self, sample_ids: &[String]) -> Result<bool> { ... }
}
