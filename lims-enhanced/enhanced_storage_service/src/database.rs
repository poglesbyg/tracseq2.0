use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, PgPool, Pool, Postgres};
use std::time::Duration;
use tracing::{info, warn};

#[derive(Clone)]
pub struct DatabasePool {
    pub pool: PgPool,
}

impl DatabasePool {
    pub async fn new(database_url: &str) -> Result<Self> {
        info!("🔗 Connecting to database: {}", database_url);

        let pool = PgPoolOptions::new()
            .max_connections(50)
            .min_connections(10)
            .acquire_timeout(Duration::from_secs(30))
            .idle_timeout(Duration::from_secs(600))
            .max_lifetime(Duration::from_secs(1800))
            .connect(database_url)
            .await?;

        info!("✅ Database connection pool created successfully");

        // Test the connection
        let connection = pool.acquire().await?;
        drop(connection);
        info!("✅ Database connection test successful");

        Ok(Self { pool })
    }

    pub async fn test_connection(&self) -> Result<()> {
        let _result = sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn run_migrations(&self) -> Result<()> {
        info!("🚀 Running database migrations...");
        
        // Check if migrations table exists
        let migrations_exist = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'migrations')"
        )
        .fetch_one(&self.pool)
        .await?;

        if !migrations_exist {
            warn!("⚠️ Migrations table not found. Creating basic storage tables...");
            self.create_basic_tables().await?;
        }

