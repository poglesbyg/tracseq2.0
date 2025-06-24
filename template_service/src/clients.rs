#[derive(Clone)]
pub struct AuthClient {
    base_url: String,
}

impl AuthClient {
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }
}

#[derive(Clone)]
pub struct SampleClient {
    base_url: String,
}

impl SampleClient {
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }
}