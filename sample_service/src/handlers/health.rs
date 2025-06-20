use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::collections::HashMap;
use std::time::Instant;

use crate::{error::SampleServiceError, AppState};

/// Basic health check endpoint
pub async fn health_check(State(state): State<AppState>) -> Result<Json<serde_json::Value>, SampleServiceError> {
    let start = Instant::now();
    
    // Test database connectivity
    match state.sample_service.health_check().await {
        Ok(_) => {
            let duration = start.elapsed();
            Ok(Json(json!({
                "status": "healthy",
                "service": "sample_service",
                "version": "1.0.0",
                "timestamp": chrono::Utc::now(),
                "checks": {
                    "database": {
                        "status": "healthy",
                        "response_time_ms": duration.as_millis()
                    }
                }
            })))
        }
        Err(e) => {
            let duration = start.elapsed();
            Ok(Json(json!({
                "status": "unhealthy",
                "service": "sample_service",
                "version": "1.0.0",
                "timestamp": chrono::Utc::now(),
                "checks": {
                    "database": {
                        "status": "unhealthy",
                        "error": e.to_string(),
                        "response_time_ms": duration.as_millis()
                    }
                }
            })))
        }
    }
}

/// Detailed readiness check endpoint
pub async fn readiness_check(State(state): State<AppState>) -> Result<Json<serde_json::Value>, SampleServiceError> {
    let mut checks = HashMap::new();
    let mut overall_healthy = true;

    // Database connectivity check
    let db_start = Instant::now();
    match sqlx::query("SELECT 1").fetch_one(&state.db_pool.pool).await {
        Ok(_) => {
            checks.insert("database", json!({
                "status": "ready",
                "response_time_ms": db_start.elapsed().as_millis(),
                "details": "Database connection successful"
            }));
        }
        Err(e) => {
            overall_healthy = false;
            checks.insert("database", json!({
                "status": "not_ready",
                "response_time_ms": db_start.elapsed().as_millis(),
                "error": e.to_string()
            }));
        }
    }

    // Auth service connectivity check
    let auth_start = Instant::now();
    match state.auth_client.health_check().await {
        Ok(_) => {
            checks.insert("auth_service", json!({
                "status": "ready",
                "response_time_ms": auth_start.elapsed().as_millis(),
                "details": "Auth service connection successful"
            }));
        }
        Err(e) => {
            overall_healthy = false;
            checks.insert("auth_service", json!({
                "status": "not_ready",
                "response_time_ms": auth_start.elapsed().as_millis(),
                "error": e.to_string()
            }));
        }
    }

    // Storage service connectivity check
    let storage_start = Instant::now();
    match state.storage_client.health_check().await {
        Ok(_) => {
            checks.insert("storage_service", json!({
                "status": "ready",
                "response_time_ms": storage_start.elapsed().as_millis(),
                "details": "Storage service connection successful"
            }));
        }
        Err(e) => {
            overall_healthy = false;
            checks.insert("storage_service", json!({
                "status": "not_ready",
                "response_time_ms": storage_start.elapsed().as_millis(),
                "error": e.to_string()
            }));
        }
    }

    // Configuration validation
    match state.config.validate() {
        Ok(_) => {
            checks.insert("configuration", json!({
                "status": "ready",
                "details": "Configuration is valid"
            }));
        }
        Err(e) => {
            overall_healthy = false;
            checks.insert("configuration", json!({
                "status": "not_ready",
                "error": e.to_string()
            }));
        }
    }

    let status = if overall_healthy { "ready" } else { "not_ready" };
    let status_code = if overall_healthy { 
        StatusCode::OK 
    } else { 
        StatusCode::SERVICE_UNAVAILABLE 
    };

    let response = Json(json!({
        "status": status,
        "service": "sample_service",
        "version": "1.0.0",
        "timestamp": chrono::Utc::now(),
        "checks": checks
    }));

    Ok(response)
}

/// Service metrics endpoint
pub async fn metrics(State(state): State<AppState>) -> Result<Json<serde_json::Value>, SampleServiceError> {
    // Get sample statistics
    let total_samples: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM samples")
        .fetch_one(&state.db_pool.pool)
        .await
        .unwrap_or(0);

    let pending_samples: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM samples WHERE status = 'pending'")
        .fetch_one(&state.db_pool.pool)
        .await
        .unwrap_or(0);

    let validated_samples: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM samples WHERE status = 'validated'")
        .fetch_one(&state.db_pool.pool)
        .await
        .unwrap_or(0);

    let completed_samples: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM samples WHERE status = 'completed'")
        .fetch_one(&state.db_pool.pool)
        .await
        .unwrap_or(0);

    // Get recent activity (last 24 hours)
    let recent_created: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM samples WHERE created_at > NOW() - INTERVAL '24 hours'"
    )
    .fetch_one(&state.db_pool.pool)
    .await
    .unwrap_or(0);

    let recent_updated: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM samples WHERE updated_at > NOW() - INTERVAL '24 hours' AND updated_at != created_at"
    )
    .fetch_one(&state.db_pool.pool)
    .await
    .unwrap_or(0);

    // Get sample type distribution
    let sample_types: Vec<(String, i64)> = sqlx::query_as::<_, (String, i64)>(
        "SELECT sample_type, COUNT(*) FROM samples GROUP BY sample_type ORDER BY COUNT(*) DESC LIMIT 10"
    )
    .fetch_all(&state.db_pool.pool)
    .await
    .unwrap_or_default();

    Ok(Json(json!({
        "service": "sample_service",
        "version": "1.0.0",
        "timestamp": chrono::Utc::now(),
        "metrics": {
            "samples": {
                "total": total_samples,
                "by_status": {
                    "pending": pending_samples,
                    "validated": validated_samples,
                    "completed": completed_samples
                },
                "recent_activity": {
                    "created_last_24h": recent_created,
                    "updated_last_24h": recent_updated
                },
                "sample_types": sample_types.into_iter().collect::<HashMap<String, i64>>()
            },
            "configuration": {
                "max_batch_size": state.config.sample.max_batch_size,
                "auto_generate_barcode": state.config.sample.auto_generate_barcode,
                "validation_timeout_seconds": state.config.sample.validation_timeout_seconds
            }
        }
    })))
} 
