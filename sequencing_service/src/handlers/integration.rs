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

/// Sync jobs with sample service
pub async fn sync_with_sample_service(
    State(state): State<AppState>,
    Json(request): Json<SyncSampleServiceRequest>,
) -> Result<Json<serde_json::Value>> {
    let mut sync_results = Vec::new();
    let mut success_count = 0;
    let mut error_count = 0;

    let sample_count = request.sample_ids.len();
    for sample_id in &request.sample_ids {
        match sync_single_sample(&state, sample_id).await {
            Ok(result) => {
                sync_results.push(json!({
                    "sample_id": sample_id,
                    "status": "success",
                    "data": result
                }));
                success_count += 1;
            }
            Err(e) => {
                sync_results.push(json!({
                    "sample_id": sample_id,
                    "status": "error",
                    "error": e.to_string()
                }));
                error_count += 1;
            }
        }
    }

    // Log sync operation
    sqlx::query(
        r#"
        INSERT INTO integration_logs (
            id, integration_type, operation, details, 
            success_count, error_count, created_at, initiated_by
        ) VALUES ($1, $2, $3, $4, $5, $6, NOW(), $7)
        "#
    )
    .bind(Uuid::new_v4())
    .bind("sample_service")
    .bind("sync_samples")
    .bind(json!({
        "sync_results": sync_results,
        "requested_samples": sample_count
    }))
    .bind(success_count)
    .bind(error_count)
    .bind(request.initiated_by.as_deref())
    .execute(&state.db_pool.pool)
    .await?;

    Ok(Json(json!({
        "success": true,
        "data": {
            "sync_results": sync_results,
            "summary": {
                "total_requested": request.sample_ids.len(),
                "successful": success_count,
                "failed": error_count,
                "success_rate": if request.sample_ids.len() > 0 {
                    (success_count as f64 / request.sample_ids.len() as f64 * 100.0).round()
                } else { 0.0 }
            }
        },
        "message": format!("Sync completed: {}/{} samples successful", success_count, request.sample_ids.len())
    })))
}

/// Push job status to notification service
pub async fn push_to_notification_service(
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
    Json(request): Json<NotificationPushRequest>,
) -> Result<Json<serde_json::Value>> {
    // Get job details
    let job = sqlx::query_as::<_, SequencingJob>(
        "SELECT * FROM sequencing_jobs WHERE id = $1"
    )
    .bind(job_id)
    .fetch_optional(&state.db_pool.pool)
    .await?
    .ok_or(SequencingError::JobNotFound(job_id.to_string()))?;

    // Prepare notification payload
    let notification_payload = json!({
        "job_id": job.id,
        "job_name": job.job_name.as_deref().unwrap_or("unknown"),
        "status": job.status,
        "platform": job.platform,
        "priority": job.priority,
        "created_at": job.created_at,
        "updated_at": job.updated_at,
        "notification_type": request.notification_type,
        "recipients": request.recipients,
        "template": request.template.unwrap_or("sequencing_job_status".to_string()),
        "custom_data": request.custom_data
    });

    // Send to notification service
    let notification_result = send_to_notification_service(&state, &notification_payload).await;

    match notification_result {
        Ok(response) => {
            // Log successful notification
            sqlx::query(
                r#"
                INSERT INTO integration_logs (
                    id, integration_type, operation, job_id, details,
                    success_count, error_count, created_at, initiated_by
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), $8)
                "#
            )
            .bind(Uuid::new_v4())
            .bind("notification_service")
            .bind("send_notification")
            .bind(job_id)
            .bind(json!({
                "notification_payload": notification_payload,
                "response": response
            }))
            .bind(1)
            .bind(0)
            .bind(request.initiated_by.as_deref())
            .execute(&state.db_pool.pool)
            .await?;

            Ok(Json(json!({
                "success": true,
                "data": {
                    "notification_sent": true,
                    "job_id": job_id,
                    "notification_type": request.notification_type,
                    "response": response
                },
                "message": "Notification sent successfully"
            })))
        }
        Err(e) => {
            // Log failed notification
            sqlx::query(
                r#"
                INSERT INTO integration_logs (
                    id, integration_type, operation, job_id, details,
                    success_count, error_count, created_at, initiated_by
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), $8)
                "#
            )
            .bind(Uuid::new_v4())
            .bind("notification_service")
            .bind("send_notification")
            .bind(job_id)
            .bind(json!({
                "notification_payload": notification_payload,
                "error": e.to_string()
            }))
            .bind(0)
            .bind(1)
            .bind(request.initiated_by.as_deref())
            .execute(&state.db_pool.pool)
            .await?;

            Err(SequencingError::IntegrationError {
                service: "notification_service".to_string(),
                message: format!("Failed to send notification: {}", e),
            })
        }
    }
}

