use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;
use tracing::{info, warn};
use uuid::Uuid;
use validator::{Validate, ValidationError, ValidationErrors};

use crate::assembly::AppComponents;
use crate::errors::{ErrorResponse, ErrorSeverity};

// Compiled regexes using OnceLock for thread-safe lazy initialization
static EMAIL_REGEX: OnceLock<Regex> = OnceLock::new();
static BARCODE_REGEX: OnceLock<Regex> = OnceLock::new();
static SAFE_PATH_REGEX: OnceLock<Regex> = OnceLock::new();
static IPV4_REGEX: OnceLock<Regex> = OnceLock::new();
static IPV6_REGEX: OnceLock<Regex> = OnceLock::new();

/// Initialize all validation regexes at startup
pub fn initialize_validation_regexes() -> Result<(), String> {
    EMAIL_REGEX
        .set(
            Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
                .map_err(|e| format!("Failed to compile email regex: {}", e))?,
        )
        .map_err(|_| "Failed to set email regex")?;

    BARCODE_REGEX
        .set(
            Regex::new(r"^[A-Z]{3,5}-\d{14}-\d{3}$")
                .map_err(|e| format!("Failed to compile barcode regex: {}", e))?,
        )
        .map_err(|_| "Failed to set barcode regex")?;

    SAFE_PATH_REGEX
        .set(
            Regex::new(r"^[a-zA-Z0-9._/-]+$")
                .map_err(|e| format!("Failed to compile safe path regex: {}", e))?,
        )
        .map_err(|_| "Failed to set safe path regex")?;

    IPV4_REGEX
        .set(
            Regex::new(r"^(?:[0-9]{1,3}\.){3}[0-9]{1,3}$")
                .map_err(|e| format!("Failed to compile IPv4 regex: {}", e))?,
        )
        .map_err(|_| "Failed to set IPv4 regex")?;

    IPV6_REGEX
        .set(
            Regex::new(r"^(?:[0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}$")
                .map_err(|e| format!("Failed to compile IPv6 regex: {}", e))?,
        )
        .map_err(|_| "Failed to set IPv6 regex")?;

    Ok(())
}

/// Get email regex, initializing if necessary
fn get_email_regex() -> Result<&'static Regex, String> {
    EMAIL_REGEX.get().ok_or_else(|| {
        "Email regex not initialized. Call initialize_validation_regexes() first.".to_string()
    })
}

/// Get barcode regex, initializing if necessary
fn get_barcode_regex() -> Result<&'static Regex, String> {
    BARCODE_REGEX.get().ok_or_else(|| {
        "Barcode regex not initialized. Call initialize_validation_regexes() first.".to_string()
    })
}

/// Get safe path regex, initializing if necessary
fn get_safe_path_regex() -> Result<&'static Regex, String> {
    SAFE_PATH_REGEX.get().ok_or_else(|| {
        "Safe path regex not initialized. Call initialize_validation_regexes() first.".to_string()
    })
}

/// Get IPv4 regex, initializing if necessary
fn get_ipv4_regex() -> Result<&'static Regex, String> {
    IPV4_REGEX.get().ok_or_else(|| {
        "IPv4 regex not initialized. Call initialize_validation_regexes() first.".to_string()
    })
}

/// Get IPv6 regex, initializing if necessary
fn get_ipv6_regex() -> Result<&'static Regex, String> {
    IPV6_REGEX.get().ok_or_else(|| {
        "IPv6 regex not initialized. Call initialize_validation_regexes() first.".to_string()
    })
}

