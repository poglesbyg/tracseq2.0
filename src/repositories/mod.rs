use async_trait::async_trait;
use uuid::Uuid;

/// Generic repository trait for data access abstraction
#[async_trait]
pub trait Repository<Entity, CreateData, UpdateData>: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn create(&self, data: CreateData) -> Result<Entity, Self::Error>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Entity>, Self::Error>;
    async fn update(&self, id: Uuid, data: UpdateData) -> Result<Entity, Self::Error>;
    async fn delete(&self, id: Uuid) -> Result<(), Self::Error>;
    async fn list(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<Entity>, Self::Error>;
}

/// Repository factory for creating different repository implementations
pub trait RepositoryFactory {
    type TemplateRepo: Repository<
        crate::models::template::Template,
        crate::models::template::CreateTemplate,
        crate::models::template::UpdateTemplate,
        Error = sqlx::Error,
    >;
    type SampleRepo: Repository<
        crate::sample_submission::Sample,
        crate::sample_submission::CreateSample,
        crate::sample_submission::CreateSample,
        Error = sqlx::Error,
    >;

    fn template_repository(&self) -> Self::TemplateRepo;
    fn sample_repository(&self) -> Self::SampleRepo;
}

/// PostgreSQL repository factory
pub struct PostgresRepositoryFactory {
    pool: sqlx::PgPool,
}

impl PostgresRepositoryFactory {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

impl RepositoryFactory for PostgresRepositoryFactory {
    type TemplateRepo = PostgresTemplateRepository;
    type SampleRepo = PostgresSampleRepository;

    fn template_repository(&self) -> Self::TemplateRepo {
        PostgresTemplateRepository::new(self.pool.clone())
    }