/// Register webhook for external integrations
pub async fn register_webhook(
    State(state): State<AppState>,
    Json(request): Json<RegisterWebhookRequest>,
) -> Result<Json<serde_json::Value>> {
    let webhook_id = Uuid::new_v4();
    
    let webhook = sqlx::query_as::<_, IntegrationWebhook>(
        r#"
        INSERT INTO integration_webhooks (
            id, name, url, secret_token, event_types, 
            is_active, retry_count, timeout_seconds,
            created_at, created_by
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), $9)
        RETURNING *
        "#
    )
    .bind(webhook_id)
    .bind(&request.name)
    .bind(&request.url)
    .bind(request.secret_token.as_deref())
    .bind(&json!(request.event_types))
    .bind(request.is_active.unwrap_or(true))
    .bind(request.retry_count.unwrap_or(3))
    .bind(request.timeout_seconds.unwrap_or(30))
    .bind(request.created_by.as_deref())
    .fetch_one(&state.db_pool.pool)
    .await?;

    Ok(Json(json!({
        "success": true,
        "data": webhook,
        "message": "Webhook registered successfully"
    })))
}

/// List registered webhooks
pub async fn list_webhooks(
    State(state): State<AppState>,
    Query(query): Query<WebhookListQuery>,
) -> Result<Json<serde_json::Value>> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).min(100);
    let offset = (page - 1) * page_size;

    let mut where_conditions = Vec::new();
    
    if let Some(is_active) = query.is_active {
        where_conditions.push(format!("is_active = {}", is_active));
    }

    let where_clause = if where_conditions.is_empty() {
        "".to_string()
    } else {
        format!("WHERE {}", where_conditions.join(" AND "))
    };

    // Get total count
    let total_count: i64 = sqlx::query_scalar(&format!(
        "SELECT COUNT(*) FROM integration_webhooks {}", where_clause
    ))
    .fetch_one(&state.db_pool.pool)
    .await?;

    // Get webhooks
    let webhooks = sqlx::query_as::<_, IntegrationWebhook>(&format!(
        "SELECT * FROM integration_webhooks {} ORDER BY created_at DESC LIMIT {} OFFSET {}",
        where_clause, page_size, offset
    ))
    .fetch_all(&state.db_pool.pool)
    .await?;

    let total_pages = (total_count + page_size - 1) / page_size;

    Ok(Json(json!({
        "success": true,
        "data": {
            "webhooks": webhooks,
            "pagination": {
                "page": page,
                "page_size": page_size,
                "total_count": total_count,
                "total_pages": total_pages
            }
        }
    })))
}

