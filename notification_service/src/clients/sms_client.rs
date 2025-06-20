use crate::config::SmsConfig;

#[derive(Clone)]
pub struct SmsClient {
    config: SmsConfig,
    http_client: reqwest::Client,
}

impl SmsClient {
    pub fn new(config: SmsConfig) -> Self {
        Self {
            config,
            http_client: reqwest::Client::new(),
        }
    }
}
