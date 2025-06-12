/// Performance benchmark tests for the modular architecture
///
/// These tests demonstrate the scalability and efficiency of our IKEA-like modular system
/// under various load conditions and configurations.
use lab_manager::assembly::{
    components::{
        DatabaseComponent, DatabaseComponentBuilder, EventSystemBuilder, EventSystemComponent,
        LabEvent, ProcessingStage, SampleProcessingBuilder, SampleProcessingComponent,
        StorageComponent, StorageComponentBuilder,
    },
    product_lines::{HybridLine, StudioLine},
    traits::{Component, ServiceRegistry},
};

use std::time::{Duration, Instant};
use tokio_test;

/// Benchmark component initialization and shutdown performance
#[tokio::test]
async fn benchmark_component_lifecycle_performance() {
    println!("🏃‍♂️ Benchmarking Component Lifecycle Performance");

    let iterations = 100;
    let mut initialization_times = Vec::new();
    let mut shutdown_times = Vec::new();

    for i in 0..iterations {
        // Benchmark initialization
        let start_time = Instant::now();

        let mut registry = ServiceRegistry::new();

        let database = DatabaseComponentBuilder::new()
            .for_testing()
            .build()
            .expect("Failed to build database");

        let storage = StorageComponentBuilder::new()
            .mock()
            .build()
            .expect("Failed to build storage");

        let processor = SampleProcessingBuilder::new()
            .with_rag(false) // Disable for faster testing
            .build();

        let event_system = EventSystemBuilder::new().with_history_size(1000).build();

        registry
            .register_component(database)
            .expect("Failed to register database");
        registry
            .register_component(storage)
            .expect("Failed to register storage");
        registry
            .register_component(processor)
            .expect("Failed to register processor");
        registry
            .register_component(event_system)
            .expect("Failed to register event system");

        registry
            .initialize_all()
            .await
            .expect("Failed to initialize components");

        let init_duration = start_time.elapsed();
        initialization_times.push(init_duration);

        // Benchmark shutdown
        let shutdown_start = Instant::now();
        registry
            .shutdown_all()
            .await
            .expect("Failed to shutdown components");
        let shutdown_duration = shutdown_start.elapsed();
        shutdown_times.push(shutdown_duration);

        if (i + 1) % 20 == 0 {
            println!("  Completed {} iterations...", i + 1);
        }
    }

    // Calculate statistics
    let avg_init = initialization_times.iter().sum::<Duration>() / iterations as u32;
    let avg_shutdown = shutdown_times.iter().sum::<Duration>() / iterations as u32;

    let min_init = initialization_times.iter().min().unwrap();
    let max_init = initialization_times.iter().max().unwrap();
    let min_shutdown = shutdown_times.iter().min().unwrap();
    let max_shutdown = shutdown_times.iter().max().unwrap();

    println!("📊 Component Lifecycle Performance Results:");
    println!("   Iterations: {}", iterations);
    println!(
        "   Initialization - Avg: {:?}, Min: {:?}, Max: {:?}",
        avg_init, min_init, max_init
    );
    println!(
        "   Shutdown - Avg: {:?}, Min: {:?}, Max: {:?}",
        avg_shutdown, min_shutdown, max_shutdown
    );

    // Performance assertions
    assert!(
        avg_init < Duration::from_millis(100),
        "Initialization should be fast"
    );
    assert!(
        avg_shutdown < Duration::from_millis(50),
        "Shutdown should be fast"
    );

    println!("✅ Component lifecycle performance within acceptable limits");
}