/// Send webhook notification
pub async fn send_webhook_notification(
    State(state): State<AppState>,
    Path(webhook_id): Path<Uuid>,
    Json(request): Json<WebhookNotificationRequest>,
) -> Result<Json<serde_json::Value>> {
    // Get webhook details
    let webhook = sqlx::query_as::<_, IntegrationWebhook>(
        "SELECT * FROM integration_webhooks WHERE id = $1 AND is_active = true"
    )
    .bind(webhook_id)
    .fetch_optional(&state.db_pool.pool)
    .await?
    .ok_or(SequencingError::WebhookNotFound { webhook_id })?;

    // Check if event type is supported
    let event_types: Vec<String> = serde_json::from_value(webhook.event_types.clone())
        .unwrap_or_default();
    
    if !event_types.contains(&request.event_type) {
        return Err(SequencingError::Validation {
            message: format!("Event type '{}' not supported by this webhook", request.event_type),
        });
    }

    // Prepare webhook payload
    let webhook_payload = json!({
        "event_type": request.event_type,
        "timestamp": Utc::now(),
        "data": request.payload,
        "webhook_id": webhook_id,
        "delivery_id": Uuid::new_v4()
    });

    // Send webhook
    let delivery_result = deliver_webhook(&webhook, &webhook_payload).await;

    // Record delivery attempt
    let delivery_id = Uuid::new_v4();
    let delivery_record = sqlx::query_as::<_, WebhookDelivery>(
        r#"
        INSERT INTO webhook_deliveries (
            id, webhook_id, event_type, payload, response_status,
            response_body, delivered_at, retry_count
        ) VALUES ($1, $2, $3, $4, $5, $6, NOW(), $7)
        RETURNING *
        "#
    )
    .bind(delivery_id)
    .bind(webhook_id)
    .bind(&request.event_type)
    .bind(&webhook_payload)
    .bind(delivery_result.status_code)
    .bind(&delivery_result.response_body)
    .bind(0)
    .fetch_one(&state.db_pool.pool)
    .await?;

    if delivery_result.success {
        Ok(Json(json!({
            "success": true,
            "data": {
                "delivery": delivery_record,
                "webhook": webhook,
                "delivered": true
            },
            "message": "Webhook delivered successfully"
        })))
    } else {
        // Schedule retry if configured
        if webhook.retry_count > 0 {
            schedule_webhook_retry(&state, delivery_id, 1).await?;
        }

        Ok(Json(json!({
            "success": false,
            "data": {
                "delivery": delivery_record,
                "webhook": webhook,
                "delivered": false,
                "will_retry": webhook.retry_count > 0
            },
            "message": "Webhook delivery failed"
        })))
    }
}

/// Get integration logs
pub async fn get_integration_logs(
    State(state): State<AppState>,
    Query(query): Query<IntegrationLogsQuery>,
) -> Result<Json<serde_json::Value>> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(50).min(200);
    let offset = (page - 1) * page_size;

    let mut where_conditions = Vec::new();
    
    if let Some(integration_type) = &query.integration_type {
        where_conditions.push(format!("integration_type = '{}'", integration_type));
    }

    if let Some(operation) = &query.operation {
        where_conditions.push(format!("operation = '{}'", operation));
    }

    if let Some(job_id) = query.job_id {
        where_conditions.push(format!("job_id = '{}'", job_id));
    }

    let where_clause = if where_conditions.is_empty() {
        "".to_string()
    } else {
        format!("WHERE {}", where_conditions.join(" AND "))
    };

    // Get total count
    let total_count: i64 = sqlx::query_scalar(&format!(
        "SELECT COUNT(*) FROM integration_logs {}", where_clause
    ))
    .fetch_one(&state.db_pool.pool)
    .await?;

    // Get logs
    let logs = sqlx::query_as::<_, IntegrationLog>(&format!(
        "SELECT * FROM integration_logs {} ORDER BY created_at DESC LIMIT {} OFFSET {}",
        where_clause, page_size, offset
    ))
    .fetch_all(&state.db_pool.pool)
    .await?;

    let total_pages = (total_count + page_size - 1) / page_size;

    Ok(Json(json!({
        "success": true,
        "data": {
            "logs": logs,
            "pagination": {
                "page": page,
                "page_size": page_size,
                "total_count": total_count,
                "total_pages": total_pages
            }
        }
    })))
}

