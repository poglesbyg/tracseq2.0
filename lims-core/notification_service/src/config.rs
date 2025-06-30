use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database_url: String,
    pub auth_service_url: String,
    pub email: EmailConfig,
    pub sms: SmsConfig,
    pub slack: SlackConfig,
    pub teams: TeamsConfig,
    pub discord: DiscordConfig,
    pub webhook: WebhookConfig,
    pub rate_limiting: RateLimitingConfig,
    pub retry: RetryConfig,
    pub templates: TemplateConfig,
    pub analytics: AnalyticsConfig,
    pub security: SecurityConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub max_connections: u32,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub enabled: bool,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub from_address: String,
    pub from_name: String,
    pub use_tls: bool,
    pub use_starttls: bool,
    pub timeout_seconds: u64,
    pub max_recipients_per_email: usize,
    pub attachment_max_size_mb: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsConfig {
    pub enabled: bool,
    pub provider: String, // "twilio", "aws_sns", "azure"
    pub account_sid: String,
    pub auth_token: String,
    pub from_number: String,
    pub webhook_url: Option<String>,
    pub max_message_length: usize,
    pub delivery_callback: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackConfig {
    pub enabled: bool,
    pub bot_token: String,
    pub app_token: String,
    pub webhook_urls: Vec<String>,
    pub default_channel: String,
    pub signing_secret: String,
    pub socket_mode: bool,
    pub mention_users: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamsConfig {
    pub enabled: bool,
    pub webhook_urls: Vec<String>,
    pub tenant_id: String,
    pub client_id: String,
    pub client_secret: String,
    pub default_team: String,
    pub default_channel: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordConfig {
    pub enabled: bool,
    pub bot_token: String,
    pub webhook_urls: Vec<String>,
    pub guild_id: String,
    pub default_channel_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub enabled: bool,
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub verify_ssl: bool,
    pub custom_headers: Vec<(String, String)>,
    pub max_payload_size_mb: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitingConfig {
    pub enabled: bool,
    pub requests_per_minute: u32,
    pub burst_size: u32,
    pub per_channel_limits: bool,
    pub email_per_hour: u32,
    pub sms_per_hour: u32,
    pub slack_per_minute: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub exponential_backoff: bool,
    pub jitter: bool,
    pub dead_letter_queue: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    pub engine: String, // "handlebars", "tera"
    pub cache_templates: bool,
    pub template_dir: String,
    pub max_template_size_kb: usize,
    pub allow_custom_helpers: bool,
    pub strict_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsConfig {
    pub enabled: bool,
    pub retention_days: u32,
    pub real_time_metrics: bool,
    pub export_format: String, // "json", "csv", "parquet"
    pub aggregate_interval_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub jwt_secret: String,
    pub api_key_header: String,
    pub encrypt_payloads: bool,
    pub audit_logging: bool,
    pub ip_whitelist: Vec<String>,
    pub cors_allowed_origins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String, // "json", "text"
    pub file_enabled: bool,
    pub file_path: Option<String>,
    pub console_enabled: bool,
    pub structured_logging: bool,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Config {
            server: ServerConfig {
                host: env::var("NOTIFICATION_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("NOTIFICATION_PORT")
                    .unwrap_or_else(|_| "8085".to_string())
                    .parse()
                    .unwrap_or(8085),
                workers: env::var("NOTIFICATION_WORKERS")
                    .unwrap_or_else(|_| "4".to_string())
                    .parse()
                    .unwrap_or(4),
                max_connections: env::var("NOTIFICATION_MAX_CONNECTIONS")
                    .unwrap_or_else(|_| "1000".to_string())
                    .parse()
                    .unwrap_or(1000),
                timeout_seconds: env::var("NOTIFICATION_TIMEOUT_SECONDS")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()
                    .unwrap_or(30),
            },
            database_url: env::var("NOTIFICATION_DATABASE_URL").unwrap_or_else(|_| {
                "postgresql://notification_user:password@localhost:5432/notification_db".to_string()
            }),
            auth_service_url: env::var("AUTH_SERVICE_URL")
                .unwrap_or_else(|_| "http://auth-service:8080".to_string()),
            email: EmailConfig {
                enabled: env::var("EMAIL_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                smtp_host: env::var("EMAIL_SMTP_HOST").unwrap_or_else(|_| "localhost".to_string()),
                smtp_port: env::var("EMAIL_SMTP_PORT")
                    .unwrap_or_else(|_| "587".to_string())
                    .parse()
                    .unwrap_or(587),
                smtp_username: env::var("EMAIL_SMTP_USERNAME").unwrap_or_else(|_| "".to_string()),
                smtp_password: env::var("EMAIL_SMTP_PASSWORD").unwrap_or_else(|_| "".to_string()),
                from_address: env::var("EMAIL_FROM_ADDRESS")
                    .unwrap_or_else(|_| "noreply@lab-management.com".to_string()),
                from_name: env::var("EMAIL_FROM_NAME")
                    .unwrap_or_else(|_| "Lab Management System".to_string()),
                use_tls: env::var("EMAIL_USE_TLS")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                use_starttls: env::var("EMAIL_USE_STARTTLS")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .unwrap_or(false),
                timeout_seconds: env::var("EMAIL_TIMEOUT_SECONDS")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()
                    .unwrap_or(30),
                max_recipients_per_email: env::var("EMAIL_MAX_RECIPIENTS")
                    .unwrap_or_else(|_| "50".to_string())
                    .parse()
                    .unwrap_or(50),
                attachment_max_size_mb: env::var("EMAIL_ATTACHMENT_MAX_SIZE_MB")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap_or(10),
            },
            sms: SmsConfig {
                enabled: env::var("SMS_ENABLED")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .unwrap_or(false),
                provider: env::var("SMS_PROVIDER").unwrap_or_else(|_| "twilio".to_string()),
                account_sid: env::var("SMS_ACCOUNT_SID").unwrap_or_else(|_| "".to_string()),
                auth_token: env::var("SMS_AUTH_TOKEN").unwrap_or_else(|_| "".to_string()),
                from_number: env::var("SMS_FROM_NUMBER").unwrap_or_else(|_| "".to_string()),
                webhook_url: env::var("SMS_WEBHOOK_URL").ok(),
                max_message_length: env::var("SMS_MAX_MESSAGE_LENGTH")
                    .unwrap_or_else(|_| "1600".to_string())
                    .parse()
                    .unwrap_or(1600),
                delivery_callback: env::var("SMS_DELIVERY_CALLBACK")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
            },
            slack: SlackConfig {
                enabled: env::var("SLACK_ENABLED")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .unwrap_or(false),
                bot_token: env::var("SLACK_BOT_TOKEN").unwrap_or_else(|_| "".to_string()),
                app_token: env::var("SLACK_APP_TOKEN").unwrap_or_else(|_| "".to_string()),
                webhook_urls: env::var("SLACK_WEBHOOK_URLS")
                    .unwrap_or_else(|_| "".to_string())
                    .split(',')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect(),
                default_channel: env::var("SLACK_DEFAULT_CHANNEL")
                    .unwrap_or_else(|_| "#lab-notifications".to_string()),
                signing_secret: env::var("SLACK_SIGNING_SECRET").unwrap_or_else(|_| "".to_string()),
                socket_mode: env::var("SLACK_SOCKET_MODE")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .unwrap_or(false),
                mention_users: env::var("SLACK_MENTION_USERS")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
            },
            teams: TeamsConfig {
                enabled: env::var("TEAMS_ENABLED")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .unwrap_or(false),
                webhook_urls: env::var("TEAMS_WEBHOOK_URLS")
                    .unwrap_or_else(|_| "".to_string())
                    .split(',')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect(),
                tenant_id: env::var("TEAMS_TENANT_ID").unwrap_or_else(|_| "".to_string()),
                client_id: env::var("TEAMS_CLIENT_ID").unwrap_or_else(|_| "".to_string()),
                client_secret: env::var("TEAMS_CLIENT_SECRET").unwrap_or_else(|_| "".to_string()),
                default_team: env::var("TEAMS_DEFAULT_TEAM")
                    .unwrap_or_else(|_| "Lab Team".to_string()),
                default_channel: env::var("TEAMS_DEFAULT_CHANNEL")
                    .unwrap_or_else(|_| "General".to_string()),
            },
            discord: DiscordConfig {
                enabled: env::var("DISCORD_ENABLED")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .unwrap_or(false),
                bot_token: env::var("DISCORD_BOT_TOKEN").unwrap_or_else(|_| "".to_string()),
                webhook_urls: env::var("DISCORD_WEBHOOK_URLS")
                    .unwrap_or_else(|_| "".to_string())
                    .split(',')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect(),
                guild_id: env::var("DISCORD_GUILD_ID").unwrap_or_else(|_| "".to_string()),
                default_channel_id: env::var("DISCORD_DEFAULT_CHANNEL_ID")
                    .unwrap_or_else(|_| "".to_string()),
            },
            webhook: WebhookConfig {
                enabled: env::var("WEBHOOK_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                timeout_seconds: env::var("WEBHOOK_TIMEOUT_SECONDS")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()
                    .unwrap_or(30),
                max_retries: env::var("WEBHOOK_MAX_RETRIES")
                    .unwrap_or_else(|_| "3".to_string())
                    .parse()
                    .unwrap_or(3),
                verify_ssl: env::var("WEBHOOK_VERIFY_SSL")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                custom_headers: vec![],
                max_payload_size_mb: env::var("WEBHOOK_MAX_PAYLOAD_SIZE_MB")
                    .unwrap_or_else(|_| "5".to_string())
                    .parse()
                    .unwrap_or(5),
            },
            rate_limiting: RateLimitingConfig {
                enabled: env::var("RATE_LIMITING_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                requests_per_minute: env::var("RATE_LIMITING_REQUESTS_PER_MINUTE")
                    .unwrap_or_else(|_| "1000".to_string())
                    .parse()
                    .unwrap_or(1000),
                burst_size: env::var("RATE_LIMITING_BURST_SIZE")
                    .unwrap_or_else(|_| "100".to_string())
                    .parse()
                    .unwrap_or(100),
                per_channel_limits: env::var("RATE_LIMITING_PER_CHANNEL")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                email_per_hour: env::var("RATE_LIMITING_EMAIL_PER_HOUR")
                    .unwrap_or_else(|_| "100".to_string())
                    .parse()
                    .unwrap_or(100),
                sms_per_hour: env::var("RATE_LIMITING_SMS_PER_HOUR")
                    .unwrap_or_else(|_| "50".to_string())
                    .parse()
                    .unwrap_or(50),
                slack_per_minute: env::var("RATE_LIMITING_SLACK_PER_MINUTE")
                    .unwrap_or_else(|_| "60".to_string())
                    .parse()
                    .unwrap_or(60),
            },
            retry: RetryConfig {
                max_attempts: env::var("RETRY_MAX_ATTEMPTS")
                    .unwrap_or_else(|_| "3".to_string())
                    .parse()
                    .unwrap_or(3),
                initial_delay_ms: env::var("RETRY_INITIAL_DELAY_MS")
                    .unwrap_or_else(|_| "1000".to_string())
                    .parse()
                    .unwrap_or(1000),
                max_delay_ms: env::var("RETRY_MAX_DELAY_MS")
                    .unwrap_or_else(|_| "30000".to_string())
                    .parse()
                    .unwrap_or(30000),
                exponential_backoff: env::var("RETRY_EXPONENTIAL_BACKOFF")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                jitter: env::var("RETRY_JITTER")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                dead_letter_queue: env::var("RETRY_DEAD_LETTER_QUEUE")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
            },
            templates: TemplateConfig {
                engine: env::var("TEMPLATE_ENGINE").unwrap_or_else(|_| "handlebars".to_string()),
                cache_templates: env::var("TEMPLATE_CACHE")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                template_dir: env::var("TEMPLATE_DIR")
                    .unwrap_or_else(|_| "./templates".to_string()),
                max_template_size_kb: env::var("TEMPLATE_MAX_SIZE_KB")
                    .unwrap_or_else(|_| "100".to_string())
                    .parse()
                    .unwrap_or(100),
                allow_custom_helpers: env::var("TEMPLATE_ALLOW_CUSTOM_HELPERS")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .unwrap_or(false),
                strict_mode: env::var("TEMPLATE_STRICT_MODE")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
            },
            analytics: AnalyticsConfig {
                enabled: env::var("ANALYTICS_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                retention_days: env::var("ANALYTICS_RETENTION_DAYS")
                    .unwrap_or_else(|_| "90".to_string())
                    .parse()
                    .unwrap_or(90),
                real_time_metrics: env::var("ANALYTICS_REAL_TIME")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                export_format: env::var("ANALYTICS_EXPORT_FORMAT")
                    .unwrap_or_else(|_| "json".to_string()),
                aggregate_interval_minutes: env::var("ANALYTICS_AGGREGATE_INTERVAL_MINUTES")
                    .unwrap_or_else(|_| "5".to_string())
                    .parse()
                    .unwrap_or(5),
            },
            security: SecurityConfig {
                jwt_secret: env::var("JWT_SECRET")
                    .unwrap_or_else(|_| "your-notification-service-jwt-secret".to_string()),
                api_key_header: env::var("API_KEY_HEADER")
                    .unwrap_or_else(|_| "X-API-Key".to_string()),
                encrypt_payloads: env::var("ENCRYPT_PAYLOADS")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .unwrap_or(false),
                audit_logging: env::var("AUDIT_LOGGING")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                ip_whitelist: env::var("IP_WHITELIST")
                    .unwrap_or_else(|_| "".to_string())
                    .split(',')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect(),
                cors_allowed_origins: env::var("CORS_ALLOWED_ORIGINS")
                    .unwrap_or_else(|_| "http://localhost:3000".to_string())
                    .split(',')
                    .map(|s| s.to_string())
                    .collect(),
            },
            logging: LoggingConfig {
                level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
                format: env::var("LOG_FORMAT").unwrap_or_else(|_| "json".to_string()),
                file_enabled: env::var("LOG_FILE_ENABLED")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .unwrap_or(false),
                file_path: env::var("LOG_FILE_PATH").ok(),
                console_enabled: env::var("LOG_CONSOLE_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                structured_logging: env::var("LOG_STRUCTURED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
            },
        })
    }

    pub fn validate(&self) -> Result<()> {
        // Validate required fields based on enabled features
        if self.email.enabled {
            if self.email.smtp_host.is_empty() {
                return Err(anyhow::anyhow!(
                    "Email SMTP host is required when email is enabled"
                ));
            }
            if self.email.from_address.is_empty() {
                return Err(anyhow::anyhow!(
                    "Email from address is required when email is enabled"
                ));
            }
        }

        if self.sms.enabled {
            if self.sms.account_sid.is_empty() || self.sms.auth_token.is_empty() {
                return Err(anyhow::anyhow!(
                    "SMS credentials are required when SMS is enabled"
                ));
            }
        }

        if self.slack.enabled {
            if self.slack.bot_token.is_empty() && self.slack.webhook_urls.is_empty() {
                return Err(anyhow::anyhow!(
                    "Slack bot token or webhook URLs are required when Slack is enabled"
                ));
            }
        }

        if self.teams.enabled && self.teams.webhook_urls.is_empty() {
            return Err(anyhow::anyhow!(
                "Teams webhook URLs are required when Teams is enabled"
            ));
        }

        if self.discord.enabled {
            if self.discord.bot_token.is_empty() && self.discord.webhook_urls.is_empty() {
                return Err(anyhow::anyhow!(
                    "Discord bot token or webhook URLs are required when Discord is enabled"
                ));
            }
        }

        Ok(())
    }
}
