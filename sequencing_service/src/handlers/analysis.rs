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

/// Start analysis pipeline for a sequencing job
pub async fn start_analysis(
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
    Json(request): Json<StartAnalysisRequest>,
) -> Result<Json<serde_json::Value>> {
    // Verify job exists and is in the right state
    let job = sqlx::query_as::<_, SequencingJob>(
        "SELECT * FROM sequencing_jobs WHERE id = $1"
    )
    .bind(job_id)
    .fetch_optional(&state.db_pool.pool)
    .await?
    .ok_or(SequencingError::JobNotFound { job_id })?;

    if job.status != JobStatus::Completed {
        return Err(SequencingError::InvalidJobState {
            current_state: job.status.to_string(),
            required_state: "completed".to_string(),
        });
    }

    // Create analysis record
    let analysis_id = Uuid::new_v4();
    let analysis = sqlx::query_as::<_, AnalysisJob>(
        r#"
        INSERT INTO analysis_jobs (
            id, job_id, pipeline_type, pipeline_version, parameters,
            status, created_at, created_by
        ) VALUES ($1, $2, $3, $4, $5, $6, NOW(), $7)
        RETURNING *
        "#
    )
    .bind(analysis_id)
    .bind(job_id)
    .bind(&request.pipeline_type)
    .bind(&request.pipeline_version)
    .bind(&request.parameters)
    .bind(AnalysisStatus::Queued)
    .bind(request.created_by.as_deref())
    .fetch_one(&state.db_pool.pool)
    .await?;

    // Update job status to indicate analysis started
    sqlx::query(
        "UPDATE sequencing_jobs SET analysis_status = 'running', updated_at = NOW() WHERE id = $1"
    )
    .bind(job_id)
    .execute(&state.db_pool.pool)
    .await?;

    Ok(Json(json!({
        "success": true,
        "data": analysis,
        "message": "Analysis pipeline started successfully"
    })))
}

/// Get analysis status and results
pub async fn get_analysis_status(
    State(state): State<AppState>,
    Path(analysis_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let analysis = sqlx::query_as::<_, AnalysisJob>(
        "SELECT * FROM analysis_jobs WHERE id = $1"
    )
    .bind(analysis_id)
    .fetch_optional(&state.db_pool.pool)
    .await?
    .ok_or(SequencingError::AnalysisNotFound { analysis_id })?;

    // Get quality metrics if available
    let quality_metrics = sqlx::query_as::<_, QualityMetrics>(
        "SELECT * FROM quality_metrics WHERE analysis_id = $1"
    )
    .bind(analysis_id)
    .fetch_optional(&state.db_pool.pool)
    .await?;

    // Get analysis results if completed
    let results = if analysis.status == AnalysisStatus::Completed {
        sqlx::query_as::<_, AnalysisResult>(
            "SELECT * FROM analysis_results WHERE analysis_id = $1"
        )
        .bind(analysis_id)
        .fetch_all(&state.db_pool.pool)
        .await?
    } else {
        Vec::new()
    };

    Ok(Json(json!({
        "success": true,
        "data": {
            "analysis": analysis,
            "quality_metrics": quality_metrics,
            "results": results,
            "result_count": results.len()
        }
    })))
}

/// List analyses for a job
pub async fn list_job_analyses(
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
    Query(query): Query<AnalysisListQuery>,
) -> Result<Json<serde_json::Value>> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).min(100);
    let offset = (page - 1) * page_size;

    let mut where_conditions = vec!["job_id = $1".to_string()];
    let mut param_count = 1;

    if let Some(status) = &query.status {
        param_count += 1;
        where_conditions.push(format!("status = ${}", param_count));
    }

    if let Some(pipeline_type) = &query.pipeline_type {
        param_count += 1;
        where_conditions.push(format!("pipeline_type = ${}", param_count));
    }

    let where_clause = where_conditions.join(" AND ");
    
    // Get total count
    let total_count: i64 = sqlx::query_scalar(&format!(
        "SELECT COUNT(*) FROM analysis_jobs WHERE {}", where_clause
    ))
    .bind(job_id)
    .fetch_one(&state.db_pool.pool)
    .await?;

    // Get analyses
    let analyses = sqlx::query_as::<_, AnalysisJob>(&format!(
        "SELECT * FROM analysis_jobs WHERE {} ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        where_clause
    ))
    .bind(job_id)
    .bind(page_size)
    .bind(offset)
    .fetch_all(&state.db_pool.pool)
    .await?;

    let total_pages = (total_count + page_size - 1) / page_size;

    Ok(Json(json!({
        "success": true,
        "data": {
            "analyses": analyses,
            "pagination": {
                "page": page,
                "page_size": page_size,
                "total_count": total_count,
                "total_pages": total_pages
            }
        }
    })))
}