/// Input validation middleware for API endpoints
pub async fn validate_input_middleware(
    State(_app): State<AppComponents>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = request.uri().path().to_string();
    let method = request.method().as_str().to_string();

    // Log request for security monitoring
    info!("Processing request: {} {}", method, path);

    // Validate request headers
    if let Err(response) = validate_headers(&headers).await {
        warn!("Header validation failed for {} {}", method, path);
        return Ok(response);
    }

    // Validate request size
    if let Some(content_length) = headers.get("content-length") {
        if let Ok(length_str) = content_length.to_str() {
            if let Ok(length) = length_str.parse::<u64>() {
                if length > MAX_REQUEST_SIZE {
                    warn!(
                        "Request too large: {} bytes for {} {}",
                        length, method, path
                    );
                    return Ok(create_validation_error_response(
                        ValidationErrorType::RequestTooLarge,
                        format!(
                            "Request size {} exceeds maximum allowed size {}",
                            length, MAX_REQUEST_SIZE
                        ),
                    ));
                }
            }
        }
    }

    // Validate content type for POST/PUT requests
    if matches!(method.as_str(), "POST" | "PUT" | "PATCH") {
        if let Err(response) = validate_content_type(&headers).await {
            warn!("Content-Type validation failed for {} {}", method, path);
            return Ok(response);
        }
    }

    // Validate URL path parameters
    if let Err(response) = validate_path_parameters(&path).await {
        warn!("Path parameter validation failed for {}", path);
        return Ok(response);
    }

    // Continue to next middleware/handler
    let response = next.run(request).await;

    // Log response status for monitoring
    info!(
        "Request {} {} completed with status: {}",
        method,
        path,
        response.status()
    );

    Ok(response)
}

/// Maximum request size (10MB)
const MAX_REQUEST_SIZE: u64 = 10 * 1024 * 1024;

/// Allowed content types
const ALLOWED_CONTENT_TYPES: &[&str] = &[
    "application/json",
    "multipart/form-data",
    "application/x-www-form-urlencoded",
    "text/plain",
];

/// Validation error types
#[derive(Debug, Clone, Serialize)]
pub enum ValidationErrorType {
    InvalidHeader,
    InvalidContentType,
    RequestTooLarge,
    InvalidPathParameter,
    InvalidQueryParameter,
    MalformedInput,
    SecurityViolation,
}

/// Input sanitization utilities
pub struct InputSanitizer;

impl InputSanitizer {
    /// Sanitize string input to prevent XSS and injection attacks
    pub fn sanitize_string(input: &str) -> String {
        // Remove potentially dangerous characters
        let dangerous_chars = ['<', '>', '"', '\'', '&', ';', '(', ')', '{', '}', '[', ']'];
        let mut sanitized = input.to_string();

        for char in dangerous_chars {
            sanitized = sanitized.replace(char, "");
        }

        // Limit length to prevent DoS
        if sanitized.len() > 1000 {
            sanitized.truncate(1000);
        }

        sanitized.trim().to_string()
    }

    /// Validate and sanitize email addresses
    pub fn validate_email(email: &str) -> Result<String, ValidationError> {
        let email_regex = match get_email_regex() {
            Ok(regex) => regex,
            Err(_) => return Err(ValidationError::new("regex_compilation_error")),
        };

        if !email_regex.is_match(email) {
            return Err(ValidationError::new("invalid_email"));
        }

        if email.len() > 254 {
            return Err(ValidationError::new("email_too_long"));
        }

        Ok(email.to_lowercase())
    }

    /// Validate UUID format
    pub fn validate_uuid(uuid_str: &str) -> Result<Uuid, ValidationError> {
        Uuid::parse_str(uuid_str).map_err(|_| ValidationError::new("invalid_uuid"))
    }

    /// Validate barcode format (laboratory-specific)
    pub fn validate_barcode(barcode: &str) -> Result<String, ValidationError> {
        let barcode_regex = match get_barcode_regex() {
            Ok(regex) => regex,
            Err(_) => return Err(ValidationError::new("regex_compilation_error")),
        };

        if !barcode_regex.is_match(barcode) {
            return Err(ValidationError::new("invalid_barcode_format"));
        }

        Ok(barcode.to_uppercase())
    }

    /// Validate file paths to prevent directory traversal
    pub fn validate_file_path(path: &str) -> Result<String, ValidationError> {
        let safe_path_regex = match get_safe_path_regex() {
            Ok(regex) => regex,
            Err(_) => return Err(ValidationError::new("regex_compilation_error")),
        };

        // Check for directory traversal attempts
        if path.contains("..") || path.contains("//") || path.starts_with('/') {
            return Err(ValidationError::new("invalid_file_path"));
        }

        if !safe_path_regex.is_match(path) {
            return Err(ValidationError::new("unsafe_file_path"));
        }

        Ok(path.to_string())
    }

