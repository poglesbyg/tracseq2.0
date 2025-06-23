use sqlx::PgPool;
use std::sync::Arc;

use crate::{
    config::AppConfig,
    models::{spreadsheet::SpreadsheetDataManager, user::UserManager},
    repositories::PostgresRepositoryFactory,
    sample_submission::SampleSubmissionManager,
    sequencing::SequencingManager,
    services::{auth_service::AuthService, spreadsheet_service::SpreadsheetService},
    storage::Storage,
};

// New modular system imports
pub mod components;
pub mod product_lines;
pub mod traits;

// Re-export key types from the new modular system
pub use product_lines::{
    CompactLine, CompactVariant, HybridLine, ProductLine, ProfessionalLine, ProfessionalVariant,
    StudioLine, StudioVariant,
};
pub use traits::{
    Component, ComponentError, Configurable, ServiceConsumer, ServiceProvider, ServiceRegistry,
};

// Local simplified type definitions to avoid workspace import issues
#[derive(Debug)]
pub struct MetricsCollector {
    // Simplified version for compilation
}

#[derive(Debug)]
pub struct TracingService {
    // Simplified version for compilation
}

#[derive(Debug)]
pub struct HealthChecker {
    // Simplified version for compilation
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn get_all_metrics(&self) -> std::collections::HashMap<String, MetricValue> {
        std::collections::HashMap::new()
    }
}

impl TracingService {
    pub fn new() -> Self {
        Self {}
    }
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn check_all(&self) -> std::collections::HashMap<String, HealthStatus> {
        std::collections::HashMap::new()
    }

    pub async fn check_single(&self, _name: &str) -> Option<HealthStatus> {
        None
    }
}

// Local type definitions to match health.rs
#[derive(Debug, Clone)]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram(Vec<f64>),
    Timer(std::time::Duration),
}

#[derive(Debug, Clone)]
pub enum ServiceStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub status: ServiceStatus,
    pub message: Option<String>,
    pub response_time_ms: u64,
    pub last_checked: chrono::DateTime<chrono::Utc>,
}

// Main application component types
#[derive(Debug, Clone)]
pub struct AppComponents {
    pub config: crate::config::AppConfig,
    pub database: DatabaseComponent,
    pub storage: StorageComponent,
    pub sample_processing: SampleProcessingComponent,
    pub sequencing: SequencingComponent,
    pub repositories: RepositoriesComponent,
    pub user_manager: crate::models::user::UserManager,
    pub auth_service: crate::services::auth_service::AuthService,
    pub spreadsheet_service: crate::services::spreadsheet_service::SpreadsheetService,
    pub observability: ObservabilityComponent,
}

#[derive(Debug, Clone)]
pub struct DatabaseComponent {
    pub pool: PgPool,
}

#[derive(Debug, Clone)]
pub struct StorageComponent {
    pub storage: Arc<crate::storage::Storage>,
}

#[derive(Debug, Clone)]
pub struct SampleProcessingComponent {
    pub manager: Arc<crate::sample_submission::SampleSubmissionManager>,
}

#[derive(Debug, Clone)]
pub struct SequencingComponent {
    pub manager: Arc<crate::sequencing::SequencingManager>,
}

#[derive(Debug, Clone)]
pub struct ObservabilityComponent {
    pub metrics: Arc<MetricsCollector>,
    pub tracing: Arc<TracingService>,
    pub health_checker: Arc<HealthChecker>,
}

/// Repositories component for data access abstraction
#[derive(Debug, Clone)]
pub struct RepositoriesComponent {
    pub factory: Arc<PostgresRepositoryFactory>,
}

/// Builder for assembling application components
#[derive(Debug)]
pub struct ComponentBuilder {
    pub config: AppConfig,
    database_pool: Option<PgPool>,
    storage: Option<Arc<Storage>>,
    sample_manager: Option<Arc<SampleSubmissionManager>>,
    sequencing_manager: Option<Arc<SequencingManager>>,
    repository_factory: Option<Arc<PostgresRepositoryFactory>>,
    user_manager: Option<UserManager>,
    auth_service: Option<AuthService>,
    spreadsheet_service: Option<SpreadsheetService>,
}

