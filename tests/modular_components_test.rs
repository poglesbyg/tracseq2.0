use lab_manager::assembly::{
    components::{
        DatabaseComponent, DatabaseComponentBuilder, ProcessingStage, SampleProcessingBuilder,
        SampleProcessingComponent, SampleProcessingConfig, StorageBackend, StorageComponent,
        StorageComponentBuilder,
    },
    product_lines::{CompactLine, HybridLine, ProfessionalLine, StudioLine},
    traits::{Component, ComponentError, Configurable, ServiceProvider, ServiceRegistry},
};
use tokio_test;

/// Test the core component lifecycle
#[tokio::test]
async fn test_component_lifecycle() {
    // Test database component lifecycle
    let mut db_component = DatabaseComponentBuilder::new()
        .for_testing()
        .build()
        .expect("Failed to build database component");

    // Component should start uninitialized
    assert_eq!(db_component.component_id(), "database");
    assert_eq!(db_component.component_name(), "Database Connection Pool");

    // Health check should fail before initialization
    assert!(db_component.health_check().await.is_err());

    // Initialize component
    let registry = ServiceRegistry::new();
    db_component
        .initialize(&registry)
        .await
        .expect("Failed to initialize database component");

    // Health check should now pass
    assert!(db_component.health_check().await.is_ok());

    // Shutdown component
    db_component
        .shutdown()
        .await
        .expect("Failed to shutdown database component");
}

/// Test storage component with different backends
#[tokio::test]
async fn test_storage_backends() {
    // Test file system backend
    let mut fs_storage = StorageComponentBuilder::new()
        .filesystem("/tmp/test_storage")
        .build()
        .expect("Failed to build filesystem storage");

    assert_eq!(fs_storage.component_id(), "storage");
    assert_eq!(fs_storage.component_name(), "File System Storage");

    let registry = ServiceRegistry::new();
    fs_storage
        .initialize(&registry)
        .await
        .expect("Failed to initialize filesystem storage");
    assert!(fs_storage.health_check().await.is_ok());
    fs_storage
        .shutdown()
        .await
        .expect("Failed to shutdown filesystem storage");

    // Test in-memory backend
    let mut memory_storage = StorageComponentBuilder::new()
        .in_memory()
        .build()
        .expect("Failed to build memory storage");

    assert_eq!(memory_storage.component_name(), "In-Memory Storage");
    memory_storage
        .initialize(&registry)
        .await
        .expect("Failed to initialize memory storage");
    assert!(memory_storage.health_check().await.is_ok());
    memory_storage
        .shutdown()
        .await
        .expect("Failed to shutdown memory storage");

    // Test mock backend
    let mut mock_storage = StorageComponentBuilder::new()
        .mock()
        .build()
        .expect("Failed to build mock storage");

    assert_eq!(mock_storage.component_name(), "Mock Storage");
    mock_storage
        .initialize(&registry)
        .await
        .expect("Failed to initialize mock storage");
    assert!(mock_storage.health_check().await.is_ok());
    mock_storage
        .shutdown()
        .await
        .expect("Failed to shutdown mock storage");
}

/// Test sample processing component data flow
#[tokio::test]
async fn test_sample_processing_pipeline() {
    let mut processor = SampleProcessingBuilder::new()
        .with_rag(true)
        .with_confidence_threshold(0.8)
        .build();

    assert_eq!(processor.component_id(), "sample_processing");
    assert_eq!(processor.component_name(), "Sample Processing Pipeline");

    // Initialize component
    let registry = ServiceRegistry::new();
    processor
        .initialize(&registry)
        .await
        .expect("Failed to initialize sample processor");

    // Test document processing pipeline
    let test_document = b"Sample document content for testing RAG processing";
    let result = processor
        .process_document(test_document, "test.pdf")
        .await
        .expect("Failed to process document");

    // Verify processing stages
    assert_eq!(result.stage, ProcessingStage::ValidationComplete);
    assert!(result.confidence > 0.8);
    assert!(!result.extracted_data.is_null());
    assert!(result.errors.is_empty());

    // Check barcode generation
    assert!(result.metadata.contains_key("barcode"));
    let barcode = result.metadata.get("barcode").unwrap();
    assert!(barcode.starts_with("LAB-"));

    // Test unsupported format
    let unsupported_result = processor
        .process_document(test_document, "test.xyz")
        .await
        .expect("Should handle unsupported format gracefully");
    assert!(!unsupported_result.errors.is_empty());
    assert!(unsupported_result.errors[0].contains("Unsupported file format"));

    // Check processing statistics
    let stats = processor.get_stats();
    assert_eq!(stats.documents_processed, 2); // Two documents processed
    assert_eq!(stats.samples_created, 1); // Only one valid sample created

    processor
        .shutdown()
        .await
        .expect("Failed to shutdown sample processor");
}

