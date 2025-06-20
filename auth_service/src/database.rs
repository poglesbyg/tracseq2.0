use anyhow::Result;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;
use tracing::{info, warn, error};

/// Database connection pool wrapper
#[derive(Debug, Clone)]
pub struct DatabasePool {
    pub pool: PgPool,
}

impl DatabasePool {
    /// Create a new database connection pool
    pub async fn new(database_url: &str) -> Result<Self> {
        info!("Connecting to database...");

        let pool = PgPoolOptions::new()
            .max_connections(20)
            .min_connections(5)
            .acquire_timeout(Duration::from_secs(10))
            .idle_timeout(Duration::from_secs(600))
            .max_lifetime(Duration::from_secs(3600))
            .connect(database_url)
            .await?;

        info!("Database connection pool created successfully");

        Ok(Self { pool })
    }

    /// Run database migrations
    pub async fn migrate(&self) -> Result<()> {
        info!("Running database migrations...");

        // Create users table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                email VARCHAR(255) UNIQUE NOT NULL,
                password_hash VARCHAR(255) NOT NULL,
                first_name VARCHAR(100) NOT NULL,
                last_name VARCHAR(100) NOT NULL,
                role VARCHAR(50) NOT NULL DEFAULT 'guest',
                status VARCHAR(20) NOT NULL DEFAULT 'active',
                email_verified BOOLEAN NOT NULL DEFAULT FALSE,
                verification_token VARCHAR(255),
                failed_login_attempts INTEGER NOT NULL DEFAULT 0,
                locked_until TIMESTAMPTZ,
                last_login_at TIMESTAMPTZ,
                password_changed_at TIMESTAMPTZ DEFAULT NOW(),
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                
                -- Lab-specific fields
                department VARCHAR(100),
                position VARCHAR(100),
                phone VARCHAR(20),
                office_location VARCHAR(100),
                lab_affiliation VARCHAR(100),
                
                -- Shibboleth integration
                shibboleth_id VARCHAR(255),
                external_id VARCHAR(255)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create sessions table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS user_sessions (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                token_hash VARCHAR(255) NOT NULL,
                refresh_token_hash VARCHAR(255),
                device_info TEXT,
                ip_address VARCHAR(45),
                user_agent TEXT,
                expires_at TIMESTAMPTZ NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                last_used_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                revoked BOOLEAN NOT NULL DEFAULT FALSE,
                revoked_at TIMESTAMPTZ
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create audit log table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS security_audit_log (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                event_id UUID NOT NULL,
                event_type VARCHAR(50) NOT NULL,
                user_id UUID REFERENCES users(id),
                user_email VARCHAR(255),
                ip_address VARCHAR(45),
                user_agent TEXT,
                details JSONB,
                severity VARCHAR(20) NOT NULL,
                timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                
                -- Indexing
                CONSTRAINT security_audit_log_event_id_key UNIQUE(event_id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create password reset tokens table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS password_reset_tokens (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                token_hash VARCHAR(255) NOT NULL,
                expires_at TIMESTAMPTZ NOT NULL,
                used BOOLEAN NOT NULL DEFAULT FALSE,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                used_at TIMESTAMPTZ
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create email verification tokens table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS email_verification_tokens (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                token_hash VARCHAR(255) NOT NULL,
                expires_at TIMESTAMPTZ NOT NULL,
                used BOOLEAN NOT NULL DEFAULT FALSE,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                used_at TIMESTAMPTZ
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create rate limiting table (if not using Redis)
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS rate_limits (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                identifier VARCHAR(255) NOT NULL,
                request_count INTEGER NOT NULL DEFAULT 1,
                window_start TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                
                UNIQUE(identifier, window_start)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create user activity log table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS user_activity_log (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                user_id UUID REFERENCES users(id),
                action VARCHAR(100) NOT NULL,
                resource_type VARCHAR(50),
                resource_id UUID,
                ip_address VARCHAR(45),
                user_agent TEXT,
                details JSONB,
                timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes for performance
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_users_status ON users(status)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_users_shibboleth_id ON users(shibboleth_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON user_sessions(user_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON user_sessions(expires_at)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_log_user_id ON security_audit_log(user_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_log_timestamp ON security_audit_log(timestamp)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_log_event_type ON security_audit_log(event_type)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_password_reset_user_id ON password_reset_tokens(user_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_password_reset_expires_at ON password_reset_tokens(expires_at)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_email_verification_user_id ON email_verification_tokens(user_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_rate_limits_identifier ON rate_limits(identifier)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_activity_log_user_id ON user_activity_log(user_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_activity_log_timestamp ON user_activity_log(timestamp)")
            .execute(&self.pool)
            .await?;

        // Create updated_at trigger function
        sqlx::query(
            r#"
            CREATE OR REPLACE FUNCTION update_updated_at_column()
            RETURNS TRIGGER AS $$
            BEGIN
                NEW.updated_at = NOW();
                RETURN NEW;
            END;
            $$ language 'plpgsql'
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create triggers for updated_at
        if let Err(e) = sqlx::query(
            r#"
            CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
            FOR EACH ROW EXECUTE FUNCTION update_updated_at_column()
            "#,
        )
        .execute(&self.pool)
        .await
        {
            if !e.to_string().contains("already exists") {
                return Err(e.into());
            }
            warn!("Trigger update_users_updated_at already exists");
        }

        if let Err(e) = sqlx::query(
            r#"
            CREATE TRIGGER update_rate_limits_updated_at BEFORE UPDATE ON rate_limits
            FOR EACH ROW EXECUTE FUNCTION update_updated_at_column()
            "#,
        )
        .execute(&self.pool)
        .await
        {
            if !e.to_string().contains("already exists") {
                return Err(e.into());
            }
            warn!("Trigger update_rate_limits_updated_at already exists");
        }

        info!("Database migrations completed successfully");
        Ok(())
    }

    /// Health check for the database
    pub async fn health_check(&self) -> Result<DatabaseHealth> {
        let start_time = std::time::Instant::now();

        match sqlx::query("SELECT 1").fetch_one(&self.pool).await {
            Ok(_) => {
                let response_time = start_time.elapsed();
                let pool_status = self.pool.size();

                Ok(DatabaseHealth {
                    is_connected: true,
                    response_time_ms: response_time.as_millis() as u64,
                    active_connections: pool_status,
                    idle_connections: self.pool.num_idle() as u32,
                    max_connections: self.pool.options().get_max_connections(),
                })
            }
            Err(e) => {
                error!("Database health check failed: {}", e);
                Ok(DatabaseHealth {
                    is_connected: false,
                    response_time_ms: 0,
                    active_connections: 0,
                    idle_connections: 0,
                    max_connections: 0,
                })
            }
        }
    }

    /// Clean up expired sessions and tokens
    pub async fn cleanup_expired(&self) -> Result<CleanupResult> {
        let start_time = std::time::Instant::now();

        // Clean up expired sessions
        let expired_sessions = sqlx::query(
            "DELETE FROM user_sessions WHERE expires_at < NOW() OR revoked = TRUE"
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        // Clean up expired password reset tokens
        let expired_password_tokens = sqlx::query(
            "DELETE FROM password_reset_tokens WHERE expires_at < NOW() OR used = TRUE"
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        // Clean up expired email verification tokens
        let expired_email_tokens = sqlx::query(
            "DELETE FROM email_verification_tokens WHERE expires_at < NOW() OR used = TRUE"
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        // Clean up old rate limit entries (older than 24 hours)
        let expired_rate_limits = sqlx::query(
            "DELETE FROM rate_limits WHERE window_start < NOW() - INTERVAL '24 hours'"
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        // Clean up old audit log entries (older than 90 days, configurable)
        let expired_audit_logs = sqlx::query(
            "DELETE FROM security_audit_log WHERE timestamp < NOW() - INTERVAL '90 days'"
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        let duration = start_time.elapsed();

        info!(
            "Database cleanup completed in {:?}: sessions={}, password_tokens={}, email_tokens={}, rate_limits={}, audit_logs={}",
            duration, expired_sessions, expired_password_tokens, expired_email_tokens, expired_rate_limits, expired_audit_logs
        );

        Ok(CleanupResult {
            expired_sessions,
            expired_password_tokens,
            expired_email_tokens,
            expired_rate_limits,
            expired_audit_logs,
            duration_ms: duration.as_millis() as u64,
        })
    }
}

/// Database health information
#[derive(Debug, Clone, serde::Serialize)]
pub struct DatabaseHealth {
    pub is_connected: bool,
    pub response_time_ms: u64,
    pub active_connections: u32,
    pub idle_connections: u32,
    pub max_connections: u32,
}

/// Result of database cleanup operations
#[derive(Debug, Clone, serde::Serialize)]
pub struct CleanupResult {
    pub expired_sessions: u64,
    pub expired_password_tokens: u64,
    pub expired_email_tokens: u64,
    pub expired_rate_limits: u64,
    pub expired_audit_logs: u64,
    pub duration_ms: u64,
} 
