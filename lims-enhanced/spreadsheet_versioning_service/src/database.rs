use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::{info, error};

#[derive(Debug, Clone)]
pub struct Database {
    pub pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        info!("Connecting to database: {}", database_url);

        // Validate database URL format
        if database_url.is_empty() {
            return Err(anyhow::anyhow!("Database URL is empty"));
        }
        
        if !database_url.starts_with("postgres://") && !database_url.starts_with("postgresql://") {
            return Err(anyhow::anyhow!("Invalid database URL format. Must start with postgres:// or postgresql://"));
        }

        let pool = PgPoolOptions::new()
            .max_connections(20)
            .min_connections(5)
            .acquire_timeout(std::time::Duration::from_secs(30))
            .idle_timeout(std::time::Duration::from_secs(600))
            .max_lifetime(std::time::Duration::from_secs(1800))
            .connect(database_url)
            .await
            .map_err(|e| {
                error!("Failed to connect to database: {}", e);
                anyhow::anyhow!("Database connection failed: {}", e)
            })?;

        info!("✅ Database connection pool established");
        info!("  Max connections: 20");
        info!("  Min connections: 5");

        Ok(Self { pool })
    }

    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn migrate(&self) -> Result<()> {
        info!("🔄 Running database migrations");

        // Create extensions
        sqlx::query("CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\"")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE EXTENSION IF NOT EXISTS \"pg_trgm\"")
            .execute(&self.pool)
            .await?;

        // Create version status enum
        sqlx::query(
            r#"
            DO $$ BEGIN
                CREATE TYPE version_status AS ENUM (
                    'draft',
                    'active',
                    'archived',
                    'deleted'
                );
            EXCEPTION
                WHEN duplicate_object THEN null;
            END $$;
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create conflict status enum
        sqlx::query(
            r#"
            DO $$ BEGIN
                CREATE TYPE conflict_status AS ENUM (
                    'pending',
                    'resolved',
                    'rejected'
                );
            EXCEPTION
                WHEN duplicate_object THEN null;
            END $$;
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create spreadsheet versions table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS spreadsheet_versions (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                spreadsheet_id UUID NOT NULL,
                version_number INTEGER NOT NULL,
                version_tag VARCHAR(50),
                status version_status NOT NULL DEFAULT 'draft',
                parent_version_id UUID REFERENCES spreadsheet_versions(id),
                
                -- Spreadsheet metadata
                name VARCHAR(255) NOT NULL,
                filename VARCHAR(255) NOT NULL,
                original_filename VARCHAR(255) NOT NULL,
                file_type VARCHAR(50) NOT NULL,
                file_size BIGINT NOT NULL,
                file_hash VARCHAR(64) NOT NULL,
                
                -- Version metadata
                changes_summary TEXT,
                change_count INTEGER DEFAULT 0,
                created_by UUID,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                
                -- Metadata
                metadata JSONB NOT NULL DEFAULT '{}',
                
                UNIQUE(spreadsheet_id, version_number)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create version data table (stores the actual spreadsheet data)
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS version_data (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                version_id UUID NOT NULL REFERENCES spreadsheet_versions(id) ON DELETE CASCADE,
                sheet_name VARCHAR(255) NOT NULL,
                sheet_index INTEGER NOT NULL,
                row_index INTEGER NOT NULL,
                column_index INTEGER NOT NULL,
                column_name VARCHAR(255),
                cell_value TEXT,
                data_type VARCHAR(50),
                formatted_value TEXT,
                cell_formula TEXT,
                cell_metadata JSONB DEFAULT '{}',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                
                UNIQUE(version_id, sheet_name, row_index, column_index)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create version diffs table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS version_diffs (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                from_version_id UUID NOT NULL REFERENCES spreadsheet_versions(id),
                to_version_id UUID NOT NULL REFERENCES spreadsheet_versions(id),
                diff_type VARCHAR(50) NOT NULL, -- 'cell_change', 'row_added', 'row_deleted', 'column_added', 'column_deleted', 'sheet_added', 'sheet_deleted'
                sheet_name VARCHAR(255),
                row_index INTEGER,
                column_index INTEGER,
                column_name VARCHAR(255),
                old_value TEXT,
                new_value TEXT,
                change_metadata JSONB DEFAULT '{}',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                
                UNIQUE(from_version_id, to_version_id, diff_type, sheet_name, row_index, column_index)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create version conflicts table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS version_conflicts (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                base_version_id UUID NOT NULL REFERENCES spreadsheet_versions(id),
                version_a_id UUID NOT NULL REFERENCES spreadsheet_versions(id),
                version_b_id UUID NOT NULL REFERENCES spreadsheet_versions(id),
                
                conflict_type VARCHAR(50) NOT NULL, -- 'cell_conflict', 'structural_conflict', 'metadata_conflict'
                sheet_name VARCHAR(255),
                row_index INTEGER,
                column_index INTEGER,
                column_name VARCHAR(255),
                
                value_a TEXT,
                value_b TEXT,
                base_value TEXT,
                
                status conflict_status NOT NULL DEFAULT 'pending',
                resolution_strategy VARCHAR(50),
                resolved_value TEXT,
                resolved_by UUID,
                resolved_at TIMESTAMPTZ,
                
                conflict_metadata JSONB DEFAULT '{}',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create version merge requests table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS version_merge_requests (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                source_version_id UUID NOT NULL REFERENCES spreadsheet_versions(id),
                target_version_id UUID NOT NULL REFERENCES spreadsheet_versions(id),
                merged_version_id UUID REFERENCES spreadsheet_versions(id),
                
                title VARCHAR(255) NOT NULL,
                description TEXT,
                status VARCHAR(50) NOT NULL DEFAULT 'pending', -- 'pending', 'approved', 'rejected', 'merged'
                
                requested_by UUID NOT NULL,
                reviewed_by UUID,
                merged_by UUID,
                
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                reviewed_at TIMESTAMPTZ,
                merged_at TIMESTAMPTZ,
                
                merge_metadata JSONB DEFAULT '{}'
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        self.create_indexes().await?;
        self.create_triggers().await?;

        info!("✅ Database migrations completed successfully");
        Ok(())
    }

    async fn create_indexes(&self) -> Result<()> {
        info!("Creating database indexes...");

        // Spreadsheet versions indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_spreadsheet_versions_spreadsheet_id ON spreadsheet_versions(spreadsheet_id)")
            .execute(&self.pool).await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_spreadsheet_versions_version_number ON spreadsheet_versions(version_number)")
            .execute(&self.pool).await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_spreadsheet_versions_status ON spreadsheet_versions(status)")
            .execute(&self.pool).await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_spreadsheet_versions_created_at ON spreadsheet_versions(created_at)")
            .execute(&self.pool).await?;

        // Version data indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_version_data_version_id ON version_data(version_id)")
            .execute(&self.pool).await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_version_data_sheet_name ON version_data(sheet_name)")
            .execute(&self.pool).await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_version_data_position ON version_data(row_index, column_index)")
            .execute(&self.pool).await?;

        // Version diffs indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_version_diffs_from_to ON version_diffs(from_version_id, to_version_id)")
            .execute(&self.pool).await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_version_diffs_type ON version_diffs(diff_type)")
            .execute(&self.pool).await?;

        // Version conflicts indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_version_conflicts_status ON version_conflicts(status)")
            .execute(&self.pool).await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_version_conflicts_versions ON version_conflicts(version_a_id, version_b_id)")
            .execute(&self.pool).await?;

        // GIN indexes for JSONB columns
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_spreadsheet_versions_metadata_gin ON spreadsheet_versions USING GIN(metadata)")
            .execute(&self.pool).await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_version_data_cell_metadata_gin ON version_data USING GIN(cell_metadata)")
            .execute(&self.pool).await?;

        // Text search indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_version_data_cell_value_trgm ON version_data USING GIN(cell_value gin_trgm_ops)")
            .execute(&self.pool).await?;

        Ok(())
    }

    async fn create_triggers(&self) -> Result<()> {
        info!("Creating database triggers...");

        // Update timestamp trigger function
        sqlx::query(
            r#"
            CREATE OR REPLACE FUNCTION update_updated_at_column()
            RETURNS TRIGGER AS $$
            BEGIN
                NEW.updated_at = NOW();
                RETURN NEW;
            END;
            $$ LANGUAGE plpgsql;
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Apply triggers to tables
        sqlx::query(
            r#"
            CREATE TRIGGER trigger_spreadsheet_versions_updated_at
                BEFORE UPDATE ON spreadsheet_versions
                FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TRIGGER trigger_version_conflicts_updated_at
                BEFORE UPDATE ON version_conflicts
                FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TRIGGER trigger_version_merge_requests_updated_at
                BEFORE UPDATE ON version_merge_requests
                FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
} 
