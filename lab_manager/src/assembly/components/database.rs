use sqlx::{Pool, Postgres, PgPool};
use sqlx::postgres::PgPoolOptions;
use tracing::{info, warn, error};
use async_trait::async_trait;
use serde_json;
use std::any::Any;

use super::super::traits::{
    Component, ComponentError, Configurable, ServiceProvider, ServiceRegistry,
};
use crate::config::DatabaseConfig;

/// Modular database component that provides connection pooling and database services
pub struct DatabaseComponent {
    config: DatabaseConfig,
    pool: Option<PgPool>,
    is_initialized: bool,
}

impl DatabaseComponent {
    pub fn new(config: DatabaseConfig) -> Self {
        Self {
            config,
            pool: None,
            is_initialized: false,
        }
    }

    /// Get the database pool (only available after initialization)
    pub fn pool(&self) -> Option<&PgPool> {
        self.pool.as_ref()
    }

    /// Get a cloned pool for sharing
    pub fn pool_cloned(&self) -> Option<PgPool> {
        self.pool.clone()
    }
}

#[async_trait]
impl Component for DatabaseComponent {
    fn component_id(&self) -> &'static str {
        "database"
    }

    fn component_name(&self) -> &'static str {
        "Database Connection Pool"
    }

    async fn initialize(&mut self, _context: &ServiceRegistry) -> Result<(), ComponentError> {
        if self.is_initialized {
            return Ok(());
        }

        info!("Initializing database connection pool");

        // Create the connection pool
        let pool = crate::config::database::create_pool(&self.config.url)
            .await
            .map_err(|e| {
                ComponentError::InitializationFailed(format!("Database connection failed: {}", e))
            })?;

        // Run migrations
        crate::config::database::run_migrations(&pool)
            .await
            .map_err(|e| {
                ComponentError::InitializationFailed(format!("Migration failed: {}", e))
            })?;

        self.pool = Some(pool);
        self.is_initialized = true;

        info!("Database component initialized successfully");
        Ok(())
    }

    async fn health_check(&self) -> Result<(), ComponentError> {
        if !self.is_initialized {
            return Err(ComponentError::InitializationFailed(
                "Component not initialized".to_string(),
            ));
        }

        if let Some(pool) = &self.pool {
            // Perform a simple health check query
            sqlx::query("SELECT 1").execute(pool).await.map_err(|e| {
                ComponentError::ServiceUnavailable(format!("Database health check failed: {}", e))
            })?;
        } else {
            return Err(ComponentError::ServiceUnavailable(
                "Database pool not available".to_string(),
            ));
        }

        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), ComponentError> {
        if let Some(pool) = &self.pool {
            info!("Closing database connection pool");
            pool.close().await;
        }

        self.pool = None;
        self.is_initialized = false;

        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[async_trait]
impl ServiceProvider for DatabaseComponent {
    fn provided_services(&self) -> Vec<&'static str> {
        vec!["database_pool", "database_connection"]
    }
}

impl Configurable for DatabaseComponent {
    type Config = DatabaseConfig;

    fn configure(&mut self, config: Self::Config) -> Result<(), ComponentError> {
        if self.is_initialized {
            return Err(ComponentError::ConfigurationError(
                "Cannot reconfigure initialized component".to_string(),
            ));
        }

        self.config = config;
        Ok(())
    }

    fn get_config(&self) -> &Self::Config {
        &self.config
    }
}

/// Builder for creating database components with different configurations
pub struct DatabaseComponentBuilder {
    config: Option<DatabaseConfig>,
}

impl DatabaseComponentBuilder {
    pub fn new() -> Self {
        Self { config: None }
    }

    /// Configure with a specific database config
    pub fn with_config(mut self, config: DatabaseConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Configure with environment variables
    pub fn with_env_config(mut self) -> Result<Self, ComponentError> {
        let config = DatabaseConfig::from_env().map_err(|e| {
            ComponentError::ConfigurationError(format!("Failed to load database config: {}", e))
        })?;
        self.config = Some(config);
        Ok(self)
    }

    /// Configure for testing (in-memory or test database)
    pub fn for_testing(mut self) -> Self {
        self.config = Some(DatabaseConfig::for_testing());
        self
    }

    /// Build the component
    pub fn build(self) -> Result<DatabaseComponent, ComponentError> {
        let config = self.config.ok_or_else(|| {
            ComponentError::ConfigurationError("Database configuration not provided".to_string())
        })?;

        Ok(DatabaseComponent::new(config))
    }
}

impl Default for DatabaseComponentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience macro for creating database components
#[macro_export]
macro_rules! database_component {
    // Create with environment config
    (env) => {
        $crate::assembly::components::database::DatabaseComponentBuilder::new()
            .with_env_config()?
            .build()?
    };

    // Create for testing
    (test) => {
        $crate::assembly::components::database::DatabaseComponentBuilder::new()
            .for_testing()
            .build()?
    };

    // Create with custom config
    ($config:expr) => {
        $crate::assembly::components::database::DatabaseComponentBuilder::new()
            .with_config($config)
            .build()?
    };
}
