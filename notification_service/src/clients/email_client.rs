use crate::config::EmailConfig;

#[derive(Clone)]
pub struct EmailClient {
    config: EmailConfig,
    http_client: reqwest::Client,
}

impl EmailClient {
    pub fn new(config: EmailConfig) -> Self {
        Self {
            config,
            http_client: reqwest::Client::new(),
        }
    }
}