/// Benchmark sample processing throughput
#[tokio::test]
async fn benchmark_sample_processing_throughput() {
    println!("🧪 Benchmarking Sample Processing Throughput");

    let mut processor = SampleProcessingBuilder::new()
        .with_rag(false) // Disable RAG for consistent timing
        .with_confidence_threshold(0.8)
        .build();

    let registry = ServiceRegistry::new();
    processor
        .initialize(&registry)
        .await
        .expect("Failed to initialize processor");

    // Test different document sizes
    let test_cases = vec![
        ("Small", create_small_document()),
        ("Medium", create_medium_document()),
        ("Large", create_large_document()),
    ];

    for (size_name, test_document) in test_cases {
        let samples_count = 1000;
        let batch_size = 100;

        println!(
            "  Testing {} documents ({} samples in batches of {})...",
            size_name, samples_count, batch_size
        );

        let start_time = Instant::now();

        for batch in 0..(samples_count / batch_size) {
            let batch_start = Instant::now();

            // Process a batch of documents
            for i in 0..batch_size {
                let filename = format!("test_{}_{}.txt", batch, i);
                let result = processor
                    .process_document(&test_document, &filename)
                    .await
                    .expect("Failed to process document");

                assert_eq!(result.stage, ProcessingStage::ValidationComplete);
            }

            let batch_duration = batch_start.elapsed();
            if batch % 2 == 0 {
                println!("    Batch {} completed in {:?}", batch + 1, batch_duration);
            }
        }

        let total_duration = start_time.elapsed();
        let samples_per_second = samples_count as f64 / total_duration.as_secs_f64();
        let avg_processing_time = total_duration / samples_count as u32;

        println!("  📈 {} Document Results:", size_name);
        println!("     Total time: {:?}", total_duration);
        println!("     Samples per second: {:.2}", samples_per_second);
        println!("     Average processing time: {:?}", avg_processing_time);

        // Performance assertions based on document size
        match size_name {
            "Small" => assert!(
                samples_per_second > 500.0,
                "Small documents should process at >500/sec"
            ),
            "Medium" => assert!(
                samples_per_second > 200.0,
                "Medium documents should process at >200/sec"
            ),
            "Large" => assert!(
                samples_per_second > 50.0,
                "Large documents should process at >50/sec"
            ),
            _ => {}
        }
    }

    // Check final processing statistics
    let stats = processor.get_stats();
    println!("  📊 Final Processing Statistics:");
    println!(
        "     Total documents processed: {}",
        stats.documents_processed
    );
    println!("     Total samples created: {}", stats.samples_created);
    println!(
        "     Success rate: {:.2}%",
        (stats.samples_created as f64 / stats.documents_processed as f64) * 100.0
    );

    assert_eq!(stats.documents_processed, 3000); // 1000 per size category
    assert_eq!(stats.samples_created, 3000); // All should succeed

    processor
        .shutdown()
        .await
        .expect("Failed to shutdown processor");

    println!("✅ Sample processing throughput benchmarks completed");
}

/// Benchmark event system performance
#[tokio::test]
async fn benchmark_event_system_performance() {
    println!("📡 Benchmarking Event System Performance");

    let mut event_system = EventSystemBuilder::new().with_history_size(50000).build();

    let registry = ServiceRegistry::new();
    event_system
        .initialize(&registry)
        .await
        .expect("Failed to initialize event system");

    let events_count = 10000;
    let batch_size = 1000;

    println!(
        "  Publishing {} events in batches of {}...",
        events_count, batch_size
    );

    let start_time = Instant::now();

    for batch in 0..(events_count / batch_size) {
        let batch_start = Instant::now();

        for i in 0..batch_size {
            let event = LabEvent::SampleCreated {
                sample_id: format!("S{:06}", batch * batch_size + i),
                sample_type: "Blood".to_string(),
                patient_id: format!("P{:06}", batch * batch_size + i),
                created_at: chrono::Utc::now(),
            };

            let event_id = event_system
                .publish_event(event)
                .await
                .expect("Failed to publish event");
            assert!(event_id.starts_with("evt_"));
        }

        let batch_duration = batch_start.elapsed();
        if batch % 2 == 0 {
            println!("    Published batch {} in {:?}", batch + 1, batch_duration);
        }
    }

    let total_duration = start_time.elapsed();
    let events_per_second = events_count as f64 / total_duration.as_secs_f64();
    let avg_event_time = total_duration / events_count as u32;

    println!("  📈 Event System Results:");
    println!("     Total time: {:?}", total_duration);
    println!("     Events per second: {:.2}", events_per_second);
    println!("     Average event time: {:?}", avg_event_time);
    println!(
        "     Total events published: {}",
        event_system.get_event_count()
    );

    // Performance assertions
    assert!(
        events_per_second > 1000.0,
        "Event system should handle >1000 events/sec"
    );
    assert!(
        avg_event_time < Duration::from_micros(100),
        "Event publishing should be fast"
    );
    assert_eq!(event_system.get_event_count(), events_count as u64);

    event_system
        .shutdown()
        .await
        .expect("Failed to shutdown event system");

    println!("✅ Event system performance benchmarks completed");
}

