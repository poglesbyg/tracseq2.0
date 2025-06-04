use lab_manager::{
    assembly::ComponentBuilder,
    config::AppConfig,
    errors::{ComponentError, ErrorHandler, ErrorResponse, HttpErrorHandler},
    events::{Event, EventBus, EventFilter, EventPriority},
    services::{Service, ServiceRegistry},
    validation::{ValidationChain, ValidationError, ValidationResult, Validator},
};
use std::sync::Arc;

/// Example showing IKEA-style modular assembly
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧱 IKEA-Style Modular Lab Manager Demo");
    println!("=====================================");
    println!("This example would demonstrate:");
    println!("✅ Democratic component assembly");
    println!("✅ Modular error handling");
    println!("✅ Event-driven communication");
    println!("✅ Composable validation");
    println!("✅ Service registry pattern");
    println!("🚀 Your lab manager is now truly modular!");

    // 1. MODULAR CONFIGURATION
    println!("📋 1. Loading Modular Configuration...");
    let config = AppConfig::for_testing();
    println!(
        "✅ Configuration loaded: {} environments",
        if config.server.cors_enabled {
            "CORS enabled"
        } else {
            "CORS disabled"
        }
    );

    // 2. MODULAR COMPONENT ASSEMBLY
    println!("\n🔧 2. Assembling Components Modularly...");

    let components = ComponentBuilder::new(config.clone())
        .with_database()
        .await?
        .with_storage()
        .await?
        .with_sample_processing()?
        .with_sequencing()?
        .build()?;

    println!("✅ All components assembled successfully!");
    println!("   - Database: Connected");
    println!("   - Storage: Ready");
    println!("   - Sample Processing: Initialized");
    println!("   - Sequencing: Initialized");

    // 3. MODULAR ERROR HANDLING
    println!("\n❌ 3. Demonstrating Modular Error Handling...");

    // Example custom error
    #[derive(Debug, thiserror::Error)]
    enum DemoError {
        #[error("Sample validation failed")]
        ValidationFailed,
        #[error("Storage quota exceeded")]
        StorageQuotaExceeded,
    }

    impl ComponentError for DemoError {
        fn error_code(&self) -> &'static str {
            match self {
                Self::ValidationFailed => "DEMO_VALIDATION_FAILED",
                Self::StorageQuotaExceeded => "DEMO_STORAGE_QUOTA_EXCEEDED",
            }
        }

        fn severity(&self) -> lab_manager::errors::ErrorSeverity {
            match self {
                Self::ValidationFailed => lab_manager::errors::ErrorSeverity::Medium,
                Self::StorageQuotaExceeded => lab_manager::errors::ErrorSeverity::High,
            }
        }

        fn is_retryable(&self) -> bool {
            matches!(self, Self::StorageQuotaExceeded)
        }
    }

    // Demonstrate error handling
    let error = DemoError::ValidationFailed;
    let http_response = HttpErrorHandler::handle_error(error);
    println!("✅ Error handled modularly:");
    println!("   - Status: {:?}", http_response.0);
    println!("   - Error ID: {}", http_response.1.error_id);

    // 4. MODULAR EVENT SYSTEM
    println!("\n📡 4. Setting Up Modular Event System...");

    let event_bus = Arc::new(EventBus::new(1000));

    // Create sample events
    use lab_manager::events::types::{SampleCreatedEvent, TemplateUploadedEvent};
    use uuid::Uuid;

    let sample_event = SampleCreatedEvent::new(
        Uuid::new_v4(),
        "Test Sample".to_string(),
        "BARCODE123".to_string(),
        "Lab A".to_string(),
        "demo_user".to_string(),
    );

    let template_event = TemplateUploadedEvent::new(
        Uuid::new_v4(),
        "Sample Template".to_string(),
        "/storage/template.xlsx".to_string(),
        "demo_user".to_string(),
    );

    // Publish events
    event_bus.publish(sample_event).await?;
    event_bus.publish(template_event).await?;

    println!("✅ Events published successfully!");

    // Get event statistics
    let stats = event_bus.get_stats().await;
    println!("   - Total events: {}", stats.total_events);
    println!(
        "   - Event types: {:?}",
        stats.events_by_type.keys().collect::<Vec<_>>()
    );

    // 5. MODULAR VALIDATION
    println!("\n✅ 5. Demonstrating Modular Validation...");

    // Example data to validate
    #[derive(Debug)]
    struct SampleData {
        name: String,
        barcode: String,
        location: String,
    }

    // Example validator
    struct SampleValidator;

    impl Validator<SampleData> for SampleValidator {
        fn validate(&self, item: &SampleData) -> ValidationResult {
            let mut errors = Vec::new();

            if item.name.is_empty() {
                errors.push(
                    ValidationError::new(
                        "EMPTY_NAME".to_string(),
                        "Sample name cannot be empty".to_string(),
                    )
                    .with_field("name".to_string()),
                );
            }

            if item.barcode.len() < 6 {
                errors.push(
                    ValidationError::new(
                        "INVALID_BARCODE".to_string(),
                        "Barcode must be at least 6 characters".to_string(),
                    )
                    .with_field("barcode".to_string()),
                );
            }

            ValidationResult {
                is_valid: errors.is_empty(),
                errors,
                warnings: Vec::new(),
                metadata: std::collections::HashMap::new(),
            }
        }

        fn config(&self) -> lab_manager::validation::ValidatorConfig {
            lab_manager::validation::ValidatorConfig {
                name: "SampleValidator".to_string(),
                version: "1.0.0".to_string(),
                strict_mode: false,
                custom_rules: std::collections::HashMap::new(),
            }
        }
    }

    // Test validation
    let valid_sample = SampleData {
        name: "Valid Sample".to_string(),
        barcode: "VALID123".to_string(),
        location: "Lab A".to_string(),
    };

    let invalid_sample = SampleData {
        name: "".to_string(),
        barcode: "ABC".to_string(),
        location: "Lab B".to_string(),
    };

    let validator = SampleValidator;

    let valid_result = validator.validate(&valid_sample);
    let invalid_result = validator.validate(&invalid_sample);

    println!("✅ Validation results:");
    println!("   - Valid sample: {}", valid_result.is_valid);
    println!(
        "   - Invalid sample: {} (errors: {})",
        invalid_result.is_valid,
        invalid_result.errors.len()
    );

    // 6. MODULAR SERVICE REGISTRY
    println!("\n🏗️ 6. Service Registry Demo...");

    let mut registry = ServiceRegistry::new();

    // In a real implementation, you'd register actual services here
    println!("✅ Service registry initialized");
    println!("   - Ready to register modular services");
    println!("   - Each service can be developed and deployed independently");

    // 7. SUMMARY
    println!("\n🎉 IKEA-Style Modular Demo Complete!");
    println!("=====================================");
    println!("🧱 Demonstrated modular features:");
    println!("   ✅ Democratic component assembly");
    println!("   ✅ Modular error handling");
    println!("   ✅ Event-driven communication");
    println!("   ✅ Composable validation");
    println!("   ✅ Service registry pattern");
    println!("   ✅ Trait-based extensibility");
    println!("\n🔧 Benefits achieved:");
    println!("   - Components can be developed independently");
    println!("   - Easy to test individual components");
    println!("   - Simple to swap implementations");
    println!("   - Clear separation of concerns");
    println!("   - Democratic architecture (no single dominant component)");

    println!("\n🚀 Your lab manager is now truly modular and IKEA-like!");

    Ok(())
}
