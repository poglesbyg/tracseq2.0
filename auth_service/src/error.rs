use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// Main error type for the authentication service
#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("Password hashing error: {0}")]
    PasswordHash(#[from] argon2::password_hash::Error),

    #[error("Configuration error: {0}")]
    Config(#[from] anyhow::Error),

    #[error("Validation error: {message}")]
    Validation { message: String },

    #[error("Authentication failed: {message}")]
    Authentication { message: String },

    #[error("Authorization failed: {message}")]
    Authorization { message: String },

    #[error("User not found")]
    UserNotFound,

    #[error("User already exists")]
    UserAlreadyExists,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Account locked: {message}")]
    AccountLocked { message: String },

    #[error("Account not verified")]
    AccountNotVerified,

    #[error("Account disabled")]
    AccountDisabled,

    #[error("Session not found")]
    SessionNotFound,

    #[error("Session expired")]
    SessionExpired,

    #[error("Token invalid")]
    TokenInvalid,

    #[error("Token expired")]
    TokenExpired,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Feature disabled: {feature}")]
    FeatureDisabled { feature: String },

    #[error("External service error: {service}")]
    ExternalService { service: String },

    #[error("Email sending failed: {0}")]
    EmailSending(String),

    #[error("Shibboleth error: {message}")]
    Shibboleth { message: String },

    #[error("Internal server error")]
    Internal,

    #[error("Bad request: {message}")]
    BadRequest { message: String },

    #[error("Not found: {resource}")]
    NotFound { resource: String },

    #[error("Conflict: {message}")]
    Conflict { message: String },

    #[error("Too many requests")]
    TooManyRequests,
}

impl AuthError {
    /// Create a validation error
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

    /// Create an authentication error
    pub fn authentication(message: impl Into<String>) -> Self {
        Self::Authentication {
            message: message.into(),
        }
    }

    /// Create an authorization error
    pub fn authorization(message: impl Into<String>) -> Self {
        Self::Authorization {
            message: message.into(),
        }
    }

    /// Create an account locked error
    pub fn account_locked(message: impl Into<String>) -> Self {
        Self::AccountLocked {
            message: message.into(),
        }
    }

    /// Create a feature disabled error
    pub fn feature_disabled(feature: impl Into<String>) -> Self {
        Self::FeatureDisabled {
            feature: feature.into(),
        }
    }

    /// Create an external service error
    pub fn external_service(service: impl Into<String>) -> Self {
        Self::ExternalService {
            service: service.into(),
        }
    }

    /// Create a Shibboleth error
    pub fn shibboleth(message: impl Into<String>) -> Self {
        Self::Shibboleth {
            message: message.into(),
        }
    }

    /// Create a bad request error
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest {
            message: message.into(),
        }
    }

    /// Create a not found error
    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::NotFound {
            resource: resource.into(),
        }
    }

    /// Create a conflict error
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict {
            message: message.into(),
        }
    }

    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Jwt(_) => StatusCode::UNAUTHORIZED,
            Self::PasswordHash(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Config(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Validation { .. } => StatusCode::BAD_REQUEST,
            Self::Authentication { .. } => StatusCode::UNAUTHORIZED,
            Self::Authorization { .. } => StatusCode::FORBIDDEN,
            Self::UserNotFound => StatusCode::NOT_FOUND,
            Self::UserAlreadyExists => StatusCode::CONFLICT,
            Self::InvalidCredentials => StatusCode::UNAUTHORIZED,
            Self::AccountLocked { .. } => StatusCode::FORBIDDEN,
            Self::AccountNotVerified => StatusCode::FORBIDDEN,
            Self::AccountDisabled => StatusCode::FORBIDDEN,
            Self::SessionNotFound => StatusCode::NOT_FOUND,
            Self::SessionExpired => StatusCode::UNAUTHORIZED,
            Self::TokenInvalid => StatusCode::UNAUTHORIZED,
            Self::TokenExpired => StatusCode::UNAUTHORIZED,
            Self::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
            Self::FeatureDisabled { .. } => StatusCode::NOT_IMPLEMENTED,
            Self::ExternalService { .. } => StatusCode::BAD_GATEWAY,
            Self::EmailSending(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Shibboleth { .. } => StatusCode::UNAUTHORIZED,
            Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            Self::BadRequest { .. } => StatusCode::BAD_REQUEST,
            Self::NotFound { .. } => StatusCode::NOT_FOUND,
            Self::Conflict { .. } => StatusCode::CONFLICT,
            Self::TooManyRequests => StatusCode::TOO_MANY_REQUESTS,
        }
    }

    /// Get the error code for this error
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::Database(_) => "DATABASE_ERROR",
            Self::Jwt(_) => "JWT_ERROR",
            Self::PasswordHash(_) => "PASSWORD_HASH_ERROR",
            Self::Config(_) => "CONFIG_ERROR",
            Self::Validation { .. } => "VALIDATION_ERROR",
            Self::Authentication { .. } => "AUTHENTICATION_FAILED",
            Self::Authorization { .. } => "AUTHORIZATION_FAILED",
            Self::UserNotFound => "USER_NOT_FOUND",
            Self::UserAlreadyExists => "USER_ALREADY_EXISTS",
            Self::InvalidCredentials => "INVALID_CREDENTIALS",
            Self::AccountLocked { .. } => "ACCOUNT_LOCKED",
            Self::AccountNotVerified => "ACCOUNT_NOT_VERIFIED",
            Self::AccountDisabled => "ACCOUNT_DISABLED",
            Self::SessionNotFound => "SESSION_NOT_FOUND",
            Self::SessionExpired => "SESSION_EXPIRED",
            Self::TokenInvalid => "TOKEN_INVALID",
            Self::TokenExpired => "TOKEN_EXPIRED",
            Self::RateLimitExceeded => "RATE_LIMIT_EXCEEDED",
            Self::FeatureDisabled { .. } => "FEATURE_DISABLED",
            Self::ExternalService { .. } => "EXTERNAL_SERVICE_ERROR",
            Self::EmailSending(_) => "EMAIL_SENDING_FAILED",
            Self::Shibboleth { .. } => "SHIBBOLETH_ERROR",
            Self::Internal => "INTERNAL_SERVER_ERROR",
            Self::BadRequest { .. } => "BAD_REQUEST",
            Self::NotFound { .. } => "NOT_FOUND",
            Self::Conflict { .. } => "CONFLICT",
            Self::TooManyRequests => "TOO_MANY_REQUESTS",
        }
    }

    /// Check if this error should be logged
    pub fn should_log(&self) -> bool {
        matches!(
            self,
            Self::Database(_)
                | Self::Config(_)
                | Self::PasswordHash(_)
                | Self::EmailSending(_)
                | Self::ExternalService { .. }
                | Self::Internal
        )
    }

    /// Check if this error contains sensitive information
    pub fn is_sensitive(&self) -> bool {
        matches!(
            self,
            Self::Database(_) | Self::Config(_) | Self::PasswordHash(_) | Self::Internal
        )
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let status_code = self.status_code();
        let error_code = self.error_code();

        // Log sensitive errors but don't expose details
        if self.should_log() {
            tracing::error!("Authentication service error: {}", self);
        }

        // Create appropriate error message for client
        let client_message = if self.is_sensitive() {
            // Don't expose sensitive error details to client
            match self {
                Self::Database(_) => "A database error occurred".to_string(),
                Self::Config(_) => "A configuration error occurred".to_string(),
                Self::PasswordHash(_) => "A password processing error occurred".to_string(),
                Self::Internal => "An internal server error occurred".to_string(),
                _ => self.to_string(),
            }
        } else {
            self.to_string()
        };

        let error_response = json!({
            "error": {
                "code": error_code,
                "message": client_message,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }
        });

        (status_code, Json(error_response)).into_response()
    }
}

