use super::components::{
    database::{DatabaseComponentBuilder},
    storage::{StorageComponentBuilder},
};
use super::traits::{ComponentError, ServiceRegistry};
use crate::config::{DatabaseConfig, StorageConfig};
use std::collections::HashMap;

/// Different "product lines" - pre-configured assemblies for specific use cases
/// These are like IKEA's different furniture collections, each optimized for different needs
///
/// The "Studio" line - minimal components for development and testing
pub struct StudioLine;

impl StudioLine {
    /// Minimal setup perfect for local development
    pub async fn developer_setup() -> Result<ServiceRegistry, ComponentError> {
        tracing::info!("🛠️  Assembling Studio Line: Developer Setup");

        let mut registry = ServiceRegistry::new();

        // In-memory database for quick development
        let db_config = DatabaseConfig {
            url: "postgres://postgres:postgres@localhost:5432/lab_manager_test".to_string(),
            max_connections: 5,
            min_connections: 1,
        };
        let database = DatabaseComponentBuilder::new()
            .with_config(db_config)
            .build()?;

        // Mock storage for fast testing
        let storage = StorageComponentBuilder::new().mock().build()?;

        registry.register_component(database)?;
        registry.register_component(storage)?;

        registry.initialize_all().await?;

        tracing::info!("✅ Studio Line: Developer Setup assembled successfully");
        Ok(registry)
    }

    /// Unit testing setup with completely isolated components
    pub async fn unit_test_setup() -> Result<ServiceRegistry, ComponentError> {
        tracing::info!("🧪 Assembling Studio Line: Unit Test Setup");

        let mut registry = ServiceRegistry::new();

        // Isolated test database
        let database = DatabaseComponentBuilder::new().for_testing().build()?;

        // In-memory storage that cleans up automatically
        let storage = StorageComponentBuilder::new().in_memory().build()?;

        registry.register_component(database)?;
        registry.register_component(storage)?;

        registry.initialize_all().await?;

        tracing::info!("✅ Studio Line: Unit Test Setup assembled successfully");
        Ok(registry)
    }
}

/// The "Professional" line - production-ready components with full features
pub struct ProfessionalLine;

impl ProfessionalLine {
    /// Full production setup with all features
    pub async fn production_setup() -> Result<ServiceRegistry, ComponentError> {
        tracing::info!("🏢 Assembling Professional Line: Production Setup");

        let mut registry = ServiceRegistry::new();

        // Production database with connection pooling
        let database = DatabaseComponentBuilder::new().with_env_config()?.build()?;

        // File system storage with configurable path
        let storage = StorageComponentBuilder::new().from_env()?.build()?;

        registry.register_component(database)?;
        registry.register_component(storage)?;

        registry.initialize_all().await?;

        tracing::info!("✅ Professional Line: Production Setup assembled successfully");
        Ok(registry)
    }

    /// High-availability setup with redundancy
    pub async fn high_availability_setup() -> Result<ServiceRegistry, ComponentError> {
        tracing::info!("🔧 Assembling Professional Line: High Availability Setup");

        let mut registry = ServiceRegistry::new();

        // Primary database
        let database = DatabaseComponentBuilder::new().with_env_config()?.build()?;

        // S3 storage for distributed access
        let storage = StorageComponentBuilder::new()
            .s3("tracseq-storage", "us-east-1")
            .build()?;

        registry.register_component(database)?;
        registry.register_component(storage)?;

        registry.initialize_all().await?;

        tracing::info!("✅ Professional Line: High Availability Setup assembled successfully");
        Ok(registry)
    }
}

/// The "Compact" line - minimal resource usage for constrained environments
pub struct CompactLine;

