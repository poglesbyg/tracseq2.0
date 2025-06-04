use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use sqlx::Row;

use crate::AppComponents;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardStats {
    total_templates: i64,
    total_samples: i64,
    pending_sequencing: i64,
    completed_sequencing: i64,
}

/// Health check endpoint
pub async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}

/// Get dashboard statistics
pub async fn get_dashboard_stats(
    State(state): State<AppComponents>,
) -> Result<Json<DashboardStats>, (StatusCode, String)> {
    let row = sqlx::query(
        r#"
        WITH stats AS (
            SELECT
                (SELECT COUNT(*) FROM templates) as total_templates,
                (SELECT COUNT(*) FROM samples) as total_samples,
                (SELECT COUNT(*) FROM sequencing_jobs WHERE status = 'pending') as pending_sequencing,
                (SELECT COUNT(*) FROM sequencing_jobs WHERE status = 'completed') as completed_sequencing
        )
        SELECT * FROM stats
        "#
    )
    .fetch_one(&state.database.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(DashboardStats {
        total_templates: row.try_get("total_templates").unwrap_or(0),
        total_samples: row.try_get("total_samples").unwrap_or(0),
        pending_sequencing: row.try_get("pending_sequencing").unwrap_or(0),
        completed_sequencing: row.try_get("completed_sequencing").unwrap_or(0),
    }))
}
