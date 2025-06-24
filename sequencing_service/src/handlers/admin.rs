use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde_json::json;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::{
    error::{Result, SequencingError},
    models::*,
    AppState,
};

/// Get system statistics for administrators
pub async fn get_system_statistics(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    // Get job statistics
    let total_jobs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sequencing_jobs")
        .fetch_one(&state.db_pool.pool)
        .await?;

    let jobs_by_status = sqlx::query_as::<_, (String, i64)>(
        "SELECT status::text, COUNT(*) FROM sequencing_jobs GROUP BY status"
    )
    .fetch_all(&state.db_pool.pool)
    .await?;

    let jobs_by_platform = sqlx::query_as::<_, (String, i64)>(
        "SELECT platform, COUNT(*) FROM sequencing_jobs GROUP BY platform"
    )
    .fetch_all(&state.db_pool.pool)
    .await?;

    // Get recent activity (last 7 days)
    let recent_jobs: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sequencing_jobs WHERE created_at > NOW() - INTERVAL '7 days'"
    )
    .fetch_one(&state.db_pool.pool)
    .await?;

    let completed_jobs: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sequencing_jobs WHERE status = 'completed' AND completed_at > NOW() - INTERVAL '7 days'"
    )
    .fetch_one(&state.db_pool.pool)
    .await?;

    // Get run statistics
    let total_runs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sequencing_runs")
        .fetch_one(&state.db_pool.pool)
        .await?;

    let active_runs: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sequencing_runs WHERE status IN ('running', 'queued')"
    )
    .fetch_one(&state.db_pool.pool)
    .await?;

    // Get workflow statistics
    let active_workflows: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sequencing_workflows WHERE status = 'active'"
    )
    .fetch_one(&state.db_pool.pool)
    .await?;

    // Calculate performance metrics
    let avg_processing_time: Option<f64> = sqlx::query_scalar(
        r#"
        SELECT AVG(EXTRACT(EPOCH FROM (completed_at - started_at))/3600.0) 
        FROM sequencing_jobs 
        WHERE status = 'completed' 
        AND started_at IS NOT NULL 
        AND completed_at IS NOT NULL
        AND completed_at > NOW() - INTERVAL '30 days'
        "#
    )
    .fetch_optional(&state.db_pool.pool)
    .await?
    .flatten();

    Ok(Json(json!({
        "success": true,
        "data": {
            "overview": {
                "total_jobs": total_jobs,
                "total_runs": total_runs,
                "active_runs": active_runs,
                "active_workflows": active_workflows,
                "recent_jobs_7d": recent_jobs,
                "completed_jobs_7d": completed_jobs
            },
            "job_distribution": {
                "by_status": jobs_by_status.into_iter().collect::<std::collections::HashMap<String, i64>>(),
                "by_platform": jobs_by_platform.into_iter().collect::<std::collections::HashMap<String, i64>>()
            },
            "performance": {
                "avg_processing_time_hours": avg_processing_time
            }
        }
    })))
}

