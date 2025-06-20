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

/// Set quality control thresholds for a platform
pub async fn set_qc_thresholds(
    State(state): State<AppState>,
    Path(platform): Path<String>,
    Json(request): Json<SetQCThresholdsRequest>,
) -> Result<Json<serde_json::Value>> {
    let threshold_id = Uuid::new_v4();
    
    let qc_threshold = sqlx::query_as::<_, QCThreshold>(
        r#"
        INSERT INTO qc_thresholds (
            id, platform, threshold_type, min_value, max_value,
            warning_min, warning_max, is_active, created_at, created_by
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), $9)
        ON CONFLICT (platform, threshold_type)
        DO UPDATE SET
            min_value = EXCLUDED.min_value,
            max_value = EXCLUDED.max_value,
            warning_min = EXCLUDED.warning_min,
            warning_max = EXCLUDED.warning_max,
            is_active = EXCLUDED.is_active,
            updated_at = NOW()
        RETURNING *
        "#
    )
    .bind(threshold_id)
    .bind(&platform)
    .bind(&request.threshold_type)
    .bind(request.min_value)
    .bind(request.max_value)
    .bind(request.warning_min)
    .bind(request.warning_max)
    .bind(request.is_active.unwrap_or(true))
    .bind(request.created_by.as_deref())
    .fetch_one(&state.db_pool.pool)
    .await?;

    Ok(Json(json!({
        "success": true,
        "data": qc_threshold,
        "message": "QC threshold set successfully"
    })))
}

/// Get quality control thresholds for a platform
pub async fn get_qc_thresholds(
    State(state): State<AppState>,
    Path(platform): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let thresholds = sqlx::query_as::<_, QCThreshold>(
        "SELECT * FROM qc_thresholds WHERE platform = $1 AND is_active = true ORDER BY threshold_type"
    )
    .bind(&platform)
    .fetch_all(&state.db_pool.pool)
    .await?;

    // Group thresholds by type for easier consumption
    let mut grouped_thresholds = std::collections::HashMap::new();
    for threshold in &thresholds {
        grouped_thresholds.insert(threshold.threshold_type.clone(), threshold);
    }

    Ok(Json(json!({
        "success": true,
        "data": {
            "platform": platform,
            "thresholds": thresholds,
            "grouped_thresholds": grouped_thresholds,
            "threshold_count": thresholds.len()
        }
    })))
}

