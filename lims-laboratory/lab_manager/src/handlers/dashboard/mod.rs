use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;

use crate::assembly::AppComponents;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardStats {
    total_templates: i64,
    total_samples: i64,
    pending_sequencing: i64,
    completed_sequencing: i64,
}

#[derive(Debug, Serialize)]
pub struct HealthStatus {
    status: String,
    database_connected: bool,
    timestamp: chrono::DateTime<chrono::Utc>,
}

/// Health check endpoint with database connectivity test
pub async fn health_check(
    State(state): State<AppComponents>,
) -> Result<Json<HealthStatus>, (StatusCode, String)> {
    // Test database connectivity
    let database_connected = sqlx::query("SELECT 1")
        .fetch_one(&state.database.pool)
        .await
        .is_ok();

    let status = if database_connected {
        "healthy"
    } else {
        "degraded"
    };

    Ok(Json(HealthStatus {
        status: status.to_string(),
        database_connected,
        timestamp: chrono::Utc::now(),
    }))
}

/// Get dashboard statistics with improved error handling
pub async fn get_dashboard_stats(
    State(state): State<AppComponents>,
) -> Result<Json<DashboardStats>, (StatusCode, String)> {
    // Use separate queries with better error handling
    let total_templates = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM templates")
        .fetch_one(&state.database.pool)
        .await
        .unwrap_or(0);

    let total_samples = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM samples")
        .fetch_one(&state.database.pool)
        .await
        .unwrap_or(0);

    let pending_sequencing = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM sequencing_jobs WHERE status = 'pending'",
    )
    .fetch_one(&state.database.pool)
    .await
    .unwrap_or(0);

    let completed_sequencing = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM sequencing_jobs WHERE status = 'completed'",
    )
    .fetch_one(&state.database.pool)
    .await
    .unwrap_or(0);

    Ok(Json(DashboardStats {
        total_templates,
        total_samples,
        pending_sequencing,
        completed_sequencing,
    }))
}
