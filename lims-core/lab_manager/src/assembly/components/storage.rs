use async_trait::async_trait;
use std::any::Any;
use std::path::PathBuf;
use std::sync::Arc;

use super::super::traits::{
    Component, ComponentError, Configurable, ServiceProvider, ServiceRegistry,
};
use crate::config::StorageConfig;
// Storage component implementation

/// Basic Storage implementation for file operations
#[derive(Debug)]
pub struct Storage {
    base_path: PathBuf,
}

impl Storage {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    pub fn base_path(&self) -> &PathBuf {
        &self.base_path
    }

    // Add basic storage operations here as needed
    pub async fn store_file(&self, filename: &str, content: &[u8]) -> Result<(), std::io::Error> {
        let file_path = self.base_path.join(filename);
        tokio::fs::write(file_path, content).await
    }

    pub async fn read_file(&self, filename: &str) -> Result<Vec<u8>, std::io::Error> {
        let file_path = self.base_path.join(filename);
        tokio::fs::read(file_path).await
    }

    pub async fn delete_file(&self, filename: &str) -> Result<(), std::io::Error> {
        let file_path = self.base_path.join(filename);
        tokio::fs::remove_file(file_path).await
    }
}

/// Modular storage component with configurable backends
pub struct StorageComponent {
    config: StorageConfig,
    storage: Option<Arc<Storage>>,
    backend_type: StorageBackend,
    is_initialized: bool,
}

#[derive(Debug, Clone)]
pub enum StorageBackend {
    /// Local filesystem storage
    FileSystem { base_path: PathBuf },
    /// In-memory storage (for testing)
    InMemory,
    /// S3-compatible storage
    S3 { bucket: String, region: String },
    /// Mock storage (for testing/development)
    Mock,
}

impl StorageComponent {
    pub fn new(config: StorageConfig, backend: StorageBackend) -> Self {
        Self {
            config,
            storage: None,
            backend_type: backend,
            is_initialized: false,
        }
    }

    /// Get the storage instance
    pub fn storage(&self) -> Option<Arc<Storage>> {
        self.storage.clone()
    }

    /// Get the backend type
    pub fn backend_type(&self) -> &StorageBackend {
        &self.backend_type
    }
}

#[async_trait]
impl Component for StorageComponent {
    fn component_id(&self) -> &'static str {
        "storage"
    }

    fn component_name(&self) -> &'static str {
        match self.backend_type {
            StorageBackend::FileSystem { .. } => "File System Storage",
            StorageBackend::InMemory => "In-Memory Storage",
            StorageBackend::S3 { .. } => "S3 Storage",
            StorageBackend::Mock => "Mock Storage",
        }
    }

    async fn initialize(&mut self, _context: &ServiceRegistry) -> Result<(), ComponentError> {
        if self.is_initialized {
            return Ok(());
        }

        tracing::info!(
            "Initializing storage component with backend: {:?}",
            self.backend_type
        );

        match &self.backend_type {
            StorageBackend::FileSystem { base_path } => {
                // Ensure storage directory exists
                tokio::fs::create_dir_all(base_path).await.map_err(|e| {
                    ComponentError::InitializationFailed(format!(
                        "Failed to create storage directory: {}",
                        e
                    ))
                })?;

                let storage = Arc::new(Storage::new(base_path.clone()));
                self.storage = Some(storage);
            }
            StorageBackend::InMemory => {
                // Create in-memory storage using temp directory
                let temp_dir = std::env::temp_dir().join("tracseq_memory");
                tokio::fs::create_dir_all(&temp_dir).await.map_err(|e| {
                    ComponentError::InitializationFailed(format!(
                        "Failed to create temp directory: {}",
                        e
                    ))
                })?;

                let storage = Arc::new(Storage::new(temp_dir));
                self.storage = Some(storage);
            }
            StorageBackend::S3 { bucket, region: _ } => {
                // TODO: Implement S3 storage initialization
                tracing::warn!(
                    "S3 storage backend not yet implemented, using file system fallback"
                );
                let fallback_path = std::env::temp_dir().join(format!("s3_fallback_{}", bucket));
                tokio::fs::create_dir_all(&fallback_path)
                    .await
                    .map_err(|e| {
                        ComponentError::InitializationFailed(format!(
                            "Failed to create S3 fallback directory: {}",
                            e
                        ))
                    })?;

                let storage = Arc::new(Storage::new(fallback_path));
                self.storage = Some(storage);
            }
            StorageBackend::Mock => {
                // Create mock storage for testing
                let mock_path = std::env::temp_dir().join("tracseq_mock");
                tokio::fs::create_dir_all(&mock_path).await.map_err(|e| {
                    ComponentError::InitializationFailed(format!(
                        "Failed to create mock directory: {}",
                        e
                    ))
                })?;

                let storage = Arc::new(Storage::new(mock_path));
                self.storage = Some(storage);
            }
        }

        self.is_initialized = true;
        tracing::info!("Storage component initialized successfully");
        Ok(())
    }

    async fn health_check(&self) -> Result<(), ComponentError> {
        if !self.is_initialized {
            return Err(ComponentError::InitializationFailed(
                "Component not initialized".to_string(),
            ));
        }

        if let Some(_storage) = &self.storage {
            // Perform basic health check - ensure we can write/read a test file
            let test_path = "health_check.tmp";
            let test_content = b"health_check";

            // This is a simplified health check - in reality, Storage would need health check methods
            match &self.backend_type {
                StorageBackend::FileSystem { base_path } => {
                    let full_path = base_path.join(test_path);
                    tokio::fs::write(&full_path, test_content)
                        .await
                        .map_err(|e| {
                            ComponentError::ServiceUnavailable(format!(
                                "Storage write failed: {}",
                                e
                            ))
                        })?;

                    let _ = tokio::fs::read(&full_path).await.map_err(|e| {
                        ComponentError::ServiceUnavailable(format!("Storage read failed: {}", e))
                    })?;

                    let _ = tokio::fs::remove_file(full_path).await; // Cleanup, ignore errors
                }
                _ => {
                    // For other backends, assume healthy if initialized
                }
            }
        }

        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), ComponentError> {
        if let Some(_storage) = &self.storage {
            tracing::info!("Shutting down storage component");

            // Perform any necessary cleanup based on backend type
            match &self.backend_type {
                StorageBackend::InMemory | StorageBackend::Mock => {
                    // Clean up temporary directories for in-memory/mock storage
                    // This is optional - we could leave them for debugging
                }
                _ => {
                    // No special cleanup needed for file system or S3
                }
            }
        }

        self.storage = None;
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
impl ServiceProvider for StorageComponent {
    fn provided_services(&self) -> Vec<&'static str> {
        vec!["storage", "file_storage", "document_storage"]
    }
}