/// Evaluate quality metrics against thresholds
pub async fn evaluate_quality_metrics(
    State(state): State<AppState>,
    Path(analysis_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    // Get the analysis and its quality metrics
    let analysis = sqlx::query_as::<_, AnalysisJob>(
        "SELECT * FROM analysis_jobs WHERE id = $1"
    )
    .bind(analysis_id)
    .fetch_optional(&state.db_pool.pool)
    .await?
    .ok_or(SequencingError::AnalysisNotFound { analysis_id })?;

    let metrics = sqlx::query_as::<_, QualityMetrics>(
        "SELECT * FROM quality_metrics WHERE analysis_id = $1"
    )
    .bind(analysis_id)
    .fetch_optional(&state.db_pool.pool)
    .await?
    .ok_or(SequencingError::QualityMetricsNotFound { analysis_id })?;

    // Get the sequencing job to determine platform
    let job = sqlx::query_as::<_, SequencingJob>(
        "SELECT * FROM sequencing_jobs WHERE id = $1"
    )
    .bind(analysis.job_id)
    .fetch_one(&state.db_pool.pool)
    .await?;

    // Get platform-specific thresholds
    let thresholds = sqlx::query_as::<_, QCThreshold>(
        "SELECT * FROM qc_thresholds WHERE platform = $1 AND is_active = true"
    )
    .bind(&job.platform)
    .fetch_all(&state.db_pool.pool)
    .await?;

    // Evaluate each metric against thresholds
    let evaluation_result = evaluate_metrics_against_thresholds(&metrics, &thresholds);

    // Save evaluation results
    let evaluation_id = Uuid::new_v4();
    let qc_evaluation = sqlx::query_as::<_, QCEvaluation>(
        r#"
        INSERT INTO qc_evaluations (
            id, analysis_id, overall_status, failed_checks, warning_checks,
            evaluation_details, evaluated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, NOW())
        RETURNING *
        "#
    )
    .bind(evaluation_id)
    .bind(analysis_id)
    .bind(&evaluation_result.overall_status)
    .bind(&evaluation_result.failed_checks)
    .bind(&evaluation_result.warning_checks)
    .bind(&evaluation_result.details)
    .fetch_one(&state.db_pool.pool)
    .await?;

    Ok(Json(json!({
        "success": true,
        "data": {
            "evaluation": qc_evaluation,
            "metrics": metrics,
            "summary": {
                "overall_status": evaluation_result.overall_status,
                "passed_checks": evaluation_result.passed_checks,
                "failed_checks": evaluation_result.failed_checks.as_array().unwrap_or(&vec![]).len(),
                "warning_checks": evaluation_result.warning_checks.as_array().unwrap_or(&vec![]).len()
            }
        },
        "message": format!("Quality evaluation completed with status: {}", evaluation_result.overall_status)
    })))
}

/// Get quality metrics summary for a job
pub async fn get_quality_summary(
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    // Get all analyses for this job
    let analyses = sqlx::query_as::<_, AnalysisJob>(
        "SELECT * FROM analysis_jobs WHERE job_id = $1 ORDER BY created_at DESC"
    )
    .bind(job_id)
    .fetch_all(&state.db_pool.pool)
    .await?;

    if analyses.is_empty() {
        return Ok(Json(json!({
            "success": true,
            "data": {
                "job_id": job_id,
                "message": "No analyses found for this job"
            }
        })));
    }

    let mut quality_summaries = Vec::new();

    for analysis in &analyses {
        // Get quality metrics
        if let Ok(Some(metrics)) = sqlx::query_as::<_, QualityMetrics>(
            "SELECT * FROM quality_metrics WHERE analysis_id = $1"
        )
        .bind(analysis.id)
        .fetch_optional(&state.db_pool.pool)
        .await
        {
            // Get latest evaluation
            let evaluation = sqlx::query_as::<_, QCEvaluation>(
                "SELECT * FROM qc_evaluations WHERE analysis_id = $1 ORDER BY evaluated_at DESC LIMIT 1"
            )
            .bind(analysis.id)
            .fetch_optional(&state.db_pool.pool)
            .await?;

            quality_summaries.push(json!({
                "analysis_id": analysis.id,
                "pipeline_type": analysis.pipeline_type,
                "metrics": metrics,
                "evaluation": evaluation,
                "created_at": analysis.created_at
            }));
        }
    }

    // Calculate overall job quality status
    let overall_status = calculate_overall_job_status(&quality_summaries);

    Ok(Json(json!({
        "success": true,
        "data": {
            "job_id": job_id,
            "overall_status": overall_status,
            "analyses": quality_summaries,
            "analysis_count": quality_summaries.len()
        }
    })))
}

/// Get quality trends across multiple jobs
pub async fn get_quality_trends(
    State(state): State<AppState>,
    Query(query): Query<QualityTrendsQuery>,
) -> Result<Json<serde_json::Value>> {
    let period_days = query.period_days.unwrap_or(30);
    let start_date = Utc::now() - chrono::Duration::days(period_days);

    // Get quality metrics trends
    let quality_trends = sqlx::query!(
        r#"
        SELECT 
            DATE(qm.calculated_at) as date,
            sj.platform,
            COUNT(*) as sample_count,
            AVG(qm.q30_percentage) as avg_q30,
            AVG(qm.mean_quality_score) as avg_quality_score,
            AVG(qm.gc_content) as avg_gc_content,
            AVG(qm.duplication_rate) as avg_duplication_rate,
            COUNT(CASE WHEN qce.overall_status = 'pass' THEN 1 END) as passed_samples,
            COUNT(CASE WHEN qce.overall_status = 'fail' THEN 1 END) as failed_samples
        FROM quality_metrics qm
        JOIN analysis_jobs aj ON qm.analysis_id = aj.id
        JOIN sequencing_jobs sj ON aj.job_id = sj.id
        LEFT JOIN qc_evaluations qce ON qm.analysis_id = qce.analysis_id
        WHERE qm.calculated_at > $1
        GROUP BY DATE(qm.calculated_at), sj.platform
        ORDER BY date DESC, sj.platform
        "#,
        start_date
    )
    .fetch_all(&state.db_pool.pool)
    .await?;

    // Group by platform
    let mut platform_trends: std::collections::HashMap<String, Vec<_>> = std::collections::HashMap::new();
    for trend in quality_trends {
        platform_trends
            .entry(trend.platform.unwrap_or_default())
            .or_insert_with(Vec::new)
            .push(json!({
                "date": trend.date,
                "sample_count": trend.sample_count,
                "avg_q30": trend.avg_q30,
                "avg_quality_score": trend.avg_quality_score,
                "avg_gc_content": trend.avg_gc_content,
                "avg_duplication_rate": trend.avg_duplication_rate,
                "passed_samples": trend.passed_samples,
                "failed_samples": trend.failed_samples,
                "pass_rate": if trend.sample_count > 0 {
                    (trend.passed_samples.unwrap_or(0) as f64 / trend.sample_count as f64 * 100.0).round()
                } else { 0.0 }
            }));
    }

    Ok(Json(json!({
        "success": true,
        "data": {
            "period_days": period_days,
            "platform_trends": platform_trends,
            "platforms": platform_trends.keys().collect::<Vec<_>>()
        }
    })))
}

/// Generate quality control report
pub async fn generate_qc_report(
    State(state): State<AppState>,
    Json(request): Json<GenerateQCReportRequest>,
) -> Result<Json<serde_json::Value>> {
    let start_date = request.start_date.unwrap_or_else(|| Utc::now() - chrono::Duration::days(7));
    let end_date = request.end_date.unwrap_or_else(Utc::now);

    // Get comprehensive quality data for the period
    let report_data = sqlx::query!(
        r#"
        SELECT 
            sj.id as job_id,
            sj.job_name,
            sj.platform,
            sj.created_at as job_created,
            aj.id as analysis_id,
            aj.pipeline_type,
            qm.q30_percentage,
            qm.mean_quality_score,
            qm.gc_content,
            qm.duplication_rate,
            qm.read_count,
            qm.base_count,
            qce.overall_status,
            qce.failed_checks,
            qce.warning_checks
        FROM sequencing_jobs sj
        JOIN analysis_jobs aj ON sj.id = aj.job_id
        LEFT JOIN quality_metrics qm ON aj.id = qm.analysis_id
        LEFT JOIN qc_evaluations qce ON aj.id = qce.analysis_id
        WHERE sj.created_at BETWEEN $1 AND $2
        ORDER BY sj.created_at DESC
        "#,
        start_date,
        end_date
    )
    .fetch_all(&state.db_pool.pool)
    .await?;

    // Calculate summary statistics
    let total_jobs = report_data.len();
    let passed_jobs = report_data.iter().filter(|r| r.overall_status.as_deref() == Some("pass")).count();
    let failed_jobs = report_data.iter().filter(|r| r.overall_status.as_deref() == Some("fail")).count();
    let warning_jobs = report_data.iter().filter(|r| r.overall_status.as_deref() == Some("warning")).count();

    let avg_q30: f64 = report_data.iter()
        .filter_map(|r| r.q30_percentage)
        .map(|q| q as f64)
        .sum::<f64>() / report_data.len().max(1) as f64;

    let avg_quality_score: f64 = report_data.iter()
        .filter_map(|r| r.mean_quality_score)
        .map(|q| q as f64)
        .sum::<f64>() / report_data.len().max(1) as f64;

    // Group by platform
    let mut platform_stats: std::collections::HashMap<String, serde_json::Value> = std::collections::HashMap::new();
    for row in &report_data {
        let platform = row.platform.as_deref().unwrap_or("unknown");
        let entry = platform_stats.entry(platform.to_string()).or_insert_with(|| json!({
            "total_jobs": 0,
            "passed_jobs": 0,
            "failed_jobs": 0,
            "avg_q30": 0.0,
            "avg_quality_score": 0.0
        }));

        entry["total_jobs"] = entry["total_jobs"].as_i64().unwrap_or(0) + 1;
        if row.overall_status.as_deref() == Some("pass") {
            entry["passed_jobs"] = entry["passed_jobs"].as_i64().unwrap_or(0) + 1;
        } else if row.overall_status.as_deref() == Some("fail") {
            entry["failed_jobs"] = entry["failed_jobs"].as_i64().unwrap_or(0) + 1;
        }
    }

    // Save report
    let report_id = Uuid::new_v4();
    let report = sqlx::query_as::<_, QCReport>(
        r#"
        INSERT INTO qc_reports (
            id, report_type, start_date, end_date, platform_filter,
            summary_data, detailed_data, generated_at, generated_by
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), $8)
        RETURNING *
        "#
    )
    .bind(report_id)
    .bind("comprehensive")
    .bind(start_date)
    .bind(end_date)
    .bind(request.platform_filter.as_deref())
    .bind(json!({
        "total_jobs": total_jobs,
        "passed_jobs": passed_jobs,
        "failed_jobs": failed_jobs,
        "warning_jobs": warning_jobs,
        "pass_rate": if total_jobs > 0 { (passed_jobs as f64 / total_jobs as f64 * 100.0).round() } else { 0.0 },
        "avg_q30": avg_q30,
        "avg_quality_score": avg_quality_score,
        "platform_stats": platform_stats
    }))
    .bind(json!(report_data))
    .bind(request.generated_by.as_deref())
    .fetch_one(&state.db_pool.pool)
    .await?;

    Ok(Json(json!({
        "success": true,
        "data": {
            "report": report,
            "summary": {
                "period": format!("{} to {}", start_date.format("%Y-%m-%d"), end_date.format("%Y-%m-%d")),
                "total_jobs": total_jobs,
                "passed_jobs": passed_jobs,
                "failed_jobs": failed_jobs,
                "warning_jobs": warning_jobs,
                "pass_rate": if total_jobs > 0 { (passed_jobs as f64 / total_jobs as f64 * 100.0).round() } else { 0.0 },
                "avg_q30": avg_q30.round(),
                "avg_quality_score": avg_quality_score.round()
            }
        },
        "message": "QC report generated successfully"
    })))
}

