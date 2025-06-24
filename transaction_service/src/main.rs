//! TracSeq Transaction Service - Distributed Transaction Management using Saga Pattern

mod saga;
mod coordinator;
mod models;
mod services;
mod persistence;
mod workflows;

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
use workflows::{
    WorkflowConfig,
    orchestrator::{EnhancedWorkflowService, EnhancedWorkflowRequest},
};
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
    enhanced_workflow_service: Option<EnhancedWorkflowService>,
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
    let coordinator = match TransactionCoordinator::with_persistence(config.clone()).await {
        Ok(coordinator) => Arc::new(coordinator),
        Err(e) => {
            error!("Failed to initialize transaction coordinator with persistence: {}", e);
            info!("Falling back to in-memory coordinator...");
            Arc::new(TransactionCoordinator::new(config))
        }
    };
    let health_service = HealthService::new(coordinator.clone());
    let workflow_service = WorkflowService::new(coordinator.clone());
    
    // Initialize enhanced workflow service with RAG integration
    let enhanced_workflow_service = match EnhancedWorkflowService::new(
        coordinator.clone(),
        load_workflow_config(),
    ).await {
        Ok(service) => {
            info!("Enhanced workflow service with RAG integration initialized successfully");
            Some(service)
        }
        Err(e) => {
            error!("Failed to initialize enhanced workflow service: {}", e);
            info!("Continuing without enhanced workflow capabilities");
            None
        }
    };
    
    let metrics_service = MetricsService::new(coordinator.clone());

    let app_state = AppState {
        coordinator,
        health_service,
        workflow_service,
        enhanced_workflow_service,
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

fn load_workflow_config() -> WorkflowConfig {
    WorkflowConfig {
        rag_service_url: std::env::var("RAG_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:8086".to_string()),
        enable_ai_decisions: std::env::var("ENABLE_AI_DECISIONS")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true),
        max_workflow_steps: std::env::var("MAX_WORKFLOW_STEPS")
            .unwrap_or_else(|_| "50".to_string())
            .parse()
            .unwrap_or(50),
        ai_timeout_seconds: std::env::var("AI_TIMEOUT_SECONDS")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .unwrap_or(30),
        lab_manager_url: std::env::var("LAB_MANAGER_URL")
            .unwrap_or_else(|_| "http://localhost:3000".to_string()),
        ai_confidence_threshold: std::env::var("AI_CONFIDENCE_THRESHOLD")
            .unwrap_or_else(|_| "0.8".to_string())
            .parse()
            .unwrap_or(0.8),
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
        .route("/api/v1/workflows/enhanced", post(execute_enhanced_workflow))
        .route("/api/v1/workflows/enhanced/templates", get(list_workflow_templates))
        .route("/api/v1/workflows/enhanced/ai-analyze", post(ai_analyze_workflow))
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

async fn execute_enhanced_workflow(
    State(app_state): State<AppState>,
    Json(request): Json<EnhancedWorkflowRequest>,
) -> Result<Json<workflows::orchestrator::EnhancedWorkflowResult>, StatusCode> {
    if let Some(enhanced_service) = &app_state.enhanced_workflow_service {
        match enhanced_service.execute_enhanced_workflow(request).await {
            Ok(result) => Ok(Json(result)),
            Err(e) => {
                error!("Enhanced workflow execution failed: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    } else {
        error!("Enhanced workflow service not available");
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

async fn list_workflow_templates(
    State(app_state): State<AppState>,
) -> Result<Json<Vec<workflows::templates::LaboratoryWorkflowTemplate>>, StatusCode> {
    // Return available workflow templates
    let templates = vec![
        workflows::templates::LaboratoryWorkflowTemplate {
            template_id: "standard_processing".to_string(),
            name: "Standard Sample Processing".to_string(),
            description: "General purpose sample processing workflow".to_string(),
            steps: vec![],
            estimated_duration_minutes: 120,
            required_equipment: vec!["centrifuge".to_string(), "pipettes".to_string()],
            quality_checkpoints: vec!["initial_qc".to_string(), "final_qc".to_string()],
            ai_generated: false,
            confidence_score: 1.0,
        },
        workflows::templates::LaboratoryWorkflowTemplate {
            template_id: "dna_extraction".to_string(),
            name: "DNA Extraction".to_string(),
            description: "Standardized DNA extraction workflow".to_string(),
            steps: vec![],
            estimated_duration_minutes: 180,
            required_equipment: vec!["extraction_kit".to_string(), "centrifuge".to_string()],
            quality_checkpoints: vec!["purity_check".to_string(), "yield_check".to_string()],
            ai_generated: false,
            confidence_score: 1.0,
        },
    ];
    
    Ok(Json(templates))
}

#[derive(serde::Deserialize)]
struct AiAnalysisRequest {
    workflow_type: String,
    sample_data: workflows::orchestrator::SampleWorkflowData,
    lab_context: workflows::orchestrator::LaboratoryContext,
}

#[derive(serde::Serialize)]
struct AiAnalysisResponse {
    analysis: workflows::orchestrator::AiInsights,
    recommendations: Vec<String>,
    confidence: f64,
}

async fn ai_analyze_workflow(
    State(app_state): State<AppState>,
    Json(request): Json<AiAnalysisRequest>,
) -> Result<Json<AiAnalysisResponse>, StatusCode> {
    if let Some(_enhanced_service) = &app_state.enhanced_workflow_service {
        // Simulate AI analysis
        let analysis = workflows::orchestrator::AiInsights {
            optimizations: vec![
                workflows::orchestrator::WorkflowOptimization {
                    optimization_type: "efficiency".to_string(),
                    description: "Reduce processing time by 15%".to_string(),
                    potential_improvement: 0.15,
                    confidence: 0.85,
                    implementation_effort: workflows::orchestrator::ImplementationEffort::Medium,
                }
            ],
            predictions: vec![
                workflows::orchestrator::AiPrediction {
                    prediction_type: "success_rate".to_string(),
                    predicted_value: serde_json::json!(0.95),
                    confidence: 0.88,
                    basis: "Historical data analysis".to_string(),
                }
            ],
            risk_assessments: vec![
                workflows::orchestrator::RiskAssessment {
                    risk_type: "contamination".to_string(),
                    risk_level: workflows::RiskLevel::Low,
                    probability: 0.05,
                    impact: 0.3,
                    mitigation_strategies: vec!["Use sterile techniques".to_string()],
                }
            ],
            learning_insights: vec![
                "Sample type is well-suited for standard processing".to_string(),
                "Consider automated quality control".to_string(),
            ],
        };

        let response = AiAnalysisResponse {
            analysis,
            recommendations: vec![
                "Proceed with standard processing protocol".to_string(),
                "Monitor temperature closely during incubation".to_string(),
                "Consider parallel processing for efficiency".to_string(),
            ],
            confidence: 0.87,
        };

        Ok(Json(response))
    } else {
        error!("Enhanced workflow service not available for AI analysis");
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}
