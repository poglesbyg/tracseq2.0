use crate::error::{Result, ServiceError};
use crate::models::*;
use crate::services::QualityControlService;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize)]
pub struct ListQCMetricsQuery {
    library_id: Option<Uuid>,
    metric_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GenerateReportRequest {
    library_id: Uuid,
}

pub async fn list_qc_metrics(
    State(service): State<Arc<QualityControlService>>,
    Query(params): Query<ListQCMetricsQuery>,
) -> Result<Json<Vec<QualityControlMetric>>> {
    let metrics = service.list_qc_metrics(params.library_id, params.metric_type).await?;
    Ok(Json(metrics))
}

pub async fn create_qc_metric(
    State(service): State<Arc<QualityControlService>>,
    Json(request): Json<CreateQCMetricRequest>,
) -> Result<(StatusCode, Json<QualityControlMetric>)> {
    request.validate().map_err(|e| ServiceError::Validation {
        message: e.to_string(),
    })?;

    let metric = service.create_qc_metric(request).await?;
    Ok((StatusCode::CREATED, Json(metric)))
}

pub async fn assess_library_quality(
    State(service): State<Arc<QualityControlService>>,
    Path(id): Path<Uuid>,
) -> Result<Json<QualityReport>> {
    let report = service.assess_library_quality(id).await?;
    Ok(Json(report))
}

pub async fn get_quality_thresholds(
    State(service): State<Arc<QualityControlService>>,
) -> Result<Json<HashMap<String, (Option<f64>, Option<f64>)>>> {
    let thresholds = service.get_quality_thresholds().await?;
    Ok(Json(thresholds))
}

pub async fn generate_quality_report(
    State(service): State<Arc<QualityControlService>>,
    Json(request): Json<GenerateReportRequest>,
) -> Result<Json<QualityReport>> {
    let report = service.generate_quality_report(request.library_id).await?;
    Ok(Json(report))
}