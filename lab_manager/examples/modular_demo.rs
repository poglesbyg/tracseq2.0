/// Demo script showcasing the new IKEA-like modular architecture
///
/// This script demonstrates how different "product lines" can be assembled
/// like IKEA furniture - mix and match components for different use cases.
///
/// Run with: cargo run --example modular_demo
use lab_manager::assembly::{
    components::{DatabaseComponentBuilder, StorageComponentBuilder},
    CompactLine, CompactVariant, ComponentError, HybridLine, ProductLine, ProfessionalLine,
    ProfessionalVariant, ServiceRegistry, StudioLine, StudioVariant,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for better visibility
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🏗️  TracSeq Modular Architecture Demo");
    println!("=====================================");
    println!();

    // Show the IKEA-like product catalog
    show_product_catalog().await?;

    // Demo 1: Studio Line - Perfect for development
    println!("🛠️  DEMO 1: Studio Line - Developer Setup");
    println!("------------------------------------------");
    demo_studio_line().await?;

    // Demo 2: Professional Line - Production ready
    println!("🏢 DEMO 2: Professional Line - Production Setup");
    println!("-----------------------------------------------");
    demo_professional_line().await?;

    // Demo 3: Compact Line - Resource efficient
    println!("📦 DEMO 3: Compact Line - Container Setup");
    println!("-----------------------------------------");
    demo_compact_line().await?;

    // Demo 4: Hybrid Line - Custom configuration
    println!("🔧 DEMO 4: Hybrid Line - Custom Assembly");
    println!("----------------------------------------");
    demo_hybrid_line().await?;

    // Demo 5: Component health monitoring
    println!("🏥 DEMO 5: Component Health Monitoring");
    println!("--------------------------------------");
    demo_health_monitoring().await?;

    println!("✅ All demos completed successfully!");
    println!("🎯 The modular architecture makes it easy to:");
    println!("   • Mix and match components for different environments");
    println!("   • Configure applications declaratively");
    println!("   • Test components in isolation");
    println!("   • Scale and deploy flexibly");
    println!();
    println!("Just like IKEA furniture - modular, democratic, and easy to assemble! 🪑");

    Ok(())
}

/// Show the available product lines like an IKEA catalog
async fn show_product_catalog() -> Result<(), ComponentError> {
    println!("📋 Available Product Lines (IKEA-style catalog):");
    println!("================================================");

    for product_line in ProductLine::catalog() {
        println!(
            "🏷️  {}: {}",
            format!("{:?}", product_line)
                .replace('(', " - ")
                .replace(')', ""),
            product_line.description()
        );
    }
    println!();

    Ok(())
}

/// Demo the Studio Line - perfect for development
async fn demo_studio_line() -> Result<(), ComponentError> {
    // Developer setup - fast and lightweight
    let mut registry = StudioLine::developer_setup().await?;

    println!("✅ Studio Developer setup assembled");
    println!("   • In-memory database for quick iterations");
    println!("   • Mock storage for fast testing");

    // Health check
    let health = registry.health_check_all().await?;
    println!(
        "   • Health check: {} components healthy",
        health.values().filter(|&&h| h).count()
    );

    // Shutdown gracefully
    registry.shutdown_all().await?;
    println!("   • Clean shutdown completed");
    println!();

    // Unit test setup - completely isolated
    let mut test_registry = StudioLine::unit_test_setup().await?;
    println!("✅ Studio Unit Test setup assembled");
    println!("   • Isolated test database");
    println!("   • In-memory storage with auto-cleanup");

    test_registry.shutdown_all().await?;
    println!("   • Test environment cleaned up");
    println!();

    Ok(())
}

/// Demo the Professional Line - production ready
async fn demo_professional_line() -> Result<(), ComponentError> {
    println!("ℹ️  Professional Line demos require production environment variables");
    println!("   In a real environment, this would set up:");
    println!("   • Production PostgreSQL with connection pooling");
    println!("   • Persistent file system storage");
    println!("   • Full authentication and authorization");
    println!("   • Comprehensive logging and monitoring");
    println!();

    // Note: We don't actually run this in the demo because it requires real infrastructure
    println!("💡 Use ASSEMBLY_MODE=professional to run this configuration");
    println!();

    Ok(())
}

/// Demo the Compact Line - resource efficient
async fn demo_compact_line() -> Result<(), ComponentError> {
    println!("ℹ️  Compact Line is optimized for:");
    println!("   • Docker containers with limited resources");
    println!("   • Edge deployments with local storage");
    println!("   • Minimal dependencies and fast startup");
    println!();

    println!("💡 Use ASSEMBLY_MODE=compact to run this configuration");
    println!();

    Ok(())
}

/// Demo the Hybrid Line - custom configurations
async fn demo_hybrid_line() -> Result<(), ComponentError> {
    println!("🔧 Building a custom hybrid configuration...");

    // Create a custom assembly - this is where the IKEA-like modularity shines!
    let custom_registry = HybridLine::custom()
        .with_database(
            DatabaseComponentBuilder::new().for_testing(), // Use test database
        )
        .with_storage(
            StorageComponentBuilder::new().in_memory(), // Use in-memory storage
        )
        .with_config("custom_setting", "demo_value")
        .build()
        .await?;

    println!("✅ Custom hybrid assembly created with:");
    println!("   • Test database component");
    println!("   • In-memory storage component");
    println!("   • Custom configuration settings");

    // The registry is automatically initialized
    println!("   • All components initialized successfully");
    println!();

    Ok(())
}

/// Demo component health monitoring
async fn demo_health_monitoring() -> Result<(), ComponentError> {
    println!("🏥 Setting up health monitoring demo...");

    // Create a simple test registry
    let mut registry = StudioLine::developer_setup().await?;

    // Perform health checks
    let health_results = registry.health_check_all().await?;

    println!("📊 Health Check Results:");
    for (component_id, is_healthy) in health_results {
        let status = if is_healthy {
            "✅ HEALTHY"
        } else {
            "❌ UNHEALTHY"
        };
        println!("   • {}: {}", component_id, status);
    }

    println!("🔄 Demonstrating graceful shutdown...");
    registry.shutdown_all().await?;
    println!("✅ All components shut down gracefully");
    println!();

    Ok(())
}

/// Helper function to simulate different environments
#[allow(dead_code)]
async fn simulate_environment(env_name: &str) -> Result<(), ComponentError> {
    println!("🌍 Simulating {} environment...", env_name);

    let product_line = match env_name {
        "development" => ProductLine::Studio(StudioVariant::Developer),
        "testing" => ProductLine::Studio(StudioVariant::UnitTest),
        "production" => ProductLine::Professional(ProfessionalVariant::Production),
        "container" => ProductLine::Compact(CompactVariant::Container),
        _ => {
            return Err(ComponentError::ConfigurationError(format!(
                "Unknown environment: {}",
                env_name
            )))
        }
    };

    println!("📋 Selected: {}", product_line.description());

    // In a real scenario, you would:
    // let registry = product_line.assemble().await?;
    // ... use the registry ...
    // registry.shutdown_all().await?;

    println!("✅ Environment simulation complete");
    Ok(())
}