impl ComponentBuilder {
    /// Create a new builder with the given configuration
    pub fn new(config: AppConfig) -> Self {
        Self {
            config,
            database_pool: None,
            storage: None,
            sample_manager: None,
            sequencing_manager: None,
            repository_factory: None,
            user_manager: None,
            auth_service: None,
            spreadsheet_service: None,
        }
    }

    /// Build the database component
    pub async fn with_database(mut self) -> Result<Self, AssemblyError> {
        let pool = crate::config::database::create_pool(&self.config.database.url)
            .await
            .map_err(AssemblyError::DatabaseConnection)?;

        // Run migrations
        crate::config::database::run_migrations(&pool)
            .await
            .map_err(|e| AssemblyError::Migration(Box::new(e)))?;

        self.database_pool = Some(pool);
        Ok(self)
    }

    /// Build the storage component
    pub async fn with_storage(mut self) -> Result<Self, AssemblyError> {
        // Create storage directory if it doesn't exist
        tokio::fs::create_dir_all(&self.config.storage.base_path)
            .await
            .map_err(AssemblyError::StorageSetup)?;

        let storage = Arc::new(Storage::new(self.config.storage.base_path.clone()));
        self.storage = Some(storage);
        Ok(self)
    }

    /// Build the sample processing component
    pub fn with_sample_processing(mut self) -> Result<Self, AssemblyError> {
        let pool = self
            .database_pool
            .as_ref()
            .ok_or(AssemblyError::MissingDependency(
                "Database pool required for sample processing",
            ))?;

        let manager = Arc::new(SampleSubmissionManager::new(pool.clone()));
        self.sample_manager = Some(manager);
        Ok(self)
    }

    /// Build the sequencing component
    pub fn with_sequencing(mut self) -> Result<Self, AssemblyError> {
        let pool = self
            .database_pool
            .as_ref()
            .ok_or(AssemblyError::MissingDependency(
                "Database pool required for sequencing",
            ))?;

        let manager = Arc::new(SequencingManager::new(pool.clone()));
        self.sequencing_manager = Some(manager);
        Ok(self)
    }

    /// Build the repository factory
    pub fn with_repositories(mut self) -> Result<Self, AssemblyError> {
        let pool = self
            .database_pool
            .as_ref()
            .ok_or(AssemblyError::MissingDependency(
                "Database pool required for repositories",
            ))?;

        let factory = Arc::new(PostgresRepositoryFactory::new(pool.clone()));
        self.repository_factory = Some(factory);
        Ok(self)
    }

    /// Build the user management component
    pub fn with_user_management(mut self) -> Result<Self, AssemblyError> {
        let pool = self
            .database_pool
            .as_ref()
            .ok_or(AssemblyError::MissingDependency(
                "Database pool required for user management",
            ))?;

        let user_manager = UserManager::new(pool.clone());
        self.user_manager = Some(user_manager);
        Ok(self)
    }

    /// Build the authentication service
    pub fn with_authentication(mut self) -> Result<Self, AssemblyError> {
        let pool = self
            .database_pool
            .as_ref()
            .ok_or(AssemblyError::MissingDependency(
                "Database pool required for authentication",
            ))?;

        // Get JWT secret from environment or use default (should be configurable)
        let jwt_secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "your-very-secure-secret-key-change-in-production".to_string());

