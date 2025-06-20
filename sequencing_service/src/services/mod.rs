use crate::{
    clients::{AuthClient, NotificationClient, SampleClient, TemplateClient},
    config::Config,
    database::DatabasePool,
    error::Result,
};

#[derive(Debug, Clone)]
pub struct SequencingServiceImpl {
    pub db_pool: DatabasePool,
    pub config: Config,
    pub auth_client: AuthClient,
    pub sample_client: SampleClient,
    pub notification_client: NotificationClient,
    pub template_client: TemplateClient,
}

impl SequencingServiceImpl {
    pub fn new(
        db_pool: DatabasePool,
        config: Config,
        auth_client: AuthClient,
        sample_client: SampleClient,
        notification_client: NotificationClient,
        template_client: TemplateClient,
    ) -> Result<Self> {
        Ok(Self {
            db_pool,
            config,
            auth_client,
            sample_client,
            notification_client,
            template_client,
        })
    }

    // Add service-level business logic methods here
    // pub async fn orchestrate_sequencing_workflow(&self, job_id: Uuid) -> Result<()> { ... }
    // pub async fn auto_schedule_jobs(&self) -> Result<()> { ... }
}
