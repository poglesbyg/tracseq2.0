use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{AppState, error::{Result, SequencingError}};

pub async fn create_run(
    State(_state): State<AppState>,
    Json(_payload): Json<Value>,
) -> Result<(StatusCode, Json<Value>)> {
    Err(SequencingError::NotImplemented("create_run not implemented yet".to_string()))
}

pub async fn list_runs(
    State(_state): State<AppState>,
) -> Result<Json<Value>> {
    Err(SequencingError::NotImplemented("list_runs not implemented yet".to_string()))
}

pub async fn get_run(
    State(_state): State<AppState>,
    Path(_run_id): Path<Uuid>,
) -> Result<Json<Value>> {
    Err(SequencingError::NotImplemented("get_run not implemented yet".to_string()))
}

pub async fn update_run(
    State(_state): State<AppState>,
    Path(_run_id): Path<Uuid>,
    Json(_payload): Json<Value>,
) -> Result<Json<Value>> {
    Err(SequencingError::NotImplemented("update_run not implemented yet".to_string()))
}

pub async fn delete_run(
    State(_state): State<AppState>,
    Path(_run_id): Path<Uuid>,
) -> Result<StatusCode> {
    Err(SequencingError::NotImplemented("delete_run not implemented yet".to_string()))
}

pub async fn start_run(
    State(_state): State<AppState>,
    Path(_run_id): Path<Uuid>,
) -> Result<Json<Value>> {
    Err(SequencingError::NotImplemented("start_run not implemented yet".to_string()))
}

pub async fn stop_run(
    State(_state): State<AppState>,
    Path(_run_id): Path<Uuid>,
) -> Result<Json<Value>> {
    Err(SequencingError::NotImplemented("stop_run not implemented yet".to_string()))
}

pub async fn get_run_metrics(
    State(_state): State<AppState>,
    Path(_run_id): Path<Uuid>,
) -> Result<Json<Value>> {
    Err(SequencingError::NotImplemented("get_run_metrics not implemented yet".to_string()))
}
