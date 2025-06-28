//! TracSeq 2.0 Test Helpers Library

pub mod database;
pub mod fixtures;
pub mod http;
pub mod mocks;

use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize test logging once per test run
pub static TEST_LOGGER: Lazy<()> = Lazy::new(|| {
    let _ = tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "warn".into()),
        ))
        .with(tracing_subscriber::fmt::layer().with_test_writer())
        .try_init();
});

/// Initialize test environment
pub fn init_test_env() {
    Lazy::force(&TEST_LOGGER);
    dotenvy::dotenv().ok();
}

/// Generate a unique test identifier
pub fn unique_test_id() -> String {
    format!("test_{}", uuid::Uuid::new_v4().to_string().replace("-", ""))
}

/// Test context for managing test resources
#[derive(Clone)]
pub struct TestContext {
    pub db_pool: Option<sqlx::PgPool>,
    pub test_id: String,
    cleanup_tasks: Arc<Mutex<Vec<Box<dyn FnOnce() + Send + 'static>>>>,
}

impl TestContext {
    /// Create a new test context
    pub async fn new() -> Self {
        init_test_env();
        
        Self {
            db_pool: None,
            test_id: unique_test_id(),
            cleanup_tasks: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Create a test context with database
    pub async fn with_database() -> anyhow::Result<Self> {
        init_test_env();
        
        let db_pool = database::create_test_pool().await?;
        
        Ok(Self {
            db_pool: Some(db_pool),
            test_id: unique_test_id(),
            cleanup_tasks: Arc::new(Mutex::new(Vec::new())),
        })
    }
    
    /// Add a cleanup task to run when the context is dropped
    pub async fn add_cleanup<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.cleanup_tasks.lock().await.push(Box::new(task));
    }
    
    /// Get the database pool
    pub fn db(&self) -> &sqlx::PgPool {
        self.db_pool
            .as_ref()
            .expect("Test context was not initialized with database")
    }
    
    /// Run cleanup tasks
    pub async fn cleanup(&self) {
        let mut tasks = self.cleanup_tasks.lock().await;
        for task in tasks.drain(..) {
            task();
        }
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        // Note: We can't run async cleanup in Drop, so cleanup() must be called explicitly
        // or use the cleanup tasks for sync cleanup only
    }
}

/// Macro to create a test with database setup
#[macro_export]
macro_rules! test_with_db {
    ($name:ident, $body:expr) => {
        #[tokio::test]
        async fn $name() {
            let ctx = $crate::TestContext::with_database()
                .await
                .expect("Failed to create test context");
            
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                tokio::runtime::Handle::current().block_on(async {
                    $body(ctx.clone()).await
                })
            }));
            
            // Always run cleanup
            ctx.cleanup().await;
            
            // Re-panic if test failed
            if let Err(e) = result {
                std::panic::resume_unwind(e);
            }
        }
    };
}

/// Macro to create a test with HTTP server
#[macro_export]
macro_rules! test_with_server {
    ($name:ident, $app:expr, $body:expr) => {
        #[tokio::test]
        async fn $name() {
            let server = $crate::http::TestServer::new($app)
                .await
                .expect("Failed to create test server");
            
            $body(server).await
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_unique_test_id() {
        let id1 = unique_test_id();
        let id2 = unique_test_id();
        
        assert_ne!(id1, id2);
        assert!(id1.starts_with("test_"));
        assert!(id2.starts_with("test_"));
    }
    
    #[tokio::test]
    async fn test_context_cleanup() {
        let ctx = TestContext::new().await;
        let cleanup_called = Arc::new(Mutex::new(false));
        let cleanup_called_clone = cleanup_called.clone();
        
        ctx.add_cleanup(move || {
            let _ = cleanup_called_clone.try_lock().map(|mut v| *v = true);
        }).await;
        
        ctx.cleanup().await;
        
        assert!(*cleanup_called.lock().await);
    }
}