    /// Validate SQL input to prevent injection
    pub fn validate_sql_input(input: &str) -> Result<String, ValidationError> {
        // Check for SQL injection patterns
        let sql_injection_patterns = [
            "DROP", "DELETE", "INSERT", "UPDATE", "UNION", "SELECT", "--", "/*", "*/", "xp_",
            "sp_", "exec", "execute",
        ];

        let upper_input = input.to_uppercase();
        for pattern in sql_injection_patterns {
            if upper_input.contains(pattern) {
                return Err(ValidationError::new("potential_sql_injection"));
            }
        }

        Ok(Self::sanitize_string(input))
    }
}

/// Validate request headers
async fn validate_headers(headers: &HeaderMap) -> Result<(), Response> {
    // Check for required headers
    if headers.get("user-agent").is_none() {
        return Err(create_validation_error_response(
            ValidationErrorType::InvalidHeader,
            "User-Agent header is required".to_string(),
        ));
    }

    // Validate User-Agent length
    if let Some(user_agent) = headers.get("user-agent") {
        if let Ok(ua_str) = user_agent.to_str() {
            if ua_str.len() > 500 {
                return Err(create_validation_error_response(
                    ValidationErrorType::InvalidHeader,
                    "User-Agent header too long".to_string(),
                ));
            }
        }
    }

    // Check for suspicious headers
    let suspicious_headers = ["x-forwarded-for", "x-real-ip"];
    for header_name in suspicious_headers {
        if let Some(header_value) = headers.get(header_name) {
            if let Ok(value_str) = header_value.to_str() {
                // Basic validation for IP format if present
                if !is_valid_ip_format(value_str) {
                    warn!(
                        "Suspicious header value detected: {} = {}",
                        header_name, value_str
                    );
                }
            }
        }
    }

    Ok(())
}

/// Validate content type
async fn validate_content_type(headers: &HeaderMap) -> Result<(), Response> {
    if let Some(content_type) = headers.get("content-type") {
        if let Ok(ct_str) = content_type.to_str() {
            let ct_main = ct_str.split(';').next().unwrap_or("").trim();

            if !ALLOWED_CONTENT_TYPES.contains(&ct_main) {
                return Err(create_validation_error_response(
                    ValidationErrorType::InvalidContentType,
                    format!("Content-Type '{}' not allowed", ct_main),
                ));
            }
        }
    }

    Ok(())
}

/// Validate path parameters
async fn validate_path_parameters(path: &str) -> Result<(), Response> {
    let path_segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    for segment in path_segments {
        // Check for directory traversal
        if segment.contains("..") {
            return Err(create_validation_error_response(
                ValidationErrorType::InvalidPathParameter,
                "Directory traversal detected in path".to_string(),
            ));
        }

        // Validate UUID parameters
        if segment.len() == 36 && segment.contains('-') {
            if InputSanitizer::validate_uuid(segment).is_err() {
                return Err(create_validation_error_response(
                    ValidationErrorType::InvalidPathParameter,
                    format!("Invalid UUID format: {}", segment),
                ));
            }
        }

        // Check for overly long segments
        if segment.len() > 100 {
            return Err(create_validation_error_response(
                ValidationErrorType::InvalidPathParameter,
                "Path segment too long".to_string(),
            ));
        }
    }

    Ok(())
}

/// Create validation error response
fn create_validation_error_response(error_type: ValidationErrorType, message: String) -> Response {
    let error_response = ErrorResponse {
        error_id: Uuid::new_v4(),
        error_code: format!("VALIDATION_{:?}", error_type).to_uppercase(),
        message,
        severity: ErrorSeverity::Medium,
        context: HashMap::new(),
        retryable: false,
        timestamp: chrono::Utc::now(),
    };

    let mut response = Response::new(serde_json::to_string(&error_response).unwrap().into());
    *response.status_mut() = StatusCode::BAD_REQUEST;
    response
        .headers_mut()
        .insert("content-type", "application/json".parse().unwrap());

    response
}

