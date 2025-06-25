use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::errors::storage::StorageError;
use crate::services::{HealthCheck, HealthStatus, Service, ServiceConfig, ServiceHealth};

/// Modular storage service trait
#[async_trait]
pub trait StorageService: Send + Sync {
    async fn save_file(
        &self,
        filename: &str,
        content: &[u8],
    ) -> Result<std::path::PathBuf, Box<dyn std::error::Error>>;
    async fn get_file(&self, path: &std::path::Path)
        -> Result<Vec<u8>, Box<dyn std::error::Error>>;

    /// Delete a file
    async fn delete_file(&self, path: &Path) -> Result<(), StorageError>;

    /// List files in a directory
    async fn list_files(&self, directory: &Path) -> Result<Vec<FileInfo>, StorageError>;

    /// Get file metadata
    async fn get_file_info(&self, path: &Path) -> Result<FileInfo, StorageError>;

    /// Check if file exists
    async fn file_exists(&self, path: &Path) -> bool;

    /// Get storage statistics
    async fn get_storage_stats(&self) -> Result<StorageStats, StorageError>;
}

/// File information structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileInfo {
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub modified_at: chrono::DateTime<chrono::Utc>,
    pub file_type: String,
    pub checksum: Option<String>,
}

/// Storage statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StorageStats {
    pub total_space: u64,
    pub used_space: u64,
    pub available_space: u64,
    pub file_count: u64,
    pub directory_count: u64,
}

/// Local filesystem storage implementation
#[derive(Debug)]
pub struct LocalStorageService {
    base_path: PathBuf,
    max_file_size: u64,
    allowed_extensions: Vec<String>,
}

impl LocalStorageService {
    pub fn new(base_path: PathBuf, max_file_size: u64, allowed_extensions: Vec<String>) -> Self {
        Self {
            base_path,
            max_file_size,
            allowed_extensions,
        }
    }

    fn validate_file(&self, filename: &str, content: &[u8]) -> Result<(), StorageError> {
        // Check file size
        if content.len() as u64 > self.max_file_size {
            return Err(StorageError::FileTooLarge);
        }

        // Check file extension
        if let Some(extension) = Path::new(filename).extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            if !self.allowed_extensions.contains(&ext) {
                return Err(StorageError::InvalidFileType);
            }
        } else if !self.allowed_extensions.is_empty() {
            return Err(StorageError::InvalidFileType);
        }

        // Check for path traversal attempts
        if filename.contains("..") || filename.contains("/") || filename.contains("\\") {
            return Err(StorageError::PathTraversalAttempt);
        }

        Ok(())
    }

    async fn ensure_directory_exists(&self, path: &Path) -> Result<(), StorageError> {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|_| StorageError::PermissionDenied)?;
        }
        Ok(())
    }
}

#[async_trait]
impl Service for LocalStorageService {
    fn name(&self) -> &'static str {
        "LocalStorageService"
    }

    async fn health_check(&self) -> ServiceHealth {
        let mut checks = HashMap::new();

        // Check if base directory exists and is writable
        let start = std::time::Instant::now();
        let directory_check = match tokio::fs::metadata(&self.base_path).await {
            Ok(metadata) if metadata.is_dir() => HealthCheck {
                status: HealthStatus::Healthy,
                duration_ms: start.elapsed().as_millis() as u64,
                details: Some("Base directory is accessible".to_string()),
            },
            Ok(_) => HealthCheck {
                status: HealthStatus::Unhealthy,
                duration_ms: start.elapsed().as_millis() as u64,
                details: Some("Base path is not a directory".to_string()),
            },
            Err(e) => HealthCheck {
                status: HealthStatus::Unhealthy,
                duration_ms: start.elapsed().as_millis() as u64,
                details: Some(format!("Cannot access base directory: {}", e)),
            },
        };
        checks.insert("directory_access".to_string(), directory_check);

        // Check available space
        let start = std::time::Instant::now();
        let space_check = match self.get_storage_stats().await {
            Ok(stats) if stats.available_space > 1024 * 1024 * 100 => HealthCheck {
                // 100MB threshold
                status: HealthStatus::Healthy,
                duration_ms: start.elapsed().as_millis() as u64,
                details: Some(format!("Available space: {} bytes", stats.available_space)),
            },
            Ok(stats) => HealthCheck {
                status: HealthStatus::Degraded,
                duration_ms: start.elapsed().as_millis() as u64,
                details: Some(format!("Low disk space: {} bytes", stats.available_space)),
            },
            Err(e) => HealthCheck {
                status: HealthStatus::Unhealthy,
                duration_ms: start.elapsed().as_millis() as u64,
                details: Some(format!("Cannot check disk space: {}", e)),
            },
        };
        checks.insert("disk_space".to_string(), space_check);

        let overall_status = if checks
            .values()
            .any(|c| matches!(c.status, HealthStatus::Unhealthy))
        {
            HealthStatus::Unhealthy
        } else if checks
            .values()
            .any(|c| matches!(c.status, HealthStatus::Degraded))
        {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };

        ServiceHealth {
            status: overall_status,
            message: Some("Local storage service health check".to_string()),
            checks,
        }
    }

    fn config(&self) -> ServiceConfig {
        let mut settings = HashMap::new();
        settings.insert(
            "base_path".to_string(),
            self.base_path.to_string_lossy().to_string(),
        );
        settings.insert("max_file_size".to_string(), self.max_file_size.to_string());
        settings.insert(
            "allowed_extensions".to_string(),
            self.allowed_extensions.join(","),
        );

        ServiceConfig {
            name: "LocalStorageService".to_string(),
            version: "1.0.0".to_string(),
            dependencies: vec!["filesystem".to_string()],
            settings,
        }
    }
}

