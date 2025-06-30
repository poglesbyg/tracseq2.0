use anyhow::Result;
use sqlx::{PgPool, postgres::PgPoolOptions};

#[derive(Clone)]
pub struct DatabasePool {
    pub pool: PgPool,
}

impl DatabasePool {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<()> {
        // TODO: Implement migrations
        // sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }

    pub fn get_pool(&self) -> &PgPool {
        &self.pool
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