/// Sync data with external LIMS
pub async fn sync_with_lims(
    State(state): State<AppState>,
    Json(request): Json<LIMSSyncRequest>,
) -> Result<Json<serde_json::Value>> {
    let sync_id = Uuid::new_v4();
    
    // Start sync operation
    let sync_record = sqlx::query_as::<_, LIMSSync>(
        r#"
        INSERT INTO lims_syncs (
            id, lims_system, sync_type, sync_direction, 
            job_ids, status, started_at, initiated_by
        ) VALUES ($1, $2, $3, $4, $5, $6, NOW(), $7)
        RETURNING *
        "#
    )
    .bind(sync_id)
    .bind(&request.lims_system)
    .bind(&request.sync_type)
    .bind(&request.sync_direction)
    .bind(&json!(request.job_ids))
    .bind("running")
    .bind(request.initiated_by.as_deref())
    .fetch_one(&state.db_pool.pool)
    .await?;

    // Perform sync operation based on type
    let sync_result = match request.sync_type.as_str() {
        "job_status" => sync_job_status_to_lims(&state, &request).await,
        "sample_data" => sync_sample_data_from_lims(&state, &request).await,
        "results" => sync_results_to_lims(&state, &request).await,
        _ => Err(SequencingError::Validation {
            message: format!("Unsupported sync type: {}", request.sync_type),
        }),
    };

    // Update sync record with results
    let (status, error_message) = match &sync_result {
        Ok(_) => ("completed", None),
        Err(e) => ("failed", Some(e.to_string())),
    };

    sqlx::query(
        r#"
        UPDATE lims_syncs 
        SET status = $2, completed_at = NOW(), error_message = $3
        WHERE id = $1
        "#
    )
    .bind(sync_id)
    .bind(status)
    .bind(error_message)
    .execute(&state.db_pool.pool)
    .await?;

    match sync_result {
        Ok(result) => Ok(Json(json!({
            "success": true,
            "data": {
                "sync_record": sync_record,
                "sync_result": result,
                "status": "completed"
            },
            "message": "LIMS sync completed successfully"
        }))),
        Err(e) => Ok(Json(json!({
            "success": false,
            "data": {
                "sync_record": sync_record,
                "status": "failed",
                "error": e.to_string()
            },
            "message": "LIMS sync failed"
        }))),
    }
}

/// Test integration connectivity
pub async fn test_integration_connectivity(
    State(state): State<AppState>,
    Json(request): Json<TestConnectivityRequest>,
) -> Result<Json<serde_json::Value>> {
    let mut test_results = Vec::new();

    for service in &request.services {
        let test_result = match service.as_str() {
            "sample_service" => test_sample_service_connectivity(&state).await,
            "notification_service" => test_notification_service_connectivity(&state).await,
            "storage_service" => test_storage_service_connectivity(&state).await,
            "auth_service" => test_auth_service_connectivity(&state).await,
            _ => TestResult {
                service: service.clone(),
                status: "unsupported".to_string(),
                response_time_ms: 0,
                details: json!({"error": "Service not supported for testing"}),
            },
        };

        test_results.push(test_result);
    }

    // Log connectivity test
    sqlx::query(
        r#"
        INSERT INTO integration_logs (
            id, integration_type, operation, details,
            success_count, error_count, created_at, initiated_by
        ) VALUES ($1, $2, $3, $4, $5, $6, NOW(), $7)
        "#
    )
    .bind(Uuid::new_v4())
    .bind("connectivity_test")
    .bind("test_services")
    .bind(json!({
        "test_results": test_results,
        "services_tested": request.services
    }))
    .bind(test_results.iter().filter(|r| r.status == "healthy").count() as i32)
    .bind(test_results.iter().filter(|r| r.status != "healthy").count() as i32)
    .bind(request.initiated_by.as_deref())
    .execute(&state.db_pool.pool)
    .await?;

    let overall_status = if test_results.iter().all(|r| r.status == "healthy") {
        "all_healthy"
    } else if test_results.iter().any(|r| r.status == "healthy") {
        "partial_healthy"
    } else {
        "all_unhealthy"
    };

    Ok(Json(json!({
        "success": true,
        "data": {
            "test_results": test_results,
            "overall_status": overall_status,
            "tested_at": Utc::now()
        },
        "message": format!("Connectivity test completed: {}", overall_status)
    })))
}

