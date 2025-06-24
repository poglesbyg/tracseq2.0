use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde_json::json;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};

use crate::{
    error::{Result, SequencingError},
    models::*,
    AppState,
};

/// Get current scheduling queue status
pub async fn get_queue_status(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    // Get queue statistics by priority
    let priority_queue = sqlx::query_as::<_, (String, i64, f64)>(
        r#"
        SELECT 
            priority::text,
            COUNT(*) as job_count,
            AVG(EXTRACT(EPOCH FROM (NOW() - created_at))/3600.0) as avg_wait_time_hours
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

    // Get platform queue distribution
    let platform_queue = sqlx::query_as::<_, (String, i64)>(
        r#"
        SELECT platform, COUNT(*) as job_count
        FROM sequencing_jobs 
        WHERE status IN ('queued', 'validated')
        GROUP BY platform
        ORDER BY job_count DESC
        "#
    )
    .fetch_all(&state.db_pool.pool)
    .await?;

    // Get currently running jobs
    let running_jobs: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sequencing_jobs WHERE status = 'running'"
    )
    .fetch_one(&state.db_pool.pool)
    .await?;

    // Get estimated wait times
    let next_available_slot = calculate_next_available_slot(&state).await?;

    Ok(Json(json!({
        "success": true,
        "data": {
            "queue_summary": {
                "total_queued": priority_queue.iter().map(|(_, count, _)| count).sum::<i64>(),
                "currently_running": running_jobs,
                "max_concurrent": state.config.sequencing.max_concurrent_runs,
                "capacity_utilization": if state.config.sequencing.max_concurrent_runs > 0 {
                    (running_jobs as f64 / state.config.sequencing.max_concurrent_runs as f64 * 100.0).round()
                } else { 0.0 }
            },
            "priority_breakdown": priority_queue.into_iter().map(|(priority, count, avg_wait)| {
                json!({
                    "priority": priority,
                    "job_count": count,
                    "avg_wait_time_hours": avg_wait
                })
            }).collect::<Vec<_>>(),
            "platform_distribution": platform_queue.into_iter().collect::<std::collections::HashMap<String, i64>>(),
            "next_available_slot": next_available_slot
        }
    })))
}

/// Get detailed queue with job information
pub async fn get_detailed_queue(
    State(state): State<AppState>,
    Query(query): Query<QueueQuery>,
) -> Result<Json<serde_json::Value>> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(50).min(200);
    let offset = (page - 1) * page_size;

    let mut where_conditions = vec!["status IN ('queued', 'validated')".to_string()];
    let mut param_count = 0;

    if let Some(priority) = &query.priority {
        param_count += 1;
        where_conditions.push(format!("priority = ${}", param_count));
    }

    if let Some(platform) = &query.platform {
        param_count += 1;
        where_conditions.push(format!("platform = ${}", param_count));
    }

    let where_clause = where_conditions.join(" AND ");

    // Get total count
    let total_count: i64 = sqlx::query_scalar(&format!(
        "SELECT COUNT(*) FROM sequencing_jobs WHERE {}", where_clause
    ))
    .fetch_one(&state.db_pool.pool)
    .await?;

    // Get queue jobs with estimated start times
    let queue_jobs = sqlx::query_as::<_, SequencingJob>(&format!(
        r#"
        SELECT * FROM sequencing_jobs 
        WHERE {} 
        ORDER BY 
            CASE priority
                WHEN 'critical' THEN 1
                WHEN 'urgent' THEN 2
                WHEN 'high' THEN 3
                WHEN 'normal' THEN 4
                WHEN 'low' THEN 5
            END,
            created_at ASC
        LIMIT ${} OFFSET ${}
        "#,
        where_clause,
        param_count + 1,
        param_count + 2
    ))
    .bind(page_size)
    .bind(offset)
    .fetch_all(&state.db_pool.pool)
    .await?;

    // Calculate estimated start times for each job
    let mut jobs_with_estimates = Vec::new();
    let mut current_time = Utc::now();
    let avg_job_duration = Duration::hours(4); // Default 4 hours per job

    for (index, job) in queue_jobs.iter().enumerate() {
        let estimated_start = current_time + Duration::minutes((index as i64) * 15); // 15-minute buffer between jobs
        let estimated_completion = estimated_start + avg_job_duration;

        jobs_with_estimates.push(json!({
            "job": job,
            "queue_position": offset + index as i64 + 1,
            "estimated_start": estimated_start,
            "estimated_completion": estimated_completion,
            "estimated_wait_hours": (estimated_start - job.created_at).num_hours()
        }));
    }

    let total_pages = (total_count + page_size - 1) / page_size;

    Ok(Json(json!({
        "success": true,
        "data": {
            "jobs": jobs_with_estimates,
            "pagination": {
                "page": page,
                "page_size": page_size,
                "total_count": total_count,
                "total_pages": total_pages
            }
        }
    })))
}

