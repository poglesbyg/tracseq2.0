use axum::{routing::{get, post, put, delete}, Router};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

mod models;
mod handlers;

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
        .route("/api/v1/projects", get(handlers::list_projects))
        .route("/api/v1/projects", post(handlers::create_project))
        .route("/api/v1/projects/:id", get(handlers::get_project))
        .route("/api/v1/projects/:id", put(handlers::update_project))
        .route("/api/v1/projects/:id", delete(handlers::delete_project))
        .route("/api/v1/projects/:id/files", get(handlers::get_project_files))
        .route("/api/v1/projects/:id/files", post(handlers::upload_project_file))
        .route("/api/v1/projects/:id/signoffs", get(handlers::list_project_signoffs))
        .route("/api/v1/batches", get(handlers::list_batches))
        .route("/api/v1/batches", post(handlers::create_batch))
        .route("/api/v1/batches/:id", get(handlers::get_batch))
        .route("/api/v1/signoffs", post(handlers::create_signoff))
        .route("/api/v1/signoffs/:id", put(handlers::update_signoff))
        .route("/api/v1/templates-repository", get(handlers::list_templates_repository))
        .route("/api/v1/templates-repository/:id/download", post(handlers::download_template_repository))
        .route("/api/v1/permission-queue", get(handlers::list_permission_queue))
        .route("/api/v1/permission-queue", post(handlers::create_permission_request))
        .route("/api/v1/permission-queue/:id", put(handlers::update_permission_request))
        .with_state(pool)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    // Start server
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8084".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid number");
    
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Project Service listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "healthy"
}

 