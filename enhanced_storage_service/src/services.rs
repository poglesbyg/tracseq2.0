use crate::{
    config::Config,
    database::DatabasePool,
    error::{StorageError, StorageResult},
    models::*,
};
use anyhow::Result;
use chrono::Utc;
use sqlx::Row;
use tracing::{info, warn, error};
use uuid::Uuid;

#[derive(Clone)]
pub struct EnhancedStorageService {
    pub db: DatabasePool,
    pub config: Config,
}

impl EnhancedStorageService {
    pub async fn new(db: DatabasePool, config: Config) -> Result<Self> {
        info!("Initializing Enhanced Storage Service");

        let service = Self { db, config };

        // Initialize default storage locations if needed
        service.initialize_default_locations().await?;

        info!("Enhanced Storage Service initialized successfully");
        Ok(service)
    }

    // ============================================================================
    // Storage Location Management
    // ============================================================================

    pub async fn create_storage_location(
        &self,
        request: CreateStorageLocationRequest,
    ) -> StorageResult<StorageLocation> {
        info!("Creating storage location: {}", request.name);

        let location_id = Uuid::new_v4();
        let metadata = request.metadata.unwrap_or_else(|| serde_json::json!({}));

        let location = sqlx::query_as::<_, StorageLocation>(
            r#"
            INSERT INTO storage_locations (
                id, name, description, location_type, temperature_zone, 
                max_capacity, coordinates, metadata
            )
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
        .map_err(|e| {
            error!("Failed to create storage location: {}", e);
            StorageError::Database(e)
        })?;

        info!("Storage location created with ID: {}", location.id);
        Ok(location)
    }

    pub async fn get_storage_location(&self, location_id: Uuid) -> StorageResult<StorageLocation> {
        let location = sqlx::query_as::<_, StorageLocation>(
            "SELECT * FROM storage_locations WHERE id = $1"
        )
        .bind(location_id)
        .fetch_optional(&self.db.pool)
        .await?
        .ok_or_else(|| StorageError::LocationNotFound(location_id.to_string()))?;

        Ok(location)
    }

    pub async fn list_storage_locations(
        &self,
        page: Option<i32>,
        per_page: Option<i32>,
    ) -> StorageResult<PaginatedResponse<StorageLocation>> {
        let page = page.unwrap_or(1);
        let per_page = per_page.unwrap_or(20);
        let offset = (page - 1) * per_page;

        let locations = sqlx::query_as::<_, StorageLocation>(
            r#"
            SELECT * FROM storage_locations 
            ORDER BY created_at DESC 
            LIMIT $1 OFFSET $2
            "#
        )
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.db.pool)
        .await?;

        let total_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM storage_locations"
        )
        .fetch_one(&self.db.pool)
        .await?;

        let total_pages = (total_count as i32 + per_page - 1) / per_page;

