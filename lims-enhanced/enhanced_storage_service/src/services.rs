use anyhow::Result;
use std::sync::Arc;
use tracing::{info, warn, error};
use uuid::Uuid;

use crate::{
    config::Config,
    database::DatabasePool,
    models::*,
    error::StorageError,
};

#[derive(Clone)]
pub struct EnhancedStorageService {
    pub db: DatabasePool,
    pub config: Arc<Config>,
}

impl EnhancedStorageService {
    pub async fn new(db: DatabasePool, config: Config) -> Result<Self> {
        info!("🏪 Initializing Enhanced Storage Service");

        let service = Self {
            db,
            config: Arc::new(config),
        };

        // Initialize the service
        service.initialize().await?;

        info!("✅ Enhanced Storage Service initialized successfully");
        Ok(service)
    }

    async fn initialize(&self) -> Result<()> {
        info!("🔧 Running service initialization...");

        // Test database connection
        self.db.test_connection().await?;
        info!("✅ Database connection verified");

        // Run database setup if needed
        if let Err(e) = self.db.run_migrations().await {
            warn!("⚠️ Database migrations failed: {}", e);
        }

        // Cleanup expired reservations
        if let Ok(expired_count) = self.db.cleanup_expired_reservations().await {
            if expired_count > 0 {
                info!("🧹 Cleaned up {} expired reservations", expired_count);
            }
        }

        // Validate configuration
        self.validate_config()?;

        info!("✅ Service initialization complete");
        Ok(())
    }

    fn validate_config(&self) -> Result<()> {
        info!("🔍 Validating service configuration...");

        // Validate feature flags
        if !self.config.features.enable_hierarchical_storage {
            warn!("⚠️ Hierarchical storage is disabled");
        }

        if !self.config.features.enable_sample_tracking {
            warn!("⚠️ Sample tracking is disabled");
        }

        // Validate storage configuration
        if self.config.storage.max_hierarchy_depth < 1 {
            return Err(anyhow::anyhow!("Invalid max hierarchy depth: must be >= 1"));
        }

        if self.config.storage.capacity_warning_threshold >= self.config.storage.capacity_critical_threshold {
            return Err(anyhow::anyhow!("Warning threshold must be less than critical threshold"));
        }

        info!("✅ Configuration validation complete");
        Ok(())
    }

    // ============================================================================
    // Storage Location Services
    // ============================================================================

