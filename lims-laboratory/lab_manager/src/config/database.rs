use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use std::time::Duration;
use tracing::info;

/// Enhanced database configuration with performance tuning
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub max_lifetime: Duration,
    pub idle_timeout: Duration,
    pub acquire_timeout: Duration,
    pub test_before_acquire: bool,
    pub enable_logging: bool,
}

impl DatabaseConfig {
    pub fn from_env() -> Result<Self> {
        let url = std::env::var("DATABASE_URL")
            .map_err(|_| anyhow::anyhow!("DATABASE_URL must be set"))?;

        // Production-ready defaults
        Ok(Self {
            url,
            max_connections: std::env::var("DB_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "20".to_string())
                .parse()
                .unwrap_or(20),
            min_connections: std::env::var("DB_MIN_CONNECTIONS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
            max_lifetime: Duration::from_secs(
                std::env::var("DB_MAX_LIFETIME_SECONDS")
                    .unwrap_or_else(|_| "1800".to_string()) // 30 minutes
                    .parse()
                    .unwrap_or(1800),
            ),
            idle_timeout: Duration::from_secs(
                std::env::var("DB_IDLE_TIMEOUT_SECONDS")
                    .unwrap_or_else(|_| "600".to_string()) // 10 minutes
                    .parse()
                    .unwrap_or(600),
            ),
            acquire_timeout: Duration::from_secs(
                std::env::var("DB_ACQUIRE_TIMEOUT_SECONDS")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()
                    .unwrap_or(30),
            ),
            test_before_acquire: std::env::var("DB_TEST_BEFORE_ACQUIRE")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            enable_logging: std::env::var("DB_ENABLE_LOGGING")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
        })
    }
}

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let config = DatabaseConfig::from_env().map_err(|e| sqlx::Error::Configuration(e.into()))?;

    info!(
        "Creating database pool with max_connections: {}",
        config.max_connections
    );

    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .max_lifetime(config.max_lifetime)
        .idle_timeout(config.idle_timeout)
        .acquire_timeout(config.acquire_timeout)
        .test_before_acquire(config.test_before_acquire)
        .connect(database_url)
        .await?;

    info!("Database pool created successfully");
    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    info!("Running database migrations...");
    sqlx::migrate!("./migrations").run(pool).await?;
    info!("Database migrations completed successfully");
    Ok(())
}

/// Database health check with detailed metrics
pub async fn health_check(pool: &PgPool) -> Result<DatabaseHealth, sqlx::Error> {
    let start = std::time::Instant::now();

    // Test connection
    let _: (i32,) = sqlx::query_as("SELECT 1").fetch_one(pool).await?;

    let response_time = start.elapsed();

    // Get pool status
    let active_connections = pool.size();
    let idle_connections = pool.num_idle();

    Ok(DatabaseHealth {
        is_healthy: true,
        response_time_ms: response_time.as_millis() as u64,
        active_connections,
        idle_connections: idle_connections as u32,
        max_connections: pool.options().get_max_connections(),
    })
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DatabaseHealth {
    pub is_healthy: bool,
    pub response_time_ms: u64,
    pub active_connections: u32,
    pub idle_connections: u32,
    pub max_connections: u32,
}

/// Database performance monitoring
pub struct DatabaseMonitor {
    pool: PgPool,
}

impl DatabaseMonitor {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_performance_metrics(&self) -> Result<DatabaseMetrics, sqlx::Error> {
        let stats = sqlx::query(
            r#"
            SELECT 
                schemaname,
                tablename,
                n_tup_ins as inserts,
                n_tup_upd as updates,
                n_tup_del as deletes,
                n_live_tup as live_tuples,
                n_dead_tup as dead_tuples
            FROM pg_stat_user_tables
            ORDER BY n_live_tup DESC
            LIMIT 10
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let active_queries = sqlx::query(
            r#"
            SELECT count(*) as active_count
            FROM pg_stat_activity 
            WHERE state = 'active' AND query NOT LIKE '%pg_stat_activity%'
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(DatabaseMetrics {
            table_stats: stats
                .into_iter()
                .map(|row| TableStats {
                    schema_name: row
                        .try_get::<Option<String>, _>("schemaname")
                        .unwrap_or_default()
                        .unwrap_or_default(),
                    table_name: row
                        .try_get::<Option<String>, _>("tablename")
                        .unwrap_or_default()
                        .unwrap_or_default(),
                    inserts: row
                        .try_get::<Option<i64>, _>("inserts")
                        .unwrap_or_default()
                        .unwrap_or(0),
                    updates: row
                        .try_get::<Option<i64>, _>("updates")
                        .unwrap_or_default()
                        .unwrap_or(0),
                    deletes: row
                        .try_get::<Option<i64>, _>("deletes")
                        .unwrap_or_default()
                        .unwrap_or(0),
                    live_tuples: row
                        .try_get::<Option<i64>, _>("live_tuples")
                        .unwrap_or_default()
                        .unwrap_or(0),
                    dead_tuples: row
                        .try_get::<Option<i64>, _>("dead_tuples")
                        .unwrap_or_default()
                        .unwrap_or(0),
                })
                .collect(),
            active_queries: active_queries
                .try_get::<Option<i64>, _>("active_count")
                .unwrap_or_default()
                .unwrap_or(0),
        })
    }

    pub async fn check_slow_queries(&self) -> Result<Vec<SlowQuery>, sqlx::Error> {
        let slow_queries = sqlx::query(
            r#"
            SELECT 
                query,
                calls,
                total_time,
                mean_time,
                stddev_time
            FROM pg_stat_statements 
            WHERE mean_time > 1000 -- queries taking more than 1 second on average
            ORDER BY mean_time DESC
            LIMIT 10
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(slow_queries
            .into_iter()
            .map(|row| SlowQuery {
                query: row
                    .try_get::<Option<String>, _>("query")
                    .unwrap_or_default()
                    .unwrap_or_default(),
                calls: row
                    .try_get::<Option<i64>, _>("calls")
                    .unwrap_or_default()
                    .unwrap_or(0),
                total_time: row
                    .try_get::<Option<f64>, _>("total_time")
                    .unwrap_or_default()
                    .unwrap_or(0.0),
                mean_time: row
                    .try_get::<Option<f64>, _>("mean_time")
                    .unwrap_or_default()
                    .unwrap_or(0.0),
                stddev_time: row
                    .try_get::<Option<f64>, _>("stddev_time")
                    .unwrap_or_default()
                    .unwrap_or(0.0),
            })
            .collect())
    }
}

#[derive(Debug, serde::Serialize)]
pub struct DatabaseMetrics {
    pub table_stats: Vec<TableStats>,
    pub active_queries: i64,
}

#[derive(Debug, serde::Serialize)]
pub struct TableStats {
    pub schema_name: String,
    pub table_name: String,
    pub inserts: i64,
    pub updates: i64,
    pub deletes: i64,
    pub live_tuples: i64,
    pub dead_tuples: i64,
}

#[derive(Debug, serde::Serialize)]
pub struct SlowQuery {
    pub query: String,
    pub calls: i64,
    pub total_time: f64,
    pub mean_time: f64,
    pub stddev_time: f64,
}