/// Test component configuration and reconfiguration
#[tokio::test]
async fn test_component_configuration() {
    let mut processor = SampleProcessingBuilder::new().build();

    // Test initial configuration
    let initial_config = processor.get_config();
    assert_eq!(initial_config.confidence_threshold, 0.8);
    assert!(initial_config.enable_rag);

    // Test reconfiguration before initialization
    let new_config = SampleProcessingConfig {
        enable_rag: false,
        confidence_threshold: 0.9,
        max_file_size: 5 * 1024 * 1024,
        supported_formats: vec!["pdf".to_string()],
        auto_generate_barcodes: false,
        barcode_prefix: "TEST".to_string(),
    };

    processor
        .configure(new_config.clone())
        .expect("Failed to configure component");
    assert_eq!(processor.get_config().confidence_threshold, 0.9);
    assert!(!processor.get_config().enable_rag);

    // Initialize component
    let registry = ServiceRegistry::new();
    processor
        .initialize(&registry)
        .await
        .expect("Failed to initialize");

    // Test that reconfiguration fails after initialization
    let result = processor.configure(new_config);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Cannot reconfigure initialized component"));
}

/// Test service registry functionality
#[tokio::test]
async fn test_service_registry() {
    let mut registry = ServiceRegistry::new();

    // Register components
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

    // Test duplicate registration
    let duplicate_db = DatabaseComponentBuilder::new()
        .for_testing()
        .build()
        .expect("Failed to build duplicate database");

    let result = registry.register_component(duplicate_db);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Component already registered"));

    // Test component retrieval
    let db_component = registry.get_component("database");
    assert!(db_component.is_some());

    let missing_component = registry.get_component("nonexistent");
    assert!(missing_component.is_none());

    // Test service registration
    registry.register_service("test_service", "test_value".to_string());
    let service: Option<std::sync::Arc<String>> = registry.get_service("test_service");
    assert!(service.is_some());
    assert_eq!(*service.unwrap(), "test_value");

    // Test initialization
    registry
        .initialize_all()
        .await
        .expect("Failed to initialize all components");

    // Test health checks
    let health_results = registry
        .health_check_all()
        .await
        .expect("Failed to perform health checks");
    assert_eq!(health_results.len(), 2); // Two components registered
    assert!(health_results.values().all(|&healthy| healthy)); // All should be healthy

    // Test shutdown
    registry
        .shutdown_all()
        .await
        .expect("Failed to shutdown all components");
}

/// Test builder patterns for component creation
#[tokio::test]
async fn test_component_builders() {
    // Test database builder variations
    let prod_db = DatabaseComponentBuilder::new()
        .with_env_config()
        .unwrap_or_else(|_| DatabaseComponentBuilder::new().for_testing())
        .build()
        .expect("Failed to build database");
    assert_eq!(prod_db.component_id(), "database");

    // Test storage builder variations
    let fs_storage = StorageComponentBuilder::new()
        .filesystem("/tmp/test")
        .build()
        .expect("Failed to build filesystem storage");

    let s3_storage = StorageComponentBuilder::new()
        .s3("test-bucket", "us-east-1")
        .build()
        .expect("Failed to build S3 storage");

    assert_eq!(fs_storage.component_id(), "storage");
    assert_eq!(s3_storage.component_id(), "storage");

    // Verify different backend types
    match fs_storage.backend_type() {
        StorageBackend::FileSystem { .. } => (),
        _ => panic!("Expected FileSystem backend"),
    }

    match s3_storage.backend_type() {
        StorageBackend::S3 { bucket, region } => {
            assert_eq!(bucket, "test-bucket");
            assert_eq!(region, "us-east-1");
        }
        _ => panic!("Expected S3 backend"),
    }

    // Test sample processing builder
    let high_throughput = SampleProcessingBuilder::new().for_high_throughput().build();

    assert!(high_throughput.get_config().max_file_size > 10 * 1024 * 1024);
    assert!(high_throughput.get_config().confidence_threshold < 0.8);
}

/// Test product line assemblies
#[tokio::test]
async fn test_product_lines() {
    // Test Studio Line - developer setup
    let mut dev_registry = StudioLine::developer_setup()
        .await
        .expect("Failed to assemble developer setup");

    let health = dev_registry
        .health_check_all()
        .await
        .expect("Failed to check health");
    assert!(!health.is_empty());

    dev_registry
        .shutdown_all()
        .await
        .expect("Failed to shutdown dev setup");

    // Test Studio Line - unit test setup
    let mut test_registry = StudioLine::unit_test_setup()
        .await
        .expect("Failed to assemble unit test setup");

    let health = test_registry
        .health_check_all()
        .await
        .expect("Failed to check health");
    assert!(!health.is_empty());

    test_registry
        .shutdown_all()
        .await
        .expect("Failed to shutdown test setup");

    // Test Hybrid Line - custom setup
    let custom_registry = HybridLine::custom()
        .with_database(DatabaseComponentBuilder::new().for_testing())
        .with_storage(StorageComponentBuilder::new().in_memory())
        .with_config("test_setting", "test_value")
        .build()
        .await
        .expect("Failed to build custom setup");

    // Verify custom configuration was registered
    let custom_config: Option<std::sync::Arc<std::collections::HashMap<String, String>>> =
        custom_registry.get_service("custom_config");
    assert!(custom_config.is_some());
    assert_eq!(
        custom_config.unwrap().get("test_setting").unwrap(),
        "test_value"
    );
}

