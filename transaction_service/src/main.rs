//! TracSeq Transaction Service - Distributed Transaction Management using Saga Pattern

mod saga;
mod coordinator;
mod models;
mod services;
mod persistence;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, delete},
    Router,
};
use coordinator::{TransactionCoordinator, CoordinatorConfig, TransactionRequest};
use models::TransactionServiceHealth;
use services::{HealthService, WorkflowService, MetricsService};
use saga::step::{SampleCreationData, StorageRequirements};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer, compression::CompressionLayer};
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    coordinator: Arc<TransactionCoordinator>,
    health_service: HealthService,
    workflow_service: WorkflowService,
    metrics_service: MetricsService,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting TracSeq Transaction Service...");

    let config = load_config();
    let coordinator = match TransactionCoordinator::with_persistence(config).await {
        Ok(coordinator) => Arc::new(coordinator),
        Err(e) => {
            error!("Failed to initialize transaction coordinator with persistence: {}", e);
            info!("Falling back to in-memory coordinator...");
            Arc::new(TransactionCoordinator::new(config))
        }
    };
    let health_service = HealthService::new(coordinator.clone());
    let workflow_service = WorkflowService::new(coordinator.clone());
    let metrics_service = MetricsService::new(coordinator.clone());

    let app_state = AppState {
        coordinator,
        health_service,
        workflow_service,
        metrics_service,
    };

    let app = create_router(app_state);
    let port = std::env::var("PORT").unwrap_or_else(|_| "8088".to_string());
    let addr = format!("0.0.0.0:{}", port);
    
    info!("TracSeq Transaction Service listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn load_config() -> CoordinatorConfig {
    use persistence::DatabaseConfig;
    
    let database_config = DatabaseConfig {
        connection_string: std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://tracseq:tracseq@localhost:5432/tracseq_transactions".to_string()),
        max_connections: std::env::var("DB_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "20".to_string())
            .parse()
            .unwrap_or(20),
        min_connections: std::env::var("DB_MIN_CONNECTIONS")
            .unwrap_or_else(|_| "5".to_string())
            .parse()
            .unwrap_or(5),
        connection_timeout_seconds: std::env::var("DB_CONNECTION_TIMEOUT_SECONDS")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .unwrap_or(30),
    };

    CoordinatorConfig {
        max_concurrent_sagas: std::env::var("MAX_CONCURRENT_SAGAS")
            .unwrap_or_else(|_| "100".to_string())
            .parse()
            .unwrap_or(100),
        default_timeout_ms: std::env::var("DEFAULT_TIMEOUT_MS")
            .unwrap_or_else(|_| "300000".to_string())
            .parse()
            .unwrap_or(300000),
        enable_events: std::env::var("ENABLE_EVENTS")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true),
        event_service_url: std::env::var("EVENT_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:8087".to_string()),
        enable_persistence: std::env::var("ENABLE_PERSISTENCE")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true),
        database: database_config,
        cleanup_after_hours: std::env::var("CLEANUP_AFTER_HOURS")
            .unwrap_or_else(|_| "24".to_string())
            .parse()
            .unwrap_or(24),
    }
}

fn create_router(app_state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/health/detailed", get(detailed_health_check))
        .route("/api/v1/transactions", post(execute_transaction))
        .route("/api/v1/transactions", get(list_active_transactions))
        .route("/api/v1/transactions/:saga_id", get(get_transaction_status))
        .route("/api/v1/transactions/:saga_id", delete(cancel_transaction))
        .route("/api/v1/workflows/sample-submission", post(execute_sample_submission))
        .route("/api/v1/metrics/coordinator", get(get_coordinator_metrics))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new())
                .layer(CorsLayer::permissive())
        )
        .with_state(app_state)
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "transaction-service",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now()
    }))
}

async fn detailed_health_check(
    State(app_state): State<AppState>,
) -> Result<Json<TransactionServiceHealth>, StatusCode> {
    let health = app_state.health_service.get_health().await;
    
    match health.status.as_str() {
        "healthy" => Ok(Json(health)),
        "degraded" => Ok(Json(health)),
        _ => Err(StatusCode::SERVICE_UNAVAILABLE),
    }
}

async fn execute_transaction(
    State(app_state): State<AppState>,
    Json(request): Json<TransactionRequest>,
) -> Result<Json<saga::SagaExecutionResult>, StatusCode> {
    let saga = saga::TransactionSaga::builder(&request.name)
        .with_timeout(request.timeout_ms.unwrap_or(300000))
        .build();

    match app_state.coordinator.execute_transaction(request, saga).await {
        Ok(result) => Ok(Json(result)),
        Err(e) => {
            error!("Transaction execution failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn list_active_transactions(
    State(app_state): State<AppState>,
) -> Json<Vec<coordinator::TransactionStatus>> {
    let transactions = app_state.coordinator.list_active_transactions().await;
    Json(transactions)
}

async fn get_transaction_status(
    State(app_state): State<AppState>,
    Path(saga_id): Path<Uuid>,
) -> Result<Json<coordinator::TransactionStatus>, StatusCode> {
    match app_state.coordinator.get_transaction_status(saga_id).await {
        Some(status) => Ok(Json(status)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn cancel_transaction(
    State(app_state): State<AppState>,
    Path(saga_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match app_state.coordinator.cancel_transaction(saga_id).await {
        Ok(_) => Ok(Json(serde_json::json!({
            "message": "Transaction cancelled successfully",
            "saga_id": saga_id
        }))),
        Err(e) => {
            error!("Failed to cancel transaction {}: {}", saga_id, e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

#[derive(serde::Deserialize)]
struct SampleSubmissionRequest {
    sample_data: SampleCreationData,
    storage_requirements: StorageRequirements,
    notification_recipients: Vec<String>,
    #[serde(flatten)]
    transaction_request: TransactionRequest,
}

async fn execute_sample_submission(
    State(app_state): State<AppState>,
    Json(req): Json<SampleSubmissionRequest>,
) -> Result<Json<saga::SagaExecutionResult>, StatusCode> {
    let saga = app_state
        .workflow_service
        .create_sample_submission_workflow(
            req.transaction_request.clone(),
            req.sample_data,
            req.storage_requirements,
            req.notification_recipients,
        )
        .await
        .map_err(|e| {
            error!("Failed to create sample submission workflow: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    match app_state
        .workflow_service
        .execute_workflow(req.transaction_request, saga)
        .await
    {
        Ok(result) => Ok(Json(result)),
        Err(e) => {
            error!("Sample submission workflow failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_coordinator_metrics(
    State(app_state): State<AppState>,
) -> Json<coordinator::CoordinatorStatistics> {
    let stats = app_state.metrics_service.get_coordinator_statistics().await;
    Json(stats)
}
