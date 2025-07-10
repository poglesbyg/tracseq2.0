use anyhow::Result;
use axum::{
    routing::{get, post, put, delete},
    Router,
    http::StatusCode,
    Json,
};
use serde_json::json;
use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

// Import only the essential modules for hierarchical storage
use enhanced_storage_service::{
    models, handlers, database::DatabasePool, config::Config, error::StorageResult,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "enhanced_storage_service=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("🏪 Starting Enhanced Storage Service with Hierarchical Storage");

    // Get configuration from environment
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8082".to_string())
        .parse::<u16>()?;
    let database_url = std::env::var("DATABASE_URL")
        .or_else(|_| std::env::var("STORAGE_DATABASE_URL"))
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/lab_manager".to_string());

    info!("📋 Configuration loaded - Port: {}, Database: {}", port, database_url);

    // Setup database connection
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;
    info!("🗄️ Database connection established");

    // Test database connectivity
    sqlx::query("SELECT 1").execute(&pool).await?;
    info!("✅ Database connectivity verified");

    // Build the application router with hierarchical storage routes
    let app = Router::new()
        // Health check routes
        .route("/health", get(health_check))
        .route("/health/ready", get(readiness_check))
        
        // Basic storage routes
        .route("/storage/locations", get(list_locations))
        .route("/storage/locations", post(create_location))
        
        // Hierarchical storage routes
        .route("/storage/containers", post(handlers::hierarchical_storage::create_container))
        .route("/storage/containers", get(handlers::hierarchical_storage::list_containers))
        .route("/storage/containers/:container_id", get(handlers::hierarchical_storage::get_container_with_details))
        .route("/storage/containers/:container_id/hierarchy", get(handlers::hierarchical_storage::get_storage_hierarchy))
        .route("/storage/containers/:container_id/grid", get(handlers::hierarchical_storage::get_container_grid))
        .route("/storage/samples/assign", post(handlers::hierarchical_storage::assign_sample_to_position))
        .route("/storage/samples/:sample_id/move", put(handlers::hierarchical_storage::move_sample_to_position))
        .route("/storage/samples/:sample_id/location-detailed", get(handlers::hierarchical_storage::get_sample_location_detailed))
        .route("/storage/samples/:sample_id/position", delete(handlers::hierarchical_storage::remove_sample_from_position))
        
        .layer(
            tower::ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive()),
        )
        .with_state(pool);

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("🚀 Enhanced Storage Service with Hierarchical Storage listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "enhanced-storage-service",
        "features": ["hierarchical_storage"],
        "timestamp": chrono::Utc::now()
    }))
}

async fn readiness_check() -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(json!({
        "status": "ready",
        "service": "enhanced-storage-service",
        "features": ["hierarchical_storage"],
        "timestamp": chrono::Utc::now()
    })))
}

async fn list_locations(
    axum::extract::State(pool): axum::extract::State<sqlx::PgPool>,
) -> StorageResult<Json<serde_json::Value>> {
    let locations = sqlx::query_as!(
        models::StorageLocation,
        "SELECT id, name, zone_type, temperature_celsius, capacity, current_usage, status, location_code, description, created_at, updated_at FROM storage_locations ORDER BY name"
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(json!({
        "locations": locations
    })))
}

async fn create_location(
    axum::extract::State(pool): axum::extract::State<sqlx::PgPool>,
    Json(request): Json<models::CreateStorageLocationRequest>,
) -> StorageResult<Json<models::StorageLocation>> {
    let location = sqlx::query_as!(
        models::StorageLocation,
        r#"
        INSERT INTO storage_locations (name, zone_type, temperature_celsius, capacity, location_code, description)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, name, zone_type, temperature_celsius, capacity, current_usage, status, location_code, description, created_at, updated_at
        "#,
        request.name,
        request.zone_type.to_string(),
        request.temperature_celsius,
        request.capacity,
        request.location_code,
        request.description
    )
    .fetch_one(&pool)
    .await?;

    Ok(Json(location))
} 