    pub async fn create_storage_location(&self, request: CreateStorageLocationRequest) -> Result<StorageLocation, StorageError> {
        info!("Creating storage location: {}", request.name);

        let location_id = Uuid::new_v4();
        let metadata = request.metadata.unwrap_or_else(|| serde_json::json!({}));

        let location = sqlx::query_as::<_, StorageLocation>(
            r#"
            INSERT INTO storage_locations (id, name, description, location_type, temperature_zone, max_capacity, coordinates, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(location_id)
        .bind(&request.name)
        .bind(request.description.as_deref())
        .bind(&request.location_type)
        .bind(&request.temperature_zone)
        .bind(request.max_capacity)
        .bind(request.coordinates.as_ref())
        .bind(&metadata)
        .fetch_one(&self.db.pool)
        .await
        .map_err(StorageError::Database)?;

        info!("Storage location created with ID: {}", location.id);
        Ok(location)
    }

    pub async fn get_storage_location(&self, location_id: Uuid) -> Result<StorageLocation, StorageError> {
        let location = sqlx::query_as::<_, StorageLocation>(
            "SELECT * FROM storage_locations WHERE id = $1"
        )
        .bind(location_id)
        .fetch_optional(&self.db.pool)
        .await
        .map_err(StorageError::Database)?
        .ok_or_else(|| StorageError::NotFound("Storage location not found".to_string()))?;

        Ok(location)
    }

    pub async fn list_storage_locations(&self) -> Result<Vec<StorageLocation>, StorageError> {
        let locations = sqlx::query_as::<_, StorageLocation>(
            "SELECT * FROM storage_locations ORDER BY name"
        )
        .fetch_all(&self.db.pool)
        .await
        .map_err(StorageError::Database)?;

        Ok(locations)
    }

    pub async fn update_storage_location(&self, location_id: Uuid, request: UpdateStorageLocationRequest) -> Result<StorageLocation, StorageError> {
        // Build dynamic update query
        let mut query_builder = sqlx::QueryBuilder::new("UPDATE storage_locations SET updated_at = NOW()");
        let mut has_updates = false;

        if let Some(name) = &request.name {
            query_builder.push(", name = ");
            query_builder.push_bind(name);
            has_updates = true;
        }

        if let Some(description) = &request.description {
            query_builder.push(", description = ");
            query_builder.push_bind(description);
            has_updates = true;
        }

        if let Some(location_type) = &request.location_type {
            query_builder.push(", location_type = ");
            query_builder.push_bind(location_type);
            has_updates = true;
        }

        if let Some(temperature_zone) = &request.temperature_zone {
            query_builder.push(", temperature_zone = ");
            query_builder.push_bind(temperature_zone);
            has_updates = true;
        }

        if let Some(max_capacity) = request.max_capacity {
            query_builder.push(", max_capacity = ");
            query_builder.push_bind(max_capacity);
            has_updates = true;
        }

        if let Some(coordinates) = &request.coordinates {
            query_builder.push(", coordinates = ");
            query_builder.push_bind(coordinates);
            has_updates = true;
        }

        if let Some(status) = &request.status {
            query_builder.push(", status = ");
            query_builder.push_bind(status);
            has_updates = true;
        }

        if let Some(metadata) = &request.metadata {
            query_builder.push(", metadata = ");
            query_builder.push_bind(metadata);
            has_updates = true;
        }

        if !has_updates {
            return Err(StorageError::Validation("No updates provided".to_string()));
        }

        query_builder.push(" WHERE id = ");
        query_builder.push_bind(location_id);
        query_builder.push(" RETURNING *");

        let location = query_builder
            .build_query_as::<StorageLocation>()
            .fetch_optional(&self.db.pool)
            .await
            .map_err(StorageError::Database)?
            .ok_or_else(|| StorageError::NotFound("Storage location not found".to_string()))?;

        Ok(location)
    }

    pub async fn delete_storage_location(&self, location_id: Uuid) -> Result<(), StorageError> {
        // Check if location has containers
        let container_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM storage_containers WHERE location_id = $1"
        )
        .bind(location_id)
        .fetch_one(&self.db.pool)
        .await
        .map_err(StorageError::Database)?;

        if container_count > 0 {
            return Err(StorageError::Validation(
                "Cannot delete location with existing containers".to_string()
            ));
        }

        let result = sqlx::query("DELETE FROM storage_locations WHERE id = $1")
            .bind(location_id)
            .execute(&self.db.pool)
            .await
            .map_err(StorageError::Database)?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound("Storage location not found".to_string()));
        }

