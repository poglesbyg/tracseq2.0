use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde_json::json;
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::{SampleResult, SampleServiceError},
    models::*,
    AppState,
};

/// Create a new sample
pub async fn create_sample(
    State(state): State<AppState>,
    Json(request): Json<CreateSampleRequest>,
) -> SampleResult<Json<serde_json::Value>> {
    // Validate request
    request.validate().map_err(|e| SampleServiceError::Validation(e.to_string()))?;

    let sample = state.sample_service.create_sample(request).await?;

    Ok(Json(json!({
        "success": true,
        "data": sample,
        "message": "Sample created successfully"
    })))
}

/// Get sample by ID
pub async fn get_sample(
    State(state): State<AppState>,
    Path(sample_id): Path<Uuid>,
) -> SampleResult<Json<serde_json::Value>> {
    let sample = state.sample_service.get_sample(sample_id).await?;

    Ok(Json(json!({
        "success": true,
        "data": sample
    })))
}

/// Get sample by barcode
pub async fn get_sample_by_barcode(
    State(state): State<AppState>,
    Path(barcode): Path<String>,
) -> SampleResult<Json<serde_json::Value>> {
    let sample = state.sample_service.get_sample_by_barcode(&barcode).await?;

    Ok(Json(json!({
        "success": true,
        "data": sample
    })))
}

/// List samples with filtering and pagination
pub async fn list_samples(
    State(state): State<AppState>,
    Query(query): Query<ListSamplesQuery>,
) -> SampleResult<Json<serde_json::Value>> {
    let response = state.sample_service.list_samples(query).await?;

    Ok(Json(json!({
        "success": true,
        "data": response
    })))
}

/// Update sample
pub async fn update_sample(
    State(state): State<AppState>,
    Path(sample_id): Path<Uuid>,
    Json(request): Json<UpdateSampleRequest>,
) -> SampleResult<Json<serde_json::Value>> {
    // Validate request
    request.validate().map_err(|e| SampleServiceError::Validation(e.to_string()))?;

    let sample = state.sample_service.update_sample(
        sample_id,
        request,
        Some("user") // TODO: Extract from authentication context
    ).await?;

    Ok(Json(json!({
        "success": true,
        "data": sample,
        "message": "Sample updated successfully"
    })))
}

/// Update sample status
pub async fn update_status(
    State(state): State<AppState>,
    Path(sample_id): Path<Uuid>,
    Json(request): Json<UpdateSampleStatusRequest>,
) -> SampleResult<Json<serde_json::Value>> {
    // Validate request
    request.validate().map_err(|e| SampleServiceError::Validation(e.to_string()))?;

    let sample = state.sample_service.update_sample_status(
        sample_id,
        request.status,
        Some("user") // TODO: Extract from authentication context
    ).await?;

    Ok(Json(json!({
        "success": true,
        "data": sample,
        "message": "Sample status updated successfully"
    })))
}

/// Validate sample
pub async fn validate_sample(
    State(state): State<AppState>,
    Path(sample_id): Path<Uuid>,
) -> SampleResult<Json<serde_json::Value>> {
    let validation_result = state.sample_service.validate_sample(sample_id).await?;

    let status_code = if validation_result.is_valid {
        StatusCode::OK
    } else {
        StatusCode::BAD_REQUEST
    };

    Ok(Json(json!({
        "success": validation_result.is_valid,
        "data": validation_result,
        "message": if validation_result.is_valid {
            "Sample validation successful"
        } else {
            "Sample validation failed"
        }
    })))
}

/// Delete sample (soft delete)
pub async fn delete_sample(
    State(state): State<AppState>,
    Path(sample_id): Path<Uuid>,
) -> SampleResult<Json<serde_json::Value>> {
    state.sample_service.delete_sample(
        sample_id,
        Some("user") // TODO: Extract from authentication context
    ).await?;

    Ok(Json(json!({
        "success": true,
        "message": "Sample deleted successfully"
    })))
}

/// Create batch samples
pub async fn create_batch_samples(
    State(state): State<AppState>,
    Json(request): Json<BatchCreateSampleRequest>,
) -> SampleResult<Json<serde_json::Value>> {
    // Validate each sample request
    for sample_request in &request.samples {
        sample_request.validate().map_err(|e| SampleServiceError::Validation(e.to_string()))?;
    }

    let response = state.sample_service.create_batch_samples(request.samples).await?;

    Ok(Json(json!({
        "success": true,
        "data": response,
        "message": format!(
            "Batch processing completed: {} created, {} failed",
            response.total_created,
            response.total_failed
        )
    })))
}

/// Validate batch samples
pub async fn validate_batch(
    State(state): State<AppState>,
    Json(request): Json<BatchValidateRequest>,
) -> SampleResult<Json<serde_json::Value>> {
    let mut validation_results = Vec::new();
    let mut overall_valid = true;

    for sample_id in request.sample_ids {
        match state.sample_service.validate_sample(sample_id).await {
            Ok(result) => {
                if !result.is_valid {
                    overall_valid = false;
                }
                validation_results.push(result);
            }
            Err(e) => {
                overall_valid = false;
                validation_results.push(SampleValidationResult {
                    sample_id,
                    is_valid: false,
                    errors: vec![e.to_string()],
                    warnings: vec![],
                });
            }
        }
    }

    Ok(Json(json!({
        "success": overall_valid,
        "data": {
            "validation_results": validation_results,
            "overall_valid": overall_valid,
            "total_samples": validation_results.len(),
            "valid_samples": validation_results.iter().filter(|r| r.is_valid).count(),
            "invalid_samples": validation_results.iter().filter(|r| !r.is_valid).count()
        },
        "message": if overall_valid {
            "All samples are valid"
        } else {
            "Some samples have validation errors"
        }
    })))
}

