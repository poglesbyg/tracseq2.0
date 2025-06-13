use async_trait::async_trait;
use sqlx::PgPool;

use crate::models::storage::{
    SampleLocation, StorageCapacityStats, StorageLocation, StorageMovementHistory, StorageState,
    TemperatureZone,
};

/// Storage repository trait for database operations
#[async_trait]
pub trait StorageRepository: Send + Sync {
    /// Storage Location Operations
    async fn create_storage_location(
        &self,
        location: CreateStorageLocation,
    ) -> Result<StorageLocation, sqlx::Error>;
    async fn get_storage_location(&self, id: i32) -> Result<Option<StorageLocation>, sqlx::Error>;
    async fn get_all_storage_locations(&self) -> Result<Vec<StorageLocation>, sqlx::Error>;
    async fn get_storage_locations_by_temperature(
        &self,
        temp_zone: TemperatureZone,
    ) -> Result<Vec<StorageLocation>, sqlx::Error>;
    async fn update_storage_location(
        &self,
        id: i32,
        updates: UpdateStorageLocation,
    ) -> Result<StorageLocation, sqlx::Error>;
    async fn update_location_usage(
        &self,
        location_id: i32,
        usage_change: i32,
    ) -> Result<(), sqlx::Error>;

    /// Sample Location Operations
    async fn store_sample(
        &self,
        sample_location: CreateSampleLocation,
    ) -> Result<SampleLocation, sqlx::Error>;
    async fn get_sample_location(
        &self,
        sample_id: i32,
    ) -> Result<Option<SampleLocation>, sqlx::Error>;
    async fn get_sample_by_barcode(
        &self,
        barcode: &str,
    ) -> Result<Option<SampleLocation>, sqlx::Error>;
    async fn get_samples_in_location(
        &self,
        location_id: i32,
    ) -> Result<Vec<SampleLocation>, sqlx::Error>;
    async fn move_sample(
        &self,
        sample_id: i32,
        new_location_id: i32,
        moved_by: &str,
        reason: &str,
    ) -> Result<SampleLocation, sqlx::Error>;
    async fn update_sample_state(
        &self,
        sample_id: i32,
        new_state: StorageState,
        updated_by: &str,
    ) -> Result<SampleLocation, sqlx::Error>;

    /// Remove a sample from storage
    async fn remove_sample(
        &self,
        sample_id: i32,
        removed_by: &str,
        reason: &str,
    ) -> Result<SampleLocation, sqlx::Error>;

    /// Movement History Operations
    async fn record_movement(
        &self,
        movement: CreateMovementHistory,
    ) -> Result<StorageMovementHistory, sqlx::Error>;
    async fn get_sample_movement_history(
        &self,
        sample_id: i32,
    ) -> Result<Vec<StorageMovementHistory>, sqlx::Error>;

    /// Capacity and Statistics
    async fn get_storage_capacity_stats(&self) -> Result<Vec<StorageCapacityStats>, sqlx::Error>;
    async fn get_location_capacity_stats(
        &self,
        location_id: i32,
    ) -> Result<Option<StorageCapacityStats>, sqlx::Error>;

    /// Barcode Operations
    async fn is_barcode_unique(&self, barcode: &str) -> Result<bool, sqlx::Error>;
    async fn reserve_barcode(&self, barcode: &str, _sample_id: i32) -> Result<(), sqlx::Error>;
}

/// Create storage location data
#[derive(Debug, Clone)]
pub struct CreateStorageLocation {
    pub name: String,
    pub description: Option<String>,
    pub temperature_zone: TemperatureZone,
    pub capacity: i32,
    pub container_type: crate::models::storage::ContainerType,
    pub location_path: Option<String>,
}

/// Update storage location data
#[derive(Debug, Clone)]
pub struct UpdateStorageLocation {
    pub name: Option<String>,
    pub description: Option<String>,
    pub capacity: Option<i32>,
    pub is_active: Option<bool>,
    pub location_path: Option<String>,
}

/// Create sample location data
#[derive(Debug, Clone)]
pub struct CreateSampleLocation {
    pub sample_id: i32,
    pub location_id: i32,
    pub barcode: String,
    pub position: Option<String>,
    pub storage_state: StorageState,
    pub stored_by: Option<String>,
    pub notes: Option<String>,
}