/// Helper functions
async fn sync_single_sample(state: &AppState, sample_id: &Uuid) -> Result<serde_json::Value> {
    // This would call the actual sample service API
    // For now, we'll simulate the call
    
    if let Ok(response) = state.sample_client.get_sample(*sample_id).await {
        Ok(json!({
            "sample_id": sample_id,
            "sync_status": "completed",
            "data": response
        }))
    } else {
        Err(SequencingError::IntegrationError {
            service: "sample_service".to_string(),
            message: "Failed to fetch sample data".to_string(),
        })
    }
}

async fn send_to_notification_service(
    state: &AppState,
    payload: &serde_json::Value,
) -> Result<serde_json::Value> {
    // This would call the actual notification service API
    state.notification_client.send_notification(payload).await
        .map_err(|e| SequencingError::IntegrationError {
            service: "notification_service".to_string(),
            message: e.to_string(),
        })
}

async fn deliver_webhook(webhook: &IntegrationWebhook, payload: &serde_json::Value) -> WebhookDeliveryResult {
    // Simulate webhook delivery
    // In a real implementation, this would make an HTTP request to the webhook URL
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(webhook.timeout_seconds as u64))
        .build()
        .unwrap();

    match client
        .post(&webhook.url)
        .header("Content-Type", "application/json")
        .header("User-Agent", "TracSeq-Sequencing-Service/1.0")
        .json(payload)
        .send()
        .await
    {
        Ok(response) => {
            let status_code = response.status().as_u16();
            let response_body = response.text().await.unwrap_or_default();
            
            WebhookDeliveryResult {
                success: status_code >= 200 && status_code < 300,
                status_code: Some(status_code as i32),
                response_body: Some(response_body),
            }
        }
        Err(e) => WebhookDeliveryResult {
            success: false,
            status_code: None,
            response_body: Some(e.to_string()),
        },
    }
}

async fn schedule_webhook_retry(state: &AppState, delivery_id: Uuid, retry_attempt: i32) -> Result<()> {
    // In a real implementation, this would schedule a retry using a job queue
    // For now, we'll just log the retry attempt
    
    sqlx::query(
        "UPDATE webhook_deliveries SET retry_count = $2 WHERE id = $1"
    )
    .bind(delivery_id)
    .bind(retry_attempt)
    .execute(&state.db_pool.pool)
    .await?;

    Ok(())
}

async fn sync_job_status_to_lims(state: &AppState, request: &LIMSSyncRequest) -> Result<serde_json::Value> {
    // Simulate LIMS sync
    Ok(json!({
        "synced_jobs": request.job_ids.len(),
        "sync_type": "job_status",
        "lims_system": request.lims_system
    }))
}

async fn sync_sample_data_from_lims(state: &AppState, request: &LIMSSyncRequest) -> Result<serde_json::Value> {
    // Simulate LIMS sync
    Ok(json!({
        "synced_samples": request.job_ids.len() * 5, // Assume 5 samples per job
        "sync_type": "sample_data",
        "lims_system": request.lims_system
    }))
}

async fn sync_results_to_lims(state: &AppState, request: &LIMSSyncRequest) -> Result<serde_json::Value> {
    // Simulate LIMS sync
    Ok(json!({
        "synced_results": request.job_ids.len(),
        "sync_type": "results",
        "lims_system": request.lims_system
    }))
}

async fn test_sample_service_connectivity(state: &AppState) -> TestResult {
    let start_time = std::time::Instant::now();
    
    match state.sample_client.health_check().await {
        Ok(_) => TestResult {
            service: "sample_service".to_string(),
            status: "healthy".to_string(),
            response_time_ms: start_time.elapsed().as_millis() as u64,
            details: json!({"message": "Service is responsive"}),
        },
        Err(e) => TestResult {
            service: "sample_service".to_string(),
            status: "unhealthy".to_string(),
            response_time_ms: start_time.elapsed().as_millis() as u64,
            details: json!({"error": e.to_string()}),
        },
    }
}

