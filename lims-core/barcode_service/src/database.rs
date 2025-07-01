use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::{info, warn};

/// Database connection pool type alias
pub type DatabasePool = PgPool;

impl From<PgPool> for DatabasePool {
    fn from(pool: PgPool) -> Self {
        pool
    }
}

/// Database connection module
pub struct Database;

impl Database {
    /// Create a new database connection pool
    pub async fn new(database_url: &str) -> Result<DatabasePool> {
        info!("🔗 Connecting to PostgreSQL database...");
        
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        info!("✅ Database connection established");
        Ok(pool)
    }
}

/// Convenience function to create a database pool
pub async fn create_pool(database_url: &str) -> Result<DatabasePool> {
    Database::new(database_url).await
}

/// Database pool constructor (alias for convenience)
pub async fn new(database_url: &str) -> Result<DatabasePool> {
    create_pool(database_url).await
}

/// Run database migrations
pub async fn run_migrations(pool: &DatabasePool) -> Result<()> {
    info!("🔄 Running database migrations...");

    // Create barcodes table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS barcodes (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            barcode VARCHAR(255) NOT NULL UNIQUE,
            prefix VARCHAR(50),
            sample_type VARCHAR(100),
            location_id INTEGER,
            is_reserved BOOLEAN NOT NULL DEFAULT false,
            reserved_by VARCHAR(255),
            reserved_at TIMESTAMPTZ,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            metadata JSONB
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Create indexes for better performance
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_barcodes_barcode ON barcodes(barcode);
        CREATE INDEX IF NOT EXISTS idx_barcodes_prefix ON barcodes(prefix);
        CREATE INDEX IF NOT EXISTS idx_barcodes_sample_type ON barcodes(sample_type);
        CREATE INDEX IF NOT EXISTS idx_barcodes_location_id ON barcodes(location_id);
        CREATE INDEX IF NOT EXISTS idx_barcodes_is_reserved ON barcodes(is_reserved);
        CREATE INDEX IF NOT EXISTS idx_barcodes_created_at ON barcodes(created_at);
        "#,
    )
    .execute(pool)
    .await?;

    info!("✅ Database migrations completed");
    Ok(())
}

/// Database health check
pub async fn health_check(pool: &DatabasePool) -> Result<bool> {
    match sqlx::query("SELECT 1").fetch_one(pool).await {
        Ok(_) => Ok(true),
        Err(e) => {
            warn!("Database health check failed: {}", e);
            Ok(false)
        }
    }
} 