use anyhow::Result;
use sqlx::PgPool;
use tracing::info;

/// Database connection pool wrapper
#[derive(Debug, Clone)]
pub struct DatabasePool {
    pub pool: PgPool,
}

impl DatabasePool {
    /// Create a new database connection pool
    pub async fn new(database_url: &str) -> Result<Self> {
        info!(
            "Connecting to database: {}",
            database_url.split('@').next().unwrap_or("***")
        );

        let pool = PgPool::connect(database_url).await?;

        // Test the connection
        let _row: (i32,) = sqlx::query_as("SELECT 1").fetch_one(&pool).await?;

        info!("Database connection established successfully");

        Ok(Self { pool })
    }

    /// Run database migrations
    pub async fn migrate(&self) -> Result<()> {
        info!("Running database migrations");

        // Create the samples table and related structures
        self.create_sample_tables().await?;
        self.create_workflow_tables().await?;
        self.create_barcode_tables().await?;
        self.create_audit_tables().await?;

        info!("Database migrations completed successfully");
        Ok(())
    }

    /// Create sample-related tables
    async fn create_sample_tables(&self) -> Result<()> {
        // Create sample status enum
        sqlx::query(
            r#"
            DO $$ BEGIN
                CREATE TYPE sample_status AS ENUM (
                    'pending',
                    'validated', 
                    'in_storage',
                    'in_sequencing',
                    'completed',
                    'failed',
                    'discarded'
                );
            EXCEPTION
                WHEN duplicate_object THEN null;
            END $$;
        "#,
        )
        .execute(&self.pool)
        .await?;

        // Create samples table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS samples (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                name VARCHAR(255) NOT NULL,
                barcode VARCHAR(100) NOT NULL UNIQUE,
                sample_type VARCHAR(50) NOT NULL,
                status sample_status NOT NULL DEFAULT 'pending',
                template_id UUID,
                source_type VARCHAR(50),
                source_identifier VARCHAR(255),
                collection_date TIMESTAMPTZ,
                collection_location VARCHAR(255),
                collector VARCHAR(255),
                concentration DECIMAL(10,4),
                volume DECIMAL(10,4),
                unit VARCHAR(20),
                quality_score DECIMAL(3,2),
                metadata JSONB DEFAULT '{}',
                notes TEXT,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                created_by VARCHAR(255),
                updated_by VARCHAR(255),
                CONSTRAINT samples_quality_score_valid CHECK (quality_score >= 0 AND quality_score <= 1)
            );
        "#)
        .execute(&self.pool)
        .await?;

