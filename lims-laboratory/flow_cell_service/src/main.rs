use axum::{routing::{get, post, put, delete}, Router};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

mod models;
mod handlers;
mod ai_optimizer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Database connection
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // Run migrations unless SKIP_MIGRATIONS is set
    if std::env::var("SKIP_MIGRATIONS").unwrap_or_default() != "true" {
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await?;
    }

    // Build application
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/flow-cells/types", get(handlers::list_flow_cell_types))
        .route("/api/v1/flow-cells/types/:id/stats", get(handlers::get_flow_cell_type_stats))
        .route("/api/v1/flow-cells/designs", get(handlers::list_flow_cell_designs))
        .route("/api/v1/flow-cells/designs", post(handlers::create_flow_cell_design))
        .route("/api/v1/flow-cells/designs/:id", get(handlers::get_flow_cell_design))
        .route("/api/v1/flow-cells/designs/:id", put(handlers::update_flow_cell_design))
        .route("/api/v1/flow-cells/designs/:id", delete(handlers::delete_flow_cell_design))
        .route("/api/v1/flow-cells/designs/:id/approve", post(handlers::approve_flow_cell_design))
        .route("/api/v1/flow-cells/designs/:id/lanes", get(handlers::get_flow_cell_lanes))
        .route("/api/v1/flow-cells/designs/:design_id/lanes/:lane_number", put(handlers::update_flow_cell_lane))
        .route("/api/v1/flow-cells/optimize", post(handlers::optimize_flow_cell))
        .with_state(pool)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    // Start server
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8086".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid number");
    
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Flow Cell Service listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "healthy"
} 