        let auth_service = AuthService::new(pool.clone(), jwt_secret);
        self.auth_service = Some(auth_service);
        Ok(self)
    }

    /// Build the spreadsheet service
    pub fn with_spreadsheet(mut self) -> Result<Self, AssemblyError> {
        let pool = self
            .database_pool
            .as_ref()
            .ok_or(AssemblyError::MissingDependency(
                "Database pool required for spreadsheet service",
            ))?;

        let manager = SpreadsheetDataManager::new(pool.clone());
        let spreadsheet_service = SpreadsheetService::new(manager);
        self.spreadsheet_service = Some(spreadsheet_service);
        Ok(self)
    }

    /// Assemble all components
    pub fn build(self) -> Result<AppComponents, AssemblyError> {
        let database_pool = self
            .database_pool
            .ok_or(AssemblyError::MissingComponent("Database"))?;
        let storage = self
            .storage
            .ok_or(AssemblyError::MissingComponent("Storage"))?;
        let sample_manager = self
            .sample_manager
            .ok_or(AssemblyError::MissingComponent("Sample Processing"))?;
        let sequencing_manager = self
            .sequencing_manager
            .ok_or(AssemblyError::MissingComponent("Sequencing"))?;
        let repository_factory = self
            .repository_factory
            .ok_or(AssemblyError::MissingComponent("Repositories"))?;
        let user_manager = self
            .user_manager
            .ok_or(AssemblyError::MissingComponent("User Manager"))?;
        let auth_service = self
            .auth_service
            .ok_or(AssemblyError::MissingComponent("Auth Service"))?;
        let spreadsheet_service = self
            .spreadsheet_service
            .ok_or(AssemblyError::MissingComponent("Spreadsheet Service"))?;

        // Create observability component
        let observability = ObservabilityComponent {
            metrics: Arc::new(MetricsCollector::new()),
            tracing: Arc::new(TracingService::new()),
            health_checker: Arc::new(HealthChecker::new()),
        };

        Ok(AppComponents {
            config: self.config,
            database: DatabaseComponent {
                pool: database_pool,
            },
            storage: StorageComponent { storage },
            sample_processing: SampleProcessingComponent {
                manager: sample_manager,
            },
            sequencing: SequencingComponent {
                manager: sequencing_manager,
            },
            repositories: RepositoriesComponent {
                factory: repository_factory,
            },
            user_manager,
            auth_service,
            spreadsheet_service,
            observability,
        })
    }
}

/// Quick assembly method for production use
pub async fn assemble_production_components() -> Result<AppComponents, AssemblyError> {
    let config = AppConfig::from_env().map_err(AssemblyError::Configuration)?;

    ComponentBuilder::new(config)
        .with_database()
        .await?
        .with_repositories()?
        .with_storage()
        .await?
        .with_sample_processing()?
        .with_sequencing()?
        .with_user_management()?
        .with_authentication()?
        .with_spreadsheet()?
        .build()
}

/// Quick assembly method for testing
pub async fn assemble_test_components() -> Result<AppComponents, AssemblyError> {
    let config = AppConfig::for_testing();

    ComponentBuilder::new(config)
        .with_database()
        .await?
        .with_repositories()?
        .with_storage()
        .await?
        .with_sample_processing()?
        .with_sequencing()?
        .with_user_management()?
        .with_authentication()?
        .with_spreadsheet()?
        .build()
}

/// Custom assembly for specific use cases
pub struct CustomAssembly;

impl CustomAssembly {
    /// Create components for API-only mode (no storage operations)
    pub async fn api_only(config: AppConfig) -> Result<AppComponents, AssemblyError> {
        // Create minimal storage that doesn't write to disk
        let storage = Arc::new(Storage::new(std::env::temp_dir()));

        let components = ComponentBuilder::new(config)
            .with_database()
            .await?
            .with_repositories()?
            .with_sample_processing()?
            .with_sequencing()?
            .with_user_management()?
            .with_authentication()?
            .with_spreadsheet()?
            .build()?;

        Ok(AppComponents {
            config: components.config,
            database: components.database,
            storage: StorageComponent { storage },
            sample_processing: components.sample_processing,
            sequencing: components.sequencing,
            repositories: components.repositories,
            user_manager: components.user_manager,
            auth_service: components.auth_service,
            spreadsheet_service: components.spreadsheet_service,
            observability: components.observability,
        })
    }

    /// Create components for storage-only mode (no database operations)
    pub async fn storage_only(config: AppConfig) -> Result<StorageComponent, AssemblyError> {
        tokio::fs::create_dir_all(&config.storage.base_path)
            .await
            .map_err(AssemblyError::StorageSetup)?;

        let storage = Arc::new(Storage::new(config.storage.base_path));
        Ok(StorageComponent { storage })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AssemblyError {
    #[error("Configuration error: {0}")]
    Configuration(#[from] crate::config::ConfigError),
    #[error("Database connection error: {0}")]
    DatabaseConnection(#[from] sqlx::Error),
    #[error("Migration error: {0}")]
    Migration(Box<dyn std::error::Error + Send + Sync>),
    #[error("Storage setup error: {0}")]
    StorageSetup(#[from] std::io::Error),
    #[error("Missing dependency: {0}")]
    MissingDependency(&'static str),
    #[error("Missing component: {0}")]
    MissingComponent(&'static str),
}