impl CompactLine {
    /// Docker container setup with minimal footprint
    pub async fn container_setup() -> Result<ServiceRegistry, ComponentError> {
        tracing::info!("📦 Assembling Compact Line: Container Setup");

        let mut registry = ServiceRegistry::new();

        // Lightweight database configuration
        let db_config = DatabaseConfig {
            url: std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgresql://postgres:password@localhost/tracseq_compact".to_string()
            }),
            max_connections: 5, // Reduced for containers
            min_connections: 1,
        };

        let database = DatabaseComponentBuilder::new()
            .with_config(db_config)
            .build()?;

        // Compact storage with limited cache
        let storage_config = StorageConfig {
            base_path: std::path::PathBuf::from("/tmp/tracseq"),
            max_file_size: 50 * 1024 * 1024, // 50MB
            allowed_extensions: vec!["xlsx".to_string(), "csv".to_string(), "txt".to_string()],
        };

        let storage = StorageComponentBuilder::new()
            .with_config(storage_config)
            .filesystem("/data/storage")
            .build()?;

        registry.register_component(database)?;
        registry.register_component(storage)?;

        registry.initialize_all().await?;

        tracing::info!("✅ Compact Line: Container Setup assembled successfully");
        Ok(registry)
    }

    /// Edge deployment with minimal dependencies
    pub async fn edge_setup() -> Result<ServiceRegistry, ComponentError> {
        tracing::info!("🌐 Assembling Compact Line: Edge Setup");

        let mut registry = ServiceRegistry::new();

        // SQLite for edge deployment
        let db_config = DatabaseConfig {
            url: "sqlite:///data/tracseq.db".to_string(),
            max_connections: 2,
            min_connections: 1,
        };

        let database = DatabaseComponentBuilder::new()
            .with_config(db_config)
            .build()?;

        // Local storage optimized for edge
        let storage = StorageComponentBuilder::new()
            .filesystem("/data/edge_storage")
            .build()?;

        registry.register_component(database)?;
        registry.register_component(storage)?;

        registry.initialize_all().await?;

        tracing::info!("✅ Compact Line: Edge Setup assembled successfully");
        Ok(registry)
    }
}

/// The "Hybrid" line - mix and match components for custom requirements
pub struct HybridLine;

impl HybridLine {
    /// Custom assembly builder for specific requirements
    pub fn custom() -> HybridAssemblyBuilder {
        HybridAssemblyBuilder::new()
    }

    /// Cloud-native setup with managed services
    pub async fn cloud_native_setup() -> Result<ServiceRegistry, ComponentError> {
        tracing::info!("☁️  Assembling Hybrid Line: Cloud Native Setup");

        let mut registry = ServiceRegistry::new();

        // Managed database (RDS, CloudSQL, etc.)
        let db_config = DatabaseConfig {
            url: std::env::var("MANAGED_DATABASE_URL").unwrap_or_else(|_| {
                std::env::var("DATABASE_URL")
                    .unwrap_or_else(|_| "postgresql://localhost/tracseq".to_string())
            }),
            max_connections: 20,
            min_connections: 2,
        };

        let database = DatabaseComponentBuilder::new()
            .with_config(db_config)
            .build()?;

        // Object storage (S3, GCS, etc.)
        let bucket =
            std::env::var("STORAGE_BUCKET").unwrap_or_else(|_| "tracseq-cloud".to_string());
        let region = std::env::var("STORAGE_REGION").unwrap_or_else(|_| "us-east-1".to_string());

        let storage = StorageComponentBuilder::new().s3(bucket, region).build()?;

        registry.register_component(database)?;
        registry.register_component(storage)?;

        registry.initialize_all().await?;

        tracing::info!("✅ Hybrid Line: Cloud Native Setup assembled successfully");
        Ok(registry)
    }
}

/// Builder for custom hybrid assemblies
pub struct HybridAssemblyBuilder {
    database_builder: Option<DatabaseComponentBuilder>,
    storage_builder: Option<StorageComponentBuilder>,
    custom_config: HashMap<String, String>,
}

impl HybridAssemblyBuilder {
    pub fn new() -> Self {
        Self {
            database_builder: None,
            storage_builder: None,
            custom_config: HashMap::new(),
        }
    }

    /// Add custom database configuration
    pub fn with_database(mut self, builder: DatabaseComponentBuilder) -> Self {
        self.database_builder = Some(builder);
        self
    }

    /// Add custom storage configuration
    pub fn with_storage(mut self, builder: StorageComponentBuilder) -> Self {
        self.storage_builder = Some(builder);
        self
    }

    /// Add custom configuration
    pub fn with_config<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.custom_config.insert(key.into(), value.into());
        self
    }

    /// Build the custom assembly
    pub async fn build(self) -> Result<ServiceRegistry, ComponentError> {
        tracing::info!("🔧 Assembling Hybrid Line: Custom Setup");

        let mut registry = ServiceRegistry::new();

        // Add database if configured
        if let Some(db_builder) = self.database_builder {
            let database = db_builder.build()?;
            registry.register_component(database)?;
        }

        // Add storage if configured
        if let Some(storage_builder) = self.storage_builder {
            let storage = storage_builder.build()?;
            registry.register_component(storage)?;
        }

        // Register custom configuration as a service (simplified for now)
        if !self.custom_config.is_empty() {
            tracing::info!(
                "Custom config available with {} keys",
                self.custom_config.len()
            );
            // TODO: Implement proper config service registration
        }

        registry.initialize_all().await?;

        tracing::info!("✅ Hybrid Line: Custom Setup assembled successfully");
        Ok(registry)
    }
}

