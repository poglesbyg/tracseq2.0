use crate::errors::{ComponentError, ErrorSeverity};
use std::collections::HashMap;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Requested file was not found")]
    FileNotFound,
    #[error("Insufficient storage space available")]
    InsufficientSpace,
    #[error("Permission denied for storage operation")]
    PermissionDenied,
    #[error("File exceeds maximum allowed size")]
    FileTooLarge,
    #[error("File type is not allowed")]
    InvalidFileType,
    #[error("File appears to be corrupted")]
    CorruptedFile,
    #[error("Storage system is currently unavailable")]
    StorageUnavailable,
    #[error("Attempted path traversal attack detected")]
    PathTraversalAttempt,
}

impl ComponentError for StorageError {
    fn error_code(&self) -> &'static str {
        match self {
            Self::FileNotFound => "STORAGE_FILE_NOT_FOUND",
            Self::InsufficientSpace => "STORAGE_INSUFFICIENT_SPACE",
            Self::PermissionDenied => "STORAGE_PERMISSION_DENIED",
            Self::FileTooLarge => "STORAGE_FILE_TOO_LARGE",
            Self::InvalidFileType => "STORAGE_INVALID_FILE_TYPE",
            Self::CorruptedFile => "STORAGE_CORRUPTED_FILE",
            Self::StorageUnavailable => "STORAGE_UNAVAILABLE",
            Self::PathTraversalAttempt => "STORAGE_PATH_TRAVERSAL",
        }
    }

    fn severity(&self) -> ErrorSeverity {
        match self {
            Self::FileNotFound | Self::CorruptedFile => ErrorSeverity::Medium,
            Self::FileTooLarge | Self::InvalidFileType => ErrorSeverity::Low,
            Self::InsufficientSpace | Self::PermissionDenied => ErrorSeverity::High,
            Self::StorageUnavailable | Self::PathTraversalAttempt => ErrorSeverity::Critical,
        }
    }

    fn is_retryable(&self) -> bool {
        matches!(self, Self::InsufficientSpace | Self::StorageUnavailable)
    }
}

impl StorageError {
    /// Create a storage error with additional context
    pub fn with_context(self, key: &str, value: String) -> StorageErrorWithContext {
        let mut context = HashMap::new();
        context.insert(key.to_string(), value);
        StorageErrorWithContext {
            error: self,
            context,
        }
    }
}

/// Storage error with additional context
#[derive(Debug)]
pub struct StorageErrorWithContext {
    error: StorageError,
    context: HashMap<String, String>,
}

impl std::fmt::Display for StorageErrorWithContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}

impl std::error::Error for StorageErrorWithContext {}

impl ComponentError for StorageErrorWithContext {
    fn error_code(&self) -> &'static str {
        self.error.error_code()
    }

    fn severity(&self) -> ErrorSeverity {
        self.error.severity()
    }

    fn context(&self) -> HashMap<String, String> {
        self.context.clone()
    }

    fn is_retryable(&self) -> bool {
        self.error.is_retryable()
    }
}
