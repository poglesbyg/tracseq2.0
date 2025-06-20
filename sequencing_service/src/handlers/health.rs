use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use tracing::{info, error};

use crate::{AppState, error::Result};

/// Basic health check endpoint
pub async fn health_check(State(state): State<AppState>) -> Result<Json<Value>> {
    info!("Health check requested");

    // Test database connection
    match state.db_pool.health_check().await {
        Ok(_) => {
            info!("Health check passed");
            Ok(Json(json!({
                "status": "healthy",
                "service": "sequencing-service",
                "timestamp": chrono::Utc::now(),
                "database": "connected"
            })))
        }
        Err(e) => {
            error!("Health check failed - database connection error: {}", e);
            Ok(Json(json!({
                "status": "unhealthy",
                "service": "sequencing-service",
                "timestamp": chrono::Utc::now(),
                "database": "disconnected",
                "error": e.to_string()
            })))
        }
    }
}

/// Readiness check endpoint - ensures service is ready to accept requests
pub async fn readiness_check(State(state): State<AppState>) -> Result<(StatusCode, Json<Value>)> {
    info!("Readiness check requested");

    let mut ready = true;
    let mut checks = json!({});

    // Check database connection
    match state.db_pool.health_check().await {
        Ok(_) => {
            checks["database"] = json!({
                "status": "ready",
                "message": "Database connection successful"
            });
        }
        Err(e) => {
            ready = false;
            checks["database"] = json!({
                "status": "not_ready",
                "message": format!("Database connection failed: {}", e)
            });
        }
    }

    // Check external service connections
    // Auth service
    match state.auth_client.health_check().await {
        Ok(_) => {
            checks["auth_service"] = json!({
                "status": "ready",
                "message": "Auth service connection successful"
            });
        }
        Err(e) => {
            error!("Auth service health check failed: {}", e);
            checks["auth_service"] = json!({
                "status": "not_ready",
                "message": format!("Auth service connection failed: {}", e)
            });
            // Auth service failure is not critical for basic functionality
        }
    }

    // Sample service
    match state.sample_client.health_check().await {
        Ok(_) => {
            checks["sample_service"] = json!({
                "status": "ready",
                "message": "Sample service connection successful"
            });
        }
        Err(e) => {
            error!("Sample service health check failed: {}", e);
            checks["sample_service"] = json!({
                "status": "not_ready",
                "message": format!("Sample service connection failed: {}", e)
            });
            // Sample service failure is not critical for basic functionality
        }
    }

    // Notification service
    match state.notification_client.health_check().await {
        Ok(_) => {
            checks["notification_service"] = json!({
                "status": "ready",
                "message": "Notification service connection successful"
            });
        }
        Err(e) => {
            error!("Notification service health check failed: {}", e);
            checks["notification_service"] = json!({
                "status": "not_ready",
                "message": format!("Notification service connection failed: {}", e)
            });
            // Notification service failure is not critical for basic functionality
        }
    }

    let status_code = if ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    let response = json!({
        "status": if ready { "ready" } else { "not_ready" },
        "service": "sequencing-service",
        "timestamp": chrono::Utc::now(),
        "checks": checks
    });

    info!("Readiness check completed - status: {}", if ready { "ready" } else { "not_ready" });

    Ok((status_code, Json(response)))
}

/// Metrics endpoint for monitoring
pub async fn metrics(State(state): State<AppState>) -> Result<Json<Value>> {
    info!("Metrics requested");

    // Get basic metrics from database
    let mut metrics = json!({
        "service": "sequencing-service",
        "timestamp": chrono::Utc::now(),
    });

    // Job metrics
    if let Ok(job_counts) = get_job_metrics(&state).await {
        metrics["jobs"] = job_counts;
    }

    // Run metrics
    if let Ok(run_counts) = get_run_metrics(&state).await {
        metrics["runs"] = run_counts;
    }

    // System metrics
    metrics["system"] = json!({
        "uptime": "N/A", // Could be implemented with a startup timestamp
        "memory_usage": "N/A", // Could be implemented with system monitoring
        "cpu_usage": "N/A" // Could be implemented with system monitoring
    });

    Ok(Json(metrics))
}

async fn get_job_metrics(state: &AppState) -> Result<Value> {
    let rows = sqlx::query!(
        r#"
        SELECT 
            status::text,
            COUNT(*) as count
        FROM sequencing_jobs 
        GROUP BY status
        "#
    )
    .fetch_all(&state.db_pool.pool)
    .await?;

    let mut job_counts = json!({});
    for row in rows {
        job_counts[row.status.unwrap_or_default()] = json!(row.count.unwrap_or(0));
    }

    Ok(json!({
        "total": job_counts.as_object().map(|obj| 
            obj.values().filter_map(|v| v.as_i64()).sum::<i64>()
        ).unwrap_or(0),
        "by_status": job_counts
    }))
}

async fn get_run_metrics(state: &AppState) -> Result<Value> {
    let rows = sqlx::query!(
        r#"
        SELECT 
            status::text,
            COUNT(*) as count
        FROM sequencing_runs 
        GROUP BY status
        "#
    )
    .fetch_all(&state.db_pool.pool)
    .await?;

    let mut run_counts = json!({});
    for row in rows {
        run_counts[row.status.unwrap_or_default()] = json!(row.count.unwrap_or(0));
    }

    Ok(json!({
        "total": run_counts.as_object().map(|obj| 
            obj.values().filter_map(|v| v.as_i64()).sum::<i64>()
        ).unwrap_or(0),
        "by_status": run_counts
    }))
}