/// Create movement history data
#[derive(Debug, Clone)]
pub struct CreateMovementHistory {
    pub sample_id: i32,
    pub barcode: String,
    pub from_location_id: Option<i32>,
    pub to_location_id: i32,
    pub from_state: Option<StorageState>,
    pub to_state: StorageState,
    pub movement_reason: String,
    pub moved_by: String,
    pub notes: Option<String>,
}

/// PostgreSQL implementation of storage repository
pub struct PostgresStorageRepository {
    pool: PgPool,
}

impl PostgresStorageRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl StorageRepository for PostgresStorageRepository {
    async fn create_storage_location(
        &self,
        location: CreateStorageLocation,
    ) -> Result<StorageLocation, sqlx::Error> {
        sqlx::query_as::<_, StorageLocation>(
            r#"
            INSERT INTO storage_locations (name, description, temperature_zone, capacity, current_usage, container_type, location_path)
            VALUES ($1, $2, $3, $4, 0, $5, $6)
            RETURNING *
            "#
        )
        .bind(&location.name)
        .bind(&location.description)
        .bind(&location.temperature_zone)
        .bind(location.capacity)
        .bind(&location.container_type)
        .bind(&location.location_path)
        .fetch_one(&self.pool)
        .await
    }

    async fn get_storage_location(&self, id: i32) -> Result<Option<StorageLocation>, sqlx::Error> {
        sqlx::query_as::<_, StorageLocation>("SELECT * FROM storage_locations WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    async fn get_all_storage_locations(&self) -> Result<Vec<StorageLocation>, sqlx::Error> {
        sqlx::query_as::<_, StorageLocation>(
            "SELECT * FROM storage_locations WHERE is_active = true ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await
    }

    async fn get_storage_locations_by_temperature(
        &self,
        temp_zone: TemperatureZone,
    ) -> Result<Vec<StorageLocation>, sqlx::Error> {
        sqlx::query_as::<_, StorageLocation>(
            "SELECT * FROM storage_locations WHERE temperature_zone = $1 AND is_active = true ORDER BY name"
        )
        .bind(&temp_zone)
        .fetch_all(&self.pool)
        .await
    }

    async fn update_storage_location(
        &self,
        id: i32,
        updates: UpdateStorageLocation,
    ) -> Result<StorageLocation, sqlx::Error> {
        let mut query_parts = Vec::new();
        let mut param_count = 1;

        if updates.name.is_some() {
            query_parts.push(format!("name = ${}", param_count));
            param_count += 1;
        }
        if updates.description.is_some() {
            query_parts.push(format!("description = ${}", param_count));
            param_count += 1;
        }
        if updates.capacity.is_some() {
            query_parts.push(format!("capacity = ${}", param_count));
            param_count += 1;
        }
        if updates.is_active.is_some() {
            query_parts.push(format!("is_active = ${}", param_count));
            param_count += 1;
        }
        if updates.location_path.is_some() {
            query_parts.push(format!("location_path = ${}", param_count));
            param_count += 1;
        }

        if query_parts.is_empty() {
            return self
                .get_storage_location(id)
                .await?
                .ok_or(sqlx::Error::RowNotFound);
        }

        let query = format!(
            "UPDATE storage_locations SET {}, updated_at = NOW() WHERE id = ${} RETURNING *",
            query_parts.join(", "),
            param_count
        );

        let mut query_builder = sqlx::query_as::<_, StorageLocation>(&query);

        if let Some(name) = updates.name {
            query_builder = query_builder.bind(name);
        }
        if let Some(description) = updates.description {
            query_builder = query_builder.bind(description);
        }
        if let Some(capacity) = updates.capacity {
            query_builder = query_builder.bind(capacity);
        }
        if let Some(is_active) = updates.is_active {
            query_builder = query_builder.bind(is_active);
        }
        if let Some(location_path) = updates.location_path {
            query_builder = query_builder.bind(location_path);
        }

        query_builder = query_builder.bind(id);
        query_builder.fetch_one(&self.pool).await
    }

    async fn update_location_usage(
        &self,
        location_id: i32,
        usage_change: i32,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE storage_locations SET current_usage = current_usage + $1, updated_at = NOW() WHERE id = $2"
        )
        .bind(usage_change)
        .bind(location_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn store_sample(
        &self,
        sample_location: CreateSampleLocation,
    ) -> Result<SampleLocation, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        // Insert sample location
        let stored_sample = sqlx::query_as::<_, SampleLocation>(
            r#"
            INSERT INTO sample_locations (sample_id, location_id, barcode, position, storage_state, stored_by, notes)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#
        )
        .bind(sample_location.sample_id)
        .bind(sample_location.location_id)
        .bind(&sample_location.barcode)
        .bind(&sample_location.position)
        .bind(&sample_location.storage_state)
        .bind(&sample_location.stored_by)
        .bind(&sample_location.notes)
        .fetch_one(&mut *tx)
        .await?;

        // Update location usage
        sqlx::query(
            "UPDATE storage_locations SET current_usage = current_usage + 1, updated_at = NOW() WHERE id = $1"
        )
        .bind(sample_location.location_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(stored_sample)
    }

    async fn get_sample_location(
        &self,
        sample_id: i32,
    ) -> Result<Option<SampleLocation>, sqlx::Error> {
        sqlx::query_as::<_, SampleLocation>(
            "SELECT * FROM sample_locations WHERE sample_id = $1 ORDER BY created_at DESC LIMIT 1",
        )
        .bind(sample_id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn get_sample_by_barcode(
        &self,
        barcode: &str,
    ) -> Result<Option<SampleLocation>, sqlx::Error> {
        sqlx::query_as::<_, SampleLocation>(
            "SELECT * FROM sample_locations WHERE barcode = $1 ORDER BY created_at DESC LIMIT 1",
        )
        .bind(barcode)
        .fetch_optional(&self.pool)
        .await
    }

    async fn get_samples_in_location(
        &self,
        location_id: i32,
    ) -> Result<Vec<SampleLocation>, sqlx::Error> {
        sqlx::query_as::<_, SampleLocation>(
            "SELECT * FROM sample_locations WHERE location_id = $1 ORDER BY stored_at DESC",
        )
        .bind(location_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn move_sample(
        &self,
        sample_id: i32,
        new_location_id: i32,
        moved_by: &str,
        reason: &str,
    ) -> Result<SampleLocation, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        // Get current sample location
        let current_location = sqlx::query_as::<_, SampleLocation>(
            "SELECT * FROM sample_locations WHERE sample_id = $1 ORDER BY created_at DESC LIMIT 1",
        )
        .bind(sample_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;

        // Create movement history record
        sqlx::query(
            r#"
            INSERT INTO storage_movement_history (sample_id, barcode, from_location_id, to_location_id, from_state, to_state, movement_reason, moved_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#
        )
        .bind(sample_id)
        .bind(&current_location.barcode)
        .bind(current_location.location_id)
        .bind(new_location_id)
        .bind(&current_location.storage_state)
        .bind(&current_location.storage_state)
        .bind(reason)
        .bind(moved_by)
        .execute(&mut *tx)
        .await?;

        // Update sample location
        let updated_sample = sqlx::query_as::<_, SampleLocation>(
            r#"
            UPDATE sample_locations 
            SET location_id = $1, moved_at = NOW(), moved_by = $2, updated_at = NOW()
            WHERE sample_id = $3
            RETURNING *
            "#,
        )
        .bind(new_location_id)
        .bind(moved_by)
        .bind(sample_id)
        .fetch_one(&mut *tx)
        .await?;

        // Update location usage counts
        sqlx::query(
            "UPDATE storage_locations SET current_usage = current_usage - 1, updated_at = NOW() WHERE id = $1"
        )
        .bind(current_location.location_id)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            "UPDATE storage_locations SET current_usage = current_usage + 1, updated_at = NOW() WHERE id = $1"
        )
        .bind(new_location_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(updated_sample)
    }

    async fn update_sample_state(
        &self,
        sample_id: i32,
        new_state: StorageState,
        updated_by: &str,
    ) -> Result<SampleLocation, sqlx::Error> {
        let updated_sample = sqlx::query_as::<_, SampleLocation>(
            r#"
            UPDATE sample_locations 
            SET storage_state = $1, moved_by = $2, moved_at = NOW(), updated_at = NOW()
            WHERE sample_id = $3
            RETURNING *
            "#,
        )
        .bind(&new_state)
        .bind(updated_by)
        .bind(sample_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(updated_sample)
    }

    async fn remove_sample(
        &self,
        sample_id: i32,
        removed_by: &str,
        reason: &str,
    ) -> Result<SampleLocation, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        // Get current sample location before deletion
        let current_location = sqlx::query_as::<_, SampleLocation>(
            "SELECT * FROM sample_locations WHERE sample_id = $1 ORDER BY created_at DESC LIMIT 1",
        )
        .bind(sample_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;

        // Create movement history record indicating removal
        sqlx::query(
            r#"
            INSERT INTO storage_movement_history (sample_id, barcode, from_location_id, to_location_id, from_state, to_state, movement_reason, moved_by)
            VALUES ($1, $2, $3, NULL, $4, $5, $6, $7)
            "#
        )
        .bind(sample_id)
        .bind(&current_location.barcode)
        .bind(current_location.location_id)
        .bind(&current_location.storage_state)
        .bind(StorageState::Discarded)
        .bind(reason)
        .bind(removed_by)
        .execute(&mut *tx)
        .await?;

        // Delete the sample location record
        sqlx::query("DELETE FROM sample_locations WHERE sample_id = $1")
            .bind(sample_id)
            .execute(&mut *tx)
            .await?;

        // Update location usage count
        sqlx::query(
            "UPDATE storage_locations SET current_usage = current_usage - 1, updated_at = NOW() WHERE id = $1"
        )
        .bind(current_location.location_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(current_location)
    }

    async fn record_movement(
        &self,
        movement: CreateMovementHistory,
    ) -> Result<StorageMovementHistory, sqlx::Error> {
        sqlx::query_as::<_, StorageMovementHistory>(
            r#"
            INSERT INTO storage_movement_history (sample_id, barcode, from_location_id, to_location_id, from_state, to_state, movement_reason, moved_by, notes)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#
        )
        .bind(movement.sample_id)
        .bind(&movement.barcode)
        .bind(movement.from_location_id)
        .bind(movement.to_location_id)
        .bind(&movement.from_state)
        .bind(&movement.to_state)
        .bind(&movement.movement_reason)
        .bind(&movement.moved_by)
        .bind(&movement.notes)
        .fetch_one(&self.pool)
        .await
    }

    async fn get_sample_movement_history(
        &self,
        sample_id: i32,
    ) -> Result<Vec<StorageMovementHistory>, sqlx::Error> {
        sqlx::query_as::<_, StorageMovementHistory>(
            "SELECT * FROM storage_movement_history WHERE sample_id = $1 ORDER BY moved_at DESC",
        )
        .bind(sample_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn get_storage_capacity_stats(&self) -> Result<Vec<StorageCapacityStats>, sqlx::Error> {
        let locations = self.get_all_storage_locations().await?;
        let mut stats = Vec::new();

        for location in locations {
            let sample_count = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM sample_locations WHERE location_id = $1",
            )
            .bind(location.id)
            .fetch_one(&self.pool)
            .await? as i32;

            stats.push(StorageCapacityStats {
                location_id: location.id,
                location_name: location.name.clone(),
                temperature_zone: location.temperature_zone,
                total_capacity: location.capacity,
                current_usage: location.current_usage,
                available_capacity: location.available_capacity(),
                utilization_percentage: location.utilization_percentage(),
                sample_count,
                is_near_capacity: location.is_near_capacity(85.0), // 85% threshold
            });
        }

        Ok(stats)
    }

    async fn get_location_capacity_stats(
        &self,
        location_id: i32,
    ) -> Result<Option<StorageCapacityStats>, sqlx::Error> {
        if let Some(location) = self.get_storage_location(location_id).await? {
            let sample_count = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM sample_locations WHERE location_id = $1",
            )
            .bind(location_id)
            .fetch_one(&self.pool)
            .await? as i32;

            Ok(Some(StorageCapacityStats {
                location_id: location.id,
                location_name: location.name.clone(),
                temperature_zone: location.temperature_zone,
                total_capacity: location.capacity,
                current_usage: location.current_usage,
                available_capacity: location.available_capacity(),
                utilization_percentage: location.utilization_percentage(),
                sample_count,
                is_near_capacity: location.is_near_capacity(85.0),
            }))
        } else {
            Ok(None)
        }
    }

    async fn is_barcode_unique(&self, barcode: &str) -> Result<bool, sqlx::Error> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM sample_locations WHERE barcode = $1",
        )
        .bind(barcode)
        .fetch_one(&self.pool)
        .await?;

        Ok(count == 0)
    }

    async fn reserve_barcode(&self, barcode: &str, _sample_id: i32) -> Result<(), sqlx::Error> {
        // This could insert into a barcode reservation table if needed
        // For now, we'll just verify it's unique when storing the sample
        if !self.is_barcode_unique(barcode).await? {
            return Err(sqlx::Error::Protocol("Barcode already exists".to_string()));
        }
        Ok(())
    }
}