#[async_trait]
impl StorageService for LocalStorageService {
    async fn save_file(
        &self,
        filename: &str,
        content: &[u8],
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        self.validate_file(filename, content)?;

        let file_id = Uuid::new_v4();
        let safe_filename = format!("{}_{}", file_id, filename);
        let file_path = self.base_path.join(&safe_filename);

        self.ensure_directory_exists(&file_path).await?;

        tokio::fs::write(&file_path, content)
            .await
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::PermissionDenied => StorageError::PermissionDenied,
                std::io::ErrorKind::OutOfMemory => StorageError::InsufficientSpace,
                _ => StorageError::StorageUnavailable,
            })?;

        Ok(file_path)
    }

    async fn get_file(&self, path: &Path) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        tokio::fs::read(path).await.map_err(|e| {
            let storage_error = match e.kind() {
                std::io::ErrorKind::NotFound => StorageError::FileNotFound,
                std::io::ErrorKind::PermissionDenied => StorageError::PermissionDenied,
                _ => StorageError::StorageUnavailable,
            };
            Box::new(storage_error) as Box<dyn std::error::Error>
        })
    }

    async fn delete_file(&self, path: &Path) -> Result<(), StorageError> {
        tokio::fs::remove_file(path)
            .await
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => StorageError::FileNotFound,
                std::io::ErrorKind::PermissionDenied => StorageError::PermissionDenied,
                _ => StorageError::StorageUnavailable,
            })
    }

    async fn list_files(&self, directory: &Path) -> Result<Vec<FileInfo>, StorageError> {
        let mut files = Vec::new();
        let mut entries = tokio::fs::read_dir(directory)
            .await
            .map_err(|_| StorageError::FileNotFound)?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|_| StorageError::StorageUnavailable)?
        {
            if let Ok(file_info) = self.get_file_info(&entry.path()).await {
                files.push(file_info);
            }
        }

        Ok(files)
    }

    async fn get_file_info(&self, path: &Path) -> Result<FileInfo, StorageError> {
        let metadata = tokio::fs::metadata(path)
            .await
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => StorageError::FileNotFound,
                _ => StorageError::StorageUnavailable,
            })?;

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let file_type = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(FileInfo {
            path: path.to_path_buf(),
            name,
            size: metadata.len(),
            created_at: metadata
                .created()
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                .into(),
            modified_at: metadata
                .modified()
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                .into(),
            file_type,
            checksum: None, // Could implement SHA256 checksum here
        })
    }

    async fn file_exists(&self, path: &Path) -> bool {
        tokio::fs::metadata(path).await.is_ok()
    }

    async fn get_storage_stats(&self) -> Result<StorageStats, StorageError> {
        // Simplified implementation - in production you'd use proper disk space APIs
        let mut file_count = 0;
        let mut directory_count = 0;
        let mut used_space = 0;

        fn count_files_recursive(
            path: &Path,
            file_count: &mut u64,
            directory_count: &mut u64,
            used_space: &mut u64,
        ) -> Result<(), std::io::Error> {
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                let metadata = entry.metadata()?;

                if metadata.is_dir() {
                    *directory_count += 1;
                    count_files_recursive(&entry.path(), file_count, directory_count, used_space)?;
                } else {
                    *file_count += 1;
                    *used_space += metadata.len();
                }
            }
            Ok(())
        }

        count_files_recursive(
            &self.base_path,
            &mut file_count,
            &mut directory_count,
            &mut used_space,
        )
        .map_err(|_| StorageError::StorageUnavailable)?;

        // Simplified - in production use proper filesystem APIs
        let total_space = 1024 * 1024 * 1024 * 100; // 100GB placeholder
        let available_space = total_space - used_space;

        Ok(StorageStats {
            total_space,
            used_space,
            available_space,
            file_count,
            directory_count,
        })
    }
}