    fn sample_repository(&self) -> Self::SampleRepo {
        PostgresSampleRepository::new(self.pool.clone())
    }
}

/// Concrete PostgreSQL template repository
pub struct PostgresTemplateRepository {
    pool: sqlx::PgPool,
}

impl PostgresTemplateRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl
    Repository<
        crate::models::template::Template,
        crate::models::template::CreateTemplate,
        crate::models::template::UpdateTemplate,
    > for PostgresTemplateRepository
{
    type Error = sqlx::Error;

    async fn create(
        &self,
        data: crate::models::template::CreateTemplate,
    ) -> Result<crate::models::template::Template, Self::Error> {
        // Extract file_path and file_type from metadata if they exist
        let metadata = data.metadata.unwrap_or(serde_json::json!({}));
        let file_path = metadata
            .get("file_path")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let file_type = metadata
            .get("file_type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let clean_metadata = metadata
            .get("original_metadata")
            .unwrap_or(&metadata)
            .clone();

        sqlx::query_as::<_, crate::models::template::Template>(
            r#"
            INSERT INTO templates (name, description, file_path, file_type, metadata)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(&data.name)
        .bind(&data.description)
        .bind(file_path)
        .bind(file_type)
        .bind(&clean_metadata)
        .fetch_one(&self.pool)
        .await
    }

    async fn find_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<crate::models::template::Template>, Self::Error> {
        sqlx::query_as::<_, crate::models::template::Template>(
            "SELECT * FROM templates WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn update(
        &self,
        id: Uuid,
        data: crate::models::template::UpdateTemplate,
    ) -> Result<crate::models::template::Template, Self::Error> {
        // Build dynamic query based on provided fields
        let mut query_parts = Vec::new();
        let mut param_count = 1;

        if data.name.is_some() {
            query_parts.push(format!("name = ${}", param_count));
            param_count += 1;
        }
        if data.description.is_some() {
            query_parts.push(format!("description = ${}", param_count));
            param_count += 1;
        }
        if data.metadata.is_some() {
            query_parts.push(format!("metadata = ${}", param_count));
            param_count += 1;
        }

        if query_parts.is_empty() {
            // No updates provided, just return the existing template
            return self
                .find_by_id(id)
                .await?
                .ok_or_else(|| sqlx::Error::RowNotFound);
        }

        let query = format!(
            r#"
            UPDATE templates 
            SET {}, updated_at = NOW()
            WHERE id = ${}
            RETURNING *
            "#,
            query_parts.join(", "),
            param_count
        );

        let mut query_builder = sqlx::query_as::<_, crate::models::template::Template>(&query);

        // Bind parameters in the same order they were added
        if let Some(name) = data.name {
            query_builder = query_builder.bind(name);
        }
        if let Some(description) = data.description {
            query_builder = query_builder.bind(description);
        }
        if let Some(metadata) = data.metadata {
            query_builder = query_builder.bind(metadata);
        }

        // Bind the template_id last
        query_builder = query_builder.bind(id);

        query_builder.fetch_one(&self.pool).await
    }

    async fn delete(&self, id: Uuid) -> Result<(), Self::Error> {
        sqlx::query("DELETE FROM templates WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn list(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<crate::models::template::Template>, Self::Error> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);

        sqlx::query_as::<_, crate::models::template::Template>(
            "SELECT * FROM templates ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
    }
}

/// Concrete PostgreSQL sample repository
pub struct PostgresSampleRepository {
    pool: sqlx::PgPool,
}

impl PostgresSampleRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl
    Repository<
        crate::sample_submission::Sample,
        crate::sample_submission::CreateSample,
        crate::sample_submission::CreateSample,
    > for PostgresSampleRepository
{
    type Error = sqlx::Error;

    async fn create(
        &self,
        data: crate::sample_submission::CreateSample,
    ) -> Result<crate::sample_submission::Sample, Self::Error> {
        sqlx::query_as::<_, crate::sample_submission::Sample>(
            r#"
            INSERT INTO samples (name, barcode, location, status, metadata)
            VALUES ($1, $2, $3, 'pending', $4)
            RETURNING *
            "#,
        )
        .bind(&data.name)
        .bind(&data.barcode)
        .bind(&data.location)
        .bind(data.metadata.unwrap_or(serde_json::json!({})))
        .fetch_one(&self.pool)
        .await
    }

    async fn find_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<crate::sample_submission::Sample>, Self::Error> {
        sqlx::query_as::<_, crate::sample_submission::Sample>("SELECT * FROM samples WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    async fn update(
        &self,
        id: Uuid,
        data: crate::sample_submission::CreateSample,
    ) -> Result<crate::sample_submission::Sample, Self::Error> {
        sqlx::query_as::<_, crate::sample_submission::Sample>(
            r#"
            UPDATE samples 
            SET name = $2, barcode = $3, location = $4, metadata = $5, updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&data.name)
        .bind(&data.barcode)
        .bind(&data.location)
        .bind(data.metadata.unwrap_or(serde_json::json!({})))
        .fetch_one(&self.pool)
        .await
    }

    async fn delete(&self, id: Uuid) -> Result<(), Self::Error> {
        sqlx::query("DELETE FROM samples WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn list(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<crate::sample_submission::Sample>, Self::Error> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);

        sqlx::query_as::<_, crate::sample_submission::Sample>(
            "SELECT * FROM samples ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
    }
}

/// In-memory repository for testing
#[allow(dead_code)]
pub struct InMemoryRepositoryFactory {
    templates: std::sync::Arc<
        std::sync::RwLock<std::collections::HashMap<Uuid, crate::models::template::Template>>,
    >,
    samples: std::sync::Arc<
        std::sync::RwLock<std::collections::HashMap<Uuid, crate::sample_submission::Sample>>,
    >,
}

impl Default for InMemoryRepositoryFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryRepositoryFactory {
    pub fn new() -> Self {
        Self {
            templates: std::sync::Arc::new(
                std::sync::RwLock::new(std::collections::HashMap::new()),
            ),
            samples: std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }
}

// Export storage repository module
pub mod storage_repository;