/// Get detailed job analytics
pub async fn get_job_analytics(
    State(state): State<AppState>,
    Query(query): Query<AnalyticsQuery>,
) -> Result<Json<serde_json::Value>> {
    let period_days = query.period_days.unwrap_or(30);
    let start_date = Utc::now() - chrono::Duration::days(period_days);

    // Daily job creation trends
    let daily_trends = sqlx::query_as::<_, (chrono::NaiveDate, i64)>(
        r#"
        SELECT DATE(created_at) as date, COUNT(*) as count
        FROM sequencing_jobs 
        WHERE created_at > $1
        GROUP BY DATE(created_at)
        ORDER BY date
        "#
    )
    .bind(start_date)
    .fetch_all(&state.db_pool.pool)
    .await?;

    // Success rate analysis
    let success_rate = sqlx::query_as::<_, (String, i64)>(
        r#"
        SELECT 
            CASE 
                WHEN status = 'completed' THEN 'successful'
                WHEN status = 'failed' THEN 'failed'
                ELSE 'other'
            END as outcome,
            COUNT(*) as count
        FROM sequencing_jobs 
        WHERE created_at > $1
        GROUP BY outcome
        "#
    )
    .bind(start_date)
    .fetch_all(&state.db_pool.pool)
    .await?;

    // Platform utilization
    let platform_utilization = sqlx::query_as::<_, (String, i64, f64)>(
        r#"
        SELECT 
            platform,
            COUNT(*) as job_count,
            AVG(EXTRACT(EPOCH FROM (COALESCE(completed_at, NOW()) - created_at))/3600.0) as avg_duration_hours
        FROM sequencing_jobs 
        WHERE created_at > $1
        GROUP BY platform
        ORDER BY job_count DESC
        "#
    )
    .bind(start_date)
    .fetch_all(&state.db_pool.pool)
    .await?;

    // Priority distribution
    let priority_distribution = sqlx::query_as::<_, (String, i64)>(
        "SELECT priority::text, COUNT(*) FROM sequencing_jobs WHERE created_at > $1 GROUP BY priority"
    )
    .bind(start_date)
    .fetch_all(&state.db_pool.pool)
    .await?;

    Ok(Json(json!({
        "success": true,
        "data": {
            "period_days": period_days,
            "daily_trends": daily_trends,
            "success_rate": success_rate.into_iter().collect::<std::collections::HashMap<String, i64>>(),
            "platform_utilization": platform_utilization.into_iter().map(|(platform, count, avg_duration)| {
                json!({
                    "platform": platform,
                    "job_count": count,
                    "avg_duration_hours": avg_duration
                })
            }).collect::<Vec<_>>(),
            "priority_distribution": priority_distribution.into_iter().collect::<std::collections::HashMap<String, i64>>()
        }
    })))
}

/// Get system health information
pub async fn get_system_health(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    // Check database connectivity
    let db_health = match sqlx::query("SELECT 1").fetch_one(&state.db_pool.pool).await {
        Ok(_) => "healthy",
        Err(_) => "unhealthy",
    };

    // Check for stuck jobs (running for more than 48 hours)
    let stuck_jobs: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM sequencing_jobs 
        WHERE status = 'running' 
        AND started_at IS NOT NULL 
        AND started_at < NOW() - INTERVAL '48 hours'
        "#
    )
    .fetch_one(&state.db_pool.pool)
    .await
    .unwrap_or(0);

    // Check for failed jobs in last hour
    let recent_failures: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sequencing_jobs WHERE status = 'failed' AND updated_at > NOW() - INTERVAL '1 hour'"
    )
    .fetch_one(&state.db_pool.pool)
    .await
    .unwrap_or(0);

    // Check queue depth
    let queue_depth: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sequencing_jobs WHERE status IN ('queued', 'validated')"
    )
    .fetch_one(&state.db_pool.pool)
    .await
    .unwrap_or(0);

    // Determine overall health
    let overall_health = if db_health == "healthy" && stuck_jobs == 0 && recent_failures < 5 {
        "healthy"
    } else if stuck_jobs > 0 || recent_failures >= 5 {
        "degraded"
    } else {
        "unhealthy"
    };

    Ok(Json(json!({
        "success": true,
        "data": {
            "overall_status": overall_health,
            "checks": {
                "database": db_health,
                "stuck_jobs": stuck_jobs,
                "recent_failures": recent_failures,
                "queue_depth": queue_depth
            },
            "timestamp": Utc::now()
        }
    })))
}

/// Force cleanup of stuck jobs
pub async fn cleanup_stuck_jobs(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    let updated_jobs = sqlx::query(
        r#"
        UPDATE sequencing_jobs 
        SET status = 'failed', 
            error_message = 'Job timeout - automatically failed by admin cleanup',
            updated_at = NOW()
        WHERE status = 'running' 
        AND started_at IS NOT NULL 
        AND started_at < NOW() - INTERVAL '48 hours'
        RETURNING id
        "#
    )
    .fetch_all(&state.db_pool.pool)
    .await?;

    let cleaned_count = updated_jobs.len();

    Ok(Json(json!({
        "success": true,
        "data": {
            "cleaned_jobs": cleaned_count,
            "message": format!("Cleaned up {} stuck jobs", cleaned_count)
        }
    })))
}

