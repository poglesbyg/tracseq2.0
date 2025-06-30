use crate::config::SlackConfig;

#[derive(Clone)]
pub struct SlackClient {
    config: SlackConfig,
    http_client: reqwest::Client,
}

impl SlackClient {
    pub fn new(config: SlackConfig) -> Self {
        Self {
            config,
            http_client: reqwest::Client::new(),
        }
    }
}
