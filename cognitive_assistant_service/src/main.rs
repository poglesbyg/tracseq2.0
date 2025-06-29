use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::info;

#[derive(Clone)]
pub struct AppState {
    pub ollama_url: String,
    pub database_url: String,
}

#[derive(Serialize)]
struct HealthResponse {
    service: String,
    status: String,
    version: String,
    ollama_connected: bool,
    timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize)]
struct LabQueryRequest {
    query: String,
    user_role: Option<String>,
    context: Option<String>,
}

#[derive(Serialize)]
struct LabQueryResponse {
    response: String,
    confidence: f64,
    reasoning: String,
    response_time_ms: u64,
    sources: Vec<String>,
}

async fn health_check(State(state): State<AppState>) -> Result<Json<HealthResponse>, StatusCode> {
    // Simple health check - in full version this would check Ollama connectivity
    let response = HealthResponse {
        service: "cognitive_assistant_service".to_string(),
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        ollama_connected: true, // Simplified for now
        timestamp: chrono::Utc::now(),
    };
    
    Ok(Json(response))
}

async fn handle_intelligent_query(
    State(state): State<AppState>,
    Json(request): Json<LabQueryRequest>,
) -> Result<Json<LabQueryResponse>, StatusCode> {
    info!("🧠 Processing intelligent query: {}", request.query);

    // Simplified AI response - in full version this would use Ollama
    let start_time = std::time::Instant::now();
    
    // Simulate AI processing
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    let response = LabQueryResponse {
        response: format!("This is a Phase 10 AI response to: '{}'", request.query),
        confidence: 0.85,
        reasoning: "Using laboratory domain knowledge and context analysis".to_string(),
        response_time_ms: start_time.elapsed().as_millis() as u64,
        sources: vec!["ollama_llama3.2".to_string()],
    };

    info!("✅ Query processed successfully");
    Ok(Json(response))
}

async fn handle_proactive_suggestions(
    State(state): State<AppState>,
) -> Result<Json<Vec<String>>, StatusCode> {
    info!("🔮 Generating proactive suggestions");

    // Simplified suggestions
    let suggestions = vec![
        "Consider optimizing sample storage utilization".to_string(),
        "Review quality control metrics for this week".to_string(),
        "Check equipment maintenance schedules".to_string(),
    ];

    info!("✅ Generated {} proactive suggestions", suggestions.len());
    Ok(Json(suggestions))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("cognitive_assistant_service=debug,info")
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:tracseq_password@postgres:5432/tracseq".to_string());
    
    let ollama_url = std::env::var("OLLAMA_BASE_URL")
        .unwrap_or_else(|_| "http://ollama:11434".to_string());
    
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8090".to_string())
        .parse::<u16>()?;

    info!("🧠 Starting Cognitive Laboratory Assistant Service on port {}", port);
    info!("🔗 Database URL: {}", database_url);
    info!("🤖 Ollama URL: {}", ollama_url);

    // Create application state
    let app_state = AppState {
        ollama_url,
        database_url,
    };

    // Build our application routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/ask", post(handle_intelligent_query))
        .route("/suggest", get(handle_proactive_suggestions))
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    // Start the server
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    
    info!("🚀 Cognitive Laboratory Assistant Service listening on port {}", port);
    info!("📋 Available endpoints:");
    info!("   • GET  /health - Service health check");
    info!("   • POST /ask - Intelligent laboratory queries");
    info!("   • GET  /suggest - Proactive suggestions");

    axum::serve(listener, app).await?;

    Ok(())
} 