impl Configurable for StorageComponent {
    type Config = StorageConfig;

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

/// Builder for creating storage components with different backends
pub struct StorageComponentBuilder {
    config: Option<StorageConfig>,
    backend: Option<StorageBackend>,
}

impl StorageComponentBuilder {
    pub fn new() -> Self {
        Self {
            config: None,
            backend: None,
        }
    }

    /// Configure with storage config
    pub fn with_config(mut self, config: StorageConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Use file system backend
    pub fn filesystem<P: Into<PathBuf>>(mut self, base_path: P) -> Self {
        self.backend = Some(StorageBackend::FileSystem {
            base_path: base_path.into(),
        });
        self
    }

    /// Use in-memory backend (for testing)
    pub fn in_memory(mut self) -> Self {
        self.backend = Some(StorageBackend::InMemory);
        self
    }

    /// Use S3 backend
    pub fn s3<B: Into<String>, R: Into<String>>(mut self, bucket: B, region: R) -> Self {
        self.backend = Some(StorageBackend::S3 {
            bucket: bucket.into(),
            region: region.into(),
        });
        self
    }

    /// Use mock backend (for testing)
    pub fn mock(mut self) -> Self {
        self.backend = Some(StorageBackend::Mock);
        self
    }

    /// Configure from environment
    pub fn from_env(mut self) -> Result<Self, ComponentError> {
        let config = StorageConfig::from_env().map_err(|e| {
            ComponentError::ConfigurationError(format!("Failed to load storage config: {}", e))
        })?;

        // Default to filesystem backend with config path
        self.config = Some(config.clone());
        self.backend = Some(StorageBackend::FileSystem {
            base_path: config.base_path,
        });

        Ok(self)
    }

    /// Build the component
    pub fn build(self) -> Result<StorageComponent, ComponentError> {
        let config = self.config.unwrap_or_else(|| StorageConfig::default());

        let backend = self.backend.unwrap_or_else(|| StorageBackend::FileSystem {
            base_path: config.base_path.clone(),
        });

        Ok(StorageComponent::new(config, backend))
    }
}

impl Default for StorageComponentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience macro for creating storage components
#[macro_export]
macro_rules! storage_component {
    // File system storage
    (fs $path:expr) => {
        $crate::assembly::components::storage::StorageComponentBuilder::new()
            .filesystem($path)
            .build()?
    };

    // In-memory storage
    (memory) => {
        $crate::assembly::components::storage::StorageComponentBuilder::new()
            .in_memory()
            .build()?
    };

    // Mock storage
    (mock) => {
        $crate::assembly::components::storage::StorageComponentBuilder::new()
            .mock()
            .build()?
    };

    // S3 storage
    (s3 $bucket:expr, $region:expr) => {
        $crate::assembly::components::storage::StorageComponentBuilder::new()
            .s3($bucket, $region)
            .build()?
    };

    // From environment
    (env) => {
        $crate::assembly::components::storage::StorageComponentBuilder::new()
            .from_env()?
            .build()?
    };
}
