use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;

/// Main configuration structure for the authentication service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database_url: String,
    pub jwt: JwtConfig,
    pub security: SecurityConfig,
    pub email: EmailConfig,
    pub redis: Option<RedisConfig>,
    pub logging: LoggingConfig,
    pub features: FeatureConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
    pub access_token_expiry_hours: i64,
    pub refresh_token_expiry_days: i64,
    pub issuer: String,
    pub audience: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub password_min_length: usize,
    pub password_require_uppercase: bool,
    pub password_require_lowercase: bool,
    pub password_require_numbers: bool,
    pub password_require_symbols: bool,
    pub max_login_attempts: u32,
    pub lockout_duration_minutes: u32,
    pub session_timeout_hours: i64,
    pub rate_limiting_enabled: bool,
    pub rate_limit_requests_per_minute: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub from_address: String,
    pub from_name: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub pool_size: u32,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub file_enabled: bool,
    pub file_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureConfig {
    pub registration_enabled: bool,
    pub email_verification_required: bool,
    pub password_reset_enabled: bool,
    pub shibboleth_enabled: bool,
    pub audit_logging_enabled: bool,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok(); // Load .env file if present

        let config = Config {
            server: ServerConfig {
                host: env::var("AUTH_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("AUTH_PORT")
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()
                    .unwrap_or(8080),
                workers: env::var("AUTH_WORKERS").ok().and_then(|w| w.parse().ok()),
            },
            database_url: env::var("AUTH_DATABASE_URL")
                .or_else(|_| env::var("DATABASE_URL"))
                .unwrap_or_else(|_| {
                    "postgresql://auth_user:auth_password@localhost:5432/auth_db".to_string()
                }),
            jwt: JwtConfig {
                secret: env::var("JWT_SECRET").unwrap_or_else(|_| {
                    "your-super-secret-jwt-key-change-in-production".to_string()
                }),
                access_token_expiry_hours: env::var("JWT_ACCESS_TOKEN_EXPIRY_HOURS")
                    .unwrap_or_else(|_| "1".to_string())
                    .parse()
                    .unwrap_or(1),
                refresh_token_expiry_days: env::var("JWT_REFRESH_TOKEN_EXPIRY_DAYS")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()
                    .unwrap_or(30),
                issuer: env::var("JWT_ISSUER").unwrap_or_else(|_| "auth-service".to_string()),
                audience: env::var("JWT_AUDIENCE")
                    .unwrap_or_else(|_| "lab-management-system".to_string()),
            },
            security: SecurityConfig {
                password_min_length: env::var("PASSWORD_MIN_LENGTH")
                    .unwrap_or_else(|_| "8".to_string())
                    .parse()
                    .unwrap_or(8),
                password_require_uppercase: env::var("PASSWORD_REQUIRE_UPPERCASE")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                password_require_lowercase: env::var("PASSWORD_REQUIRE_LOWERCASE")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                password_require_numbers: env::var("PASSWORD_REQUIRE_NUMBERS")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                password_require_symbols: env::var("PASSWORD_REQUIRE_SYMBOLS")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .unwrap_or(false),
                max_login_attempts: env::var("MAX_LOGIN_ATTEMPTS")
                    .unwrap_or_else(|_| "5".to_string())
                    .parse()
                    .unwrap_or(5),
                lockout_duration_minutes: env::var("LOCKOUT_DURATION_MINUTES")
                    .unwrap_or_else(|_| "15".to_string())
                    .parse()
                    .unwrap_or(15),
                session_timeout_hours: env::var("SESSION_TIMEOUT_HOURS")
                    .unwrap_or_else(|_| "8".to_string())
                    .parse()
                    .unwrap_or(8),
                rate_limiting_enabled: env::var("RATE_LIMITING_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                rate_limit_requests_per_minute: env::var("RATE_LIMIT_REQUESTS_PER_MINUTE")
                    .unwrap_or_else(|_| "60".to_string())
                    .parse()
                    .unwrap_or(60),
            },
            email: EmailConfig {
                smtp_host: env::var("SMTP_HOST").unwrap_or_else(|_| "localhost".to_string()),
                smtp_port: env::var("SMTP_PORT")
                    .unwrap_or_else(|_| "587".to_string())
                    .parse()
                    .unwrap_or(587),
                smtp_username: env::var("SMTP_USERNAME").unwrap_or_else(|_| "".to_string()),
                smtp_password: env::var("SMTP_PASSWORD").unwrap_or_else(|_| "".to_string()),
                from_address: env::var("EMAIL_FROM_ADDRESS")
                    .unwrap_or_else(|_| "noreply@lab-management.com".to_string()),
                from_name: env::var("EMAIL_FROM_NAME")
                    .unwrap_or_else(|_| "Lab Management System".to_string()),
                enabled: env::var("EMAIL_ENABLED")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .unwrap_or(false),
            },
            redis: if env::var("REDIS_URL").is_ok() {
                Some(RedisConfig {
                    url: env::var("REDIS_URL").unwrap(),
                    pool_size: env::var("REDIS_POOL_SIZE")
                        .unwrap_or_else(|_| "10".to_string())
                        .parse()
                        .unwrap_or(10),
                    timeout_seconds: env::var("REDIS_TIMEOUT_SECONDS")
                        .unwrap_or_else(|_| "5".to_string())
                        .parse()
                        .unwrap_or(5),
                })
            } else {
                None
            },
            logging: LoggingConfig {
                level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
                format: env::var("LOG_FORMAT").unwrap_or_else(|_| "json".to_string()),
                file_enabled: env::var("LOG_FILE_ENABLED")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .unwrap_or(false),
                file_path: env::var("LOG_FILE_PATH").ok(),
            },
            features: FeatureConfig {
                registration_enabled: env::var("REGISTRATION_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                email_verification_required: env::var("EMAIL_VERIFICATION_REQUIRED")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .unwrap_or(false),
                password_reset_enabled: env::var("PASSWORD_RESET_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                shibboleth_enabled: env::var("SHIBBOLETH_ENABLED")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .unwrap_or(false),
                audit_logging_enabled: env::var("AUDIT_LOGGING_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
            },
        };

        config.validate()?;
        Ok(config)
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Validate JWT secret
        if self.jwt.secret.len() < 32 {
            return Err(anyhow::anyhow!(
                "JWT secret must be at least 32 characters long"
            ));
        }

        // Validate password requirements
        if self.security.password_min_length < 6 {
            return Err(anyhow::anyhow!(
                "Password minimum length must be at least 6"
            ));
        }

        // Validate server port
        if self.server.port == 0 {
            return Err(anyhow::anyhow!("Server port must be greater than 0"));
        }

        // Validate database URL
        if self.database_url.is_empty() {
            return Err(anyhow::anyhow!("Database URL cannot be empty"));
        }

        Ok(())
    }

    /// Get environment-specific configuration
    pub fn environment(&self) -> String {
        env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string())
    }

    /// Check if running in production
    pub fn is_production(&self) -> bool {
        self.environment().to_lowercase() == "production"
    }

    /// Check if running in development
    pub fn is_development(&self) -> bool {
        self.environment().to_lowercase() == "development"
    }
}
