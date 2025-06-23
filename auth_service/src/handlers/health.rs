use axum::{extract::State, http::StatusCode, Json};
use chrono::Utc;
use serde_json::json;

use crate::AppState;

/// Basic health check endpoint
#[allow(dead_code)]
pub async fn health_check() -> Result<Json<serde_json::Value>, StatusCode> {
    let response = json!({
        "status": "healthy",
        "timestamp": Utc::now().to_rfc3339(),
        "service": "auth-service",
        "version": env!("CARGO_PKG_VERSION")
    });

    Ok(Json(response))
}

/// Readiness probe endpoint  
#[allow(dead_code)]
pub async fn readiness_check(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Check database connectivity
    match state.auth_service.health_check().await {
        Ok(_) => {
            let response = json!({
                "status": "ready",
                "timestamp": Utc::now().to_rfc3339(),
                "service": "auth-service",
                "version": env!("CARGO_PKG_VERSION"),
                "checks": {
                    "database": "healthy"
                }
            });
            Ok(Json(response))
        }
        Err(_) => {
            let _response = json!({
                "status": "not_ready",
                "timestamp": Utc::now().to_rfc3339(),
                "service": "auth-service",
                "version": env!("CARGO_PKG_VERSION"),
                "checks": {
                    "database": "unhealthy"
                }
            });
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }
}

/// Metrics endpoint
#[allow(dead_code)]
pub async fn metrics(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Get database health
    let db_health = match state.db_pool.health_check().await {
        Ok(health) => health,
        Err(_) => {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Get basic metrics from database
    let active_users_result = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM users WHERE status = 'active' AND last_login_at > NOW() - INTERVAL '24 hours'"
    )
    .fetch_one(&state.db_pool.pool)
    .await;

    let active_sessions_result = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM user_sessions WHERE expires_at > NOW() AND revoked = FALSE"
    )
    .fetch_one(&state.db_pool.pool)
    .await;

    let total_users_result = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM users"
    )
    .fetch_one(&state.db_pool.pool)
    .await;

    let failed_logins_result = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM security_audit_log WHERE event_type = 'LOGIN_FAILED' AND timestamp > NOW() - INTERVAL '1 hour'"
    )
    .fetch_one(&state.db_pool.pool)
    .await;

    let metrics = json!({
        "timestamp": Utc::now().to_rfc3339(),
        "service": "auth-service",
        "version": env!("CARGO_PKG_VERSION"),
        "metrics": {
            "active_users_24h": active_users_result.unwrap_or(0),
            "active_sessions": active_sessions_result.unwrap_or(0),
            "total_users": total_users_result.unwrap_or(0),
            "failed_logins_last_hour": failed_logins_result.unwrap_or(0)
        },
        "database": {
            "connected": db_health.is_connected,
            "response_time_ms": db_health.response_time_ms,
            "active_connections": db_health.active_connections,
            "idle_connections": db_health.idle_connections,
            "max_connections": db_health.max_connections
        }
    });

    Ok(Json(metrics))
} 
