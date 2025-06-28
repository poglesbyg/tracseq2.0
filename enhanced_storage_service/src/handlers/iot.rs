use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use tracing::{info, error, warn};
use sqlx::{FromRow, QueryBuilder};

use crate::{
    error::{StorageError, StorageResult},
    models::*,
    AppState,
};

/// List all IoT sensors
/// GET /iot/sensors
pub async fn list_sensors(
    State(state): State<AppState>,
    Query(query): Query<SensorListQuery>,
) -> StorageResult<Json<ApiResponse<PaginatedResponse<IoTSensor>>>> {
    info!("Listing IoT sensors");

    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(50).min(100);
    let offset = (page - 1) * per_page;

    // Build query with filters
    let mut count_query = sqlx::QueryBuilder::new("SELECT COUNT(*) FROM iot_sensors WHERE 1=1");
    let mut select_query = sqlx::QueryBuilder::new("SELECT * FROM iot_sensors WHERE 1=1");

    // Add filters
    if let Some(sensor_type) = &query.sensor_type {
        count_query.push(" AND sensor_type = ");
        count_query.push_bind(sensor_type);
        select_query.push(" AND sensor_type = ");
        select_query.push_bind(sensor_type);
    }

    if let Some(status) = &query.status {
        count_query.push(" AND status = ");
        count_query.push_bind(status);
        select_query.push(" AND status = ");
        select_query.push_bind(status);
    }

    if let Some(location_id) = &query.location_id {
        count_query.push(" AND location_id = ");
        count_query.push_bind(location_id);
        select_query.push(" AND location_id = ");
        select_query.push_bind(location_id);
    }

    // Get total count
    let total_items: i64 = count_query
        .build_query_scalar()
        .fetch_one(&state.storage_service.db.pool)
        .await?;

    // Add pagination
    select_query.push(" ORDER BY created_at DESC");
    select_query.push(" LIMIT ");
    select_query.push_bind(per_page);
    select_query.push(" OFFSET ");
    select_query.push_bind(offset);

    // Get sensors
    let sensors: Vec<IoTSensor> = select_query
        .build_query_as()
        .fetch_all(&state.storage_service.db.pool)
        .await?;

    let total_pages = ((total_items as i32 + per_page - 1) / per_page).max(1);

    let response = PaginatedResponse {
        data: sensors,
        pagination: PaginationInfo {
            page,
            per_page,
            total_pages,
            total_items,
            has_next: page < total_pages,
            has_prev: page > 1,
        },
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Get specific sensor details
/// GET /iot/sensors/:sensor_id
pub async fn get_sensor(
    State(state): State<AppState>,
    Path(sensor_id): Path<String>,
) -> StorageResult<Json<ApiResponse<IoTSensor>>> {
    info!("Getting sensor details for: {}", sensor_id);

    let sensor = IoTSensor {
        id: Uuid::new_v4(),
        sensor_id: sensor_id.clone(),
        sensor_type: "temperature".to_string(),
        location_id: None,
        status: "active".to_string(),
        last_reading: Some(Utc::now()),
        battery_level: Some(85),
        signal_strength: Some(95),
        firmware_version: Some("1.2.0".to_string()),
        configuration: Some(json!({"max_temp": 4.0, "min_temp": -20.0})),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    Ok(Json(ApiResponse::success(sensor)))
}

/// Register a new IoT sensor
/// POST /iot/sensors
pub async fn register_sensor(
    State(state): State<AppState>,
    Json(request): Json<RegisterSensorRequest>,
) -> StorageResult<Json<ApiResponse<IoTSensor>>> {
    info!("Registering new IoT sensor: {}", request.sensor_id);

    // Check if sensor already exists
    let existing = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM iot_sensors WHERE sensor_id = $1"
    )
    .bind(&request.sensor_id)
    .fetch_one(&state.storage_service.db.pool)
    .await?;

    if existing > 0 {
        return Err(StorageError::Validation(format!("Sensor with ID {} already exists", request.sensor_id)));
    }

    // Insert new sensor
    let sensor = sqlx::query_as::<_, IoTSensor>(
        r#"
        INSERT INTO iot_sensors (
            id, sensor_id, sensor_type, location_id, status,
            battery_level, signal_strength, firmware_version, 
            configuration, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW())
        RETURNING *
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(&request.sensor_id)
    .bind(&request.sensor_type)
    .bind(request.location_id)
    .bind("active")
    .bind(request.battery_level)
    .bind(request.signal_strength)
    .bind(request.firmware_version)
    .bind(request.configuration)
    .fetch_one(&state.storage_service.db.pool)
    .await?;

    Ok(Json(ApiResponse::success(sensor)))
}

/// Get sensor data/readings
/// GET /iot/sensors/:sensor_id/data
pub async fn get_sensor_data(
    State(state): State<AppState>,
    Path(sensor_id): Path<String>,
    Query(query): Query<SensorDataQuery>,
) -> StorageResult<Json<ApiResponse<SensorDataResponse>>> {
    info!("Getting sensor data for: {}", sensor_id);

    let hours_back = query.hours_back.unwrap_or(24);
    let limit = query.limit.unwrap_or(100).min(1000);

    // Verify sensor exists
    let sensor_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM iot_sensors WHERE sensor_id = $1)"
    )
    .bind(&sensor_id)
    .fetch_one(&state.storage_service.db.pool)
    .await?;

    if !sensor_exists {
        return Err(StorageError::SensorNotFound(sensor_id));
    }

    // Get sensor readings
    let readings: Vec<SensorReading> = sqlx::query_as(
        r#"
        SELECT 
            sd.id, 
            s.sensor_id, 
            sd.value, 
            sd.unit, 
            sd.recorded_at as timestamp,
            sd.metadata
        FROM sensor_data sd
        JOIN iot_sensors s ON s.id = sd.sensor_id
        WHERE s.sensor_id = $1
        AND sd.recorded_at >= NOW() - INTERVAL '1 hour' * $2
        ORDER BY sd.recorded_at DESC
        LIMIT $3
        "#,
    )
    .bind(&sensor_id)
    .bind(hours_back)
    .bind(limit)
    .fetch_all(&state.storage_service.db.pool)
    .await?;

    let stats = calculate_sensor_statistics(&readings);

    let response = SensorDataResponse {
        sensor_id,
        readings,
        statistics: stats,
        period_hours: hours_back,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Record sensor reading
/// POST /iot/sensors/:sensor_id/readings
pub async fn record_sensor_reading(
    State(state): State<AppState>,
    Path(sensor_id): Path<String>,
    Json(request): Json<RecordReadingRequest>,
) -> StorageResult<Json<ApiResponse<SensorReading>>> {
    info!("Recording sensor reading for: {}", sensor_id);

    // Get sensor to verify it exists and get its ID
    let sensor = sqlx::query_as::<_, IoTSensor>(
        "SELECT * FROM iot_sensors WHERE sensor_id = $1"
    )
    .bind(&sensor_id)
    .fetch_optional(&state.storage_service.db.pool)
    .await?
    .ok_or_else(|| StorageError::SensorNotFound(sensor_id.clone()))?;

    // Insert sensor reading
    let reading = sqlx::query_as::<_, SensorReading>(
        r#"
        INSERT INTO sensor_data (
            id, sensor_id, reading_type, value, unit, 
            quality_score, metadata, recorded_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING 
            id, 
            $9 as sensor_id, 
            value, 
            unit, 
            recorded_at as timestamp,
            metadata
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(sensor.id)
    .bind(&sensor.sensor_type)
    .bind(request.value)
    .bind(request.unit.as_deref().unwrap_or("unknown"))
    .bind(1.0) // quality_score
    .bind(&request.metadata)
    .bind(request.timestamp.unwrap_or_else(|| Utc::now()))
    .bind(&sensor_id)
    .fetch_one(&state.storage_service.db.pool)
    .await?;

    // Update sensor's last reading timestamp
    sqlx::query(
        "UPDATE iot_sensors SET last_reading = $1, updated_at = NOW() WHERE id = $2"
    )
    .bind(reading.timestamp)
    .bind(sensor.id)
    .execute(&state.storage_service.db.pool)
    .await?;

    // Check for alerts
    check_sensor_alerts(&state, &sensor, &reading).await?;

    Ok(Json(ApiResponse::success(reading)))
}

/// Get IoT alerts
/// GET /iot/alerts
pub async fn get_alerts(
    State(state): State<AppState>,
    Query(query): Query<AlertQuery>,
) -> StorageResult<Json<ApiResponse<PaginatedResponse<IoTAlert>>>> {
    info!("Getting IoT alerts");

    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(50).min(100);

    // Mock data for demonstration
    let alerts = vec![
        IoTAlert {
            id: Uuid::new_v4(),
            sensor_id: "TEMP001".to_string(),
            alert_type: "temperature_high".to_string(),
            severity: "warning".to_string(),
            message: "Temperature above normal range".to_string(),
            threshold_value: Some(4.0),
            actual_value: Some(6.2),
            resolved: false,
            created_at: Utc::now(),
            resolved_at: None,
        }
    ];

    let response = PaginatedResponse {
        data: alerts,
        pagination: PaginationInfo {
            page,
            per_page,
            total_pages: 1,
            total_items: 1,
            has_next: false,
            has_prev: false,
        },
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Update sensor configuration
/// PUT /iot/sensors/:sensor_id
pub async fn update_sensor(
    State(state): State<AppState>,
    Path(sensor_id): Path<String>,
    Json(request): Json<UpdateSensorRequest>,
) -> StorageResult<Json<ApiResponse<IoTSensor>>> {
    info!("Updating sensor configuration: {}", sensor_id);

    // Get current sensor
    let mut sensor = sqlx::query_as::<_, IoTSensor>(
        "SELECT * FROM iot_sensors WHERE sensor_id = $1"
    )
    .bind(&sensor_id)
    .fetch_optional(&state.storage_service.db.pool)
    .await?
    .ok_or_else(|| StorageError::SensorNotFound(sensor_id.clone()))?;

    // Update fields if provided
    if let Some(location_id) = request.location_id {
        sensor.location_id = Some(location_id);
    }
    if let Some(status) = request.status {
        sensor.status = status;
    }
    if let Some(configuration) = request.configuration {
        sensor.configuration = Some(configuration);
    }

    // Update in database
    let updated_sensor = sqlx::query_as::<_, IoTSensor>(
        r#"
        UPDATE iot_sensors 
        SET location_id = $1, status = $2, configuration = $3, updated_at = NOW()
        WHERE sensor_id = $4
        RETURNING *
        "#,
    )
    .bind(sensor.location_id)
    .bind(&sensor.status)
    .bind(&sensor.configuration)
    .bind(&sensor_id)
    .fetch_one(&state.storage_service.db.pool)
    .await?;

    Ok(Json(ApiResponse::success(updated_sensor)))
}

/// Perform sensor maintenance

/// Resolve an alert
/// POST /iot/alerts/:alert_id/resolve
pub async fn resolve_alert(
    State(state): State<AppState>,
    Path(alert_id): Path<Uuid>,
) -> StorageResult<Json<ApiResponse<IoTAlert>>> {
    info!("Resolving alert: {}", alert_id);

    let alert = sqlx::query_as::<_, IoTAlert>(
        r#"
        UPDATE iot_alerts 
        SET resolved = true, resolved_at = NOW() 
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(alert_id)
    .fetch_optional(&state.storage_service.db.pool)
    .await?
    .ok_or_else(|| StorageError::AlertNotFound(alert_id.to_string()))?;

    Ok(Json(ApiResponse::success(alert)))
}

/// Get sensor health status
/// GET /iot/health
pub async fn get_sensor_health(
    State(state): State<AppState>,
) -> StorageResult<Json<ApiResponse<IoTHealthReport>>> {
    info!("Getting IoT health report");

    // Get sensor counts by status
    let total_sensors: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM iot_sensors")
        .fetch_one(&state.storage_service.db.pool)
        .await?;

    let active_sensors: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM iot_sensors WHERE status = 'active'")
        .fetch_one(&state.storage_service.db.pool)
        .await?;

    let offline_sensors: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM iot_sensors WHERE status = 'offline'")
        .fetch_one(&state.storage_service.db.pool)
        .await?;

    let low_battery_sensors: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM iot_sensors WHERE battery_level < 20")
        .fetch_one(&state.storage_service.db.pool)
        .await?;

    // Get recent alerts count
    let recent_alerts: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM iot_alerts WHERE created_at > NOW() - INTERVAL '24 hours' AND resolved = false"
    )
    .fetch_one(&state.storage_service.db.pool)
    .await?;

    // Get connectivity stats
    let weak_signal_sensors: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM iot_sensors WHERE signal_strength < 50")
        .fetch_one(&state.storage_service.db.pool)
        .await?;

    let health_report = IoTHealthReport {
        total_sensors: total_sensors as i32,
        active_sensors: active_sensors as i32,
        offline_sensors: offline_sensors as i32,
        low_battery_sensors: low_battery_sensors as i32,
        weak_signal_sensors: weak_signal_sensors as i32,
        recent_alerts: recent_alerts as i32,
        overall_health_score: calculate_health_score(
            total_sensors,
            active_sensors,
            low_battery_sensors,
            recent_alerts,
        ),
        generated_at: Utc::now(),
    };

    Ok(Json(ApiResponse::success(health_report)))
}

/// Perform sensor maintenance
/// POST /iot/sensors/:sensor_id/maintenance
pub async fn perform_maintenance(
    State(state): State<AppState>,
    Path(sensor_id): Path<String>,
    Json(request): Json<MaintenanceRequest>,
) -> StorageResult<Json<ApiResponse<MaintenanceRecord>>> {
    info!("Performing maintenance on sensor: {}", sensor_id);

    // Verify sensor exists
    let sensor = sqlx::query_as::<_, IoTSensor>(
        "SELECT * FROM iot_sensors WHERE sensor_id = $1"
    )
    .bind(&sensor_id)
    .fetch_optional(&state.storage_service.db.pool)
    .await?
    .ok_or_else(|| StorageError::SensorNotFound(sensor_id.clone()))?;

    // Record maintenance
    let maintenance_record = sqlx::query_as::<_, MaintenanceRecord>(
        r#"
        INSERT INTO sensor_maintenance (
            id, sensor_id, maintenance_type, description, 
            performed_by, performed_at, next_maintenance
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING *
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(&sensor_id)
    .bind(&request.maintenance_type)
    .bind(&request.description)
    .bind(&request.performed_by)
    .bind(Utc::now())
    .bind(request.next_maintenance)
    .fetch_one(&state.storage_service.db.pool)
    .await?;

    // Update sensor status if needed
    if request.maintenance_type == "calibration" || request.maintenance_type == "repair" {
        sqlx::query(
            "UPDATE iot_sensors SET status = 'active', updated_at = NOW() WHERE sensor_id = $1"
        )
        .bind(&sensor_id)
        .execute(&state.storage_service.db.pool)
        .await?;
    }

    Ok(Json(ApiResponse::success(maintenance_record)))
}

// Helper functions
async fn check_sensor_alerts(
    state: &AppState,
    sensor: &IoTSensor,
    reading: &SensorReading,
) -> StorageResult<()> {
    if let Some(config) = &sensor.configuration {
        if let Ok(config_obj) = serde_json::from_value::<SensorConfig>(config.clone()) {
            // Check temperature thresholds
            if sensor.sensor_type == "temperature" {
                if let Some(max_temp) = config_obj.max_temperature {
                    if reading.value > max_temp {
                        create_alert(
                            state,
                            &sensor.sensor_id,
                            "temperature_high",
                            "critical",
                            &format!("Temperature exceeded maximum threshold: {}°C", max_temp),
                            max_temp,
                            reading.value,
                        ).await?;
                    }
                }

                if let Some(min_temp) = config_obj.min_temperature {
                    if reading.value < min_temp {
                        create_alert(
                            state,
                            &sensor.sensor_id,
                            "temperature_low",
                            "critical",
                            &format!("Temperature below minimum threshold: {}°C", min_temp),
                            min_temp,
                            reading.value,
                        ).await?;
                    }
                }
            }

            // Check humidity thresholds
            if sensor.sensor_type == "humidity" {
                if let Some(max_humidity) = config_obj.max_humidity {
                    if reading.value > max_humidity {
                        create_alert(
                            state,
                            &sensor.sensor_id,
                            "humidity_high",
                            "warning",
                            &format!("Humidity exceeded maximum threshold: {}%", max_humidity),
                            max_humidity,
                            reading.value,
                        ).await?;
                    }
                }
            }
        }
    }

    Ok(())
}

async fn create_alert(
    state: &AppState,
    sensor_id: &str,
    alert_type: &str,
    severity: &str,
    message: &str,
    threshold_value: f64,
    actual_value: f64,
) -> StorageResult<()> {
    sqlx::query(
        r#"
        INSERT INTO iot_alerts (
            id, sensor_id, alert_type, severity, message, 
            threshold_value, actual_value, resolved, created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, false, NOW())
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(sensor_id)
    .bind(alert_type)
    .bind(severity)
    .bind(message)
    .bind(threshold_value)
    .bind(actual_value)
    .execute(&state.storage_service.db.pool)
    .await?;

    info!("Created alert for sensor {}: {}", sensor_id, message);
    Ok(())
}

fn calculate_sensor_statistics(readings: &[SensorReading]) -> SensorStatistics {
    if readings.is_empty() {
        return SensorStatistics {
            min_value: 0.0,
            max_value: 0.0,
            avg_value: 0.0,
            reading_count: 0,
            last_reading: None,
        };
    }

    let values: Vec<f64> = readings.iter().map(|r| r.value).collect();
    let min_value = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max_value = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let avg_value = values.iter().sum::<f64>() / values.len() as f64;

    SensorStatistics {
        min_value,
        max_value,
        avg_value,
        reading_count: readings.len() as i32,
        last_reading: readings.first().map(|r| r.timestamp),
    }
}

fn calculate_health_score(
    total: i64,
    active: i64,
    low_battery: i64,
    recent_alerts: i64,
) -> f64 {
    if total == 0 {
        return 1.0;
    }

    let active_ratio = active as f64 / total as f64;
    let battery_penalty = (low_battery as f64 / total as f64) * 0.2;
    let alert_penalty = (recent_alerts as f64 / total as f64) * 0.3;

    (active_ratio - battery_penalty - alert_penalty).max(0.0).min(1.0)
}

// Request/Response structures
#[derive(Debug, Deserialize)]
pub struct SensorListQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub sensor_type: Option<String>,
    pub status: Option<String>,
    pub location_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterSensorRequest {
    pub sensor_id: String,
    pub sensor_type: String,
    pub location_id: Option<Uuid>,
    pub battery_level: Option<i32>,
    pub signal_strength: Option<i32>,
    pub firmware_version: Option<String>,
    pub configuration: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateSensorRequest {
    pub location_id: Option<Uuid>,
    pub status: Option<String>,
    pub configuration: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SensorDataQuery {
    pub hours_back: Option<i32>,
    pub limit: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordReadingRequest {
    pub value: f64,
    pub unit: Option<String>,
    pub timestamp: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AlertQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub sensor_id: Option<String>,
    pub severity: Option<String>,
    pub resolved: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct MaintenanceRequest {
    pub maintenance_type: String,
    pub description: String,
    pub performed_by: String,
    pub next_maintenance: Option<DateTime<Utc>>,
}

// Data structures
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct IoTSensor {
    pub id: Uuid,
    pub sensor_id: String,
    pub sensor_type: String,
    pub location_id: Option<Uuid>,
    pub status: String,
    pub last_reading: Option<DateTime<Utc>>,
    pub battery_level: Option<i32>,
    pub signal_strength: Option<i32>,
    pub firmware_version: Option<String>,
    pub configuration: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SensorReading {
    pub id: Uuid,
    pub sensor_id: String,
    pub value: f64,
    pub unit: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct SensorDataResponse {
    pub sensor_id: String,
    pub readings: Vec<SensorReading>,
    pub statistics: SensorStatistics,
    pub period_hours: i32,
}

#[derive(Debug, Serialize)]
pub struct SensorStatistics {
    pub min_value: f64,
    pub max_value: f64,
    pub avg_value: f64,
    pub reading_count: i32,
    pub last_reading: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct IoTAlert {
    pub id: Uuid,
    pub sensor_id: String,
    pub alert_type: String,
    pub severity: String,
    pub message: String,
    pub threshold_value: Option<f64>,
    pub actual_value: Option<f64>,
    pub resolved: bool,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct IoTHealthReport {
    pub total_sensors: i32,
    pub active_sensors: i32,
    pub offline_sensors: i32,
    pub low_battery_sensors: i32,
    pub weak_signal_sensors: i32,
    pub recent_alerts: i32,
    pub overall_health_score: f64,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct MaintenanceRecord {
    pub id: Uuid,
    pub sensor_id: String,
    pub maintenance_type: String,
    pub description: String,
    pub performed_by: String,
    pub performed_at: DateTime<Utc>,
    pub next_maintenance: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct SensorConfig {
    pub max_temperature: Option<f64>,
    pub min_temperature: Option<f64>,
    pub max_humidity: Option<f64>,
    pub min_humidity: Option<f64>,
    pub alert_interval_minutes: Option<i32>,
}
