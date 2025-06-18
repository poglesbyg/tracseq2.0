use axum::{extract::State, Json};
use crate::{error::StorageResult, models::ApiResponse, AppState};

pub async fn list_sensors(State(_state): State<AppState>) -> StorageResult<Json<ApiResponse<Vec<String>>>> {
    Ok(Json(ApiResponse::success(vec!["sensor-1".to_string()])))
}

pub async fn get_sensor_data(State(_state): State<AppState>) -> StorageResult<Json<ApiResponse<String>>> {
    Ok(Json(ApiResponse::success("Sensor data".to_string())))
}

pub async fn get_alerts(State(_state): State<AppState>) -> StorageResult<Json<ApiResponse<Vec<String>>>> {
    Ok(Json(ApiResponse::success(vec!["No alerts".to_string()])))
}