/// Prioritize a job in the queue
pub async fn prioritize_job(
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
    Json(request): Json<PriorityUpdateRequest>,
) -> Result<Json<serde_json::Value>> {
    // Verify job exists and is in queue
    let job = sqlx::query_as::<_, SequencingJob>(
        "SELECT * FROM sequencing_jobs WHERE id = $1"
    )
    .bind(job_id)
    .fetch_optional(&state.db_pool.pool)
    .await?
    .ok_or(SequencingError::JobNotFound(job_id.to_string()))?;

    if !matches!(job.status, JobStatus::Queued | JobStatus::Validated) {
        return Err(SequencingError::InvalidJobState {
            current_state: job.status.to_string(),
            required_state: "queued or validated".to_string(),
        });
    }

    // Update job priority
    let updated_job = sqlx::query_as::<_, SequencingJob>(
        r#"
        UPDATE sequencing_jobs 
        SET priority = $2, updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#
    )
    .bind(job_id)
    .bind(&request.new_priority)
    .fetch_one(&state.db_pool.pool)
    .await?;

    // Log priority change
    sqlx::query(
        r#"
        INSERT INTO job_audit_log (job_id, action, old_values, new_values, performed_by, performed_at)
        VALUES ($1, 'priority_changed', $2, $3, $4, NOW())
        "#
    )
    .bind(job_id)
    .bind(json!({"old_priority": job.priority}))
    .bind(json!({"new_priority": request.new_priority}))
    .bind(request.updated_by.as_deref())
    .execute(&state.db_pool.pool)
    .await?;

    Ok(Json(json!({
        "success": true,
        "data": updated_job,
        "message": "Job priority updated successfully"
    })))
}

/// Schedule next available jobs based on capacity and priority
pub async fn schedule_next_jobs(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    let max_concurrent = state.config.sequencing.max_concurrent_runs as i64;
    
    // Get current running job count
    let current_running: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sequencing_jobs WHERE status = 'running'"
    )
    .fetch_one(&state.db_pool.pool)
    .await?;

    let available_slots = max_concurrent - current_running;

    if available_slots <= 0 {
        return Ok(Json(json!({
            "success": true,
            "data": {
                "scheduled_jobs": 0,
                "message": "No available slots for scheduling"
            }
        })));
    }

    // Get next jobs to schedule (highest priority first)
    let jobs_to_schedule = sqlx::query_as::<_, SequencingJob>(
        r#"
        SELECT * FROM sequencing_jobs 
        WHERE status = 'validated'
        ORDER BY 
            CASE priority
                WHEN 'critical' THEN 1
                WHEN 'urgent' THEN 2
                WHEN 'high' THEN 3
                WHEN 'normal' THEN 4
                WHEN 'low' THEN 5
            END,
            created_at ASC
        LIMIT $1
        "#
    )
    .bind(available_slots)
    .fetch_all(&state.db_pool.pool)
    .await?;

    let mut scheduled_jobs = Vec::new();

    for job in jobs_to_schedule {
        // Check platform capacity (simplified - could be more sophisticated)
        let platform_running: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sequencing_jobs WHERE status = 'running' AND platform = $1"
        )
        .bind(job.platform.as_deref().unwrap_or("unknown"))
        .fetch_one(&state.db_pool.pool)
        .await?;

        // Assume max 2 concurrent jobs per platform (configurable)
        if platform_running >= 2 {
            continue;
        }

        // Update job status to running
        let updated_job = sqlx::query_as::<_, SequencingJob>(
            r#"
            UPDATE sequencing_jobs 
            SET status = 'running', started_at = NOW(), updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(job.id)
        .fetch_one(&state.db_pool.pool)
        .await?;

        scheduled_jobs.push(updated_job);

        // Create sequencing run record
        sqlx::query(
            r#"
            INSERT INTO sequencing_runs (
                id, job_id, run_name, platform, status, started_at, created_at
            ) VALUES ($1, $2, $3, $4, 'running', NOW(), NOW())
            "#
        )
        .bind(Uuid::new_v4())
        .bind(job.id)
        .bind(format!("Run_{}", job.job_name.as_deref().unwrap_or("unknown")))
        .bind(job.platform.as_deref().unwrap_or("unknown"))
        .bind(RunStatus::Running)
        .execute(&state.db_pool.pool)
        .await?;
    }

    Ok(Json(json!({
        "success": true,
        "data": {
            "scheduled_jobs": scheduled_jobs.len(),
            "jobs": scheduled_jobs,
            "message": format!("Successfully scheduled {} jobs", scheduled_jobs.len())
        }
    })))
}

