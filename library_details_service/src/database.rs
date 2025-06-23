use sqlx::{PgPool, Row};
use std::str::FromStr;


pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let connect_options = sqlx::postgres::PgConnectOptions::from_str(database_url)?
        .application_name("library_details_service");
    
    let pool = PgPool::connect_with(connect_options).await?;
    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Create tables for library details service
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS libraries (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR NOT NULL,
            sample_id UUID,
            library_type VARCHAR NOT NULL,
            concentration DECIMAL,
            volume DECIMAL,
            fragment_size_min INTEGER,
            fragment_size_max INTEGER,
            preparation_protocol_id UUID,
            preparation_date TIMESTAMP WITH TIME ZONE,
            barcode VARCHAR,
            adapter_sequence VARCHAR,
            quality_score DECIMAL,
            status VARCHAR NOT NULL DEFAULT 'pending',
            metadata JSONB,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS protocols (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR NOT NULL,
            version VARCHAR NOT NULL,
            library_type VARCHAR NOT NULL,
            description TEXT,
            steps JSONB NOT NULL,
            parameters JSONB,
            kit_id UUID,
            platform_compatibility JSONB,
            quality_thresholds JSONB,
            is_active BOOLEAN DEFAULT true,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS kits (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR NOT NULL,
            manufacturer VARCHAR NOT NULL,
            catalog_number VARCHAR,
            version VARCHAR,
            library_types JSONB NOT NULL,
            reagents JSONB,
            specifications JSONB,
            storage_conditions VARCHAR,
            expiry_date DATE,
            cost_per_reaction DECIMAL,
            throughput_capacity INTEGER,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS platforms (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR NOT NULL,
            manufacturer VARCHAR NOT NULL,
            model VARCHAR NOT NULL,
            capabilities JSONB NOT NULL,
            supported_library_types JSONB NOT NULL,
            read_configurations JSONB,
            flow_cell_types JSONB,
            throughput_specs JSONB,
            quality_metrics JSONB,
            is_active BOOLEAN DEFAULT true,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS quality_control_metrics (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            library_id UUID REFERENCES libraries(id) ON DELETE CASCADE,
            metric_type VARCHAR NOT NULL,
            value DECIMAL NOT NULL,
            unit VARCHAR,
            threshold_min DECIMAL,
            threshold_max DECIMAL,
            status VARCHAR NOT NULL,
            measured_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            equipment_id VARCHAR,
            operator_id VARCHAR,
            notes TEXT,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Create indices for better performance
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_libraries_sample_id ON libraries(sample_id);")
        .execute(pool)
        .await?;
    
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_libraries_status ON libraries(status);")
        .execute(pool)
        .await?;
    
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_qc_metrics_library_id ON quality_control_metrics(library_id);")
        .execute(pool)
        .await?;

    tracing::info!("Database migrations completed successfully");
    Ok(())
}

pub async fn health_check(pool: &PgPool) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("SELECT 1")
        .fetch_one(pool)
        .await?;
    
    Ok(result.get::<i32, _>(0) == 1)
}