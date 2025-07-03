use crate::error::{Result, ServiceError};
use crate::models::*;
use crate::services::ProtocolService;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize)]
pub struct ListProtocolsQuery {
    library_type: Option<String>,
    is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct RecommendProtocolRequest {
    library_type: String,
    sample_requirements: serde_json::Value,
}

pub async fn list_protocols(
    State(service): State<Arc<ProtocolService>>,
    Query(params): Query<ListProtocolsQuery>,
) -> Result<Json<Vec<Protocol>>> {
    let protocols = service.list_protocols(params.library_type, params.is_active).await?;
    Ok(Json(protocols))
}

pub async fn create_protocol(
    State(service): State<Arc<ProtocolService>>,
    Json(request): Json<CreateProtocolRequest>,
) -> Result<(StatusCode, Json<Protocol>)> {
    request.validate().map_err(|e| ServiceError::Validation {
        message: e.to_string(),
    })?;

    let protocol = service.create_protocol(request).await?;
    Ok((StatusCode::CREATED, Json(protocol)))
}

pub async fn get_protocol(
    State(service): State<Arc<ProtocolService>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Protocol>> {
    let protocol = service.get_protocol(id).await?;
    Ok(Json(protocol))
}

pub async fn update_protocol(
    State(service): State<Arc<ProtocolService>>,
    Path(id): Path<Uuid>,
    Json(request): Json<CreateProtocolRequest>,
) -> Result<Json<Protocol>> {
    request.validate().map_err(|e| ServiceError::Validation {
        message: e.to_string(),
    })?;

    let protocol = service.update_protocol(id, request).await?;
    Ok(Json(protocol))
}

pub async fn validate_protocol(
    State(service): State<Arc<ProtocolService>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<String>>> {
    let errors = service.validate_protocol(id).await?;
    Ok(Json(errors))
}

pub async fn get_protocol_steps(
    State(service): State<Arc<ProtocolService>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let steps = service.get_protocol_steps(id).await?;
    Ok(Json(steps))
}

pub async fn recommend_protocol(
    State(service): State<Arc<ProtocolService>>,
    Json(request): Json<RecommendProtocolRequest>,
) -> Result<Json<Vec<ProtocolRecommendation>>> {
    let recommendations = service.recommend_protocol(
        request.library_type,
        request.sample_requirements,
    ).await?;
    Ok(Json(recommendations))
}