        Ok(PaginatedResponse {
            data: locations,
            pagination: PaginationInfo {
                page,
                per_page,
                total_pages,
                total_items: total_count,
                has_next: page < total_pages,
                has_prev: page > 1,
            },
        })
    }

    pub async fn get_location_capacity(&self, location_id: Uuid) -> StorageResult<f64> {
        let row = sqlx::query(
            r#"
            SELECT 
                max_capacity,
                current_capacity,
                (current_capacity::float / max_capacity::float) as utilization
            FROM storage_locations 
            WHERE id = $1
            "#
        )
        .bind(location_id)
        .fetch_optional(&self.db.pool)
        .await?
        .ok_or_else(|| StorageError::LocationNotFound(location_id.to_string()))?;

        let utilization: f64 = row.get("utilization");
        Ok(utilization)
    }

    // ============================================================================
    // Sample Management
    // ============================================================================

    pub async fn store_sample(&self, request: StoreSampleRequest) -> StorageResult<Sample> {
        info!("Storing sample with barcode: {}", request.barcode);

        // Check if location exists and has capacity
        let location = self.get_storage_location(request.storage_location_id).await?;
        
        if location.current_capacity >= location.max_capacity {
            return Err(StorageError::CapacityExceeded);
        }

        // Validate temperature requirements
        if let Some(temp_req) = &request.temperature_requirements {
            self.validate_temperature_compatibility(temp_req, &location.temperature_zone)?;
        }

        let sample_id = Uuid::new_v4();
        let metadata = request.metadata.unwrap_or_else(|| serde_json::json!({}));
        let chain_of_custody = serde_json::json!([{
            "action": "stored",
            "timestamp": Utc::now(),
            "location_id": request.storage_location_id,
            "user_id": null // Would be set from auth context
        }]);

        // Start transaction
        let mut tx = self.db.pool.begin().await?;

        // Insert sample
        let sample = sqlx::query_as::<_, Sample>(
            r#"
            INSERT INTO samples (
                id, barcode, sample_type, storage_location_id, position,
                temperature_requirements, metadata, chain_of_custody, stored_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(sample_id)
        .bind(&request.barcode)
        .bind(&request.sample_type)
        .bind(request.storage_location_id)
        .bind(request.position.as_ref())
        .bind(request.temperature_requirements.as_deref())
        .bind(&metadata)
        .bind(&chain_of_custody)
        .bind(Utc::now())
        .fetch_one(&mut *tx)
        .await?;

        // Update location capacity
        sqlx::query(
            "UPDATE storage_locations SET current_capacity = current_capacity + 1 WHERE id = $1"
        )
        .bind(request.storage_location_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        info!("Sample stored successfully with ID: {}", sample.id);
        Ok(sample)
    }

    pub async fn get_sample_location(&self, sample_id: Uuid) -> StorageResult<Option<StorageLocation>> {
        let location = sqlx::query_as::<_, StorageLocation>(
            r#"
            SELECT l.* FROM storage_locations l
            JOIN samples s ON l.id = s.storage_location_id
            WHERE s.id = $1
            "#
        )
        .bind(sample_id)
        .fetch_optional(&self.db.pool)
        .await?;

        Ok(location)
    }

    pub async fn move_sample(
        &self,
        sample_id: Uuid,
        request: MoveSampleRequest,
    ) -> StorageResult<Sample> {
        info!("Moving sample {} to location {}", sample_id, request.new_location_id);

        // Get current sample
        let sample = sqlx::query_as::<_, Sample>(
            "SELECT * FROM samples WHERE id = $1"
        )
        .bind(sample_id)
        .fetch_optional(&self.db.pool)
        .await?
        .ok_or_else(|| StorageError::SampleNotFound(sample_id.to_string()))?;

        // Check new location capacity
        let new_location = self.get_storage_location(request.new_location_id).await?;
        if new_location.current_capacity >= new_location.max_capacity {
            return Err(StorageError::CapacityExceeded);
        }

        // Start transaction
        let mut tx = self.db.pool.begin().await?;

        // Update sample location
        let updated_sample = sqlx::query_as::<_, Sample>(
            r#"
            UPDATE samples 
            SET storage_location_id = $1, position = $2, 
                chain_of_custody = chain_of_custody || $3,
                updated_at = NOW()
            WHERE id = $4
            RETURNING *
            "#,
        )
        .bind(request.new_location_id)
        .bind(request.new_position.as_ref())
        .bind(serde_json::json!({
            "action": "moved",
            "timestamp": Utc::now(),
            "from_location_id": sample.storage_location_id,
            "to_location_id": request.new_location_id,
            "reason": request.reason
        }))
        .bind(sample_id)
        .fetch_one(&mut *tx)
        .await?;

        // Update location capacities
        if let Some(old_location_id) = sample.storage_location_id {
            sqlx::query(
                "UPDATE storage_locations SET current_capacity = current_capacity - 1 WHERE id = $1"
            )
            .bind(old_location_id)
            .execute(&mut *tx)
            .await?;
        }

        sqlx::query(
            "UPDATE storage_locations SET current_capacity = current_capacity + 1 WHERE id = $1"
        )
        .bind(request.new_location_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        info!("Sample moved successfully");
        Ok(updated_sample)
    }

    // ============================================================================
    // IoT Integration
    // ============================================================================

    pub async fn register_sensor(&self, sensor_id: String, sensor_type: String, location_id: Option<Uuid>) -> StorageResult<IoTSensor> {
        info!("Registering IoT sensor: {}", sensor_id);

        let id = Uuid::new_v4();
        let sensor = sqlx::query_as::<_, IoTSensor>(
            r#"
            INSERT INTO iot_sensors (id, sensor_id, sensor_type, location_id)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&sensor_id)
        .bind(&sensor_type)
        .bind(location_id)
        .fetch_one(&self.db.pool)
        .await?;

        info!("IoT sensor registered with ID: {}", sensor.id);
        Ok(sensor)
    }

    pub async fn record_sensor_data(&self, reading: SensorReading) -> StorageResult<()> {
        // Get sensor by sensor_id
        let sensor = sqlx::query_as::<_, IoTSensor>(
            "SELECT * FROM iot_sensors WHERE sensor_id = $1"
        )
        .bind(&reading.sensor_id)
        .fetch_optional(&self.db.pool)
        .await?
        .ok_or_else(|| StorageError::IoTSensorError(format!("Sensor not found: {}", reading.sensor_id)))?;

        // Insert sensor readings
        for reading_value in &reading.readings {
            sqlx::query(
                r#"
                INSERT INTO sensor_data (sensor_id, reading_type, value, unit, quality_score, recorded_at)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
            )
            .bind(sensor.id)
            .bind(&reading_value.reading_type)
            .bind(reading_value.value)
            .bind(&reading_value.unit)
            .bind(reading_value.quality_score.unwrap_or(1.0))
            .bind(reading.timestamp)
            .execute(&self.db.pool)
            .await?;

            // Check for alerts
            self.check_sensor_alerts(&sensor, reading_value).await?;
        }

        // Update sensor last reading
        sqlx::query(
            "UPDATE iot_sensors SET last_reading = $1, updated_at = NOW() WHERE id = $2"
        )
        .bind(serde_json::to_value(&reading.readings)?)
        .bind(sensor.id)
        .execute(&self.db.pool)
        .await?;

        Ok(())
    }

    // ============================================================================
    // Private Helper Methods
    // ============================================================================

    async fn initialize_default_locations(&self) -> Result<()> {
        info!("Initializing default storage locations");

        for temp_zone in &self.config.storage.default_temperature_zones {
            let location_name = format!("Storage-{}", temp_zone);
            
            // Check if location already exists
            let exists = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM storage_locations WHERE name = $1)"
            )
            .bind(&location_name)
            .fetch_one(&self.db.pool)
            .await?;

            if !exists {
                let request = CreateStorageLocationRequest {
                    name: location_name,
                    description: Some(format!("Default {} storage location", temp_zone)),
                    location_type: "rack".to_string(),
                    temperature_zone: temp_zone.clone(),
                    max_capacity: 100,
                    coordinates: None,
                    metadata: None,
                };

                self.create_storage_location(request).await?;
            }
        }

        info!("Default storage locations initialized");
        Ok(())
    }

    fn validate_temperature_compatibility(
        &self,
        temp_requirement: &str,
        zone_temp: &str,
    ) -> StorageResult<()> {
        // Simple temperature compatibility check
        if temp_requirement != zone_temp {
            return Err(StorageError::TemperatureViolation(
                format!("Sample requires {} but location is {}", temp_requirement, zone_temp)
            ));
        }
        Ok(())
    }

    async fn check_sensor_alerts(
        &self,
        sensor: &IoTSensor,
        reading: &SensorReadingValue,
    ) -> StorageResult<()> {
        let mut should_alert = false;
        let mut alert_message = String::new();

        match reading.reading_type.as_str() {
            "temperature" => {
                if (reading.value - self.parse_zone_temperature(&sensor))
                    .abs() > self.config.iot.alert_threshold_temperature as f64
                {
                    should_alert = true;
                    alert_message = format!(
                        "Temperature deviation detected: {} {}",
                        reading.value, reading.unit
                    );
                }
            }
            "humidity" => {
                if reading.value > (50.0 + self.config.iot.alert_threshold_humidity as f64)
                    || reading.value < (50.0 - self.config.iot.alert_threshold_humidity as f64)
                {
                    should_alert = true;
                    alert_message = format!(
                        "Humidity out of range: {} {}",
                        reading.value, reading.unit
                    );
                }
            }
            _ => {}
        }

        if should_alert {
            self.create_alert(CreateAlertRequest {
                alert_type: "environmental".to_string(),
                severity: AlertSeverity::High,
                title: "Environmental Alert".to_string(),
                message: alert_message,
                source_type: "sensor".to_string(),
                source_id: Some(sensor.id),
                metadata: None,
            }).await?;
        }

        Ok(())
    }

    async fn create_alert(&self, request: CreateAlertRequest) -> StorageResult<Alert> {
        let alert_id = Uuid::new_v4();
        let severity_str = match request.severity {
            AlertSeverity::Low => "low",
            AlertSeverity::Medium => "medium", 
            AlertSeverity::High => "high",
            AlertSeverity::Critical => "critical",
        };

        let alert = sqlx::query_as::<_, Alert>(
            r#"
            INSERT INTO alerts (
                id, alert_type, severity, title, message, source_type, source_id, metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(alert_id)
        .bind(&request.alert_type)
        .bind(severity_str)
        .bind(&request.title)
        .bind(&request.message)
        .bind(&request.source_type)
        .bind(request.source_id)
        .bind(request.metadata.unwrap_or_else(|| serde_json::json!({})))
        .fetch_one(&self.db.pool)
        .await?;

        Ok(alert)
    }

    fn parse_zone_temperature(&self, sensor: &IoTSensor) -> f64 {
        // Parse temperature zone to numeric value
        // This is a simplified implementation
        match sensor.sensor_type.as_str() {
            "temperature" => match sensor.location_id {
                Some(_) => {
                    // Would lookup actual location temperature zone
                    // For now, return a default
                    -20.0
                }
                None => 20.0,
            }
            _ => 20.0,
        }
    }
}