impl Default for HybridAssemblyBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Product line catalog - easy access to all available configurations
#[derive(Debug, Clone)]
pub enum ProductLine {
    /// Studio line for development and testing
    Studio(StudioVariant),
    /// Professional line for production use
    Professional(ProfessionalVariant),
    /// Compact line for resource-constrained environments
    Compact(CompactVariant),
    /// Hybrid line for custom requirements
    Hybrid,
}

#[derive(Debug, Clone)]
pub enum StudioVariant {
    Developer,
    UnitTest,
}

#[derive(Debug, Clone)]
pub enum ProfessionalVariant {
    Production,
    HighAvailability,
}

#[derive(Debug, Clone)]
pub enum CompactVariant {
    Container,
    Edge,
}

impl ProductLine {
    /// Get a description of this product line
    pub fn description(&self) -> &'static str {
        match self {
            ProductLine::Studio(StudioVariant::Developer) => {
                "Studio Developer: Minimal setup perfect for local development with in-memory components"
            }
            ProductLine::Studio(StudioVariant::UnitTest) => {
                "Studio Unit Test: Isolated testing environment with automatic cleanup"
            }
            ProductLine::Professional(ProfessionalVariant::Production) => {
                "Professional Production: Full-featured production setup with persistent storage"
            }
            ProductLine::Professional(ProfessionalVariant::HighAvailability) => {
                "Professional HA: High-availability setup with distributed storage and redundancy"
            }
            ProductLine::Compact(CompactVariant::Container) => {
                "Compact Container: Minimal resource usage optimized for Docker deployments"
            }
            ProductLine::Compact(CompactVariant::Edge) => {
                "Compact Edge: Lightweight edge deployment with local storage and SQLite"
            }
            ProductLine::Hybrid => {
                "Hybrid Custom: Mix and match components for specific requirements"
            }
        }
    }

    /// Assemble this product line
    pub async fn assemble(&self) -> Result<ServiceRegistry, ComponentError> {
        match self {
            ProductLine::Studio(StudioVariant::Developer) => StudioLine::developer_setup().await,
            ProductLine::Studio(StudioVariant::UnitTest) => StudioLine::unit_test_setup().await,
            ProductLine::Professional(ProfessionalVariant::Production) => {
                ProfessionalLine::production_setup().await
            }
            ProductLine::Professional(ProfessionalVariant::HighAvailability) => {
                ProfessionalLine::high_availability_setup().await
            }
            ProductLine::Compact(CompactVariant::Container) => CompactLine::container_setup().await,
            ProductLine::Compact(CompactVariant::Edge) => CompactLine::edge_setup().await,
            ProductLine::Hybrid => HybridLine::cloud_native_setup().await, // Default hybrid setup
        }
    }

    /// List all available product lines
    pub fn catalog() -> Vec<ProductLine> {
        vec![
            ProductLine::Studio(StudioVariant::Developer),
            ProductLine::Studio(StudioVariant::UnitTest),
            ProductLine::Professional(ProfessionalVariant::Production),
            ProductLine::Professional(ProfessionalVariant::HighAvailability),
            ProductLine::Compact(CompactVariant::Container),
            ProductLine::Compact(CompactVariant::Edge),
            ProductLine::Hybrid,
        ]
    }
}

/// Convenience macro for quickly assembling product lines
#[macro_export]
macro_rules! assemble {
    (studio::developer) => {
        $crate::assembly::product_lines::StudioLine::developer_setup().await?
    };

    (studio::test) => {
        $crate::assembly::product_lines::StudioLine::unit_test_setup().await?
    };

    (professional::production) => {
        $crate::assembly::product_lines::ProfessionalLine::production_setup().await?
    };

    (professional::ha) => {
        $crate::assembly::product_lines::ProfessionalLine::high_availability_setup().await?
    };

    (compact::container) => {
        $crate::assembly::product_lines::CompactLine::container_setup().await?
    };

    (compact::edge) => {
        $crate::assembly::product_lines::CompactLine::edge_setup().await?
    };

    (hybrid::cloud) => {
        $crate::assembly::product_lines::HybridLine::cloud_native_setup().await?
    };

    (custom) => {
        $crate::assembly::product_lines::HybridLine::custom()
    };
}
