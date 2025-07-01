use anyhow::Result;
use sqlx::{PgPool, Postgres, migrate::MigrateDatabase};
use tracing::info;

#[derive(Debug, Clone)]
pub struct DatabasePool {
    pub pool: PgPool,
}

impl DatabasePool {
    pub async fn new(database_url: &str) -> Result<Self> {
        info!("Connecting to database: {}", database_url);

        // Create database if it doesn't exist
        if !Postgres::database_exists(database_url)
            .await
            .unwrap_or(false)
        {
            info!("Creating database...");
            Postgres::create_database(database_url).await?;
        }

        let pool = PgPool::connect(database_url).await?;

        info!("Database connection established successfully");

        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<()> {
        info!("Running database migrations...");

        // Create custom types
        self.create_enums().await?;

        // Create tables
        self.create_tables().await?;

        // Create indexes
        self.create_indexes().await?;

        info!("Database migrations completed successfully");
        Ok(())
    }

    async fn create_enums(&self) -> Result<()> {
        // Job status enum
        sqlx::query(
            r#"
            DO $$ BEGIN
                CREATE TYPE job_status AS ENUM (
                    'draft', 'submitted', 'validated', 'queued', 'running',
                    'completed', 'failed', 'cancelled', 'on_hold'
                );
            EXCEPTION
                WHEN duplicate_object THEN null;
            END $$;
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Priority enum
        sqlx::query(
            r#"
            DO $$ BEGIN
                CREATE TYPE priority AS ENUM (
                    'low', 'normal', 'high', 'urgent', 'critical'
                );
            EXCEPTION
                WHEN duplicate_object THEN null;
            END $$;
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Run status enum
        sqlx::query(
            r#"
            DO $$ BEGIN
                CREATE TYPE run_status AS ENUM (
                    'preparing', 'ready', 'running', 'analyzing', 'completed', 'failed', 'aborted'
                );
            EXCEPTION
                WHEN duplicate_object THEN null;
            END $$;
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Sample sheet status enum
        sqlx::query(
            r#"
            DO $$ BEGIN
                CREATE TYPE sample_sheet_status AS ENUM (
                    'draft', 'validating', 'valid', 'invalid', 'in_use', 'archived'
                );
            EXCEPTION
                WHEN duplicate_object THEN null;
            END $$;
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Workflow type enum
        sqlx::query(
            r#"
            DO $$ BEGIN
                CREATE TYPE workflow_type AS ENUM (
                    'whole_genome', 'exome', 'targeted_sequencing', 'rna_seq',
                    'chip_seq', 'bisulfite', 'single_cell', 'metagenomics', 'custom'
                );
            EXCEPTION
                WHEN duplicate_object THEN null;
            END $$;
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Execution status enum
        sqlx::query(
            r#"
            DO $$ BEGIN
                CREATE TYPE execution_status AS ENUM (
                    'pending', 'running', 'paused', 'completed', 'failed', 'aborted', 'retrying'
                );
            EXCEPTION
                WHEN duplicate_object THEN null;
            END $$;
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Pipeline type enum
        sqlx::query(
            r#"
            DO $$ BEGIN
                CREATE TYPE pipeline_id AS ENUM (
                    'quality_control', 'preprocessing', 'alignment', 'variant_calling',
                    'expression', 'annotation', 'reporting', 'custom'
                );
            EXCEPTION
                WHEN duplicate_object THEN null;
            END $$;
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Quality entity type enum
        sqlx::query(
            r#"
            DO $$ BEGIN
                CREATE TYPE quality_entity_type AS ENUM (
                    'job', 'run', 'sample', 'lane', 'analysis'
                );
            EXCEPTION
                WHEN duplicate_object THEN null;
            END $$;
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Quality status enum
        sqlx::query(
            r#"
            DO $$ BEGIN
                CREATE TYPE quality_status AS ENUM (
                    'pass', 'warning', 'fail', 'unknown'
                );
            EXCEPTION
                WHEN duplicate_object THEN null;
            END $$;
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Schedule status enum
        sqlx::query(
            r#"
            DO $$ BEGIN
                CREATE TYPE schedule_status AS ENUM (
                    'pending', 'scheduled', 'running', 'completed', 'cancelled', 'rescheduled'
                );
            EXCEPTION
                WHEN duplicate_object THEN null;
            END $$;
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn create_tables(&self) -> Result<()> {
        // Sequencing jobs table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sequencing_jobs (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                name VARCHAR(255) NOT NULL,
                description TEXT,
                status job_status NOT NULL DEFAULT 'draft',
                priority priority NOT NULL DEFAULT 'normal',
                platform_id VARCHAR(100) NOT NULL,
                workflow_id VARCHAR(100) NOT NULL,
                sample_sheet_id UUID,
                run_id UUID,
                created_by UUID NOT NULL,
                assigned_to UUID,
                estimated_start TIMESTAMPTZ,
                estimated_completion TIMESTAMPTZ,
                actual_start TIMESTAMPTZ,
                actual_completion TIMESTAMPTZ,
                metadata JSONB NOT NULL DEFAULT '{}',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Sequencing runs table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sequencing_runs (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                name VARCHAR(255) NOT NULL,
                platform_id VARCHAR(100) NOT NULL,
                chemistry VARCHAR(100) NOT NULL,
                flowcell_id VARCHAR(100) NOT NULL,
                status run_status NOT NULL DEFAULT 'preparing',
                cluster_generation_kit VARCHAR(100),
                sequencing_kit VARCHAR(100),
                read_structure VARCHAR(100) NOT NULL,
                sample_count INTEGER NOT NULL DEFAULT 0,
                estimated_yield_gb DECIMAL(10,3),
                actual_yield_gb DECIMAL(10,3),
                quality_score_mean DECIMAL(5,2),
                percent_pf DECIMAL(5,2),
                percent_q30 DECIMAL(5,2),
                error_rate DECIMAL(5,4),
                started_at TIMESTAMPTZ,
                completed_at TIMESTAMPTZ,
                data_path VARCHAR(500),
                created_by UUID NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Sample sheets table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sample_sheets (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                name VARCHAR(255) NOT NULL,
                platform_id VARCHAR(100) NOT NULL,
                version VARCHAR(50) NOT NULL DEFAULT '1.0',
                status sample_sheet_status NOT NULL DEFAULT 'draft',
                sample_count INTEGER NOT NULL DEFAULT 0,
                file_path VARCHAR(500),
                validation_errors JSONB,
                metadata JSONB NOT NULL DEFAULT '{}',
                created_by UUID NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Sample sheet entries table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sample_sheet_entries (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                sample_sheet_id UUID NOT NULL REFERENCES sample_sheets(id) ON DELETE CASCADE,
                sample_id VARCHAR(100) NOT NULL,
                sample_name VARCHAR(255) NOT NULL,
                sample_plate VARCHAR(50),
                sample_well VARCHAR(10),
                index1 VARCHAR(50) NOT NULL,
                index2 VARCHAR(50),
                project VARCHAR(255) NOT NULL,
                description TEXT
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Workflows table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS workflows (
                id VARCHAR(100) PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                description TEXT NOT NULL,
                version VARCHAR(50) NOT NULL,
                platform_ids TEXT[] NOT NULL,
                workflow_type workflow_type NOT NULL,
                steps JSONB NOT NULL,
                default_parameters JSONB NOT NULL DEFAULT '{}',
                estimated_duration_hours DECIMAL(5,2),
                is_active BOOLEAN NOT NULL DEFAULT true,
                created_by UUID NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Workflow executions table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS workflow_executions (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                job_id UUID NOT NULL REFERENCES sequencing_jobs(id) ON DELETE CASCADE,
                workflow_id VARCHAR(100) NOT NULL REFERENCES workflows(id),
                status execution_status NOT NULL DEFAULT 'pending',
                current_step VARCHAR(100),
                parameters JSONB NOT NULL DEFAULT '{}',
                outputs JSONB NOT NULL DEFAULT '{}',
                logs JSONB NOT NULL DEFAULT '{}',
                error_message TEXT,
                started_at TIMESTAMPTZ,
                completed_at TIMESTAMPTZ,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Analysis pipelines table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS analysis_pipelines (
                id VARCHAR(100) PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                description TEXT NOT NULL,
                version VARCHAR(50) NOT NULL,
                pipeline_id pipeline_id NOT NULL,
                container_image VARCHAR(500),
                command_template TEXT NOT NULL,
                input_types TEXT[] NOT NULL,
                output_types TEXT[] NOT NULL,
                resource_requirements JSONB NOT NULL DEFAULT '{}',
                parameters_schema JSONB NOT NULL DEFAULT '{}',
                is_active BOOLEAN NOT NULL DEFAULT true,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Analysis jobs table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS analysis_jobs (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                sequencing_job_id UUID NOT NULL REFERENCES sequencing_jobs(id) ON DELETE CASCADE,
                pipeline_id VARCHAR(100) NOT NULL REFERENCES analysis_pipelines(id),
                status job_status NOT NULL DEFAULT 'draft',
                parameters JSONB NOT NULL DEFAULT '{}',
                input_files TEXT[] NOT NULL DEFAULT '{}',
                output_files TEXT[] NOT NULL DEFAULT '{}',
                log_file VARCHAR(500),
                compute_resources_used JSONB NOT NULL DEFAULT '{}',
                started_at TIMESTAMPTZ,
                completed_at TIMESTAMPTZ,
                error_message TEXT,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Quality metrics table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS quality_metrics (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                entity_type quality_entity_type NOT NULL,
                entity_id UUID NOT NULL,
                metric_type VARCHAR(100) NOT NULL,
                value DECIMAL(15,6) NOT NULL,
                threshold_min DECIMAL(15,6),
                threshold_max DECIMAL(15,6),
                status quality_status NOT NULL,
                measured_at TIMESTAMPTZ NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Quality reports table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS quality_reports (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                entity_type quality_entity_type NOT NULL,
                entity_id UUID NOT NULL,
                report_type VARCHAR(100) NOT NULL,
                overall_status quality_status NOT NULL,
                metrics_summary JSONB NOT NULL DEFAULT '{}',
                recommendations TEXT[] NOT NULL DEFAULT '{}',
                report_data JSONB NOT NULL DEFAULT '{}',
                generated_at TIMESTAMPTZ NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Scheduled jobs table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS scheduled_jobs (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                sequencing_job_id UUID NOT NULL REFERENCES sequencing_jobs(id) ON DELETE CASCADE,
                platform_id VARCHAR(100) NOT NULL,
                priority priority NOT NULL,
                estimated_duration_hours DECIMAL(5,2) NOT NULL,
                earliest_start TIMESTAMPTZ NOT NULL,
                latest_start TIMESTAMPTZ,
                scheduled_start TIMESTAMPTZ,
                actual_start TIMESTAMPTZ,
                status schedule_status NOT NULL DEFAULT 'pending',
                constraints JSONB NOT NULL DEFAULT '{}',
                created_by UUID NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn create_indexes(&self) -> Result<()> {
        // Performance indexes - Sequencing Jobs
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_sequencing_jobs_status ON sequencing_jobs(status)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_sequencing_jobs_platform ON sequencing_jobs(platform_id)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_sequencing_jobs_created_by ON sequencing_jobs(created_by)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_sequencing_jobs_created_at ON sequencing_jobs(created_at)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_sequencing_jobs_priority ON sequencing_jobs(priority)")
            .execute(&self.pool)
            .await?;
            
        // Sequencing Runs indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_sequencing_runs_status ON sequencing_runs(status)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_sequencing_runs_platform ON sequencing_runs(platform_id)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_sequencing_runs_flowcell ON sequencing_runs(flowcell_id)")
            .execute(&self.pool)
            .await?;
            
        // Sample Sheets indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_sample_sheets_status ON sample_sheets(status)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_sample_sheets_platform ON sample_sheets(platform_id)")
            .execute(&self.pool)
            .await?;
            
        // Sample Sheet Entries indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_sample_sheet_entries_sheet_id ON sample_sheet_entries(sample_sheet_id)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_sample_sheet_entries_sample_id ON sample_sheet_entries(sample_id)")
            .execute(&self.pool)
            .await?;
            
        // Workflow Executions indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_workflow_executions_job_id ON workflow_executions(job_id)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_workflow_executions_status ON workflow_executions(status)")
            .execute(&self.pool)
            .await?;
            
        // Analysis Jobs indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_analysis_jobs_sequencing_job_id ON analysis_jobs(sequencing_job_id)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_analysis_jobs_status ON analysis_jobs(status)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_analysis_jobs_pipeline_id ON analysis_jobs(pipeline_id)")
            .execute(&self.pool)
            .await?;
            
        // Quality Metrics indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_quality_metrics_entity ON quality_metrics(entity_type, entity_id)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_quality_metrics_type ON quality_metrics(metric_type)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_quality_metrics_status ON quality_metrics(status)")
            .execute(&self.pool)
            .await?;
            
        // Quality Reports indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_quality_reports_entity ON quality_reports(entity_type, entity_id)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_quality_reports_type ON quality_reports(report_type)")
            .execute(&self.pool)
            .await?;
            
        // Scheduled Jobs indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_scheduled_jobs_sequencing_job_id ON scheduled_jobs(sequencing_job_id)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_scheduled_jobs_status ON scheduled_jobs(status)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_scheduled_jobs_platform ON scheduled_jobs(platform_id)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_scheduled_jobs_scheduled_start ON scheduled_jobs(scheduled_start)")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1").execute(&self.pool).await?;
        Ok(())
    }
}

/// Create a new database connection pool (compatibility function)
pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    let pool = DatabasePool::new(database_url).await?;
    Ok(pool.pool)
}

/// Run database migrations (compatibility function)
pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    let db_pool = DatabasePool { pool: pool.clone() };
    db_pool.migrate().await
}