/// Update analysis status
pub async fn update_analysis_status(
    State(state): State<AppState>,
    Path(analysis_id): Path<Uuid>,
    Json(request): Json<UpdateAnalysisStatusRequest>,
) -> Result<Json<serde_json::Value>> {
    let analysis = sqlx::query_as::<_, AnalysisJob>(
        r#"
        UPDATE analysis_jobs 
        SET status = $2, 
            progress_percentage = $3,
            error_message = $4,
            updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#
    )
    .bind(analysis_id)
    .bind(&request.status)
    .bind(request.progress_percentage)
    .bind(request.error_message.as_deref())
    .fetch_optional(&state.db_pool.pool)
    .await?
    .ok_or(SequencingError::AnalysisNotFound { analysis_id })?;

    // If analysis completed, update the parent job
    if request.status == AnalysisStatus::Completed {
        sqlx::query(
            "UPDATE sequencing_jobs SET analysis_status = 'completed', updated_at = NOW() WHERE id = $1"
        )
        .bind(analysis.job_id)
        .execute(&state.db_pool.pool)
        .await?;
    } else if request.status == AnalysisStatus::Failed {
        sqlx::query(
            "UPDATE sequencing_jobs SET analysis_status = 'failed', updated_at = NOW() WHERE id = $1"
        )
        .bind(analysis.job_id)
        .execute(&state.db_pool.pool)
        .await?;
    }

    Ok(Json(json!({
        "success": true,
        "data": analysis,
        "message": "Analysis status updated successfully"
    })))
}

/// Submit quality metrics for an analysis
pub async fn submit_quality_metrics(
    State(state): State<AppState>,
    Path(analysis_id): Path<Uuid>,
    Json(request): Json<QualityMetricsRequest>,
) -> Result<Json<serde_json::Value>> {
    // Verify analysis exists
    let analysis = sqlx::query("SELECT id FROM analysis_jobs WHERE id = $1")
        .bind(analysis_id)
        .fetch_optional(&state.db_pool.pool)
        .await?
        .ok_or(SequencingError::AnalysisNotFound { analysis_id })?;

    let metrics = sqlx::query_as::<_, QualityMetrics>(
        r#"
        INSERT INTO quality_metrics (
            id, analysis_id, read_count, base_count, q30_percentage,
            gc_content, mean_quality_score, duplication_rate,
            adapter_content, custom_metrics, calculated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW())
        ON CONFLICT (analysis_id) 
        DO UPDATE SET
            read_count = EXCLUDED.read_count,
            base_count = EXCLUDED.base_count,
            q30_percentage = EXCLUDED.q30_percentage,
            gc_content = EXCLUDED.gc_content,
            mean_quality_score = EXCLUDED.mean_quality_score,
            duplication_rate = EXCLUDED.duplication_rate,
            adapter_content = EXCLUDED.adapter_content,
            custom_metrics = EXCLUDED.custom_metrics,
            calculated_at = EXCLUDED.calculated_at
        RETURNING *
        "#
    )
    .bind(Uuid::new_v4())
    .bind(analysis_id)
    .bind(request.read_count)
    .bind(request.base_count)
    .bind(request.q30_percentage)
    .bind(request.gc_content)
    .bind(request.mean_quality_score)
    .bind(request.duplication_rate)
    .bind(request.adapter_content)
    .bind(&request.custom_metrics)
    .fetch_one(&state.db_pool.pool)
    .await?;

    Ok(Json(json!({
        "success": true,
        "data": metrics,
        "message": "Quality metrics submitted successfully"
    })))
}