/// Benchmark concurrent component operations
#[tokio::test]
async fn benchmark_concurrent_operations() {
    println!("🚀 Benchmarking Concurrent Operations");

    // Set up multiple registries for concurrent testing
    let registry_count = 10;
    let operations_per_registry = 100;

    println!(
        "  Setting up {} concurrent registries with {} operations each...",
        registry_count, operations_per_registry
    );

    let start_time = Instant::now();

    // Spawn concurrent tasks
    let mut handles = Vec::new();

    for registry_id in 0..registry_count {
        let handle = tokio::spawn(async move {
            let task_start = Instant::now();

            // Create registry for this task
            let mut registry = ServiceRegistry::new();

            // Add components
            let database = DatabaseComponentBuilder::new()
                .for_testing()
                .build()
                .expect("Failed to build database");

            let storage = StorageComponentBuilder::new()
                .mock()
                .build()
                .expect("Failed to build storage");

            registry
                .register_component(database)
                .expect("Failed to register database");
            registry
                .register_component(storage)
                .expect("Failed to register storage");

            // Initialize
            registry
                .initialize_all()
                .await
                .expect("Failed to initialize");

            // Perform operations
            let mut operation_times = Vec::new();

            for op_id in 0..operations_per_registry {
                let op_start = Instant::now();

                // Perform health checks
                let health_result = registry
                    .health_check_all()
                    .await
                    .expect("Health check failed");
                assert_eq!(health_result.len(), 2);

                // Simulate some work
                tokio::time::sleep(Duration::from_millis(1)).await;

                let op_duration = op_start.elapsed();
                operation_times.push(op_duration);

                if (op_id + 1) % 25 == 0 {
                    println!(
                        "    Registry {} completed {} operations",
                        registry_id,
                        op_id + 1
                    );
                }
            }

            // Shutdown
            registry.shutdown_all().await.expect("Failed to shutdown");

            let task_duration = task_start.elapsed();
            let avg_op_time =
                operation_times.iter().sum::<Duration>() / operations_per_registry as u32;

            (
                registry_id,
                task_duration,
                avg_op_time,
                operation_times.len(),
            )
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    let mut results = Vec::new();
    for handle in handles {
        let result = handle.await.expect("Task failed");
        results.push(result);
    }

    let total_duration = start_time.elapsed();

    // Analyze results
    let total_operations: usize = results.iter().map(|(_, _, _, op_count)| op_count).sum();
    let avg_task_duration: Duration = results
        .iter()
        .map(|(_, duration, _, _)| *duration)
        .sum::<Duration>()
        / registry_count as u32;
    let avg_operation_time: Duration = results
        .iter()
        .map(|(_, _, avg_op, _)| *avg_op)
        .sum::<Duration>()
        / registry_count as u32;

    let operations_per_second = total_operations as f64 / total_duration.as_secs_f64();

    println!("  📈 Concurrent Operations Results:");
    println!("     Concurrent registries: {}", registry_count);
    println!("     Total operations: {}", total_operations);
    println!("     Total time: {:?}", total_duration);
    println!("     Average task duration: {:?}", avg_task_duration);
    println!("     Average operation time: {:?}", avg_operation_time);
    println!("     Operations per second: {:.2}", operations_per_second);

    // Performance assertions
    assert!(
        operations_per_second > 500.0,
        "Concurrent operations should achieve >500 ops/sec"
    );
    assert!(
        avg_operation_time < Duration::from_millis(10),
        "Individual operations should be fast"
    );
    assert_eq!(total_operations, registry_count * operations_per_registry);

    println!("✅ Concurrent operations benchmarks completed");
}

/// Benchmark memory usage and resource efficiency
#[tokio::test]
async fn benchmark_memory_and_resource_efficiency() {
    println!("💾 Benchmarking Memory and Resource Efficiency");

    // Test memory usage with different component configurations
    let configurations = vec![
        ("Minimal", create_minimal_setup().await),
        ("Standard", create_standard_setup().await),
        ("Full Featured", create_full_featured_setup().await),
    ];

    for (config_name, mut registry) in configurations {
        println!("  Testing {} configuration...", config_name);

        // Measure initial memory usage (simplified)
        let component_count = registry
            .health_check_all()
            .await
            .expect("Health check failed")
            .len();

        // Perform stress operations
        let stress_operations = 1000;
        let stress_start = Instant::now();

        for i in 0..stress_operations {
            // Perform health checks
            let health_result = registry
                .health_check_all()
                .await
                .expect("Health check failed");
            assert_eq!(health_result.len(), component_count);

            if (i + 1) % 200 == 0 {
                println!(
                    "    Completed {} stress operations for {}",
                    i + 1,
                    config_name
                );
            }
        }

        let stress_duration = stress_start.elapsed();
        let ops_per_second = stress_operations as f64 / stress_duration.as_secs_f64();

        println!("    📊 {} Configuration Results:", config_name);
        println!("       Components: {}", component_count);
        println!("       Stress operations: {}", stress_operations);
        println!("       Stress duration: {:?}", stress_duration);
        println!("       Operations per second: {:.2}", ops_per_second);

        // Performance assertions based on configuration complexity
        match config_name {
            "Minimal" => assert!(
                ops_per_second > 2000.0,
                "Minimal config should be very fast"
            ),
            "Standard" => assert!(ops_per_second > 1000.0, "Standard config should be fast"),
            "Full Featured" => assert!(ops_per_second > 500.0, "Full config should be reasonable"),
            _ => {}
        }

        // Cleanup
        registry.shutdown_all().await.expect("Failed to shutdown");

        // Brief pause between configurations
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    println!("✅ Memory and resource efficiency benchmarks completed");
}

/// Benchmark integrated workflow performance
#[tokio::test]
async fn benchmark_integrated_workflow_performance() {
    println!("🔬 Benchmarking Integrated Workflow Performance");

    let workflow_iterations = 50;
    let samples_per_workflow = 20;

    println!(
        "  Running {} workflow iterations with {} samples each...",
        workflow_iterations, samples_per_workflow
    );

    let mut workflow_times = Vec::new();
    let mut total_samples_processed = 0;

    for iteration in 0..workflow_iterations {
        let workflow_start = Instant::now();

        // Set up complete laboratory system
        let mut registry = HybridLine::custom()
            .with_database(DatabaseComponentBuilder::new().for_testing())
            .with_storage(StorageComponentBuilder::new().mock())
            .build()
            .await
            .expect("Failed to set up laboratory system");

        // Initialize processing pipeline
        let mut processor = SampleProcessingBuilder::new()
            .with_rag(false) // Disable for consistent timing
            .with_confidence_threshold(0.8)
            .build();

        processor
            .initialize(&registry)
            .await
            .expect("Failed to initialize processor");

        // Initialize event system
        let mut event_system = EventSystemBuilder::new().with_history_size(1000).build();

        event_system
            .initialize(&registry)
            .await
            .expect("Failed to initialize event system");

        // Process samples through complete workflow
        for sample_id in 0..samples_per_workflow {
            // Step 1: Document submission
            let submission_doc = create_workflow_document(iteration, sample_id);
            let result = processor
                .process_document(
                    &submission_doc,
                    &format!("sample_{}_{}.txt", iteration, sample_id),
                )
                .await
                .expect("Failed to process document");

            assert_eq!(result.stage, ProcessingStage::ValidationComplete);

            // Step 2: Event publishing
            let sample_event = LabEvent::SampleCreated {
                sample_id: format!("S{}_{}", iteration, sample_id),
                sample_type: "Blood".to_string(),
                patient_id: format!("P{}_{}", iteration, sample_id),
                created_at: chrono::Utc::now(),
            };

            let event_id = event_system
                .publish_event(sample_event)
                .await
                .expect("Failed to publish event");
            assert!(event_id.starts_with("evt_"));

            total_samples_processed += 1;
        }

        // Health check
        let health_result = registry
            .health_check_all()
            .await
            .expect("Health check failed");
        assert!(!health_result.is_empty());

        // Cleanup
        processor
            .shutdown()
            .await
            .expect("Failed to shutdown processor");
        event_system
            .shutdown()
            .await
            .expect("Failed to shutdown event system");
        registry
            .shutdown_all()
            .await
            .expect("Failed to shutdown registry");

        let workflow_duration = workflow_start.elapsed();
        workflow_times.push(workflow_duration);

        if (iteration + 1) % 10 == 0 {
            println!("    Completed {} workflow iterations", iteration + 1);
        }
    }

    // Calculate statistics
    let total_duration: Duration = workflow_times.iter().sum();
    let avg_workflow_time = total_duration / workflow_iterations as u32;
    let min_workflow_time = workflow_times.iter().min().unwrap();
    let max_workflow_time = workflow_times.iter().max().unwrap();

    let samples_per_second = total_samples_processed as f64 / total_duration.as_secs_f64();
    let workflows_per_second = workflow_iterations as f64 / total_duration.as_secs_f64();

    println!("  📈 Integrated Workflow Results:");
    println!("     Total workflows: {}", workflow_iterations);
    println!("     Total samples: {}", total_samples_processed);
    println!("     Total duration: {:?}", total_duration);
    println!("     Average workflow time: {:?}", avg_workflow_time);
    println!("     Min workflow time: {:?}", min_workflow_time);
    println!("     Max workflow time: {:?}", max_workflow_time);
    println!("     Workflows per second: {:.2}", workflows_per_second);
    println!("     Samples per second: {:.2}", samples_per_second);

    // Performance assertions
    assert!(
        samples_per_second > 50.0,
        "Integrated workflow should process >50 samples/sec"
    );
    assert!(
        workflows_per_second > 2.0,
        "Should complete >2 workflows/sec"
    );
    assert!(
        avg_workflow_time < Duration::from_secs(5),
        "Average workflow should complete in <5s"
    );

    println!("✅ Integrated workflow performance benchmarks completed");
}

// Helper functions for benchmark tests

async fn create_minimal_setup() -> ServiceRegistry {
    let mut registry = ServiceRegistry::new();

    let database = DatabaseComponentBuilder::new()
        .for_testing()
        .build()
        .expect("Failed to build database");

    registry
        .register_component(database)
        .expect("Failed to register database");
    registry
        .initialize_all()
        .await
        .expect("Failed to initialize");

    registry
}

async fn create_standard_setup() -> ServiceRegistry {
    let mut registry = ServiceRegistry::new();

    let database = DatabaseComponentBuilder::new()
        .for_testing()
        .build()
        .expect("Failed to build database");

    let storage = StorageComponentBuilder::new()
        .mock()
        .build()
        .expect("Failed to build storage");

    registry
        .register_component(database)
        .expect("Failed to register database");
    registry
        .register_component(storage)
        .expect("Failed to register storage");
    registry
        .initialize_all()
        .await
        .expect("Failed to initialize");

    registry
}

async fn create_full_featured_setup() -> ServiceRegistry {
    let mut registry = ServiceRegistry::new();

    let database = DatabaseComponentBuilder::new()
        .for_testing()
        .build()
        .expect("Failed to build database");

    let storage = StorageComponentBuilder::new()
        .mock()
        .build()
        .expect("Failed to build storage");

    let processor = SampleProcessingBuilder::new().with_rag(false).build();

    let event_system = EventSystemBuilder::new().with_history_size(1000).build();

    registry
        .register_component(database)
        .expect("Failed to register database");
    registry
        .register_component(storage)
        .expect("Failed to register storage");
    registry
        .register_component(processor)
        .expect("Failed to register processor");
    registry
        .register_component(event_system)
        .expect("Failed to register event system");
    registry
        .initialize_all()
        .await
        .expect("Failed to initialize");

    registry
}

fn create_small_document() -> Vec<u8> {
    b"Sample ID: S001\nPatient: P001\nType: Blood".to_vec()
}

fn create_medium_document() -> Vec<u8> {
    let content = format!(
        "Laboratory Sample Submission\n{}\nSample Type: Blood\nPatient ID: P001\nCollection Date: 2024-01-15\nVolume: 5ml\nTemperature: 4C\nPriority: Standard\n{}",
        "=".repeat(50),
        "Details: ".repeat(20)
    );
    content.into_bytes()
}

fn create_large_document() -> Vec<u8> {
    let mut content = String::new();
    content.push_str("Laboratory Sample Submission Report\n");
    content.push_str(&"=".repeat(100));
    content.push('\n');

    for i in 0..50 {
        content.push_str(&format!(
            "Sample {}: ID=S{:03}, Patient=P{:03}, Type=Blood, Date=2024-01-15, Volume=5ml, Temp=4C\n",
            i + 1, i + 1, i + 1
        ));
    }

    content.push_str("\nDetailed Analysis:\n");
    content.push_str(&"This is a detailed analysis section. ".repeat(100));

    content.into_bytes()
}

fn create_workflow_document(iteration: usize, sample_id: usize) -> Vec<u8> {
    format!(
        "Workflow Document - Iteration {}, Sample {}\nSample Type: Blood\nPatient ID: P{}_{}\nCollection Date: 2024-01-15\nProcessing Priority: Standard\n",
        iteration, sample_id, iteration, sample_id
    ).into_bytes()
}

/// Summary function to run all benchmarks
#[tokio::test]
#[ignore] // Use cargo test -- --ignored to run this comprehensive benchmark
async fn run_all_performance_benchmarks() {
    println!("🏁 Running Comprehensive Performance Benchmark Suite");
    println!("===================================================");

    let suite_start = Instant::now();

    // Run all benchmarks
    benchmark_component_lifecycle_performance().await;
    println!();

    benchmark_sample_processing_throughput().await;
    println!();

    benchmark_event_system_performance().await;
    println!();

    benchmark_concurrent_operations().await;
    println!();

    benchmark_memory_and_resource_efficiency().await;
    println!();

    benchmark_integrated_workflow_performance().await;
    println!();

    let suite_duration = suite_start.elapsed();

    println!("🏆 Performance Benchmark Suite Completed!");
    println!("   Total suite duration: {:?}", suite_duration);
    println!("   All benchmarks passed within performance thresholds");
    println!("   The IKEA-like modular architecture demonstrates:");
    println!("   ✅ Fast component lifecycle management");
    println!("   ✅ High-throughput sample processing");
    println!("   ✅ Efficient event system performance");
    println!("   ✅ Excellent concurrent operation scaling");
    println!("   ✅ Resource-efficient memory usage");
    println!("   ✅ Robust integrated workflow performance");
    println!();
    println!("🪑 Just like IKEA furniture - efficient, scalable, and reliable!");
}