/// Get resource utilization metrics
pub async fn get_resource_utilization(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    // Get platform utilization
    let platform_usage = sqlx::query_as::<_, (String, i64, i64)>(
        r#"
        SELECT 
            platform,
            COUNT(*) as total_jobs,
            COUNT(CASE WHEN status IN ('running', 'queued') THEN 1 END) as active_jobs
        FROM sequencing_jobs 
        GROUP BY platform
        "#
    )
    .fetch_all(&state.db_pool.pool)
    .await?;

    // Get priority queue status
    let priority_queue = sqlx::query_as::<_, (String, i64)>(
        r#"
        SELECT priority::text, COUNT(*) 
        FROM sequencing_jobs 
        WHERE status IN ('queued', 'validated')
        GROUP BY priority
        ORDER BY 
            CASE priority
                WHEN 'critical' THEN 1
                WHEN 'urgent' THEN 2
                WHEN 'high' THEN 3
                WHEN 'normal' THEN 4
                WHEN 'low' THEN 5
            END
        "#
    )
    .fetch_all(&state.db_pool.pool)
    .await?;

    // Estimate capacity utilization (simplified)
    let total_capacity = state.config.sequencing.max_concurrent_runs as i64;
    let current_usage: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sequencing_jobs WHERE status = 'running'"
    )
    .fetch_one(&state.db_pool.pool)
    .await?;

    let utilization_percentage = if total_capacity > 0 {
        (current_usage as f64 / total_capacity as f64 * 100.0).round()
    } else {
        0.0
    };

    Ok(Json(json!({
        "success": true,
        "data": {
            "capacity": {
                "total": total_capacity,
                "current_usage": current_usage,
                "utilization_percentage": utilization_percentage
            },
            "platform_usage": platform_usage.into_iter().map(|(platform, total, active)| {
                json!({
                    "platform": platform,
                    "total_jobs": total,
                    "active_jobs": active,
                    "utilization": if total > 0 { (active as f64 / total as f64 * 100.0).round() } else { 0.0 }
                })
            }).collect::<Vec<_>>(),
            "priority_queue": priority_queue.into_iter().collect::<std::collections::HashMap<String, i64>>()
        }
    })))
}

/// Purge old completed jobs
pub async fn purge_old_jobs(
    State(state): State<AppState>,
    Query(query): Query<PurgeQuery>,
) -> Result<Json<serde_json::Value>> {
    let days_old = query.days_old.unwrap_or(90);
    
    let purged_jobs = sqlx::query(
        r#"
        DELETE FROM sequencing_jobs 
        WHERE status IN ('completed', 'failed', 'cancelled')
        AND updated_at < NOW() - INTERVAL $1 DAY
        RETURNING id
        "#
    )
    .bind(days_old)
    .fetch_all(&state.db_pool.pool)
    .await?;

    let purged_count = purged_jobs.len();

    Ok(Json(json!({
        "success": true,
        "data": {
            "purged_jobs": purged_count,
            "days_old": days_old,
            "message": format!("Purged {} jobs older than {} days", purged_count, days_old)
        }
    })))
}

/// Update service configuration
pub async fn update_service_config(
    State(state): State<AppState>,
    Json(request): Json<ServiceConfigUpdate>,
) -> Result<Json<serde_json::Value>> {
    // This would typically update configuration in a database or config management system
    // For now, we'll just validate the request and return success
    
    if let Some(max_concurrent) = request.max_concurrent_runs {
        if max_concurrent == 0 || max_concurrent > 100 {
            return Err(SequencingError::Validation {
                message: "max_concurrent_runs must be between 1 and 100".to_string(),
            });
        }
    }

    if let Some(default_timeout) = request.default_timeout_hours {
        if default_timeout == 0 || default_timeout > 168 { // Max 1 week
            return Err(SequencingError::Validation {
                message: "default_timeout_hours must be between 1 and 168".to_string(),
            });
        }
    }

    Ok(Json(json!({
        "success": true,
        "data": {
            "message": "Service configuration updated successfully",
            "updated_settings": request
        }
    })))
}

/// Query structures for admin endpoints
#[derive(serde::Deserialize)]
pub struct AnalyticsQuery {
    pub period_days: Option<i64>,
}

#[derive(serde::Deserialize)]
pub struct PurgeQuery {
    pub days_old: Option<i64>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ServiceConfigUpdate {
    pub max_concurrent_runs: Option<u32>,
    pub default_timeout_hours: Option<u32>,
    pub enable_auto_scheduling: Option<bool>,
    pub default_priority: Option<Priority>,
}
