use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub struct DatabasePool {
    pub pool: PgPool,
}

impl DatabasePool {
    pub async fn new(database_url: &str) -> Result<Self> {
        info!("Connecting to database: {}", database_url);

        let pool = PgPoolOptions::new()
            .max_connections(20)
            .min_connections(5)
            .connect(database_url)
            .await?;

        info!("Database connection pool established");

        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<()> {
        info!("Running database migrations");

        // Create extensions
        sqlx::query("CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\"")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE EXTENSION IF NOT EXISTS \"pg_trgm\"")
            .execute(&self.pool)
            .await?;

        // Create storage locations table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS storage_locations (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                name VARCHAR NOT NULL UNIQUE,
                description TEXT,
                location_type VARCHAR NOT NULL,
                temperature_zone VARCHAR NOT NULL,
                max_capacity INTEGER NOT NULL DEFAULT 100,
                current_capacity INTEGER NOT NULL DEFAULT 0,
                coordinates JSONB,
                status VARCHAR NOT NULL DEFAULT 'active',
                metadata JSONB DEFAULT '{}',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create samples table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS samples (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                barcode VARCHAR NOT NULL UNIQUE,
                sample_type VARCHAR NOT NULL,
                storage_location_id UUID REFERENCES storage_locations(id),
                position JSONB,
                temperature_requirements VARCHAR,
                status VARCHAR NOT NULL DEFAULT 'stored',
                metadata JSONB DEFAULT '{}',
                chain_of_custody JSONB DEFAULT '[]',
                stored_at TIMESTAMPTZ,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create IoT sensors table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS iot_sensors (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                sensor_id VARCHAR NOT NULL UNIQUE,
                sensor_type VARCHAR NOT NULL,
                location_id UUID REFERENCES storage_locations(id),
                status VARCHAR NOT NULL DEFAULT 'active',
                last_reading JSONB,
                calibration_data JSONB DEFAULT '{}',
                maintenance_schedule JSONB DEFAULT '{}',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create sensor data table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sensor_data (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                sensor_id UUID REFERENCES iot_sensors(id),
                reading_type VARCHAR NOT NULL,
                value DECIMAL NOT NULL,
                unit VARCHAR NOT NULL,
                quality_score DECIMAL DEFAULT 1.0,
                metadata JSONB DEFAULT '{}',
                recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create alerts table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS alerts (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                alert_type VARCHAR NOT NULL,
                severity VARCHAR NOT NULL,
                title VARCHAR NOT NULL,
                message TEXT NOT NULL,
                source_type VARCHAR NOT NULL,
                source_id UUID,
                status VARCHAR NOT NULL DEFAULT 'active',
                acknowledged_by UUID,
                acknowledged_at TIMESTAMPTZ,
                resolved_at TIMESTAMPTZ,
                metadata JSONB DEFAULT '{}',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create analytics models table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS analytics_models (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                model_name VARCHAR NOT NULL,
                model_type VARCHAR NOT NULL,
                version VARCHAR NOT NULL,
                model_data JSONB NOT NULL,
                performance_metrics JSONB DEFAULT '{}',
                training_metadata JSONB DEFAULT '{}',
                status VARCHAR NOT NULL DEFAULT 'active',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create predictions table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS predictions (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                model_id UUID REFERENCES analytics_models(id),
                prediction_type VARCHAR NOT NULL,
                input_data JSONB NOT NULL,
                prediction_result JSONB NOT NULL,
                confidence_score DECIMAL,
                prediction_horizon INTEGER,
                metadata JSONB DEFAULT '{}',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create blockchain transactions table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS blockchain_transactions (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                transaction_hash VARCHAR NOT NULL UNIQUE,
                block_number BIGINT,
                transaction_type VARCHAR NOT NULL,
                data_hash VARCHAR NOT NULL,
                previous_hash VARCHAR,
                timestamp TIMESTAMPTZ NOT NULL,
                signature VARCHAR NOT NULL,
                metadata JSONB DEFAULT '{}',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create automation tasks table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS automation_tasks (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                task_type VARCHAR NOT NULL,
                priority INTEGER NOT NULL DEFAULT 5,
                status VARCHAR NOT NULL DEFAULT 'pending',
                input_parameters JSONB NOT NULL,
                output_results JSONB,
                assigned_robot_id VARCHAR,
                scheduled_at TIMESTAMPTZ,
                started_at TIMESTAMPTZ,
                completed_at TIMESTAMPTZ,
                error_message TEXT,
                metadata JSONB DEFAULT '{}',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create energy consumption table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS energy_consumption (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                location_id UUID REFERENCES storage_locations(id),
                equipment_type VARCHAR NOT NULL,
                consumption_kwh DECIMAL NOT NULL,
                cost_usd DECIMAL,
                efficiency_ratio DECIMAL,
                optimization_suggestions JSONB DEFAULT '[]',
                period_start TIMESTAMPTZ NOT NULL,
                period_end TIMESTAMPTZ NOT NULL,
                metadata JSONB DEFAULT '{}',
                recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create compliance events table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS compliance_events (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                event_type VARCHAR NOT NULL,
                regulatory_standard VARCHAR NOT NULL,
                compliance_status VARCHAR NOT NULL,
                description TEXT NOT NULL,
                affected_entity_type VARCHAR NOT NULL,
                affected_entity_id UUID NOT NULL,
                remediation_required BOOLEAN DEFAULT false,
                remediation_actions JSONB DEFAULT '[]',
                auditor_notes TEXT,
                metadata JSONB DEFAULT '{}',
                occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes for performance
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_samples_barcode ON samples(barcode)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_samples_location ON samples(storage_location_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_sensor_data_sensor ON sensor_data(sensor_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_sensor_data_recorded_at ON sensor_data(recorded_at)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_alerts_status ON alerts(status)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_predictions_model ON predictions(model_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_blockchain_hash ON blockchain_transactions(transaction_hash)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_automation_status ON automation_tasks(status)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_energy_location ON energy_consumption(location_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_compliance_status ON compliance_events(compliance_status)")
            .execute(&self.pool)
            .await?;

        info!("Database migrations completed successfully");

        Ok(())
    }

    pub async fn health_check(&self) -> Result<bool> {
        let result = sqlx::query_scalar::<_, i64>("SELECT 1")
            .fetch_one(&self.pool)
            .await?;

        Ok(result == 1)
    }
}
