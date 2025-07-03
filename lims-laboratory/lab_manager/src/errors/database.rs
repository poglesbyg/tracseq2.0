use crate::errors::{ComponentError, ErrorSeverity};

/// Database-specific errors
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
    #[error("Query execution failed: {0}")]
    QueryFailed(String),
    #[error("Migration failed: {0}")]
    MigrationFailed(String),
    #[error("Connection pool exhausted")]
    PoolExhausted,
    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),
    #[error("Record not found")]
    RecordNotFound,
}

impl ComponentError for DatabaseError {
    fn error_code(&self) -> &'static str {
        match self {
            Self::ConnectionFailed(_) => "DB_CONNECTION_FAILED",
            Self::TransactionFailed(_) => "DB_TRANSACTION_FAILED",
            Self::QueryFailed(_) => "DB_QUERY_FAILED",
            Self::MigrationFailed(_) => "DB_MIGRATION_FAILED",
            Self::PoolExhausted => "DB_POOL_EXHAUSTED",
            Self::ConstraintViolation(_) => "DB_CONSTRAINT_VIOLATION",
            Self::RecordNotFound => "DB_RECORD_NOT_FOUND",
        }
    }

    fn severity(&self) -> ErrorSeverity {
        match self {
            Self::ConnectionFailed(_) | Self::PoolExhausted => ErrorSeverity::High,
            Self::TransactionFailed(_) | Self::QueryFailed(_) => ErrorSeverity::Medium,
            Self::MigrationFailed(_) => ErrorSeverity::Critical,
            Self::ConstraintViolation(_) => ErrorSeverity::Low,
            Self::RecordNotFound => ErrorSeverity::Low,
        }
    }

    fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::ConnectionFailed(_) | Self::TransactionFailed(_) | Self::PoolExhausted
        )
    }
}
