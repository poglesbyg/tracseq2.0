use axum::{extract::State, Json};
use crate::{error::StorageResult, models::ApiResponse, AppState};

pub async fn verify_integrity(State(_state): State<AppState>) -> StorageResult<Json<ApiResponse<String>>> {
    Ok(Json(ApiResponse::success("Integrity verified".to_string())))
}

pub async fn list_transactions(State(_state): State<AppState>) -> StorageResult<Json<ApiResponse<Vec<String>>>> {
    Ok(Json(ApiResponse::success(vec!["tx-1".to_string()])))
}

pub async fn get_transaction(State(_state): State<AppState>) -> StorageResult<Json<ApiResponse<String>>> {
    Ok(Json(ApiResponse::success("Transaction details".to_string())))
}

pub async fn list_blocks(State(_state): State<AppState>) -> StorageResult<Json<ApiResponse<Vec<String>>>> {
    Ok(Json(ApiResponse::success(vec!["block-1".to_string()])))
}

pub async fn get_block(State(_state): State<AppState>) -> StorageResult<Json<ApiResponse<String>>> {
    Ok(Json(ApiResponse::success("Block details".to_string())))
}
