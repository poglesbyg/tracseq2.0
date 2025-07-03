use crate::config::TeamsConfig;

#[derive(Clone)]
pub struct TeamsClient {
    config: TeamsConfig,
    http_client: reqwest::Client,
}

impl TeamsClient {
    pub fn new(config: TeamsConfig) -> Self {
        Self {
            config,
            http_client: reqwest::Client::new(),
        }
    }
}
