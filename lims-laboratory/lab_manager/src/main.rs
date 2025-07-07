pub mod assembly;
pub mod config;
pub mod errors;
pub mod events;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod observability;
pub mod plugins;
pub mod repositories;
pub mod router;
pub mod sample_submission;
pub mod sequencing;
pub mod services;
pub mod validation;

use std::net::SocketAddr;
use std::process;
use std::sync::Arc;

use assembly::assemble_production_components;
use middleware::validation::initialize_validation_regexes;
use router::create_app_router;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize validation regexes
    if let Err(e) = initialize_validation_regexes() {
        tracing::error!("Failed to initialize validation regexes: {}", e);
        eprintln!("❌ Error: Failed to initialize validation regexes");
        eprintln!("   Details: {}", e);
        process::exit(1);
    }

    // Initialize proxy system if enabled
    router::proxy_routes::init_proxy_system().await;

    // Assemble all components using the new modular system
    let components = match assemble_production_components().await {
        Ok(components) => Arc::new(components),
        Err(e) => {
            tracing::error!("Failed to assemble application components: {}", e);
            eprintln!("❌ Error: Failed to initialize application components");
            eprintln!("   Details: {}", e);
            eprintln!("   Please check your configuration and database connectivity.");
            process::exit(1);
        }
    };

    // Create the application router
    let app = create_app_router().with_state(components);

    // Get server configuration
    let config = match config::AppConfig::from_env() {
        Ok(config) => config,
        Err(e) => {
            tracing::error!("Failed to load configuration: {}", e);
            eprintln!("❌ Error: Failed to load configuration");
            eprintln!("   Details: {}", e);
            eprintln!("   Please check your environment variables.");
            process::exit(1);
        }
    };

    // Parse the address
    let addr = match format!("{}:{}", config.server.host, config.server.port).parse::<SocketAddr>()
    {
        Ok(addr) => addr,
        Err(e) => {
            tracing::error!(
                "Invalid host:port combination: {}:{}",
                config.server.host,
                config.server.port
            );
            eprintln!(
                "❌ Error: Invalid host:port combination: {}:{}",
                config.server.host, config.server.port
            );
            eprintln!("   Details: {}", e);
            process::exit(1);
        }
    };

    tracing::info!("Starting server on {}", addr);
    println!("🚀 TracSeq 2.0 Laboratory Management System starting...");
    println!("📡 Server listening on: http://{}", addr);

    // Bind the TCP listener
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(e) => {
            tracing::error!("Failed to bind to address {}: {}", addr, e);
            eprintln!("❌ Error: Failed to bind to address {}", addr);
            eprintln!("   Details: {}", e);
            eprintln!("   Please check if the port is already in use or if you have permission to bind to it.");
            process::exit(1);
        }
    };

    // Start the server
    if let Err(e) = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    {
        tracing::error!("Server error: {}", e);
        eprintln!("❌ Server error: {}", e);
        process::exit(1);
    }
}