async fn test_notification_service_connectivity(state: &AppState) -> TestResult {
    let start_time = std::time::Instant::now();
    
    match state.notification_client.health_check().await {
        Ok(_) => TestResult {
            service: "notification_service".to_string(),
            status: "healthy".to_string(),
            response_time_ms: start_time.elapsed().as_millis() as u64,
            details: json!({"message": "Service is responsive"}),
        },
        Err(e) => TestResult {
            service: "notification_service".to_string(),
            status: "unhealthy".to_string(),
            response_time_ms: start_time.elapsed().as_millis() as u64,
            details: json!({"error": e.to_string()}),
        },
    }
}

async fn test_storage_service_connectivity(state: &AppState) -> TestResult {
    let start_time = std::time::Instant::now();
    
    match state.storage_client.health_check().await {
        Ok(_) => TestResult {
            service: "storage_service".to_string(),
            status: "healthy".to_string(),
            response_time_ms: start_time.elapsed().as_millis() as u64,
            details: json!({"message": "Service is responsive"}),
        },
        Err(e) => TestResult {
            service: "storage_service".to_string(),
            status: "unhealthy".to_string(),
            response_time_ms: start_time.elapsed().as_millis() as u64,
            details: json!({"error": e.to_string()}),
        },
    }
}

async fn test_auth_service_connectivity(state: &AppState) -> TestResult {
    let start_time = std::time::Instant::now();
    
    match state.auth_client.health_check().await {
        Ok(_) => TestResult {
            service: "auth_service".to_string(),
            status: "healthy".to_string(),
            response_time_ms: start_time.elapsed().as_millis() as u64,
            details: json!({"message": "Service is responsive"}),
        },
        Err(e) => TestResult {
            service: "auth_service".to_string(),
            status: "unhealthy".to_string(),
            response_time_ms: start_time.elapsed().as_millis() as u64,
            details: json!({"error": e.to_string()}),
        },
    }
}

/// Request and response structures
#[derive(serde::Deserialize)]
pub struct SyncSampleServiceRequest {
    pub sample_ids: Vec<Uuid>,
    pub initiated_by: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct NotificationPushRequest {
    pub notification_type: String,
    pub recipients: Vec<String>,
    pub template: Option<String>,
    pub custom_data: serde_json::Value,
    pub initiated_by: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct RegisterWebhookRequest {
    pub name: String,
    pub url: String,
    pub secret_token: Option<String>,
    pub event_types: Vec<String>,
    pub is_active: Option<bool>,
    pub retry_count: Option<i32>,
    pub timeout_seconds: Option<i32>,
    pub created_by: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct WebhookNotificationRequest {
    pub event_type: String,
    pub payload: serde_json::Value,
}

#[derive(serde::Deserialize)]
pub struct LIMSSyncRequest {
    pub lims_system: String,
    pub sync_type: String, // "job_status", "sample_data", "results"
    pub sync_direction: String, // "to_lims", "from_lims", "bidirectional"
    pub job_ids: Vec<Uuid>,
    pub initiated_by: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct TestConnectivityRequest {
    pub services: Vec<String>,
    pub initiated_by: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct WebhookListQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub is_active: Option<bool>,
}

#[derive(serde::Deserialize)]
pub struct IntegrationLogsQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub integration_type: Option<String>,
    pub operation: Option<String>,
    pub job_id: Option<Uuid>,
}

#[derive(serde::Serialize)]
struct TestResult {
    pub service: String,
    pub status: String,
    pub response_time_ms: u64,
    pub details: serde_json::Value,
}

struct WebhookDeliveryResult {
    pub success: bool,
    pub status_code: Option<i32>,
    pub response_body: Option<String>,
}