/// Basic IP format validation
fn is_valid_ip_format(ip: &str) -> bool {
    match (get_ipv4_regex(), get_ipv6_regex()) {
        (Ok(ipv4_regex), Ok(ipv6_regex)) => ipv4_regex.is_match(ip) || ipv6_regex.is_match(ip),
        _ => {
            warn!("Failed to get IP validation regexes");
            false
        }
    }
}

/// Validation traits for request models
#[async_trait::async_trait]
pub trait ValidatedRequest: Validate + Send {
    async fn validate_business_rules(&self) -> Result<(), ValidationErrors> {
        Ok(())
    }
}

/// Sample creation request validation
#[derive(Debug, Deserialize, Validate)]
pub struct CreateSampleRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,

    #[validate(custom(function = "validate_sample_type"))]
    pub sample_type: String,

    #[validate(custom(function = "validate_barcode_format"))]
    pub barcode: Option<String>,

    #[validate(length(max = 500))]
    pub description: Option<String>,

    #[validate(custom(function = "validate_storage_conditions"))]
    pub storage_conditions: Option<String>,
}

#[async_trait::async_trait]
impl ValidatedRequest for CreateSampleRequest {
    async fn validate_business_rules(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();

        // Custom business logic validation
        if let Some(barcode) = &self.barcode {
            if InputSanitizer::validate_barcode(barcode).is_err() {
                errors.add("barcode", ValidationError::new("invalid_barcode_format"));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Custom validation functions
fn validate_sample_type(sample_type: &str) -> Result<(), ValidationError> {
    let valid_types = [
        "DNA", "RNA", "Protein", "Blood", "Saliva", "Tissue", "Urine",
    ];

    if !valid_types.contains(&sample_type) {
        return Err(ValidationError::new("invalid_sample_type"));
    }

    Ok(())
}

fn validate_barcode_format(barcode: &str) -> Result<(), ValidationError> {
    InputSanitizer::validate_barcode(barcode).map(|_| ())
}

fn validate_storage_conditions(conditions: &str) -> Result<(), ValidationError> {
    let valid_conditions = ["frozen", "refrigerated", "room_temperature", "cryogenic"];

    if !valid_conditions.contains(&conditions) {
        return Err(ValidationError::new("invalid_storage_conditions"));
    }

    Ok(())
}

/// Rate limiting validation
pub struct RateLimitValidator {
    requests_per_minute: HashMap<String, Vec<chrono::DateTime<chrono::Utc>>>,
}

impl RateLimitValidator {
    pub fn new() -> Self {
        Self {
            requests_per_minute: HashMap::new(),
        }
    }

    pub fn check_rate_limit(&mut self, identifier: &str, max_requests: usize) -> bool {
        let now = chrono::Utc::now();
        let minute_ago = now - chrono::Duration::minutes(1);

        let requests = self
            .requests_per_minute
            .entry(identifier.to_string())
            .or_insert_with(Vec::new);

        // Remove old requests
        requests.retain(|&request_time| request_time > minute_ago);

        // Check limit
        if requests.len() >= max_requests {
            false
        } else {
            requests.push(now);
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_string() {
        let input = "<script>alert('xss')</script>";
        let sanitized = InputSanitizer::sanitize_string(input);
        assert!(!sanitized.contains('<'));
        assert!(!sanitized.contains('>'));
    }

    #[test]
    fn test_validate_email() {
        assert!(InputSanitizer::validate_email("test@example.com").is_ok());
        assert!(InputSanitizer::validate_email("invalid-email").is_err());
    }

    #[test]
    fn test_validate_barcode() {
        assert!(InputSanitizer::validate_barcode("DNA-20240320123456-001").is_ok());
        assert!(InputSanitizer::validate_barcode("invalid-barcode").is_err());
    }
}