/// Get scheduling recommendations
pub async fn get_scheduling_recommendations(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    // Analyze queue patterns and provide recommendations
    
    // Check for old queued jobs
    let old_jobs: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM sequencing_jobs 
        WHERE status IN ('queued', 'validated')
        AND created_at < NOW() - INTERVAL '24 hours'
        "#
    )
    .fetch_one(&state.db_pool.pool)
    .await?;

    // Check priority distribution
    let priority_imbalance = sqlx::query_as::<_, (String, i64)>(
        r#"
        SELECT priority::text, COUNT(*) 
        FROM sequencing_jobs 
        WHERE status IN ('queued', 'validated')
        GROUP BY priority
        HAVING COUNT(*) > 10
        "#
    )
    .fetch_all(&state.db_pool.pool)
    .await?;

    // Check platform bottlenecks
    let platform_bottlenecks = sqlx::query_as::<_, (String, i64, i64)>(
        r#"
        SELECT 
            platform,
            COUNT(*) as queued_count,
            COUNT(CASE WHEN status = 'running' THEN 1 END) as running_count
        FROM sequencing_jobs 
        WHERE status IN ('queued', 'validated', 'running')
        GROUP BY platform
        HAVING COUNT(*) > 5
        "#
    )
    .fetch_all(&state.db_pool.pool)
    .await?;

    let mut recommendations = Vec::new();

    if old_jobs > 0 {
        recommendations.push(json!({
            "type": "old_jobs_warning",
            "severity": "medium",
            "message": format!("{} jobs have been waiting for more than 24 hours", old_jobs),
            "action": "Consider reviewing job priorities or increasing capacity"
        }));
    }

    for (priority, count) in priority_imbalance {
        recommendations.push(json!({
            "type": "priority_imbalance",
            "severity": "low",
            "message": format!("High queue depth for {} priority jobs: {}", priority, count),
            "action": "Consider load balancing or priority adjustment"
        }));
    }

    for (platform, queued, running) in platform_bottlenecks {
        if queued > running * 3 {
            recommendations.push(json!({
                "type": "platform_bottleneck",
                "severity": "high",
                "message": format!("Platform {} has {} queued vs {} running jobs", platform, queued, running),
                "action": "Consider adding more capacity for this platform"
            }));
        }
    }

    if recommendations.is_empty() {
        recommendations.push(json!({
            "type": "all_good",
            "severity": "info",
            "message": "Queue is operating efficiently",
            "action": "No immediate action required"
        }));
    }

    Ok(Json(json!({
        "success": true,
        "data": {
            "recommendations": recommendations,
            "analysis_timestamp": Utc::now()
        }
    })))
}

/// Pause/resume scheduling
pub async fn toggle_scheduling(
    State(state): State<AppState>,
    Json(request): Json<SchedulingToggleRequest>,
) -> Result<Json<serde_json::Value>> {
    // This would typically update a configuration flag
    // For now, we'll simulate by returning the requested state
    
    let action = if request.enable_scheduling {
        "enabled"
    } else {
        "disabled"
    };

    Ok(Json(json!({
        "success": true,
        "data": {
            "scheduling_enabled": request.enable_scheduling,
            "changed_by": request.changed_by,
            "timestamp": Utc::now(),
            "message": format!("Automatic scheduling has been {}", action)
        }
    })))
}