/// Get quality control alerts
pub async fn get_qc_alerts(
    State(state): State<AppState>,
    Query(query): Query<QCAlertQuery>,
) -> Result<Json<serde_json::Value>> {
    let severity = query.severity.as_deref().unwrap_or("all");
    let limit = query.limit.unwrap_or(50).min(200);

    let mut where_conditions = vec!["1=1".to_string()];
    
    if severity != "all" {
        where_conditions.push(format!("severity = '{}'", severity));
    }

    let where_clause = where_conditions.join(" AND ");

    let alerts = sqlx::query_as::<_, QCAlert>(&format!(
        r#"
        SELECT * FROM qc_alerts 
        WHERE {} 
        ORDER BY created_at DESC 
        LIMIT {}
        "#,
        where_clause, limit
    ))
    .fetch_all(&state.db_pool.pool)
    .await?;

    // Group alerts by severity
    let mut alert_summary = std::collections::HashMap::new();
    for alert in &alerts {
        *alert_summary.entry(alert.severity.clone()).or_insert(0) += 1;
    }

    Ok(Json(json!({
        "success": true,
        "data": {
            "alerts": alerts,
            "summary": alert_summary,
            "total_alerts": alerts.len(),
            "filters_applied": {
                "severity": severity,
                "limit": limit
            }
        }
    })))
}