/// Get sample history
pub async fn get_sample_history(
    State(state): State<AppState>,
    Path(sample_id): Path<Uuid>,
    Query(query): Query<SampleHistoryQuery>,
) -> SampleResult<Json<serde_json::Value>> {
    // First verify sample exists
    let _ = state.sample_service.get_sample(sample_id).await?;

    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(50).min(100);
    let offset = (page - 1) * page_size;

    // Get status history
    let status_history = sqlx::query_as::<_, SampleStatusHistory>(
        r#"
        SELECT * FROM sample_status_history 
        WHERE sample_id = $1 
        ORDER BY changed_at DESC 
        LIMIT $2 OFFSET $3
        "#
    )
    .bind(sample_id)
    .bind(page_size)
    .bind(offset)
    .fetch_all(&state.db_pool.pool)
    .await?;

    // Get audit log
    let audit_log = sqlx::query_as::<_, SampleAuditLog>(
        r#"
        SELECT * FROM sample_audit_log 
        WHERE sample_id = $1 
        ORDER BY performed_at DESC 
        LIMIT $2 OFFSET $3
        "#
    )
    .bind(sample_id)
    .bind(page_size)
    .bind(offset)
    .fetch_all(&state.db_pool.pool)
    .await?;

    Ok(Json(json!({
        "success": true,
        "data": {
            "sample_id": sample_id,
            "status_history": status_history,
            "audit_log": audit_log,
            "pagination": {
                "page": page,
                "page_size": page_size,
                "has_more": status_history.len() == page_size as usize
            }
        }
    })))
}

/// Export samples to CSV
pub async fn export_samples(
    State(state): State<AppState>,
    Query(query): Query<ExportSamplesQuery>,
) -> SampleResult<impl axum::response::IntoResponse> {
    // Get samples based on filters
    let list_query = ListSamplesQuery {
        status: query.status,
        sample_type: query.sample_type,
        template_id: query.template_id,
        search: query.search,
        created_after: query.created_after,
        created_before: query.created_before,
        page: None, // Get all samples for export
        page_size: Some(10000), // Reasonable limit for export
        sort_by: Some("created_at".to_string()),
    };

    let response = state.sample_service.list_samples(list_query).await?;

    // Generate CSV content
    let mut csv_content = String::new();
    csv_content.push_str("ID,Name,Barcode,Type,Status,Created At,Updated At\n");

    for sample in response.samples {
        csv_content.push_str(&format!(
            "{},{},{},{},{},{},{}\n",
            sample.id,
            sample.name,
            sample.barcode,
            sample.sample_type,
            sample.status,
            sample.created_at.format("%Y-%m-%d %H:%M:%S"),
            sample.updated_at.format("%Y-%m-%d %H:%M:%S")
        ));
    }

    let headers = [
        ("Content-Type", "text/csv"),
        ("Content-Disposition", "attachment; filename=\"samples.csv\""),
    ];

    Ok((headers, csv_content))
}

/// Search samples with advanced filters
pub async fn search_samples(
    State(state): State<AppState>,
    Json(request): Json<SampleSearchRequest>,
) -> SampleResult<Json<serde_json::Value>> {
    // Build search query
    let list_query = ListSamplesQuery {
        status: request.filters.status,
        sample_type: request.filters.sample_type,
        template_id: request.filters.template_id,
        search: request.query,
        created_after: request.filters.created_after,
        created_before: request.filters.created_before,
        page: request.page,
        page_size: request.page_size,
        sort_by: request.sort_by,
    };

    let response = state.sample_service.list_samples(list_query).await?;

    Ok(Json(json!({
        "success": true,
        "data": response,
        "search_query": request.query,
        "filters_applied": request.filters
    })))
}

/// Get sample statistics
pub async fn get_sample_statistics(
    State(state): State<AppState>,
    Query(query): Query<StatisticsQuery>,
) -> SampleResult<Json<serde_json::Value>> {
    let period_days = query.period_days.unwrap_or(30);

    // Get overall statistics
    let total_samples: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM samples")
        .fetch_one(&state.db_pool.pool)
        .await?;

    // Get status distribution
    let status_stats = sqlx::query_as::<_, (String, i64)>(
        "SELECT status::text, COUNT(*) FROM samples GROUP BY status"
    )
    .fetch_all(&state.db_pool.pool)
    .await?;

    // Get sample type distribution
    let type_stats = sqlx::query_as::<_, (String, i64)>(
        "SELECT sample_type, COUNT(*) FROM samples GROUP BY sample_type ORDER BY COUNT(*) DESC"
    )
    .fetch_all(&state.db_pool.pool)
    .await?;

    // Get recent activity
    let recent_created: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM samples WHERE created_at > NOW() - INTERVAL $1 DAY"
    )
    .bind(period_days)
    .fetch_one(&state.db_pool.pool)
    .await?;

    // Get daily creation trends
    let daily_trends = sqlx::query_as::<_, (chrono::NaiveDate, i64)>(
        r#"
        SELECT DATE(created_at) as date, COUNT(*) as count
        FROM samples 
        WHERE created_at > NOW() - INTERVAL $1 DAY
        GROUP BY DATE(created_at)
        ORDER BY date DESC
        "#
    )
    .bind(period_days)
    .fetch_all(&state.db_pool.pool)
    .await?;

    Ok(Json(json!({
        "success": true,
        "data": {
            "overview": {
                "total_samples": total_samples,
                "recent_created": recent_created,
                "period_days": period_days
            },
            "status_distribution": status_stats.into_iter().collect::<std::collections::HashMap<String, i64>>(),
            "type_distribution": type_stats.into_iter().collect::<std::collections::HashMap<String, i64>>(),
            "daily_trends": daily_trends
        }
    })))
} 
