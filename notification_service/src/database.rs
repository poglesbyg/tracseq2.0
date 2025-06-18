use anyhow::Result;
use sqlx::{PgPool, Postgres, migrate::MigrateDatabase};
use tracing::{info, error};

#[derive(Debug, Clone)]
pub struct DatabasePool {
    pub pool: PgPool,
}

impl DatabasePool {
    pub async fn new(database_url: &str) -> Result<Self> {
        info!("Connecting to database: {}", database_url);

        // Create database if it doesn't exist
        if !Postgres::database_exists(database_url).await.unwrap_or(false) {
            info!("Creating database...");
            Postgres::create_database(database_url).await?;
        }

        let pool = PgPool::connect(database_url).await?;
        
        info!("Database connection established successfully");

        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<()> {
        info!("Running database migrations...");

        // Create notification types
        sqlx::query(
            r#"
            DO $$ BEGIN
                CREATE TYPE notification_type AS ENUM (
                    'alert', 'info', 'warning', 'error', 'success', 
                    'reminder', 'update', 'system'
                );
            EXCEPTION
                WHEN duplicate_object THEN null;
            END $$;
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            DO $$ BEGIN
                CREATE TYPE priority AS ENUM (
                    'low', 'medium', 'high', 'critical', 'urgent'
                );
            EXCEPTION
                WHEN duplicate_object THEN null;
            END $$;
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            DO $$ BEGIN
                CREATE TYPE notification_status AS ENUM (
                    'pending', 'scheduled', 'sending', 'sent', 
                    'delivered', 'failed', 'cancelled', 'retrying'
                );
            EXCEPTION
                WHEN duplicate_object THEN null;
            END $$;
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            DO $$ BEGIN
                CREATE TYPE channel AS ENUM (
                    'email', 'sms', 'slack', 'teams', 'discord', 
                    'webhook', 'push', 'in_app'
                );
            EXCEPTION
                WHEN duplicate_object THEN null;
            END $$;
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create notifications table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS notifications (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                title VARCHAR(255) NOT NULL,
                message TEXT NOT NULL,
                notification_type notification_type NOT NULL,
                priority priority NOT NULL DEFAULT 'medium',
                status notification_status NOT NULL DEFAULT 'pending',
                channels channel[] NOT NULL,
                recipients TEXT[] NOT NULL,
                template_id UUID,
                template_data JSONB,
                scheduled_at TIMESTAMPTZ,
                sent_at TIMESTAMPTZ,
                delivery_attempts INTEGER NOT NULL DEFAULT 0,
                metadata JSONB NOT NULL DEFAULT '{}',
                created_by UUID NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create notification templates table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS notification_templates (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                name VARCHAR(255) NOT NULL,
                description TEXT,
                template_type channel NOT NULL,
                channels channel[] NOT NULL,
                subject_template TEXT NOT NULL,
                body_template TEXT NOT NULL,
                variables TEXT[] NOT NULL DEFAULT '{}',
                default_data JSONB NOT NULL DEFAULT '{}',
                is_active BOOLEAN NOT NULL DEFAULT true,
                version INTEGER NOT NULL DEFAULT 1,
                created_by UUID NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create subscriptions table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS subscriptions (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                user_id UUID NOT NULL,
                event_type VARCHAR(255) NOT NULL,
                channels channel[] NOT NULL,
                filters JSONB NOT NULL DEFAULT '{}',
                preferences JSONB NOT NULL DEFAULT '{}',
                is_active BOOLEAN NOT NULL DEFAULT true,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create delivery attempts table
        sqlx::query(
            r#"
            DO $$ BEGIN
                CREATE TYPE delivery_status AS ENUM (
                    'pending', 'sent', 'delivered', 'failed', 
                    'bounced', 'spam', 'unsubscribed', 'blocked'
                );
            EXCEPTION
                WHEN duplicate_object THEN null;
            END $$;
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS delivery_attempts (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                notification_id UUID NOT NULL REFERENCES notifications(id) ON DELETE CASCADE,
                channel channel NOT NULL,
                recipient VARCHAR(255) NOT NULL,
                status delivery_status NOT NULL DEFAULT 'pending',
                attempt_number INTEGER NOT NULL,
                sent_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                delivered_at TIMESTAMPTZ,
                failed_at TIMESTAMPTZ,
                error_message TEXT,
                response_data JSONB,
                provider_message_id VARCHAR(255),
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create events table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS events (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                event_type VARCHAR(255) NOT NULL,
                source_service VARCHAR(255) NOT NULL,
                source_id VARCHAR(255),
                payload JSONB NOT NULL,
                processed_at TIMESTAMPTZ,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create rate limits table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS rate_limits (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                key_identifier VARCHAR(255) NOT NULL,
                channel channel,
                request_count INTEGER NOT NULL DEFAULT 0,
                window_start TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                window_duration_seconds INTEGER NOT NULL,
                limit_value INTEGER NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                
                UNIQUE(key_identifier, channel)
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes for performance
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_notifications_status ON notifications(status);
            CREATE INDEX IF NOT EXISTS idx_notifications_created_at ON notifications(created_at);
            CREATE INDEX IF NOT EXISTS idx_notifications_scheduled_at ON notifications(scheduled_at);
            CREATE INDEX IF NOT EXISTS idx_notifications_priority ON notifications(priority);
            CREATE INDEX IF NOT EXISTS idx_delivery_attempts_notification_id ON delivery_attempts(notification_id);
            CREATE INDEX IF NOT EXISTS idx_delivery_attempts_status ON delivery_attempts(status);
            CREATE INDEX IF NOT EXISTS idx_subscriptions_user_id ON subscriptions(user_id);
            CREATE INDEX IF NOT EXISTS idx_subscriptions_event_type ON subscriptions(event_type);
            CREATE INDEX IF NOT EXISTS idx_events_event_type ON events(event_type);
            CREATE INDEX IF NOT EXISTS idx_events_source_service ON events(source_service);
            CREATE INDEX IF NOT EXISTS idx_rate_limits_key ON rate_limits(key_identifier);
            "#,
        )
        .execute(&self.pool)
        .await?;

        info!("Database migrations completed successfully");
        Ok(())
    }

    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1").execute(&self.pool).await?;
        Ok(())
    }
} 
