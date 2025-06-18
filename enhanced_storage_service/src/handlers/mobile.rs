use axum::{extract::State, Json};
use crate::{error::StorageResult, models::ApiResponse, AppState};

pub async fn mobile_login(State(_state): State<AppState>) -> StorageResult<Json<ApiResponse<String>>> {
    Ok(Json(ApiResponse::success("Mobile login successful".to_string())))
}

pub async fn get_nearby_locations(State(_state): State<AppState>) -> StorageResult<Json<ApiResponse<Vec<String>>>> {
    Ok(Json(ApiResponse::success(vec!["Location A".to_string()])))
}

pub async fn scan_barcode(State(_state): State<AppState>) -> StorageResult<Json<ApiResponse<String>>> {
    Ok(Json(ApiResponse::success("Barcode scanned".to_string())))
}

pub async fn get_tasks(State(_state): State<AppState>) -> StorageResult<Json<ApiResponse<Vec<String>>>> {
    Ok(Json(ApiResponse::success(vec!["Task 1".to_string()])))
}

pub async fn complete_task(State(_state): State<AppState>) -> StorageResult<Json<ApiResponse<String>>> {
    Ok(Json(ApiResponse::success("Task completed".to_string())))
}
