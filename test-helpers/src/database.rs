//! Database test utilities

use anyhow::Result;
use once_cell::sync::Lazy;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;
use tokio::sync::Mutex;

/// Shared test database pool
static TEST_DB_POOL: Lazy<Mutex<Option<PgPool>>> = Lazy::new(|| Mutex::new(None));

/// Get or create a test database pool
pub async fn create_test_pool() -> Result<PgPool> {
    let mut pool_guard = TEST_DB_POOL.lock().await;
    
    if let Some(pool) = pool_guard.as_ref() {
        return Ok(pool.clone());
    }
    
    let database_url = get_test_database_url();
    
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .connect(&database_url)
        .await?;
    
    *pool_guard = Some(pool.clone());
    
    Ok(pool)
}

/// Get test database URL from environment or use default
pub fn get_test_database_url() -> String {
    env::var("TEST_DATABASE_URL")
        .or_else(|_| env::var("DATABASE_URL"))
        .unwrap_or_else(|_| "postgresql://tracseq_admin:tracseq_secure_password@localhost:5433/tracseq_main".to_string())
}

/// Create a unique test database for isolated testing
pub async fn create_isolated_test_db() -> Result<(PgPool, String)> {
    let base_url = get_test_database_url();
    let test_db_name = format!("test_{}_{}", 
        chrono::Utc::now().timestamp_micros(),
        uuid::Uuid::new_v4().to_string().replace("-", "")[..8].to_lowercase()
    );
    
    // Connect to tracseq_main database to create the test database
    let admin_url = base_url.replace("/tracseq_main", "/tracseq_main");
    let admin_pool = PgPool::connect(&admin_url).await?;
    
    // Create the test database
    sqlx::query(&format!("CREATE DATABASE {}", test_db_name))
        .execute(&admin_pool)
        .await?;
    
    // Connect to the new test database
    let test_db_url = base_url.replace("/tracseq_main", &format!("/{}", test_db_name));
    let test_pool = PgPool::connect(&test_db_url).await?;
    
    Ok((test_pool, test_db_name))
}

/// Drop a test database
pub async fn drop_test_db(db_name: &str) -> Result<()> {
    let base_url = get_test_database_url();
    let admin_url = base_url.replace("/tracseq_main", "/tracseq_main");
    let admin_pool = PgPool::connect(&admin_url).await?;
    
    // Terminate all connections to the test database
    sqlx::query(&format!(
        "SELECT pg_terminate_backend(pid) FROM pg_stat_activity 
         WHERE datname = '{}' AND pid <> pg_backend_pid()",
        db_name
    ))
    .execute(&admin_pool)
    .await?;
    
    // Drop the database
    sqlx::query(&format!("DROP DATABASE IF EXISTS {}", db_name))
        .execute(&admin_pool)
        .await?;
    
    Ok(())
}

/// Clean up test data for a specific test ID
pub async fn cleanup_test_data(pool: &PgPool, test_id: &str) -> Result<()> {
    // This is a generic cleanup function. Services can extend this
    // with their specific cleanup queries.
    
    // Example: Clean up users created in tests
    sqlx::query("DELETE FROM users WHERE email LIKE $1")
        .bind(format!("%{}%", test_id))
        .execute(pool)
        .await?;
    
    Ok(())
}

/// Transaction helper for test isolation
pub struct TestTransaction {
    tx: sqlx::Transaction<'static, sqlx::Postgres>,
}

impl TestTransaction {
    /// Begin a new test transaction
    pub async fn begin(pool: &PgPool) -> Result<Self> {
        let tx = pool.begin().await?;
        Ok(Self { tx })
    }
    
    /// Get a reference to the transaction
    pub fn tx(&mut self) -> &mut sqlx::Transaction<'static, sqlx::Postgres> {
        &mut self.tx
    }
    
    /// Rollback the transaction (happens automatically on drop)
    pub async fn rollback(self) -> Result<()> {
        self.tx.rollback().await?;
        Ok(())
    }
}

/// Run migrations for a test database
pub async fn run_migrations(pool: &PgPool, _migrations_path: &str) -> Result<()> {
    // For now, we'll comment out the migrations call since it needs a literal path
    // In a real implementation, you would use a specific migration path like:
    // sqlx::migrate!("../migrations").run(pool).await?;
    
    // As a placeholder, we'll just return Ok(())
    tracing::warn!("Migration running is not implemented in test-helpers");
    Ok(())
}

/// Create test tables for a service
pub async fn create_test_tables(pool: &PgPool, schema_sql: &str) -> Result<()> {
    sqlx::query(schema_sql).execute(pool).await?;
    Ok(())
}

/// Database test builder for fluent test setup
pub struct DatabaseTestBuilder {
    pool: Option<PgPool>,
    isolated: bool,
    migrations_path: Option<String>,
    schema_sql: Option<String>,
}

impl DatabaseTestBuilder {
    pub fn new() -> Self {
        Self {
            pool: None,
            isolated: false,
            migrations_path: None,
            schema_sql: None,
        }
    }
    
    /// Use an isolated database for this test
    pub fn isolated(mut self) -> Self {
        self.isolated = true;
        self
    }
    
    /// Run migrations from the specified path
    pub fn with_migrations(mut self, path: &str) -> Self {
        self.migrations_path = Some(path.to_string());
        self
    }
    
    /// Create tables using raw SQL
    pub fn with_schema(mut self, sql: &str) -> Self {
        self.schema_sql = Some(sql.to_string());
        self
    }
    
    /// Build the test database
    pub async fn build(self) -> Result<(PgPool, Option<String>)> {
        let (pool, db_name) = if self.isolated {
            let (pool, name) = create_isolated_test_db().await?;
            (pool, Some(name))
        } else {
            (create_test_pool().await?, None)
        };
        
        if let Some(migrations_path) = self.migrations_path {
            run_migrations(&pool, &migrations_path).await?;
        }
        
        if let Some(schema_sql) = self.schema_sql {
            create_test_tables(&pool, &schema_sql).await?;
        }
        
        Ok((pool, db_name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_test_pool() {
        let pool = create_test_pool().await;
        assert!(pool.is_ok());
    }
    
    #[tokio::test]
    async fn test_database_builder() {
        let result = DatabaseTestBuilder::new()
            .with_schema("CREATE TABLE IF NOT EXISTS test_table (id SERIAL PRIMARY KEY)")
            .build()
            .await;
            
        assert!(result.is_ok());
        
        if let Ok((pool, _)) = result {
            // Verify table was created
            let exists = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'test_table')"
            )
            .fetch_one(&pool)
            .await
            .unwrap();
            
            assert!(exists);
        }
    }
}