/// Create quality control alert
pub async fn create_qc_alert(
    State(state): State<AppState>,
    Json(request): Json<CreateQCAlertRequest>,
) -> Result<Json<serde_json::Value>> {
    let alert_id = Uuid::new_v4();
    
    let alert = sqlx::query_as::<_, QCAlert>(
        r#"
        INSERT INTO qc_alerts (
            id, analysis_id, job_id, alert_type, severity,
            message, details, created_at, created_by
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), $8)
        RETURNING *
        "#
    )
    .bind(alert_id)
    .bind(request.analysis_id)
    .bind(request.job_id)
    .bind(&request.alert_type)
    .bind(&request.severity)
    .bind(&request.message)
    .bind(&request.details)
    .bind(request.created_by.as_deref())
    .fetch_one(&state.db_pool.pool)
    .await?;

    Ok(Json(json!({
        "success": true,
        "data": alert,
        "message": "QC alert created successfully"
    })))
}

/// Helper functions
fn evaluate_metrics_against_thresholds(
    metrics: &QualityMetrics,
    thresholds: &[QCThreshold],
) -> QCEvaluationResult {
    let mut passed_checks = 0;
    let mut failed_checks = Vec::new();
    let mut warning_checks = Vec::new();
    let mut all_checks = Vec::new();

    for threshold in thresholds {
        let metric_value = match threshold.threshold_type.as_str() {
            "q30_percentage" => metrics.q30_percentage.map(|v| v as f64),
            "mean_quality_score" => metrics.mean_quality_score.map(|v| v as f64),
            "gc_content" => metrics.gc_content.map(|v| v as f64),
            "duplication_rate" => metrics.duplication_rate.map(|v| v as f64),
            "adapter_content" => metrics.adapter_content.map(|v| v as f64),
            _ => None,
        };

        if let Some(value) = metric_value {
            let check_result = evaluate_single_metric(threshold, value);
            all_checks.push(check_result.clone());

            match check_result["status"].as_str().unwrap_or("unknown") {
                "pass" => passed_checks += 1,
                "fail" => failed_checks.push(check_result),
                "warning" => warning_checks.push(check_result),
                _ => {}
            }
        }
    }

    let overall_status = if !failed_checks.is_empty() {
        "fail"
    } else if !warning_checks.is_empty() {
        "warning"
    } else {
        "pass"
    };

    QCEvaluationResult {
        overall_status: overall_status.to_string(),
        passed_checks,
        failed_checks: json!(failed_checks),
        warning_checks: json!(warning_checks),
        details: json!({
            "all_checks": all_checks,
            "evaluation_timestamp": Utc::now()
        }),
    }
}