/// Estimate job completion times
pub async fn estimate_completion_times(
    State(state): State<AppState>,
    Query(query): Query<EstimationQuery>,
) -> Result<Json<serde_json::Value>> {
    let job_ids = query.job_ids.unwrap_or_default();
    
    if job_ids.is_empty() {
        return Err(SequencingError::Validation {
            message: "At least one job_id must be provided".to_string(),
        });
    }

    let mut estimations = Vec::new();

    for job_id in job_ids {
        let job = sqlx::query_as::<_, SequencingJob>(
            "SELECT * FROM sequencing_jobs WHERE id = $1"
        )
        .bind(job_id)
        .fetch_optional(&state.db_pool.pool)
        .await?;

        if let Some(job) = job {
            let estimation = match job.status {
                JobStatus::Running => {
                    // Estimate based on current progress and historical data
                    let avg_duration = get_average_duration_for_platform(&state, job.platform.as_deref().unwrap_or("unknown")).await?;
                    let elapsed = Utc::now() - job.started_at.unwrap_or(job.created_at);
                    let estimated_total = avg_duration;
                    let estimated_remaining = estimated_total - elapsed;
                    
                    json!({
                        "job_id": job_id,
                        "status": "running",
                        "estimated_completion": Utc::now() + estimated_remaining,
                        "estimated_remaining_hours": estimated_remaining.num_hours().max(0),
                        "progress_percentage": ((elapsed.num_minutes() as f64 / estimated_total.num_minutes() as f64) * 100.0).min(95.0)
                    })
                }
                JobStatus::Queued | JobStatus::Validated => {
                    // Estimate based on queue position and platform capacity
                    let queue_position = get_queue_position(&state, job_id).await?;
                    let avg_duration = get_average_duration_for_platform(&state, job.platform.as_deref().unwrap_or("unknown")).await?;
                    let estimated_start = Utc::now() + Duration::hours(queue_position * 2); // Rough estimate
                    let estimated_completion = estimated_start + avg_duration;
                    
                    json!({
                        "job_id": job_id,
                        "status": "queued",
                        "queue_position": queue_position,
                        "estimated_start": estimated_start,
                        "estimated_completion": estimated_completion,
                        "estimated_wait_hours": (estimated_start - job.created_at).num_hours()
                    })
                }
                JobStatus::Completed => {
                    json!({
                        "job_id": job_id,
                        "status": "completed",
                        "actual_completion": job.completed_at,
                        "message": "Job already completed"
                    })
                }
                _ => {
                    json!({
                        "job_id": job_id,
                        "status": job.status,
                        "message": "Cannot estimate completion time for current status"
                    })
                }
            };

            estimations.push(estimation);
        } else {
            estimations.push(json!({
                "job_id": job_id,
                "error": "Job not found"
            }));
        }
    }

    Ok(Json(json!({
        "success": true,
        "data": {
            "estimations": estimations,
            "generated_at": Utc::now()
        }
    })))
}

/// Helper functions
async fn calculate_next_available_slot(state: &AppState) -> Result<DateTime<Utc>> {
    let running_jobs = sqlx::query_as::<_, SequencingJob>(
        "SELECT * FROM sequencing_jobs WHERE status = 'running' ORDER BY started_at ASC"
    )
    .fetch_all(&state.db_pool.pool)
    .await?;

    if running_jobs.is_empty() {
        return Ok(Utc::now());
    }

    // Simple estimation: assume shortest running job will complete in 2 hours
    let earliest_completion = running_jobs
        .iter()
        .filter_map(|job| job.started_at)
        .min()
        .unwrap_or_else(Utc::now)
        + Duration::hours(2);

    Ok(earliest_completion)
}

async fn get_average_duration_for_platform(state: &AppState, platform: &str) -> Result<Duration> {
    let avg_hours: Option<f64> = sqlx::query_scalar(
        r#"
        SELECT AVG(EXTRACT(EPOCH FROM (completed_at - started_at))/3600.0)
        FROM sequencing_jobs 
        WHERE platform = $1 
        AND status = 'completed' 
        AND started_at IS NOT NULL 
        AND completed_at IS NOT NULL
        AND completed_at > NOW() - INTERVAL '30 days'
        "#
    )
    .bind(platform)
    .fetch_optional(&state.db_pool.pool)
    .await?
    .flatten();

    Ok(Duration::hours(avg_hours.unwrap_or(4.0) as i64))
}

async fn get_queue_position(state: &AppState, job_id: Uuid) -> Result<i64> {
    let position: Option<i64> = sqlx::query_scalar(
        r#"
        WITH ranked_jobs AS (
            SELECT 
                id,
                ROW_NUMBER() OVER (
                    ORDER BY 
                        CASE priority
                            WHEN 'critical' THEN 1
                            WHEN 'urgent' THEN 2
                            WHEN 'high' THEN 3
                            WHEN 'normal' THEN 4
                            WHEN 'low' THEN 5
                        END,
                        created_at ASC
                ) as position
            FROM sequencing_jobs 
            WHERE status IN ('queued', 'validated')
        )
        SELECT position FROM ranked_jobs WHERE id = $1
        "#
    )
    .bind(job_id)
    .fetch_optional(&state.db_pool.pool)
    .await?;

    Ok(position.unwrap_or(999))
}

/// Request structures
#[derive(serde::Deserialize)]
pub struct QueueQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub priority: Option<JobPriority>,
    pub platform: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct PriorityUpdateRequest {
    pub new_priority: JobPriority,
    pub updated_by: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct SchedulingToggleRequest {
    pub enable_scheduling: bool,
    pub changed_by: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct EstimationQuery {
    pub job_ids: Option<Vec<Uuid>>,
}
