/// Comprehensive Laboratory Management Demo
///
/// This example showcases the full power of our IKEA-like modular architecture
/// by simulating a complete laboratory workflow using all available components.
///
/// Run with: cargo run --example comprehensive_laboratory_demo
use lab_manager::assembly::{
    components::{
        DatabaseComponent, DatabaseComponentBuilder, EventSystemBuilder, EventSystemComponent,
        LabEvent, MonitoringBuilder, MonitoringComponent, ProcessingStage, SampleProcessingBuilder,
        SampleProcessingComponent, StorageComponent, StorageComponentBuilder, SystemMetrics,
        TemplateProcessingBuilder, TemplateProcessingComponent, TemplateStage,
    },
    product_lines::{HybridLine, StudioLine},
    traits::{Component, ComponentError, ServiceRegistry},
};

use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging for better visibility
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🔬 Comprehensive Laboratory Management Demo");
    println!("==========================================");
    println!("Demonstrating IKEA-like modular architecture in action!");
    println!();

    // Demo 1: Basic Component Assembly
    println!("📦 DEMO 1: Component Assembly");
    println!("-----------------------------");
    demo_component_assembly().await?;

    // Demo 2: Complete Laboratory Workflow
    println!("🏥 DEMO 2: Complete Laboratory Workflow");
    println!("---------------------------------------");
    demo_complete_workflow().await?;

    // Demo 3: Template Processing Pipeline
    println!("📋 DEMO 3: Template Processing Pipeline");
    println!("----------------------------------------");
    demo_template_processing().await?;

    // Demo 4: Event-Driven System Integration
    println!("📡 DEMO 4: Event-Driven Integration");
    println!("-----------------------------------");
    demo_event_system().await?;

    // Demo 5: System Monitoring & Observability
    println!("📊 DEMO 5: System Monitoring");
    println!("----------------------------");
    demo_monitoring_system().await?;

    // Demo 6: Performance & Scalability
    println!("🚀 DEMO 6: Performance & Scalability");
    println!("------------------------------------");
    demo_performance_scalability().await?;

    println!("✅ All demos completed successfully!");
    println!();
    println!("🎯 Key Takeaways:");
    println!("   • Components can be mixed and matched like IKEA furniture");
    println!("   • Each component is self-contained and testable");
    println!("   • System scales horizontally by adding more components");
    println!("   • Configuration-driven assembly for different environments");
    println!("   • Built-in monitoring and observability");
    println!("   • Event-driven architecture enables loose coupling");
    println!();
    println!("🪑 Just like IKEA - modular, democratic, and easy to assemble!");

    Ok(())
}

/// Demo 1: Basic component assembly showing IKEA-like modularity
async fn demo_component_assembly() -> Result<(), ComponentError> {
    println!("🔧 DEMO 1: Component Assembly");
    println!("-----------------------------");

    let mut registry = ServiceRegistry::new();

    let database = DatabaseComponentBuilder::new().for_testing().build()?;
    let storage = StorageComponentBuilder::new().mock().build()?;
    let processor = SampleProcessingBuilder::new().build();
    let event_system = EventSystemBuilder::new().build();

    registry.register_component(database)?;
    registry.register_component(storage)?;
    registry.register_component(processor)?;
    registry.register_component(event_system)?;

    registry.initialize_all().await?;
    println!("  ✅ Assembled and initialized 4 components");

    let health = registry.health_check_all().await?;
    println!("  📊 System health: {} components healthy", health.len());

    registry.shutdown_all().await?;
    println!("  ✅ System shutdown completed");
    println!();

    Ok(())
}