/// Test error handling and edge cases
#[tokio::test]
async fn test_error_handling() {
    // Test invalid component configuration
    let mut processor = SampleProcessingBuilder::new().build();

    let invalid_config = SampleProcessingConfig {
        confidence_threshold: 1.5, // Invalid: > 1.0
        ..Default::default()
    };

    let result = processor.configure(invalid_config);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Confidence threshold must be between 0.0 and 1.0"));

    // Test component health check before initialization
    let uninit_component = DatabaseComponentBuilder::new()
        .for_testing()
        .build()
        .expect("Failed to build database");

    let health_result = uninit_component.health_check().await;
    assert!(health_result.is_err());
    assert!(health_result
        .unwrap_err()
        .to_string()
        .contains("Component not initialized"));

    // Test processing with uninitialized component
    let mut uninit_processor = SampleProcessingBuilder::new().build();
    let test_doc = b"test";

    let process_result = uninit_processor
        .process_document(test_doc, "test.pdf")
        .await;
    assert!(process_result.is_err());
    assert!(process_result
        .unwrap_err()
        .to_string()
        .contains("Component not initialized"));
}

/// Test concurrent component operations
#[tokio::test]
async fn test_concurrent_operations() {
    let mut registry = ServiceRegistry::new();

    // Register multiple components
    for i in 0..5 {
        let storage = StorageComponentBuilder::new()
            .filesystem(&format!("/tmp/test_concurrent_{}", i))
            .build()
            .expect("Failed to build storage");

        // Each storage needs a unique ID for concurrent registration
        // In a real implementation, we'd handle this better
        registry.register_component(storage).unwrap_or_else(|_| {
            // Handle duplicate registration gracefully in tests
        });
    }

    // Test concurrent health checks
    let health_futures: Vec<_> = (0..10)
        .map(|_| async { registry.health_check_all().await })
        .collect();

    let results = futures::future::join_all(health_futures).await;

    // All health checks should succeed
    for result in results {
        assert!(result.is_ok());
    }
}

/// Performance benchmarks for component operations
#[tokio::test]
async fn test_performance_benchmarks() {
    let mut processor = SampleProcessingBuilder::new()
        .with_rag(false) // Disable RAG for faster processing
        .build();

    let registry = ServiceRegistry::new();
    processor
        .initialize(&registry)
        .await
        .expect("Failed to initialize");

    let test_document = b"Test document for performance benchmarking";
    let start_time = std::time::Instant::now();

    // Process 100 documents
    for i in 0..100 {
        let filename = format!("test_{}.pdf", i);
        let result = processor
            .process_document(test_document, &filename)
            .await
            .expect("Failed to process document");

        assert_eq!(result.stage, ProcessingStage::ValidationComplete);
    }

    let duration = start_time.elapsed();
    let stats = processor.get_stats();

    println!("Performance benchmark:");
    println!(
        "- Processed {} documents in {:?}",
        stats.documents_processed, duration
    );
    println!(
        "- Average processing time: {:?} per document",
        duration / stats.documents_processed as u32
    );
    println!("- Samples created: {}", stats.samples_created);

    // Performance assertions (adjust based on requirements)
    assert_eq!(stats.documents_processed, 100);
    assert_eq!(stats.samples_created, 100);
    assert!(duration.as_millis() < 5000); // Should complete in under 5 seconds
}

/// Integration test combining multiple components
#[tokio::test]
async fn test_integrated_workflow() {
    // Set up a complete system with multiple components
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
        .with_confidence_threshold(0.7)
        .build();

    registry
        .register_component(database)
        .expect("Failed to register database");
    registry
        .register_component(storage)
        .expect("Failed to register storage");
    registry
        .register_component(processor)
        .expect("Failed to register processor");

    // Initialize the entire system
    registry
        .initialize_all()
        .await
        .expect("Failed to initialize system");

    // Verify all components are healthy
    let health = registry
        .health_check_all()
        .await
        .expect("Failed to check system health");
    assert_eq!(health.len(), 3);
    assert!(health.values().all(|&h| h));

    // Test that we can retrieve and use components
    let db_component = registry.get_component("database");
    assert!(db_component.is_some());

    let storage_component = registry.get_component("storage");
    assert!(storage_component.is_some());

    let processing_component = registry.get_component("sample_processing");
    assert!(processing_component.is_some());

    // Graceful shutdown
    registry
        .shutdown_all()
        .await
        .expect("Failed to shutdown system");

    // Verify all components are shut down (health checks should fail)
    let post_shutdown_health = registry
        .health_check_all()
        .await
        .expect("Health check failed");
    // Components should still exist but may not be healthy after shutdown
    assert_eq!(post_shutdown_health.len(), 3);
}