fn evaluate_single_metric(threshold: &QCThreshold, value: f64) -> serde_json::Value {
    let status = if let (Some(min), Some(max)) = (threshold.min_value, threshold.max_value) {
        if value < min || value > max {
            "fail"
        } else if let (Some(warn_min), Some(warn_max)) = (threshold.warning_min, threshold.warning_max) {
            if value < warn_min || value > warn_max {
                "warning"
            } else {
                "pass"
            }
        } else {
            "pass"
        }
    } else if let Some(min) = threshold.min_value {
        if value < min {
            "fail"
        } else if let Some(warn_min) = threshold.warning_min {
            if value < warn_min {
                "warning"
            } else {
                "pass"
            }
        } else {
            "pass"
        }
    } else if let Some(max) = threshold.max_value {
        if value > max {
            "fail"
        } else if let Some(warn_max) = threshold.warning_max {
            if value > warn_max {
                "warning"
            } else {
                "pass"
            }
        } else {
            "pass"
        }
    } else {
        "unknown"
    };

    json!({
        "metric_type": threshold.threshold_type,
        "value": value,
        "status": status,
        "threshold": {
            "min_value": threshold.min_value,
            "max_value": threshold.max_value,
            "warning_min": threshold.warning_min,
            "warning_max": threshold.warning_max
        }
    })
}

fn calculate_overall_job_status(quality_summaries: &[serde_json::Value]) -> String {
    if quality_summaries.is_empty() {
        return "unknown".to_string();
    }

    let has_failed = quality_summaries.iter()
        .any(|s| s["evaluation"]["overall_status"].as_str() == Some("fail"));
    
    let has_warning = quality_summaries.iter()
        .any(|s| s["evaluation"]["overall_status"].as_str() == Some("warning"));

    if has_failed {
        "fail".to_string()
    } else if has_warning {
        "warning".to_string()
    } else {
        "pass".to_string()
    }
}

/// Request and response structures
#[derive(serde::Deserialize)]
pub struct SetQCThresholdsRequest {
    pub threshold_type: String,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub warning_min: Option<f64>,
    pub warning_max: Option<f64>,
    pub is_active: Option<bool>,
    pub created_by: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct QualityTrendsQuery {
    pub period_days: Option<i64>,
}

#[derive(serde::Deserialize)]
pub struct GenerateQCReportRequest {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub platform_filter: Option<String>,
    pub generated_by: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct QCAlertQuery {
    pub severity: Option<String>,
    pub limit: Option<i64>,
}

#[derive(serde::Deserialize)]
pub struct CreateQCAlertRequest {
    pub analysis_id: Option<Uuid>,
    pub job_id: Option<Uuid>,
    pub alert_type: String,
    pub severity: String,
    pub message: String,
    pub details: serde_json::Value,
    pub created_by: Option<String>,
}

struct QCEvaluationResult {
    pub overall_status: String,
    pub passed_checks: i32,
    pub failed_checks: serde_json::Value,
    pub warning_checks: serde_json::Value,
    pub details: serde_json::Value,
}