/// Demo 2: Complete laboratory workflow simulation
async fn demo_complete_workflow() -> Result<(), ComponentError> {
    println!("🧪 Setting up complete laboratory environment...");

    // Use Hybrid Line for custom configuration
    let mut registry = HybridLine::custom()
        .with_database(DatabaseComponentBuilder::new().for_testing())
        .with_storage(StorageComponentBuilder::new().mock())
        .build()
        .await?;

    // Add specialized components
    let mut sample_processor = SampleProcessingBuilder::new()
        .with_rag(true)
        .with_confidence_threshold(0.8)
        .build();

    let mut event_system = EventSystemBuilder::new().build();

    let mut template_processor = TemplateProcessingBuilder::new()
        .with_formats(vec!["csv".to_string(), "json".to_string()])
        .build();

    // Initialize all components
    sample_processor.initialize(&registry).await?;
    event_system.initialize(&registry).await?;
    template_processor.initialize(&registry).await?;

    println!("  ✅ Laboratory environment ready");

    // Simulate sample submission workflow
    println!("  📄 Processing laboratory sample submission...");

    let lab_submission = create_sample_submission();
    let result = sample_processor
        .process_document(&lab_submission, "submission.txt")
        .await?;

    println!("    • Document processed successfully");
    println!("    • Processing stage: {:?}", result.stage);
    println!("    • Confidence score: {:.2}", result.confidence);

    if let Some(barcode) = result.metadata.get("barcode") {
        println!("    • Generated barcode: {}", barcode);

        // Create sample creation event
        let sample_event = LabEvent::SampleCreated {
            sample_id: barcode.clone(),
            sample_type: "Blood".to_string(),
            patient_id: "P12345".to_string(),
            created_at: chrono::Utc::now(),
        };

        // Publish event with source component
        event_system
            .publish_event(sample_event, "sample_processor")
            .await?;
        println!("    • Sample creation event published");

        // Simulate sample state transitions
        let state_transitions = vec![
            ("Pending", "Validated"),
            ("Validated", "InStorage"),
            ("InStorage", "InSequencing"),
        ];

        for (from_state, to_state) in state_transitions {
            let transition_event = LabEvent::SampleStateChanged {
                sample_id: barcode.clone(),
                from_state: from_state.to_string(),
                to_state: to_state.to_string(),
                changed_at: chrono::Utc::now(),
                changed_by: "system".to_string(),
            };

            event_system
                .publish_event(transition_event, "workflow_manager")
                .await?;
            println!("    • State transition: {} → {}", from_state, to_state);

            // Small delay to simulate real-world timing
            sleep(Duration::from_millis(100)).await;
        }
    }

    // Template processing demonstration
    println!("  📊 Processing CSV template...");
    let csv_template = create_csv_template();
    let template_result = template_processor
        .process_template(&csv_template, "samples.csv")
        .await?;

    println!("    • Template stage: {:?}", template_result.stage);
    println!(
        "    • Rows extracted: {}",
        template_result.processing_stats.rows_processed
    );
    println!(
        "    • Validation errors: {}",
        template_result.validation_errors.len()
    );

    // Final system status
    let final_health = registry.health_check_all().await?;
    println!(
        "  📈 Final workflow status: {} components operational",
        final_health.len()
    );

    // Cleanup
    sample_processor.shutdown().await?;
    event_system.shutdown().await?;
    template_processor.shutdown().await?;
    registry.shutdown_all().await?;

    println!("  ✅ Complete workflow demonstration finished");
    println!();

    Ok(())
}

/// Demo 3: Template processing pipeline
async fn demo_template_processing() -> Result<(), ComponentError> {
    println!("📋 DEMO 3: Template Processing");
    println!("------------------------------");

    let mut template_processor = TemplateProcessingBuilder::new()
        .with_formats(vec!["csv".to_string(), "json".to_string()])
        .build();

    let registry = ServiceRegistry::new();
    template_processor.initialize(&registry).await?;

    // Test CSV processing
    let csv_data = create_csv_template();
    let csv_result = template_processor
        .process_template(&csv_data, "samples.csv")
        .await?;

    println!(
        "  📊 CSV processing: stage={:?}, rows={}",
        csv_result.stage, csv_result.processing_stats.rows_processed
    );

    // Test JSON processing
    let json_data = create_json_template();
    let json_result = template_processor
        .process_template(&json_data, "data.json")
        .await?;

    println!(
        "  📊 JSON processing: stage={:?}, rows={}",
        json_result.stage, json_result.processing_stats.rows_processed
    );

    println!("  📈 Templates processed successfully",);

    template_processor.shutdown().await?;
    println!("  ✅ Template processing demo completed");
    println!();

    Ok(())
}

/// Demo 4: Event-driven system integration
async fn demo_event_system() -> Result<(), ComponentError> {
    println!("📡 DEMO 4: Event System");
    println!("-----------------------");

    let mut event_system = EventSystemBuilder::new().with_history_size(100).build();

    let registry = ServiceRegistry::new();
    event_system.initialize(&registry).await?;

    // Publish various events
    let events = vec![
        LabEvent::SampleCreated {
            sample_id: "S001".to_string(),
            sample_type: "Blood".to_string(),
            patient_id: "P001".to_string(),
            created_at: chrono::Utc::now(),
        },
        LabEvent::SampleStored {
            sample_id: "S001".to_string(),
            storage_location: "Freezer_A".to_string(),
            temperature_zone: "-20C".to_string(),
            stored_at: chrono::Utc::now(),
        },
    ];

    for event in events {
        let event_id = event_system.publish_event(event, "demo_system").await?;
        println!("  📤 Event published: {}", event_id);
    }

    let metrics = event_system.get_metrics();
    println!("  📊 Total events: {}", metrics.total_events_processed);

    event_system.shutdown().await?;
    println!("  ✅ Event system demo completed");
    println!();

    Ok(())
}

/// Demo 5: System monitoring and observability
async fn demo_monitoring_system() -> Result<(), ComponentError> {
    println!("📊 DEMO 5: System Monitoring");
    println!("----------------------------");

    let mut monitoring = MonitoringBuilder::new()
        .with_health_check_interval(5)
        .for_development()
        .build();

    let registry = ServiceRegistry::new();
    monitoring.initialize(&registry).await?;

    // Collect metrics
    for i in 1..=3 {
        let metrics = monitoring.collect_metrics().await?;
        println!(
            "  📈 Metrics {}: CPU {:.1}%, Memory {:.1}%",
            i, metrics.cpu_usage, metrics.memory_usage
        );
        sleep(Duration::from_millis(200)).await;
    }

    let report = monitoring.generate_report();
    println!(
        "  📋 Report: uptime={:?}, metrics={}",
        report.uptime, report.metrics_collected
    );

    monitoring.shutdown().await?;
    println!("  ✅ Monitoring demo completed");
    println!();

    Ok(())
}