/// Submit analysis results
pub async fn submit_analysis_results(
    State(state): State<AppState>,
    Path(analysis_id): Path<Uuid>,
    Json(request): Json<SubmitResultsRequest>,
) -> Result<Json<serde_json::Value>> {
    // Verify analysis exists
    let analysis = sqlx::query("SELECT id FROM analysis_jobs WHERE id = $1")
        .bind(analysis_id)
        .fetch_optional(&state.db_pool.pool)
        .await?
        .ok_or(SequencingError::AnalysisNotFound { analysis_id })?;

    let mut created_results = Vec::new();

    for result_data in request.results {
        let result = sqlx::query_as::<_, AnalysisResult>(
            r#"
            INSERT INTO analysis_results (
                id, analysis_id, result_type, file_path, file_size,
                checksum, description, metadata, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(analysis_id)
        .bind(&result_data.result_type)
        .bind(&result_data.file_path)
        .bind(result_data.file_size)
        .bind(result_data.checksum.as_deref())
        .bind(result_data.description.as_deref())
        .bind(&result_data.metadata)
        .fetch_one(&state.db_pool.pool)
        .await?;

        created_results.push(result);
    }

    Ok(Json(json!({
        "success": true,
        "data": {
            "results": created_results,
            "count": created_results.len()
        },
        "message": "Analysis results submitted successfully"
    })))
}

/// Get quality control summary for multiple jobs
pub async fn get_qc_summary(
    State(state): State<AppState>,
    Query(query): Query<QCSummaryQuery>,
) -> Result<Json<serde_json::Value>> {
    let period_days = query.period_days.unwrap_or(30);
    let start_date = Utc::now() - chrono::Duration::days(period_days);

    // Get QC metrics summary
    let qc_stats = sqlx::query!(
        r#"
        SELECT 
            COUNT(*) as total_analyses,
            AVG(qm.q30_percentage) as avg_q30,
            AVG(qm.gc_content) as avg_gc_content,
            AVG(qm.mean_quality_score) as avg_quality_score,
            AVG(qm.duplication_rate) as avg_duplication_rate,
            COUNT(CASE WHEN qm.q30_percentage >= 80 THEN 1 END) as high_quality_count,
            COUNT(CASE WHEN qm.q30_percentage < 70 THEN 1 END) as low_quality_count
        FROM analysis_jobs aj
        LEFT JOIN quality_metrics qm ON aj.id = qm.analysis_id
        WHERE aj.created_at > $1 AND aj.status = 'completed'
        "#,
        start_date
    )
    .fetch_one(&state.db_pool.pool)
    .await?;

    // Get platform-specific QC metrics
    let platform_qc = sqlx::query!(
        r#"
        SELECT 
            sj.platform,
            COUNT(*) as job_count,
            AVG(qm.q30_percentage) as avg_q30,
            AVG(qm.mean_quality_score) as avg_quality_score
        FROM analysis_jobs aj
        JOIN sequencing_jobs sj ON aj.job_id = sj.id
        LEFT JOIN quality_metrics qm ON aj.id = qm.analysis_id
        WHERE aj.created_at > $1 AND aj.status = 'completed'
        GROUP BY sj.platform
        ORDER BY job_count DESC
        "#,
        start_date
    )
    .fetch_all(&state.db_pool.pool)
    .await?;

    // Get daily QC trends
    let daily_trends = sqlx::query!(
        r#"
        SELECT 
            DATE(aj.created_at) as date,
            COUNT(*) as analysis_count,
            AVG(qm.q30_percentage) as avg_q30,
            AVG(qm.mean_quality_score) as avg_quality_score
        FROM analysis_jobs aj
        LEFT JOIN quality_metrics qm ON aj.id = qm.analysis_id
        WHERE aj.created_at > $1 AND aj.status = 'completed'
        GROUP BY DATE(aj.created_at)
        ORDER BY date
        "#,
        start_date
    )
    .fetch_all(&state.db_pool.pool)
    .await?;

    Ok(Json(json!({
        "success": true,
        "data": {
            "period_days": period_days,
            "overview": {
                "total_analyses": qc_stats.total_analyses,
                "avg_q30_percentage": qc_stats.avg_q30,
                "avg_gc_content": qc_stats.avg_gc_content,
                "avg_quality_score": qc_stats.avg_quality_score,
                "avg_duplication_rate": qc_stats.avg_duplication_rate,
                "high_quality_count": qc_stats.high_quality_count,
                "low_quality_count": qc_stats.low_quality_count
            },
            "platform_metrics": platform_qc.into_iter().map(|row| json!({
                "platform": row.platform,
                "job_count": row.job_count,
                "avg_q30": row.avg_q30,
                "avg_quality_score": row.avg_quality_score
            })).collect::<Vec<_>>(),
            "daily_trends": daily_trends.into_iter().map(|row| json!({
                "date": row.date,
                "analysis_count": row.analysis_count,
                "avg_q30": row.avg_q30,
                "avg_quality_score": row.avg_quality_score
            })).collect::<Vec<_>>()
        }
    })))
}

/// Get available analysis pipelines
pub async fn get_analysis_pipelines(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    let pipelines = sqlx::query_as::<_, AnalysisPipeline>(
        "SELECT * FROM analysis_pipelines WHERE is_active = true ORDER BY pipeline_type, version DESC"
    )
    .fetch_all(&state.db_pool.pool)
    .await?;

    // Group by pipeline type
    let mut grouped_pipelines: std::collections::HashMap<String, Vec<AnalysisPipeline>> = std::collections::HashMap::new();
    
    for pipeline in pipelines {
        grouped_pipelines
            .entry(pipeline.pipeline_type.clone())
            .or_insert_with(Vec::new)
            .push(pipeline);
    }

    Ok(Json(json!({
        "success": true,
        "data": {
            "pipelines": grouped_pipelines,
            "available_types": grouped_pipelines.keys().collect::<Vec<_>>()
        }
    })))
}

/// Cancel running analysis
pub async fn cancel_analysis(
    State(state): State<AppState>,
    Path(analysis_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let analysis = sqlx::query_as::<_, AnalysisJob>(
        r#"
        UPDATE analysis_jobs 
        SET status = 'cancelled', 
            updated_at = NOW(),
            error_message = 'Analysis cancelled by user request'
        WHERE id = $1 AND status IN ('queued', 'running')
        RETURNING *
        "#
    )
    .bind(analysis_id)
    .fetch_optional(&state.db_pool.pool)
    .await?
    .ok_or(SequencingError::AnalysisNotFound { analysis_id })?;

    // Update parent job analysis status
    sqlx::query(
        "UPDATE sequencing_jobs SET analysis_status = 'cancelled', updated_at = NOW() WHERE id = $1"
    )
    .bind(analysis.job_id)
    .execute(&state.db_pool.pool)
    .await?;

    Ok(Json(json!({
        "success": true,
        "data": analysis,
        "message": "Analysis cancelled successfully"
    })))
}

/// Request and response structures
#[derive(serde::Deserialize)]
pub struct StartAnalysisRequest {
    pub pipeline_type: String,
    pub pipeline_version: String,
    pub parameters: serde_json::Value,
    pub created_by: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct UpdateAnalysisStatusRequest {
    pub status: AnalysisStatus,
    pub progress_percentage: Option<f32>,
    pub error_message: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct QualityMetricsRequest {
    pub read_count: Option<i64>,
    pub base_count: Option<i64>,
    pub q30_percentage: Option<f32>,
    pub gc_content: Option<f32>,
    pub mean_quality_score: Option<f32>,
    pub duplication_rate: Option<f32>,
    pub adapter_content: Option<f32>,
    pub custom_metrics: serde_json::Value,
}

#[derive(serde::Deserialize)]
pub struct SubmitResultsRequest {
    pub results: Vec<ResultData>,
}

#[derive(serde::Deserialize)]
pub struct ResultData {
    pub result_type: String,
    pub file_path: String,
    pub file_size: Option<i64>,
    pub checksum: Option<String>,
    pub description: Option<String>,
    pub metadata: serde_json::Value,
}

#[derive(serde::Deserialize)]
pub struct AnalysisListQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub status: Option<AnalysisStatus>,
    pub pipeline_type: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct QCSummaryQuery {
    pub period_days: Option<i64>,
}
