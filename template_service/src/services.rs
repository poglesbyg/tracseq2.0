use anyhow::Result;
use crate::{config::Config, database::DatabasePool, clients::{AuthClient, SampleClient}};

#[derive(Clone)]
pub struct TemplateServiceImpl {
    db_pool: DatabasePool,
    config: Config,
    auth_client: AuthClient,
    sample_client: SampleClient,
}

impl TemplateServiceImpl {
    pub fn new(
        db_pool: DatabasePool,
        config: Config,
        auth_client: AuthClient,
        sample_client: SampleClient,
    ) -> Result<Self> {
        Ok(Self {
            db_pool,
            config,
            auth_client,
            sample_client,
        })
    }
}