        // Create indexes for performance
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_samples_barcode ON samples(barcode);")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_samples_status ON samples(status);")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_samples_created_at ON samples(created_at);")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_samples_template_id ON samples(template_id);")
            .execute(&self.pool)
            .await?;

        // Create sample relationships table for batch tracking
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sample_relationships (
                id SERIAL PRIMARY KEY,
                parent_sample_id UUID NOT NULL REFERENCES samples(id) ON DELETE CASCADE,
                child_sample_id UUID NOT NULL REFERENCES samples(id) ON DELETE CASCADE,
                relationship_type VARCHAR(50) NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                UNIQUE(parent_sample_id, child_sample_id, relationship_type)
            );
        "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Create workflow-related tables
    async fn create_workflow_tables(&self) -> Result<()> {
        // Create sample status history table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sample_status_history (
                id SERIAL PRIMARY KEY,
                sample_id UUID NOT NULL REFERENCES samples(id) ON DELETE CASCADE,
                previous_status sample_status,
                new_status sample_status NOT NULL,
                changed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                changed_by VARCHAR(255),
                reason VARCHAR(500),
                automated BOOLEAN NOT NULL DEFAULT FALSE,
                metadata JSONB DEFAULT '{}'
            );
        "#,
        )
        .execute(&self.pool)
        .await?;

        // Create sample validation rules table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sample_validation_rules (
                id SERIAL PRIMARY KEY,
                rule_name VARCHAR(100) NOT NULL UNIQUE,
                sample_type VARCHAR(50),
                rule_expression TEXT NOT NULL,
                error_message VARCHAR(500),
                is_active BOOLEAN NOT NULL DEFAULT TRUE,
                severity VARCHAR(20) NOT NULL DEFAULT 'error',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
        "#,
        )
        .execute(&self.pool)
        .await?;

        // Create sample validation results table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sample_validation_results (
                id SERIAL PRIMARY KEY,
                sample_id UUID NOT NULL REFERENCES samples(id) ON DELETE CASCADE,
                rule_id INTEGER REFERENCES sample_validation_rules(id),
                validation_passed BOOLEAN NOT NULL,
                error_message TEXT,
                validated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                validated_by VARCHAR(255)
            );
        "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Create barcode-related tables
    async fn create_barcode_tables(&self) -> Result<()> {
        // Create barcode sequence table for unique ID generation
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS barcode_sequences (
                id SERIAL PRIMARY KEY,
                prefix VARCHAR(20) NOT NULL,
                sequence_number BIGINT NOT NULL DEFAULT 1,
                last_generated TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                UNIQUE(prefix)
            );
        "#,
        )
        .execute(&self.pool)
        .await?;

        // Create barcode audit table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS barcode_audit (
                id SERIAL PRIMARY KEY,
                barcode VARCHAR(100) NOT NULL,
                sample_id UUID REFERENCES samples(id),
                action VARCHAR(50) NOT NULL, -- 'generated', 'assigned', 'scanned'
                performed_by VARCHAR(255),
                performed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                metadata JSONB DEFAULT '{}'
            );
        "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Create audit-related tables
    async fn create_audit_tables(&self) -> Result<()> {
        // Create sample audit log table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sample_audit_log (
                id SERIAL PRIMARY KEY,
                sample_id UUID REFERENCES samples(id),
                action VARCHAR(100) NOT NULL,
                old_values JSONB,
                new_values JSONB,
                performed_by VARCHAR(255),
                performed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                session_id VARCHAR(255),
                ip_address INET,
                user_agent TEXT
            );
        "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes for audit log
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_sample_audit_log_sample_id ON sample_audit_log(sample_id);")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_sample_audit_log_performed_at ON sample_audit_log(performed_at);")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Check database health
    pub async fn health_check(&self) -> Result<DatabaseHealth> {
        let start_time = std::time::Instant::now();

        // Test basic connectivity
        let _row: (i32,) = sqlx::query_as("SELECT 1").fetch_one(&self.pool).await?;

        // Get connection stats
        let pool_info = self.pool.options();
        let connections = self.pool.size();

        // Get database version
        let version_row: (String,) = sqlx::query_as("SELECT version()")
            .fetch_one(&self.pool)
            .await?;

        Ok(DatabaseHealth {
            is_healthy: true,
            response_time_ms: start_time.elapsed().as_millis() as u64,
            active_connections: self.pool.size().saturating_sub(self.pool.num_idle() as u32),
            idle_connections: self.pool.num_idle() as u32,
            max_connections: pool_info.get_max_connections(),
            database_version: Some(version_row.0),
        })
    }

    /// Get the raw pool reference
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

/// Database health information
#[derive(Debug, Clone)]
pub struct DatabaseHealth {
    pub is_healthy: bool,
    pub response_time_ms: u64,
    pub active_connections: u32,
    pub idle_connections: u32,
    pub max_connections: u32,
    pub database_version: Option<String>,
}

/// Create a new database connection pool (compatibility function)
pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    let pool = PgPool::connect(database_url).await?;

    // Test the connection
    let _row: (i32,) = sqlx::query_as("SELECT 1").fetch_one(&pool).await?;

    Ok(pool)
}

/// Run database migrations (compatibility function)
pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    let db_pool = DatabasePool { pool: pool.clone() };
    db_pool.migrate().await
}
