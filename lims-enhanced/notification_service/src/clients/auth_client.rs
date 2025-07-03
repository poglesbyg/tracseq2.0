#[derive(Clone)]
pub struct AuthClient {
    base_url: String,
    http_client: reqwest::Client,
}

impl AuthClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            http_client: reqwest::Client::new(),
        }
    }
}
