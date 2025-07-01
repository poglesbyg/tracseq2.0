use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{PgPool, FromRow};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    assembly::AppComponents,
    models::qc::*,
};

// Request/Response structs
#[derive(Debug, Deserialize)]
pub struct ListQcReviewsQuery {
    pub status: Option<String>,
    pub review_type: Option<String>,
    pub priority: Option<String>,
    pub reviewer_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct CreateQcReviewRequest {
    pub entity_type: String,
    pub entity_id: Uuid,
    pub review_type: String,
    pub priority: Option<String>,
    pub requested_by: Uuid,
    pub notes: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct CompleteQcReviewRequest {
    pub decision: String,
    pub comments: Option<String>,
    pub deviations: Option<Vec<String>>,
    pub corrective_actions: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLibraryPrepQcRequest {
    pub library_prep_id: Uuid,
    pub concentration_ng_ul: Option<f64>,
    pub volume_ul: Option<f64>,
    pub total_yield_ng: Option<f64>,
    pub fragment_size_bp: Option<i32>,
    pub quality_score: Option<f64>,
    pub performed_by: Uuid,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateControlSampleRequest {
    pub name: String,
    pub control_type: String,
    pub expected_concentration: Option<f64>,
    pub expected_fragment_size: Option<i32>,
    pub tolerance_percent: Option<f64>,
    pub lot_number: Option<String>,
    pub expiry_date: Option<DateTime<Utc>>,
    pub storage_location: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct RecordControlResultRequest {
    pub control_sample_id: Uuid,
    pub run_id: Option<Uuid>,
    pub measured_concentration: Option<f64>,
    pub measured_fragment_size: Option<i32>,
    pub performed_by: Uuid,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct QcDashboardStats {
    pub pending_reviews: i64,
    pub completed_today: i64,
    pub failed_today: i64,
    pub pass_rate_week: f64,
    pub average_turnaround_hours: f64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct QcMetricTrend {
    pub metric_name: String,
    pub date: DateTime<Utc>,
    pub average_value: f64,
    pub sample_count: i64,
    pub pass_rate: f64,
}

// Manager struct for QC database operations
#[derive(Clone)]
pub struct QcManager {
    pool: PgPool,
}

impl QcManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // Dashboard operations
    pub async fn get_dashboard_stats(&self) -> Result<QcDashboardStats, sqlx::Error> {
        let pending = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM qc_reviews WHERE status = 'pending'"
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let completed_today = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM qc_reviews 
            WHERE status = 'completed' 
            AND DATE(completed_at) = CURRENT_DATE
            "#
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let failed_today = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM qc_reviews 
            WHERE decision = 'failed' 
            AND DATE(completed_at) = CURRENT_DATE
            "#
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let week_stats = sqlx::query!(
            r#"
            SELECT 
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE decision = 'passed') as passed,
                AVG(EXTRACT(EPOCH FROM (completed_at - created_at)) / 3600) as avg_hours
            FROM qc_reviews 
            WHERE completed_at >= CURRENT_DATE - INTERVAL '7 days'
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        let pass_rate = if week_stats.total.unwrap_or(0) > 0 {
            (week_stats.passed.unwrap_or(0) as f64 / week_stats.total.unwrap_or(1) as f64) * 100.0
        } else {
            0.0
        };

        Ok(QcDashboardStats {
            pending_reviews: pending,
            completed_today,
            failed_today,
            pass_rate_week: pass_rate,
            average_turnaround_hours: week_stats.avg_hours.unwrap_or(0.0),
        })
    }

    // QC Review operations
    pub async fn create_qc_review(&self, request: CreateQcReviewRequest) -> Result<QcReview, sqlx::Error> {
        sqlx::query_as::<_, QcReview>(
            r#"
            INSERT INTO qc_reviews (
                entity_type, entity_id, review_type, status, priority,
                requested_by, notes, metadata
            )
            VALUES ($1, $2, $3, 'pending', $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(&request.entity_type)
        .bind(&request.entity_id)
        .bind(&request.review_type)
        .bind(&request.priority.unwrap_or("normal".to_string()))
        .bind(&request.requested_by)
        .bind(&request.notes)
        .bind(&request.metadata)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_qc_review(&self, review_id: Uuid) -> Result<Option<QcReview>, sqlx::Error> {
        sqlx::query_as::<_, QcReview>(
            "SELECT * FROM qc_reviews WHERE id = $1"
        )
        .bind(review_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list_qc_reviews(&self, filters: ListQcReviewsQuery) -> Result<Vec<QcReview>, sqlx::Error> {
        let mut query = String::from("SELECT * FROM qc_reviews WHERE 1=1");
        let mut bindings = vec![];
        let mut param_count = 1;

        if let Some(status) = filters.status {
            query.push_str(&format!(" AND status = ${}", param_count));
            bindings.push(status);
            param_count += 1;
        }

        if let Some(review_type) = filters.review_type {
            query.push_str(&format!(" AND review_type = ${}", param_count));
            bindings.push(review_type);
            param_count += 1;
        }

        if let Some(priority) = filters.priority {
            query.push_str(&format!(" AND priority = ${}", param_count));
            bindings.push(priority);
            param_count += 1;
        }

        if let Some(reviewer_id) = filters.reviewer_id {
            query.push_str(&format!(" AND reviewer_id = ${}", param_count));
            bindings.push(format!("{}", reviewer_id));
            param_count += 1;
        }

        query.push_str(" ORDER BY created_at DESC LIMIT 100");

        let mut query_builder = sqlx::query_as::<_, QcReview>(&query);
        for binding in &bindings {
            query_builder = query_builder.bind(binding);
        }

        query_builder.fetch_all(&self.pool).await
    }

    pub async fn complete_qc_review(&self, review_id: Uuid, request: CompleteQcReviewRequest, reviewer_id: Uuid) -> Result<QcReview, sqlx::Error> {
        sqlx::query_as::<_, QcReview>(
            r#"
            UPDATE qc_reviews
            SET status = 'completed',
                decision = $2,
                reviewer_id = $3,
                completed_at = NOW(),
                review_comments = $4,
                deviations = $5,
                corrective_actions = $6,
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(review_id)
        .bind(&request.decision)
        .bind(reviewer_id)
        .bind(&request.comments)
        .bind(&request.deviations)
        .bind(&request.corrective_actions)
        .fetch_one(&self.pool)
        .await
    }

    // Library Prep QC operations
    pub async fn create_library_prep_qc(&self, request: CreateLibraryPrepQcRequest) -> Result<LibraryPrepQc, sqlx::Error> {
        // Calculate overall status based on metrics
        let overall_status = self.calculate_qc_status(
            request.concentration_ng_ul,
            request.total_yield_ng,
            request.fragment_size_bp,
            request.quality_score
        );

        sqlx::query_as::<_, LibraryPrepQc>(
            r#"
            INSERT INTO library_prep_qc (
                library_prep_id, concentration_ng_ul, volume_ul, total_yield_ng,
                fragment_size_bp, quality_score, overall_status, performed_by,
                notes
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(&request.library_prep_id)
        .bind(&request.concentration_ng_ul)
        .bind(&request.volume_ul)
        .bind(&request.total_yield_ng)
        .bind(&request.fragment_size_bp)
        .bind(&request.quality_score)
        .bind(&overall_status)
        .bind(&request.performed_by)
        .bind(&request.notes)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_library_prep_qc(&self, library_prep_id: Uuid) -> Result<Option<LibraryPrepQc>, sqlx::Error> {
        sqlx::query_as::<_, LibraryPrepQc>(
            "SELECT * FROM library_prep_qc WHERE library_prep_id = $1 ORDER BY created_at DESC LIMIT 1"
        )
        .bind(library_prep_id)
        .fetch_optional(&self.pool)
        .await
    }

    // Control Sample operations
    pub async fn create_control_sample(&self, request: CreateControlSampleRequest, created_by: Uuid) -> Result<QcControlSample, sqlx::Error> {
        sqlx::query_as::<_, QcControlSample>(
            r#"
            INSERT INTO qc_control_samples (
                name, control_type, expected_concentration, expected_fragment_size,
                tolerance_percent, lot_number, expiry_date, storage_location,
                is_active, metadata, created_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, true, $9, $10)
            RETURNING *
            "#,
        )
        .bind(&request.name)
        .bind(&request.control_type)
        .bind(&request.expected_concentration)
        .bind(&request.expected_fragment_size)
        .bind(&request.tolerance_percent.unwrap_or(10.0))
        .bind(&request.lot_number)
        .bind(&request.expiry_date)
        .bind(&request.storage_location)
        .bind(&request.metadata)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn list_control_samples(&self, is_active: Option<bool>) -> Result<Vec<QcControlSample>, sqlx::Error> {
        let query = if let Some(active) = is_active {
            "SELECT * FROM qc_control_samples WHERE is_active = $1 ORDER BY name"
        } else {
            "SELECT * FROM qc_control_samples ORDER BY name"
        };

        if let Some(active) = is_active {
            sqlx::query_as::<_, QcControlSample>(query)
                .bind(active)
                .fetch_all(&self.pool)
                .await
        } else {
            sqlx::query_as::<_, QcControlSample>(query)
                .fetch_all(&self.pool)
                .await
        }
    }

    pub async fn record_control_result(&self, request: RecordControlResultRequest) -> Result<QcControlResult, sqlx::Error> {
        // Get control sample to check expected values
        let control = sqlx::query!(
            "SELECT expected_concentration, expected_fragment_size, tolerance_percent FROM qc_control_samples WHERE id = $1",
            request.control_sample_id
        )
        .fetch_one(&self.pool)
        .await?;

        // Calculate if results passed
        let mut passed = true;
        if let (Some(measured), Some(expected), Some(tolerance)) = 
            (request.measured_concentration, control.expected_concentration, control.tolerance_percent) {
            let diff_percent = ((measured - expected).abs() / expected) * 100.0;
            if diff_percent > tolerance {
                passed = false;
            }
        }

        if let (Some(measured), Some(expected)) = 
            (request.measured_fragment_size, control.expected_fragment_size) {
            let tolerance = control.tolerance_percent.unwrap_or(10.0);
            let diff_percent = ((measured - expected).abs() as f64 / expected as f64) * 100.0;
            if diff_percent > tolerance {
                passed = false;
            }
        }

        sqlx::query_as::<_, QcControlResult>(
            r#"
            INSERT INTO qc_control_results (
                control_sample_id, run_id, measured_concentration,
                measured_fragment_size, passed, performed_by, notes
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(&request.control_sample_id)
        .bind(&request.run_id)
        .bind(&request.measured_concentration)
        .bind(&request.measured_fragment_size)
        .bind(passed)
        .bind(&request.performed_by)
        .bind(&request.notes)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_control_results(&self, control_sample_id: Uuid) -> Result<Vec<QcControlResult>, sqlx::Error> {
        sqlx::query_as::<_, QcControlResult>(
            "SELECT * FROM qc_control_results WHERE control_sample_id = $1 ORDER BY created_at DESC LIMIT 50"
        )
        .bind(control_sample_id)
        .fetch_all(&self.pool)
        .await
    }

    // Metric operations
    pub async fn list_qc_metrics(&self, metric_type: Option<String>) -> Result<Vec<QcMetricDefinition>, sqlx::Error> {
        let query = if metric_type.is_some() {
            "SELECT * FROM qc_metric_definitions WHERE metric_type = $1 AND is_active = true ORDER BY name"
        } else {
            "SELECT * FROM qc_metric_definitions WHERE is_active = true ORDER BY metric_type, name"
        };

        if let Some(mt) = metric_type {
            sqlx::query_as::<_, QcMetricDefinition>(query)
                .bind(mt)
                .fetch_all(&self.pool)
                .await
        } else {
            sqlx::query_as::<_, QcMetricDefinition>(query)
                .fetch_all(&self.pool)
                .await
        }
    }

    pub async fn get_metric_trends(&self, metric_type: String, days: i32) -> Result<Vec<QcMetricTrend>, sqlx::Error> {
        sqlx::query_as::<_, QcMetricTrend>(
            r#"
            SELECT 
                metric_name,
                DATE(created_at) as date,
                AVG(value) as average_value,
                COUNT(*) as sample_count,
                CAST(COUNT(*) FILTER (WHERE passed = true) AS FLOAT) / COUNT(*) * 100 as pass_rate
            FROM qc_metric_history
            WHERE metric_type = $1 
            AND created_at >= CURRENT_DATE - INTERVAL '%d days'
            GROUP BY metric_name, DATE(created_at)
            ORDER BY date DESC, metric_name
            "#
        )
        .bind(metric_type)
        .bind(days)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_recent_metrics(&self) -> Result<Vec<serde_json::Value>, sqlx::Error> {
        let results = sqlx::query!(
            r#"
            SELECT 
                id, metric_type, metric_name, value, unit, 
                CASE WHEN passed THEN 'pass' ELSE 'fail' END as status,
                created_at
            FROM qc_metric_history
            ORDER BY created_at DESC
            LIMIT 10
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results.into_iter().map(|r| {
            json!({
                "id": r.id,
                "type": r.metric_type,
                "name": r.metric_name,
                "value": r.value,
                "unit": r.unit,
                "status": r.status,
                "timestamp": r.created_at
            })
        }).collect())
    }

    // Helper methods
    fn calculate_qc_status(&self, concentration: Option<f64>, yield_ng: Option<f64>, 
                          fragment_size: Option<i32>, quality_score: Option<f64>) -> String {
        let mut passed = true;

        // Basic thresholds (should be configurable)
        if let Some(conc) = concentration {
            if conc < 10.0 || conc > 100.0 {
                passed = false;
            }
        }

        if let Some(yield_val) = yield_ng {
            if yield_val < 100.0 {
                passed = false;
            }
        }

        if let Some(size) = fragment_size {
            if size < 200 || size > 600 {
                passed = false;
            }
        }

        if let Some(score) = quality_score {
            if score < 30.0 {
                passed = false;
            }
        }

        if passed { "passed".to_string() } else { "failed".to_string() }
    }
}

/// Get QC dashboard statistics
pub async fn get_qc_dashboard(
    State(state): State<Arc<AppComponents>>,
) -> Result<Json<QcDashboardStats>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = QcManager::new(pool.clone());
    
    match manager.get_dashboard_stats().await {
        Ok(stats) => Ok(Json(stats)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// List QC reviews with filters
pub async fn list_qc_reviews(
    State(state): State<Arc<AppComponents>>,
    Query(query): Query<ListQcReviewsQuery>,
) -> Result<Json<Vec<QcReview>>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = QcManager::new(pool.clone());
    
    match manager.list_qc_reviews(query).await {
        Ok(reviews) => Ok(Json(reviews)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Get a QC review by ID
pub async fn get_qc_review(
    State(state): State<Arc<AppComponents>>,
    Path(review_id): Path<Uuid>,
) -> Result<Json<QcReview>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = QcManager::new(pool.clone());
    
    match manager.get_qc_review(review_id).await {
        Ok(Some(review)) => Ok(Json(review)),
        Ok(None) => Err((StatusCode::NOT_FOUND, "QC review not found".to_string())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Create a new QC review request
pub async fn create_qc_review(
    State(state): State<Arc<AppComponents>>,
    Json(request): Json<CreateQcReviewRequest>,
) -> Result<Json<QcReview>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = QcManager::new(pool.clone());
    
    match manager.create_qc_review(request).await {
        Ok(review) => Ok(Json(review)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Complete a QC review with decision
pub async fn complete_qc_review(
    State(state): State<Arc<AppComponents>>,
    Path(review_id): Path<Uuid>,
    Json(request): Json<CompleteQcReviewRequest>,
) -> Result<Json<QcReview>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = QcManager::new(pool.clone());
    
    // TODO: Get actual user ID from authentication
    let reviewer_id = Uuid::new_v4();
    
    match manager.complete_qc_review(review_id, request, reviewer_id).await {
        Ok(review) => Ok(Json(review)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Create library prep QC results
pub async fn create_library_prep_qc(
    State(state): State<Arc<AppComponents>>,
    Json(request): Json<CreateLibraryPrepQcRequest>,
) -> Result<Json<LibraryPrepQc>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = QcManager::new(pool.clone());
    
    match manager.create_library_prep_qc(request).await {
        Ok(qc) => Ok(Json(qc)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Get library prep QC results
pub async fn get_library_prep_qc(
    State(state): State<Arc<AppComponents>>,
    Path(library_prep_id): Path<Uuid>,
) -> Result<Json<LibraryPrepQc>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = QcManager::new(pool.clone());
    
    match manager.get_library_prep_qc(library_prep_id).await {
        Ok(Some(qc)) => Ok(Json(qc)),
        Ok(None) => Err((StatusCode::NOT_FOUND, "Library prep QC not found".to_string())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Get QC metric trends
pub async fn get_qc_metric_trends(
    State(state): State<Arc<AppComponents>>,
    Query(params): Query<serde_json::Value>,
) -> Result<Json<Vec<QcMetricTrend>>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = QcManager::new(pool.clone());
    
    let metric_type = params.get("metric_type")
        .and_then(|v| v.as_str())
        .unwrap_or("library_prep")
        .to_string();
    
    let days = params.get("days")
        .and_then(|v| v.as_i64())
        .unwrap_or(30) as i32;
    
    match manager.get_metric_trends(metric_type, days).await {
        Ok(trends) => Ok(Json(trends)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// List control samples
pub async fn list_control_samples(
    State(state): State<Arc<AppComponents>>,
    Query(params): Query<serde_json::Value>,
) -> Result<Json<Vec<QcControlSample>>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = QcManager::new(pool.clone());
    
    let is_active = params.get("is_active")
        .and_then(|v| v.as_bool());
    
    match manager.list_control_samples(is_active).await {
        Ok(samples) => Ok(Json(samples)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Create a control sample
pub async fn create_control_sample(
    State(state): State<Arc<AppComponents>>,
    Json(request): Json<CreateControlSampleRequest>,
) -> Result<Json<QcControlSample>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = QcManager::new(pool.clone());
    
    // TODO: Get actual user ID from authentication
    let created_by = Uuid::new_v4();
    
    match manager.create_control_sample(request, created_by).await {
        Ok(sample) => Ok(Json(sample)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Record control sample results
pub async fn record_control_result(
    State(state): State<Arc<AppComponents>>,
    Json(request): Json<RecordControlResultRequest>,
) -> Result<Json<QcControlResult>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = QcManager::new(pool.clone());
    
    match manager.record_control_result(request).await {
        Ok(result) => Ok(Json(result)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Get control sample results history
pub async fn get_control_results(
    State(state): State<Arc<AppComponents>>,
    Path(control_sample_id): Path<Uuid>,
    Query(params): Query<serde_json::Value>,
) -> Result<Json<Vec<QcControlResult>>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = QcManager::new(pool.clone());
    
    match manager.get_control_results(control_sample_id).await {
        Ok(results) => Ok(Json(results)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// List QC metric definitions
pub async fn list_qc_metrics(
    State(state): State<Arc<AppComponents>>,
    Query(params): Query<serde_json::Value>,
) -> Result<Json<Vec<QcMetricDefinition>>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = QcManager::new(pool.clone());
    
    let metric_type = params.get("metric_type")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    match manager.list_qc_metrics(metric_type).await {
        Ok(metrics) => Ok(Json(metrics)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Create or update a QC metric definition
pub async fn upsert_qc_metric(
    State(state): State<Arc<AppComponents>>,
    Json(metric): Json<QcMetricDefinition>,
) -> Result<Json<QcMetricDefinition>, (StatusCode, String)> {
    // TODO: Implement metric definition upsert
    Err((StatusCode::NOT_IMPLEMENTED, "Not implemented yet".to_string()))
}

/// Get recent QC metrics for display
pub async fn get_recent_qc_metrics(
    State(state): State<Arc<AppComponents>>,
    Query(params): Query<serde_json::Value>,
) -> Result<Json<Vec<serde_json::Value>>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = QcManager::new(pool.clone());
    
    match manager.get_recent_metrics().await {
        Ok(metrics) => Ok(Json(metrics)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
} 