        Ok(())
    }

    // ============================================================================
    // Container Services
    // ============================================================================

    pub async fn create_container(&self, request: CreateStorageContainerRequest) -> Result<StorageContainer, StorageError> {
        self.create_storage_container(request).await
    }

    pub async fn create_storage_container(&self, request: CreateStorageContainerRequest) -> Result<StorageContainer, StorageError> {
        info!("Creating storage container: {} (type: {})", request.name, request.container_type);

        // Validate container type
        if !["freezer", "rack", "box", "position"].contains(&request.container_type.as_str()) {
            return Err(StorageError::Validation("Invalid container type".to_string()));
        }

        // Validate parent-child relationship
        if let Some(parent_id) = request.parent_container_id {
            let parent = self.get_storage_container(parent_id).await?;
            
            // Validate hierarchy rules
            match (parent.container_type.as_str(), request.container_type.as_str()) {
                ("freezer", "rack") | ("rack", "box") | ("box", "position") => {
                    // Valid hierarchy
                }
                _ => {
                    return Err(StorageError::Validation(
                        format!("Invalid hierarchy: {} cannot contain {}", parent.container_type, request.container_type)
                    ));
                }
            }
        }

        let container_id = Uuid::new_v4();
        let metadata = request.container_metadata.unwrap_or_else(|| serde_json::json!({}));
        let access_restrictions = request.access_restrictions.unwrap_or_else(|| serde_json::json!({}));

        let container = sqlx::query_as::<_, StorageContainer>(
            r#"
            INSERT INTO storage_containers (
                id, name, container_type, parent_container_id, location_id,
                grid_position, dimensions, capacity, temperature_zone, barcode,
                description, container_metadata, access_restrictions
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING *
            "#,
        )
        .bind(container_id)
        .bind(&request.name)
        .bind(&request.container_type)
        .bind(request.parent_container_id)
        .bind(request.location_id)
        .bind(request.grid_position.as_ref())
        .bind(request.dimensions.as_ref())
        .bind(request.capacity)
        .bind(request.temperature_zone.as_deref())
        .bind(request.barcode.as_deref())
        .bind(request.description.as_deref())
        .bind(&metadata)
        .bind(&access_restrictions)
        .fetch_one(&self.db.pool)
        .await
        .map_err(StorageError::Database)?;

        info!("Storage container created with ID: {}", container.id);
        Ok(container)
    }

    pub async fn get_container(&self, container_id: Uuid) -> Result<StorageContainer, StorageError> {
        self.get_storage_container(container_id).await
    }

    pub async fn get_storage_container(&self, container_id: Uuid) -> Result<StorageContainer, StorageError> {
        let container = sqlx::query_as::<_, StorageContainer>(
            "SELECT * FROM storage_containers WHERE id = $1"
        )
        .bind(container_id)
        .fetch_optional(&self.db.pool)
        .await
        .map_err(StorageError::Database)?
        .ok_or_else(|| StorageError::NotFound("Storage container not found".to_string()))?;

        Ok(container)
    }

    pub async fn list_containers(&self, query: serde_json::Value) -> Result<Vec<StorageContainer>, StorageError> {
        let mut sql_builder = sqlx::QueryBuilder::new("SELECT * FROM storage_containers WHERE 1=1");

        if let Some(container_type) = query.get("container_type").and_then(|v| v.as_str()) {
            sql_builder.push(" AND container_type = ");
            sql_builder.push_bind(container_type);
        }

        if let Some(temperature_zone) = query.get("temperature_zone").and_then(|v| v.as_str()) {
            sql_builder.push(" AND temperature_zone = ");
            sql_builder.push_bind(temperature_zone);
        }

        if let Some(parent_id) = query.get("parent_container_id").and_then(|v| v.as_str()) {
            if let Ok(uuid) = Uuid::parse_str(parent_id) {
                sql_builder.push(" AND parent_container_id = ");
                sql_builder.push_bind(uuid);
            }
        }

        sql_builder.push(" ORDER BY name");

        let containers = sql_builder
            .build_query_as::<StorageContainer>()
            .fetch_all(&self.db.pool)
            .await
            .map_err(StorageError::Database)?;

        Ok(containers)
    }

    pub async fn update_container(&self, container_id: Uuid, request: UpdateStorageContainerRequest) -> Result<StorageContainer, StorageError> {
        // Build dynamic update query
        let mut query_builder = sqlx::QueryBuilder::new("UPDATE storage_containers SET updated_at = NOW()");
        let mut has_updates = false;

        if let Some(name) = &request.name {
            query_builder.push(", name = ");
            query_builder.push_bind(name);
            has_updates = true;
        }

        if let Some(capacity) = request.capacity {
            query_builder.push(", capacity = ");
            query_builder.push_bind(capacity);
            has_updates = true;
        }

        if let Some(description) = &request.description {
            query_builder.push(", description = ");
            query_builder.push_bind(description);
            has_updates = true;
        }

        if let Some(status) = &request.status {
            query_builder.push(", status = ");
            query_builder.push_bind(status);
            has_updates = true;
        }

        if !has_updates {
            return Err(StorageError::Validation("No updates provided".to_string()));
        }

        query_builder.push(" WHERE id = ");
        query_builder.push_bind(container_id);
        query_builder.push(" RETURNING *");

        let container = query_builder
            .build_query_as::<StorageContainer>()
            .fetch_optional(&self.db.pool)
            .await
            .map_err(StorageError::Database)?
            .ok_or_else(|| StorageError::NotFound("Storage container not found".to_string()))?;

        Ok(container)
    }

    pub async fn delete_container(&self, container_id: Uuid) -> Result<(), StorageError> {
        // Check if container has children or samples
        let children_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM storage_containers WHERE parent_container_id = $1"
        ).bind(container_id).fetch_one(&self.db.pool).await.map_err(StorageError::Database)?;

        let samples_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sample_positions WHERE container_id = $1 AND removed_at IS NULL"
        ).bind(container_id).fetch_one(&self.db.pool).await.map_err(StorageError::Database)?;

        if children_count > 0 || samples_count > 0 {
            return Err(StorageError::Validation(
                "Cannot delete container with children or samples".to_string()
            ));
        }

        let result = sqlx::query("DELETE FROM storage_containers WHERE id = $1")
            .bind(container_id)
            .execute(&self.db.pool)
            .await
            .map_err(StorageError::Database)?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound("Storage container not found".to_string()));
        }

        Ok(())
    }

    pub async fn get_container_capacity(&self, container_id: Uuid) -> Result<serde_json::Value, StorageError> {
        let container = self.get_storage_container(container_id).await?;

        let children_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM storage_containers WHERE parent_container_id = $1"
        ).bind(container_id).fetch_one(&self.db.pool).await.map_err(StorageError::Database)?;

        let samples_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sample_positions WHERE container_id = $1 AND removed_at IS NULL"
        ).bind(container_id).fetch_one(&self.db.pool).await.map_err(StorageError::Database)?;

        Ok(serde_json::json!({
            "total_capacity": container.capacity,
            "occupied_count": container.occupied_count,
            "available_count": container.capacity - container.occupied_count,
            "utilization_percentage": if container.capacity > 0 {
                (container.occupied_count as f64 / container.capacity as f64) * 100.0
            } else {
                0.0
            },
            "capacity_status": self.get_capacity_status(&container),
            "child_containers_count": children_count,
            "direct_samples_count": samples_count,
        }))
    }

    pub async fn get_container_children(&self, container_id: Uuid) -> Result<Vec<StorageContainer>, StorageError> {
        let children = sqlx::query_as::<_, StorageContainer>(
            "SELECT * FROM storage_containers WHERE parent_container_id = $1 ORDER BY name"
        )
        .bind(container_id)
        .fetch_all(&self.db.pool)
        .await
        .map_err(StorageError::Database)?;

        Ok(children)
    }

    pub async fn get_storage_hierarchy(&self, container_id: Uuid) -> Result<serde_json::Value, StorageError> {
        // Get the container and build hierarchy
        let container = self.get_storage_container(container_id).await?;
        let children = self.get_container_children(container_id).await?;

        Ok(serde_json::json!({
            "container": container,
            "children": children,
            "hierarchy_level": self.get_container_level(container_id).await.unwrap_or(0),
            "path": self.get_container_path(container_id).await.unwrap_or_default()
        }))
    }

    pub async fn get_container_grid(&self, container_id: Uuid) -> Result<serde_json::Value, StorageError> {
        let container = self.get_storage_container(container_id).await?;
        
        if container.container_type != "box" {
            return Err(StorageError::Validation("Grid view only available for box containers".to_string()));
        }

        // Get all positions in this box
        let positions = sqlx::query_as::<_, StorageContainer>(
            "SELECT * FROM storage_containers WHERE parent_container_id = $1 AND container_type = 'position' ORDER BY name"
        )
        .bind(container_id)
        .fetch_all(&self.db.pool)
        .await
        .map_err(StorageError::Database)?;

        // Get sample assignments for these positions
        let position_ids: Vec<Uuid> = positions.iter().map(|p| p.id).collect();
        let samples = if !position_ids.is_empty() {
            sqlx::query_as::<_, SamplePosition>(
                "SELECT * FROM sample_positions WHERE container_id = ANY($1) AND removed_at IS NULL"
            )
            .bind(&position_ids)
            .fetch_all(&self.db.pool)
            .await
            .map_err(StorageError::Database)?
        } else {
            Vec::new()
        };

        Ok(serde_json::json!({
            "container": container,
            "positions": positions,
            "samples": samples,
            "grid_layout": "10x10" // Default layout
        }))
    }

    async fn get_container_level(&self, container_id: Uuid) -> Result<i32, StorageError> {
        let level: Option<i32> = sqlx::query_scalar(
            r#"
            WITH RECURSIVE container_hierarchy AS (
                SELECT id, parent_container_id, 0 as level
                FROM storage_containers
                WHERE id = $1
                
                UNION ALL
                
                SELECT c.id, c.parent_container_id, ch.level + 1
                FROM storage_containers c
                JOIN container_hierarchy ch ON c.id = ch.parent_container_id
            )
            SELECT MAX(level) FROM container_hierarchy
            "#
        )
        .bind(container_id)
        .fetch_optional(&self.db.pool)
        .await
        .map_err(StorageError::Database)?;

        Ok(level.unwrap_or(0))
    }

    async fn get_container_path(&self, container_id: Uuid) -> Result<Vec<String>, StorageError> {
        let path: Vec<(String,)> = sqlx::query_as(
            r#"
            WITH RECURSIVE container_path AS (
                SELECT id, parent_container_id, name, 0 as level
                FROM storage_containers
                WHERE id = $1
                
                UNION ALL
                
                SELECT c.id, c.parent_container_id, c.name, cp.level + 1
                FROM storage_containers c
                JOIN container_path cp ON c.id = cp.parent_container_id
            )
            SELECT name FROM container_path ORDER BY level DESC
            "#
        )
        .bind(container_id)
        .fetch_all(&self.db.pool)
        .await
        .map_err(StorageError::Database)?;

        Ok(path.into_iter().map(|(name,)| name).collect())
    }

    pub async fn get_container_with_children(&self, container_id: Uuid, include_samples: bool) -> Result<ContainerWithChildren, StorageError> {
        let container = self.get_storage_container(container_id).await?;

        // Get child containers
        let children = sqlx::query_as::<_, StorageContainer>(
            "SELECT * FROM storage_containers WHERE parent_container_id = $1 ORDER BY name"
        )
        .bind(container_id)
        .fetch_all(&self.db.pool)
        .await
        .map_err(StorageError::Database)?;

        // Get samples if requested
        let samples = if include_samples {
            sqlx::query_as::<_, SamplePosition>(
                "SELECT * FROM sample_positions WHERE container_id = $1 AND removed_at IS NULL"
            )
            .bind(container_id)
            .fetch_all(&self.db.pool)
            .await
            .map_err(StorageError::Database)?
        } else {
            Vec::new()
        };

        // Calculate capacity info
        let capacity_info = ContainerCapacityInfo {
            total_capacity: container.capacity,
            occupied_count: container.occupied_count,
            available_count: container.capacity - container.occupied_count,
            utilization_percentage: if container.capacity > 0 {
                (container.occupied_count as f64 / container.capacity as f64) * 100.0
            } else {
                0.0
            },
            capacity_status: self.get_capacity_status(&container),
            child_containers_count: children.len() as i32,
            direct_samples_count: samples.len() as i32,
        };

        Ok(ContainerWithChildren {
            container,
            children,
            samples,
            capacity_info,
        })
    }

    fn get_capacity_status(&self, container: &StorageContainer) -> String {
        if container.capacity > 0 {
            let utilization = container.occupied_count as f64 / container.capacity as f64;
            if utilization >= self.config.storage.capacity_critical_threshold {
                "critical".to_string()
            } else if utilization >= self.config.storage.capacity_warning_threshold {
                "warning".to_string()
            } else {
                "normal".to_string()
            }
        } else {
            "normal".to_string()
        }
    }

    // ============================================================================
    // Sample Services
    // ============================================================================

    pub async fn get_sample_movement_history(&self, sample_id: Uuid) -> Result<Vec<serde_json::Value>, StorageError> {
        let history = sqlx::query_as::<_, SamplePosition>(
            "SELECT * FROM sample_positions WHERE sample_id = $1 ORDER BY assigned_at DESC"
        )
        .bind(sample_id)
        .fetch_all(&self.db.pool)
        .await
        .map_err(StorageError::Database)?;

        let history_json: Vec<serde_json::Value> = history.into_iter()
            .map(|pos| serde_json::to_value(pos).unwrap_or_default())
            .collect();

        Ok(history_json)
    }

    pub async fn assign_sample_to_position(&self, request: AssignSampleToPositionRequest) -> Result<SamplePosition, StorageError> {
        info!("Assigning sample {} to position {}", request.sample_id, request.container_id);

        // Verify container is a position type
        let container = self.get_storage_container(request.container_id).await?;
        if container.container_type != "position" {
            return Err(StorageError::Validation(
                "Can only assign samples to position-type containers".to_string()
            ));
        }

        // Check if position is available
        let existing_assignment: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM sample_positions WHERE container_id = $1 AND removed_at IS NULL"
        )
        .bind(request.container_id)
        .fetch_optional(&self.db.pool)
        .await
        .map_err(StorageError::Database)?;

        if existing_assignment.is_some() {
            return Err(StorageError::Validation(
                "Position is already occupied".to_string()
            ));
        }

        // Check if sample is already assigned elsewhere
        let existing_sample: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM sample_positions WHERE sample_id = $1 AND removed_at IS NULL"
        )
        .bind(request.sample_id)
        .fetch_optional(&self.db.pool)
        .await
        .map_err(StorageError::Database)?;

        if existing_sample.is_some() {
            return Err(StorageError::Validation(
                "Sample is already assigned to another position".to_string()
            ));
        }

        let position_id = Uuid::new_v4();
        let storage_conditions = request.storage_conditions.unwrap_or_else(|| serde_json::json!({}));
        let special_requirements = request.special_requirements.unwrap_or_else(|| serde_json::json!({}));

        let sample_position = sqlx::query_as::<_, SamplePosition>(
            r#"
            INSERT INTO sample_positions (
                id, sample_id, container_id, position_identifier, assigned_by,
                storage_conditions, special_requirements, notes, chain_of_custody
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(position_id)
        .bind(request.sample_id)
        .bind(request.container_id)
        .bind(request.position_identifier.as_deref())
        .bind(request.assigned_by)
        .bind(&storage_conditions)
        .bind(&special_requirements)
        .bind(request.notes.as_deref())
        .bind(serde_json::json!([{
            "action": "assigned",
            "timestamp": chrono::Utc::now(),
            "assigned_by": request.assigned_by,
            "container_id": request.container_id
        }]))
        .fetch_one(&self.db.pool)
        .await
        .map_err(StorageError::Database)?;

        // Update container occupancy
        self.db.update_container_occupancy(request.container_id).await.map_err(StorageError::Anyhow)?;

        info!("Sample assigned to position successfully");
        Ok(sample_position)
    }

    pub async fn move_sample_to_position(&self, sample_id: Uuid, request: MoveSampleRequest) -> Result<SamplePosition, StorageError> {
        info!("Moving sample {} to new position {}", sample_id, request.new_container_id);

        // Get current position
        let current_position = sqlx::query_as::<_, SamplePosition>(
            "SELECT * FROM sample_positions WHERE sample_id = $1 AND removed_at IS NULL"
        )
        .bind(sample_id)
        .fetch_optional(&self.db.pool)
        .await
        .map_err(StorageError::Database)?
        .ok_or_else(|| StorageError::NotFound("Sample not found in storage".to_string()))?;

        // Verify new container is a position type
        let new_container = self.get_storage_container(request.new_container_id).await?;
        if new_container.container_type != "position" {
            return Err(StorageError::Validation(
                "Can only move samples to position-type containers".to_string()
            ));
        }

        // Check if new position is available
        let existing_assignment: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM sample_positions WHERE container_id = $1 AND removed_at IS NULL"
        )
        .bind(request.new_container_id)
        .fetch_optional(&self.db.pool)
        .await
        .map_err(StorageError::Database)?;

        if existing_assignment.is_some() {
            return Err(StorageError::Validation(
                "New position is already occupied".to_string()
            ));
        }

        // Mark old position as removed
        sqlx::query(
            "UPDATE sample_positions SET removed_at = NOW(), removed_by = $1 WHERE id = $2"
        )
        .bind(request.moved_by)
        .bind(current_position.id)
        .execute(&self.db.pool)
        .await
        .map_err(StorageError::Database)?;

        // Create new position assignment
        let new_position_id = Uuid::new_v4();
        let mut chain_of_custody = current_position.chain_of_custody.clone();
        
        // Add move event to chain of custody
        if let Some(array) = chain_of_custody.as_array_mut() {
            array.push(serde_json::json!({
                "action": "moved",
                "timestamp": chrono::Utc::now(),
                "moved_by": request.moved_by,
                "from_container_id": current_position.container_id,
                "to_container_id": request.new_container_id,
                "reason": request.reason
            }));
        }

        let new_sample_position = sqlx::query_as::<_, SamplePosition>(
            r#"
            INSERT INTO sample_positions (
                id, sample_id, container_id, position_identifier, assigned_by,
                storage_conditions, special_requirements, notes, chain_of_custody
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(new_position_id)
        .bind(sample_id)
        .bind(request.new_container_id)
        .bind(request.new_position_identifier.as_deref())
        .bind(request.moved_by)
        .bind(&current_position.storage_conditions)
        .bind(&current_position.special_requirements)
        .bind(request.notes.as_deref())
        .bind(&chain_of_custody)
        .fetch_one(&self.db.pool)
        .await
        .map_err(StorageError::Database)?;

        // Update container occupancy for both containers
        self.db.update_container_occupancy(current_position.container_id).await.map_err(StorageError::Anyhow)?;
        self.db.update_container_occupancy(request.new_container_id).await.map_err(StorageError::Anyhow)?;

        info!("Sample moved to new position successfully");
        Ok(new_sample_position)
    }

    pub async fn remove_sample_from_position(&self, sample_id: Uuid, removed_by: Option<Uuid>) -> Result<(), StorageError> {
        info!("Removing sample {} from position", sample_id);

        // Get current position
        let current_position = sqlx::query_as::<_, SamplePosition>(
            "SELECT * FROM sample_positions WHERE sample_id = $1 AND removed_at IS NULL"
        )
        .bind(sample_id)
        .fetch_optional(&self.db.pool)
        .await
        .map_err(StorageError::Database)?
        .ok_or_else(|| StorageError::NotFound("Sample not found in storage".to_string()))?;

        // Mark position as removed
        let result = sqlx::query(
            "UPDATE sample_positions SET removed_at = NOW(), removed_by = $1, status = 'retrieved' WHERE sample_id = $2 AND removed_at IS NULL"
        )
        .bind(removed_by)
        .bind(sample_id)
        .execute(&self.db.pool)
        .await
        .map_err(StorageError::Database)?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound("Sample not found in storage".to_string()));
        }

        // Update container occupancy
        self.db.update_container_occupancy(current_position.container_id).await.map_err(StorageError::Anyhow)?;

        info!("Sample removed from position successfully");
        Ok(())
    }

    // ============================================================================
    // Search and Analytics Services
    // ============================================================================

    pub async fn search_storage_items(&self, request: serde_json::Value) -> Result<Vec<serde_json::Value>, StorageError> {
        // Placeholder implementation for search
        Ok(vec![serde_json::json!({
            "message": "Search functionality not yet implemented",
            "query": request
        })])
    }

    // ============================================================================
    // Bulk Operations Services
    // ============================================================================

    pub async fn bulk_assign_samples(&self, request: serde_json::Value) -> Result<serde_json::Value, StorageError> {
        // Placeholder implementation for bulk assign
        Ok(serde_json::json!({
            "message": "Bulk assign not yet implemented",
            "request": request
        }))
    }

    pub async fn bulk_move_samples(&self, request: serde_json::Value) -> Result<serde_json::Value, StorageError> {
        // Placeholder implementation for bulk move
        Ok(serde_json::json!({
            "message": "Bulk move not yet implemented",
            "request": request
        }))
    }

    pub async fn bulk_create_containers(&self, request: serde_json::Value) -> Result<serde_json::Value, StorageError> {
        // Placeholder implementation for bulk create
        Ok(serde_json::json!({
            "message": "Bulk create not yet implemented",
            "request": request
        }))
    }

    // ============================================================================
    // Advanced Analytics Services
    // ============================================================================

    pub async fn get_usage_analytics(&self) -> Result<serde_json::Value, StorageError> {
        // Placeholder implementation for usage analytics
        Ok(serde_json::json!({
            "message": "Usage analytics not yet implemented"
        }))
    }

    pub async fn get_trend_analytics(&self) -> Result<serde_json::Value, StorageError> {
        // Placeholder implementation for trend analytics
        Ok(serde_json::json!({
            "message": "Trend analytics not yet implemented"
        }))
    }

    pub async fn generate_inventory_report(&self) -> Result<serde_json::Value, StorageError> {
        // Placeholder implementation for inventory report
        Ok(serde_json::json!({
            "message": "Inventory report not yet implemented"
        }))
    }

    pub async fn generate_audit_report(&self) -> Result<serde_json::Value, StorageError> {
        // Placeholder implementation for audit report
        Ok(serde_json::json!({
            "message": "Audit report not yet implemented"
        }))
    }

    // ============================================================================
    // Analytics and Reporting Services
    // ============================================================================

    pub async fn get_capacity_summary(&self) -> Result<Vec<StorageCapacitySummary>, StorageError> {
        let summary = sqlx::query_as::<_, StorageCapacitySummary>(
            r#"
            SELECT 
                id, name, container_type, capacity, occupied_count,
                capacity - occupied_count as available_count,
                ROUND((occupied_count::float / NULLIF(capacity, 0)) * 100, 2) as utilization_percentage,
                CASE 
                    WHEN occupied_count::float / NULLIF(capacity, 0) >= 0.95 THEN 'critical'
                    WHEN occupied_count::float / NULLIF(capacity, 0) >= 0.80 THEN 'warning'
                    ELSE 'normal'
                END as capacity_status,
                temperature_zone, status, created_at, updated_at
            FROM storage_containers 
            WHERE capacity > 0
            ORDER BY utilization_percentage DESC
            "#
        )
        .fetch_all(&self.db.pool)
        .await
        .map_err(StorageError::Database)?;

        Ok(summary)
    }

    pub async fn get_utilization_report(&self) -> Result<serde_json::Value, StorageError> {
        let total_capacity: i64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(capacity), 0) FROM storage_containers"
        ).fetch_one(&self.db.pool).await.map_err(StorageError::Database)?;

        let total_occupied: i64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(occupied_count), 0) FROM storage_containers"
        ).fetch_one(&self.db.pool).await.map_err(StorageError::Database)?;

        let utilization_by_type: Vec<(String, i64, i64, f64)> = sqlx::query_as(
            r#"
            SELECT 
                container_type,
                COALESCE(SUM(capacity), 0) as total_capacity,
                COALESCE(SUM(occupied_count), 0) as total_occupied,
                ROUND((COALESCE(SUM(occupied_count), 0)::float / NULLIF(COALESCE(SUM(capacity), 0), 0)) * 100, 2) as utilization_percentage
            FROM storage_containers 
            GROUP BY container_type
            ORDER BY utilization_percentage DESC
            "#
        ).fetch_all(&self.db.pool).await.map_err(StorageError::Database)?;

        Ok(serde_json::json!({
            "overall": {
                "total_capacity": total_capacity,
                "total_occupied": total_occupied,
                "total_available": total_capacity - total_occupied,
                "utilization_percentage": if total_capacity > 0 {
                    (total_occupied as f64 / total_capacity as f64) * 100.0
                } else { 0.0 }
            },
            "by_container_type": utilization_by_type
        }))
    }

    // ============================================================================
    // Health and Maintenance Services
    // ============================================================================

    pub async fn health_check(&self) -> Result<serde_json::Value, StorageError> {
        let db_health = self.db.get_health_info().await?;
        let db_stats = self.db.get_database_stats().await?;

        Ok(serde_json::json!({
            "service": "enhanced-storage-service",
            "status": "healthy",
            "version": "1.0.0",
            "features": {
                "hierarchical_storage": self.config.features.enable_hierarchical_storage,
                "sample_tracking": self.config.features.enable_sample_tracking,
                "capacity_management": self.config.features.enable_capacity_management,
                "analytics": self.config.features.enable_analytics,
                "bulk_operations": self.config.features.enable_bulk_operations,
                "audit_logging": self.config.features.enable_audit_logging,
            },
            "database": db_health,
            "statistics": db_stats,
            "timestamp": chrono::Utc::now()
        }))
    }

    pub async fn cleanup_expired_reservations(&self) -> Result<i64, StorageError> {
        self.db.cleanup_expired_reservations().await.map_err(StorageError::Anyhow)
    }

    pub async fn update_all_container_occupancy(&self) -> Result<(), StorageError> {
        info!("🔄 Updating all container occupancy counts...");
        
        sqlx::query(
            r#"
            UPDATE storage_containers 
            SET occupied_count = (
                SELECT COUNT(*) 
                FROM sample_positions 
                WHERE container_id = storage_containers.id AND removed_at IS NULL
            )
            "#
        )
        .execute(&self.db.pool)
        .await
        .map_err(StorageError::Database)?;

        info!("✅ All container occupancy counts updated");
        Ok(())
    }
}