/// Result type alias for authentication operations
pub type AuthResult<T> = Result<T, AuthError>;

/// Validation error details
#[derive(Debug, Clone, serde::Serialize)]
pub struct ValidationError {
    pub field: String,
    pub code: String,
    pub message: String,
}

impl ValidationError {
    pub fn new(
        field: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            field: field.into(),
            code: code.into(),
            message: message.into(),
        }
    }
}

/// Multiple validation errors
#[derive(Debug, Clone, serde::Serialize)]
pub struct ValidationErrors {
    pub errors: Vec<ValidationError>,
}

impl ValidationErrors {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn add(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    pub fn add_field_error(
        &mut self,
        field: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
    ) {
        self.errors.push(ValidationError::new(field, code, message));
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn into_auth_error(self) -> AuthError {
        let messages: Vec<String> = self.errors.into_iter().map(|e| e.message).collect();
        AuthError::validation(messages.join(", "))
    }
}

impl Default for ValidationErrors {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoResponse for ValidationErrors {
    fn into_response(self) -> Response {
        let error_response = json!({
            "error": {
                "code": "VALIDATION_ERROR",
                "message": "Validation failed",
                "details": self.errors,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }
        });

        (StatusCode::BAD_REQUEST, Json(error_response)).into_response()
    }
}

/// Convert validator errors to our validation errors
impl From<validator::ValidationErrors> for ValidationErrors {
    fn from(errors: validator::ValidationErrors) -> Self {
        let mut validation_errors = ValidationErrors::new();

        for (field, field_errors) in errors.field_errors() {
            for error in field_errors {
                let message = error
                    .message
                    .as_ref()
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| format!("Invalid value for field '{}'", field));

                validation_errors.add_field_error(field, error.code.as_ref(), message);
            }
        }

        validation_errors
    }
}