/// Demo 6: Performance and scalability testing
async fn demo_performance_scalability() -> Result<(), ComponentError> {
    println!("🚀 Demonstrating performance and scalability...");

    // Test 1: Rapid component initialization
    println!("  ⚡ Test 1: Rapid component initialization");
    let start_time = std::time::Instant::now();

    let mut registries = Vec::new();
    for i in 0..10 {
        let registry = StudioLine::developer_setup().await?;
        registries.push(registry);

        if (i + 1) % 3 == 0 {
            println!("    • Initialized {} systems", i + 1);
        }
    }

    let init_duration = start_time.elapsed();
    println!("    • Total initialization time: {:?}", init_duration);
    println!("    • Average per system: {:?}", init_duration / 10);

    // Shutdown all systems
    for mut registry in registries {
        registry.shutdown_all().await?;
    }

    // Test 2: High-throughput sample processing
    println!("  📊 Test 2: High-throughput processing");

    let mut processor = SampleProcessingBuilder::new()
        .with_rag(false) // Disable RAG for speed
        .with_confidence_threshold(0.7)
        .build();

    let registry = ServiceRegistry::new();
    processor.initialize(&registry).await?;

    let processing_start = std::time::Instant::now();
    let sample_count = 100;

    for i in 0..sample_count {
        let document = create_sample_document(i);
        let filename = format!("sample_{}.txt", i);

        let result = processor.process_document(&document, &filename).await?;
        assert_eq!(result.stage, ProcessingStage::ValidationComplete);

        if (i + 1) % 25 == 0 {
            println!("    • Processed {} samples", i + 1);
        }
    }

    let processing_duration = processing_start.elapsed();
    let samples_per_second = sample_count as f64 / processing_duration.as_secs_f64();

    println!("    • Processing time: {:?}", processing_duration);
    println!("    • Throughput: {:.1} samples/second", samples_per_second);

    let stats = processor.get_stats();
    println!(
        "    • Total samples processed: {}",
        stats.documents_processed
    );

    processor.shutdown().await?;

    // Test 3: Event system performance
    println!("  📡 Test 3: Event system performance");

    let mut event_system = EventSystemBuilder::new().with_history_size(1000).build();

    event_system.initialize(&registry).await?;

    let event_start = std::time::Instant::now();
    let event_count = 500;

    for i in 0..event_count {
        let event = LabEvent::SampleCreated {
            sample_id: format!("PERF_S{:03}", i),
            sample_type: "Blood".to_string(),
            patient_id: format!("PERF_P{:03}", i),
            created_at: chrono::Utc::now(),
        };

        event_system
            .publish_event(event, "performance_test")
            .await?;

        if (i + 1) % 100 == 0 {
            println!("    • Published {} events", i + 1);
        }
    }

    let event_duration = event_start.elapsed();
    let events_per_second = event_count as f64 / event_duration.as_secs_f64();

    println!("    • Event publishing time: {:?}", event_duration);
    println!(
        "    • Event throughput: {:.1} events/second",
        events_per_second
    );

    event_system.shutdown().await?;

    println!("  🏆 Performance summary:");
    println!(
        "    • System initialization: {:.1}ms per system",
        init_duration.as_millis() as f64 / 10.0
    );
    println!(
        "    • Sample processing: {:.1} samples/sec",
        samples_per_second
    );
    println!(
        "    • Event publishing: {:.1} events/sec",
        events_per_second
    );

    println!("  ✅ Performance and scalability demo completed");
    println!();

    Ok(())
}

// Helper functions for creating test data

fn create_sample_submission() -> Vec<u8> {
    r#"Laboratory Sample Submission
Sample Type: Blood
Patient ID: P12345
Collection Date: 2024-01-15
Volume: 5ml
Temperature: 4°C"#
        .as_bytes()
        .to_vec()
}

fn create_csv_template() -> Vec<u8> {
    r#"Sample_ID,Sample_Type,Patient_ID
S001,Blood,P001
S002,Urine,P002
S003,Tissue,P003"#
        .as_bytes()
        .to_vec()
}

fn create_json_template() -> Vec<u8> {
    r#"[
  {"sample_id": "J001", "sample_type": "Blood", "patient_id": "P_JSON_001"},
  {"sample_id": "J002", "sample_type": "Plasma", "patient_id": "P_JSON_002"}
]"#
    .as_bytes()
    .to_vec()
}

fn create_sample_document(index: usize) -> Vec<u8> {
    format!(
        "Sample Submission {}\n\
        Sample ID: PERF_S{:03}\n\
        Patient ID: PERF_P{:03}\n\
        Sample Type: Blood\n\
        Collection Date: 2024-01-15\n\
        Processing Priority: Standard",
        index, index, index
    )
    .into_bytes()
}
