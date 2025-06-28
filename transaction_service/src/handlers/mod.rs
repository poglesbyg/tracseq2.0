//! Transaction Service HTTP handlers

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    coordinator::{TransactionRequest, TransactionStatus},
    AppState,
};

/// Health check handler
pub async fn health_check() -> Json<Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "transaction-service",
        "timestamp": chrono::Utc::now()
    }))
}

/// Get transaction status handler
pub async fn get_transaction_status(
    State(app_state): State<AppState>,
    Path(saga_id): Path<Uuid>,
) -> Result<Json<TransactionStatus>, StatusCode> {
    match app_state.coordinator.get_transaction_status(saga_id).await {
        Some(status) => Ok(Json(status)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Execute transaction handler
pub async fn execute_transaction(
    State(app_state): State<AppState>,
    Json(request): Json<TransactionRequest>,
) -> Result<Json<crate::saga::SagaExecutionResult>, StatusCode> {
    let saga = crate::saga::TransactionSaga::builder(&request.name)
        .with_timeout(request.timeout_ms.unwrap_or(300000))
        .build();

    match app_state.coordinator.execute_transaction(request, saga).await {
        Ok(result) => Ok(Json(result)),
        Err(e) => {
            tracing::error!("Transaction execution failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// List active transactions handler
pub async fn list_active_transactions(
    State(app_state): State<AppState>,
) -> Json<Vec<TransactionStatus>> {
    let transactions = app_state.coordinator.list_active_transactions().await;
    Json(transactions)
}

/// Cancel transaction handler
pub async fn cancel_transaction(
    State(app_state): State<AppState>,
    Path(saga_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    match app_state.coordinator.cancel_transaction(saga_id).await {
        Ok(_) => Ok(Json(serde_json::json!({
            "message": "Transaction cancelled successfully",
            "saga_id": saga_id
        }))),
        Err(e) => {
            tracing::error!("Failed to cancel transaction {}: {}", saga_id, e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check() {
        let response = health_check().await;
        let value = response.0;
        
        assert_eq!(value["status"], "healthy");
        assert_eq!(value["service"], "transaction-service");
        assert!(value["timestamp"].is_string());
    }
}