        Ok(())
    }

    async fn create_basic_tables(&self) -> Result<()> {
        info!("📋 Creating basic storage tables...");

        // Create storage_locations table if it doesn't exist
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS storage_locations (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                name VARCHAR(255) NOT NULL,
                description TEXT,
                location_type VARCHAR(100) NOT NULL,
                temperature_zone VARCHAR(100) NOT NULL,
                max_capacity INTEGER NOT NULL DEFAULT 0,
                current_capacity INTEGER NOT NULL DEFAULT 0,
                coordinates JSONB,
                status VARCHAR(50) NOT NULL DEFAULT 'active',
                metadata JSONB NOT NULL DEFAULT '{}',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create storage_containers table if it doesn't exist
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS storage_containers (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                name VARCHAR(255) NOT NULL,
                container_type VARCHAR(100) NOT NULL,
                parent_container_id UUID REFERENCES storage_containers(id) ON DELETE CASCADE,
                location_id UUID REFERENCES storage_locations(id) ON DELETE SET NULL,
                grid_position JSONB,
                dimensions JSONB,
                capacity INTEGER NOT NULL DEFAULT 0,
                occupied_count INTEGER NOT NULL DEFAULT 0,
                temperature_zone VARCHAR(100),
                barcode VARCHAR(255) UNIQUE,
                description TEXT,
                status VARCHAR(50) NOT NULL DEFAULT 'active',
                installation_date TIMESTAMPTZ,
                last_maintenance_date TIMESTAMPTZ,
                next_maintenance_date TIMESTAMPTZ,
                container_metadata JSONB NOT NULL DEFAULT '{}',
                access_restrictions JSONB NOT NULL DEFAULT '{}',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                created_by UUID
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create sample_positions table if it doesn't exist
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sample_positions (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                sample_id UUID NOT NULL,
                container_id UUID NOT NULL REFERENCES storage_containers(id) ON DELETE CASCADE,
                position_identifier VARCHAR(100),
                assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                assigned_by UUID,
                removed_at TIMESTAMPTZ,
                removed_by UUID,
                status VARCHAR(50) NOT NULL DEFAULT 'occupied',
                reservation_expires_at TIMESTAMPTZ,
                storage_conditions JSONB NOT NULL DEFAULT '{}',
                special_requirements JSONB NOT NULL DEFAULT '{}',
                chain_of_custody JSONB NOT NULL DEFAULT '[]',
                position_metadata JSONB NOT NULL DEFAULT '{}',
                notes TEXT,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create samples table if it doesn't exist
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS samples (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                barcode VARCHAR(255) UNIQUE NOT NULL,
                sample_type VARCHAR(100) NOT NULL,
                storage_location_id UUID REFERENCES storage_locations(id) ON DELETE SET NULL,
                position JSONB,
                temperature_requirements VARCHAR(100),
                status VARCHAR(50) NOT NULL DEFAULT 'active',
                metadata JSONB NOT NULL DEFAULT '{}',
                chain_of_custody JSONB NOT NULL DEFAULT '[]',
                stored_at TIMESTAMPTZ,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes for better performance
        self.create_indexes().await?;

        info!("✅ Basic storage tables created successfully");
        Ok(())
    }

    async fn create_indexes(&self) -> Result<()> {
        info!("📊 Creating database indexes...");

        // Indexes for storage_containers
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_storage_containers_type ON storage_containers(container_type)")
            .execute(&self.pool).await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_storage_containers_parent ON storage_containers(parent_container_id)")
            .execute(&self.pool).await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_storage_containers_location ON storage_containers(location_id)")
            .execute(&self.pool).await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_storage_containers_temperature ON storage_containers(temperature_zone)")
            .execute(&self.pool).await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_storage_containers_barcode ON storage_containers(barcode)")
            .execute(&self.pool).await?;

        // Indexes for sample_positions
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_sample_positions_sample ON sample_positions(sample_id)")
            .execute(&self.pool).await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_sample_positions_container ON sample_positions(container_id)")
            .execute(&self.pool).await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_sample_positions_status ON sample_positions(status)")
            .execute(&self.pool).await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_sample_positions_removed ON sample_positions(removed_at)")
            .execute(&self.pool).await?;

        // Indexes for samples
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_samples_barcode ON samples(barcode)")
            .execute(&self.pool).await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_samples_type ON samples(sample_type)")
            .execute(&self.pool).await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_samples_status ON samples(status)")
            .execute(&self.pool).await?;

        // Indexes for storage_locations
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_storage_locations_type ON storage_locations(location_type)")
            .execute(&self.pool).await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_storage_locations_temperature ON storage_locations(temperature_zone)")
            .execute(&self.pool).await?;

        info!("✅ Database indexes created successfully");
        Ok(())
    }

    pub async fn get_health_info(&self) -> Result<DatabaseHealthInfo> {
        let pool_info = self.pool.size();
        
        let total_connections = pool_info as i32;
        let active_connections = self.pool.num_idle() as i32;
        
        // Test query performance
        let start = std::time::Instant::now();
        sqlx::query("SELECT 1").fetch_one(&self.pool).await?;
        let query_time_ms = start.elapsed().as_millis() as i32;

        // Get table counts
        let total_containers: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM storage_containers"
        ).fetch_one(&self.pool).await.unwrap_or(0);

        let total_positions: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sample_positions WHERE removed_at IS NULL"
        ).fetch_one(&self.pool).await.unwrap_or(0);

        let total_samples: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM samples"
        ).fetch_one(&self.pool).await.unwrap_or(0);

        Ok(DatabaseHealthInfo {
            connected: true,
            total_connections,
            active_connections,
            query_time_ms,
            total_containers,
            total_positions,
            total_samples,
        })
    }

    pub async fn cleanup_expired_reservations(&self) -> Result<i64> {
        let result = sqlx::query(
            r#"
            UPDATE sample_positions 
            SET status = 'available', reservation_expires_at = NULL
            WHERE status = 'reserved' AND reservation_expires_at < NOW()
            "#
        ).execute(&self.pool).await?;

        Ok(result.rows_affected() as i64)
    }

    pub async fn update_container_occupancy(&self, container_id: uuid::Uuid) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE storage_containers 
            SET occupied_count = (
                SELECT COUNT(*) 
                FROM sample_positions 
                WHERE container_id = $1 AND removed_at IS NULL
            )
            WHERE id = $1
            "#
        )
        .bind(container_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_database_stats(&self) -> Result<DatabaseStats> {
        let stats = sqlx::query_as::<_, DatabaseStats>(
            r#"
            SELECT 
                (SELECT COUNT(*) FROM storage_locations) as total_locations,
                (SELECT COUNT(*) FROM storage_containers) as total_containers,
                (SELECT COUNT(*) FROM sample_positions WHERE removed_at IS NULL) as active_positions,
                (SELECT COUNT(*) FROM samples) as total_samples,
                (SELECT COUNT(*) FROM storage_containers WHERE container_type = 'freezer') as freezer_count,
                (SELECT COUNT(*) FROM storage_containers WHERE container_type = 'rack') as rack_count,
                (SELECT COUNT(*) FROM storage_containers WHERE container_type = 'box') as box_count,
                (SELECT COUNT(*) FROM storage_containers WHERE container_type = 'position') as position_count,
                (SELECT COALESCE(SUM(capacity), 0) FROM storage_containers) as total_capacity,
                (SELECT COALESCE(SUM(occupied_count), 0) FROM storage_containers) as total_occupied
            "#
        ).fetch_one(&self.pool).await?;

        Ok(stats)
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DatabaseHealthInfo {
    pub connected: bool,
    pub total_connections: i32,
    pub active_connections: i32,
    pub query_time_ms: i32,
    pub total_containers: i64,
    pub total_positions: i64,
    pub total_samples: i64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct DatabaseStats {
    pub total_locations: i64,
    pub total_containers: i64,
    pub active_positions: i64,
    pub total_samples: i64,
    pub freezer_count: i64,
    pub rack_count: i64,
    pub box_count: i64,
    pub position_count: i64,
    pub total_capacity: i64,
    pub total_occupied: i64,
}

// Helper functions for database operations
pub async fn ensure_database_schema(pool: &PgPool) -> Result<()> {
    info!("🔍 Ensuring database schema is up to date...");
    
    // Check if our tables exist
    let tables_exist = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM information_schema.tables 
            WHERE table_name IN ('storage_containers', 'sample_positions', 'storage_locations', 'samples')
        )
        "#
    ).fetch_one(pool).await?;

    if !tables_exist {
        warn!("⚠️ Storage tables not found. Please run migrations first.");
        return Err(anyhow::anyhow!("Database schema not initialized"));
    }

    info!("✅ Database schema verification complete");
    Ok(())
}

pub async fn cleanup_database(pool: &PgPool) -> Result<()> {
    info!("🧹 Running database cleanup...");
    
    // Clean up expired reservations
    let expired_reservations = sqlx::query(
        "UPDATE sample_positions SET status = 'available', reservation_expires_at = NULL WHERE status = 'reserved' AND reservation_expires_at < NOW()"
    ).execute(pool).await?;

    // Update container occupancy counts
    sqlx::query(
        r#"
        UPDATE storage_containers 
        SET occupied_count = (
            SELECT COUNT(*) 
            FROM sample_positions 
            WHERE container_id = storage_containers.id AND removed_at IS NULL
        )
        "#
    ).execute(pool).await?;

    info!("✅ Database cleanup complete. Expired reservations: {}", expired_reservations.rows_affected());
    Ok(())
}
