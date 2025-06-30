use reqwest::Client;
use crate::error::{Result, SequencingError};

#[derive(Debug, Clone)]
pub struct StorageClient {
    client: Client,
    base_url: String,
}

impl StorageClient {
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
                        service: "storage".to_string(),
                        message: format!("Health check failed: {}", response.status())
                    })
                }
            }
            Err(e) => {
                Err(SequencingError::ExternalService {
                    service: "storage".to_string(),
                    message: format!("Health check request failed: {}", e)
                })
            }
        }
    }
}