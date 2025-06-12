/// Integration tests for data flow patterns in the modular architecture
///
/// These tests demonstrate the key data flows identified in the system-dataflow context:
/// 1. Sample Submission Pipeline
/// 2. Storage Location Management Flow
/// 3. Template Processing Pipeline
/// 4. Event-Driven Communication
/// 5. Document Processing Flow
/// 6. Sequencing Job Pipeline
use lab_manager::assembly::{
    components::{
        DatabaseComponent, DatabaseComponentBuilder, ProcessingResult, ProcessingStage,
        SampleProcessingBuilder, SampleProcessingComponent, StorageComponent,
        StorageComponentBuilder,
    },
    product_lines::{HybridLine, StudioLine},
    traits::{ComponentError, ServiceRegistry},
};

use serde_json::json;
use std::collections::HashMap;
use tokio_test;

/// Test the complete sample submission pipeline data flow
/// This demonstrates the core data flow: Document upload → RAG processing → Sample creation
#[tokio::test]
async fn test_sample_submission_pipeline_flow() {
    println!("🧪 Testing Sample Submission Pipeline Data Flow");

    // Set up the pipeline components
    let mut registry = StudioLine::developer_setup()
        .await
        .expect("Failed to set up development environment");

    // Create a sample processing component for the pipeline
    let mut processor = SampleProcessingBuilder::new()
        .with_rag(true)
        .with_confidence_threshold(0.8)
        .build();

    processor
        .initialize(&registry)
        .await
        .expect("Failed to initialize processor");

    // Simulate document upload with laboratory submission data
    let lab_submission_doc = r#"
        Laboratory Sample Submission Form
        ================================
        
        Sample Type: Blood
        Patient ID: P12345
        Collection Date: 2024-01-15
        Collection Time: 14:30
        Laboratory: Central Diagnostics Lab
        Volume: 5ml
        Temperature Requirements: 4°C
        Processing Priority: Standard
        
        Clinical Notes:
        - Patient fasting for 12 hours
        - Routine blood chemistry panel
        - Store in refrigerated conditions
        
        Submitted by: Dr. Sarah Johnson
        Contact: sarah.johnson@hospital.com
        Phone: (555) 123-4567
    "#
    .as_bytes();

    // Process through the pipeline
    println!("📄 Processing laboratory submission document...");
    let result = processor
        .process_document(lab_submission_doc, "lab_submission.txt")
        .await
        .expect("Failed to process lab submission");

    // Verify the data flow stages
    assert_eq!(result.stage, ProcessingStage::ValidationComplete);
    assert!(result.confidence >= 0.8, "Confidence should meet threshold");
    assert!(result.errors.is_empty(), "No errors should occur");

    // Verify extracted data structure
    assert!(
        !result.extracted_data.is_null(),
        "Should extract structured data"
    );

    // Verify barcode generation (part of sample creation flow)
    assert!(
        result.metadata.contains_key("barcode"),
        "Should generate sample barcode"
    );
    let barcode = result.metadata.get("barcode").unwrap();
    assert!(
        barcode.starts_with("LAB-"),
        "Barcode should follow naming convention"
    );

    // Verify pipeline statistics
    let stats = processor.get_stats();
    assert_eq!(stats.documents_processed, 1);
    assert_eq!(stats.samples_created, 1);
    assert!(stats.average_confidence >= 0.8);

    println!("✅ Sample submission pipeline completed successfully");
    println!("   - Barcode generated: {}", barcode);
    println!("   - Confidence score: {:.2}", result.confidence);
    println!("   - Pipeline stage: {:?}", result.stage);

    registry
        .shutdown_all()
        .await
        .expect("Failed to shutdown registry");
}

/// Test storage location management flow with temperature validation
/// This demonstrates: Temperature zone validation → Capacity checking → Position assignment
#[tokio::test]
async fn test_storage_location_management_flow() {
    println!("🌡️  Testing Storage Location Management Flow");

    let mut registry = ServiceRegistry::new();

    // Set up storage component with different temperature zones
    let storage_configs = vec![
        (
            "freezer_minus80",
            StorageComponentBuilder::new().filesystem("/tmp/storage_minus80"),
        ),
        (
            "freezer_minus20",
            StorageComponentBuilder::new().filesystem("/tmp/storage_minus20"),
        ),
        (
            "refrigerator_4c",
            StorageComponentBuilder::new().filesystem("/tmp/storage_4c"),
        ),
        (
            "room_temp",
            StorageComponentBuilder::new().filesystem("/tmp/storage_rt"),
        ),
    ];

    // Register storage components for different temperature zones
    for (zone_name, builder) in storage_configs {
        let mut storage = builder.build().expect("Failed to build storage");
        storage
            .initialize(&registry)
            .await
            .expect("Failed to initialize storage");

        // Simulate temperature zone validation
        let zone_temp = match zone_name {
            "freezer_minus80" => -80,
            "freezer_minus20" => -20,
            "refrigerator_4c" => 4,
            "room_temp" => 22,
            _ => 22,
        };

        // Test temperature compatibility for sample storage
        let sample_temp_requirements = vec![-80, -20, 4, 22];

        for required_temp in sample_temp_requirements {
            let is_compatible = validate_temperature_compatibility(zone_temp, required_temp);

            if zone_temp == required_temp {
                assert!(
                    is_compatible,
                    "Exact temperature match should be compatible"
                );
                println!(
                    "✅ Sample requiring {}°C can be stored in {} zone",
                    required_temp, zone_name
                );
            } else if zone_temp < required_temp {
                assert!(is_compatible, "Colder storage should be acceptable");
                println!(
                    "✅ Sample requiring {}°C can be stored in colder {} zone",
                    required_temp, zone_name
                );
            } else {
                assert!(!is_compatible, "Warmer storage should not be acceptable");
                println!(
                    "❌ Sample requiring {}°C cannot be stored in warmer {} zone",
                    required_temp, zone_name
                );
            }
        }

        // Test storage capacity simulation
        let capacity_info = simulate_storage_capacity_check(zone_name);
        println!(
            "📊 {} capacity: {:.1}% used",
            zone_name, capacity_info.utilization_percent
        );

        if capacity_info.utilization_percent > 95.0 {
            println!("🚨 Critical: {} storage over 95% capacity!", zone_name);
        } else if capacity_info.utilization_percent > 80.0 {
            println!("⚠️  Warning: {} storage over 80% capacity", zone_name);
        }

        storage
            .shutdown()
            .await
            .expect("Failed to shutdown storage");
    }

    println!("✅ Storage location management flow validated");
}

/// Test template processing pipeline flow
/// This demonstrates: Template upload → Format detection → Data extraction → Validation
#[tokio::test]
async fn test_template_processing_pipeline_flow() {
    println!("📋 Testing Template Processing Pipeline Flow");

    let mut processor = SampleProcessingBuilder::new()
        .with_rag(true)
        .for_high_throughput() // Configure for template processing
        .build();

    let registry = ServiceRegistry::new();
    processor
        .initialize(&registry)
        .await
        .expect("Failed to initialize processor");

    // Simulate CSV template data
    let csv_template = r#"
Sample_ID,Sample_Type,Collection_Date,Patient_ID,Volume,Temperature,Priority
S001,Blood,2024-01-15,P12345,5ml,4C,Standard
S002,Urine,2024-01-15,P12346,10ml,RT,Urgent
S003,Tissue,2024-01-16,P12347,2g,-80C,Standard
S004,Plasma,2024-01-16,P12348,3ml,-20C,Standard
    "#
    .trim()
    .as_bytes();

    // Test CSV format detection and processing
    println!("📊 Processing CSV template...");
    let csv_result = processor
        .process_document(csv_template, "lab_samples.csv")
        .await
        .expect("Failed to process CSV template");

    assert_eq!(csv_result.stage, ProcessingStage::ValidationComplete);
    assert!(csv_result.errors.is_empty());

    // Simulate Excel template data (simplified as we're testing the flow)
    let excel_template = r#"
Laboratory Sample Template
=========================

Batch ID: BATCH_2024_001
Processing Date: 2024-01-15
Laboratory: Central Lab

Samples:
1. Sample ID: S005, Type: Blood, Patient: P12349
2. Sample ID: S006, Type: Serum, Patient: P12350
3. Sample ID: S007, Type: CSF, Patient: P12351

Quality Control:
- All samples collected under sterile conditions
- Temperature maintained during transport
- Chain of custody documented
    "#
    .trim()
    .as_bytes();

    // Test document format processing
    println!("📄 Processing document template...");
    let doc_result = processor
        .process_document(excel_template, "lab_batch.txt")
        .await
        .expect("Failed to process document template");

    assert_eq!(doc_result.stage, ProcessingStage::ValidationComplete);

    // Simulate multi-format batch processing
    let templates = vec![
        (csv_template, "samples_batch1.csv"),
        (excel_template, "samples_batch2.txt"),
    ];

    let mut batch_results = Vec::new();

    for (template_data, filename) in templates {
        let result = processor
            .process_document(template_data, filename)
            .await
            .expect("Failed to process template in batch");
        batch_results.push(result);
    }

    // Verify batch processing results
    assert_eq!(batch_results.len(), 2);
    assert!(batch_results
        .iter()
        .all(|r| r.stage == ProcessingStage::ValidationComplete));

    let stats = processor.get_stats();
    println!("✅ Template processing completed:");
    println!(
        "   - Total templates processed: {}",
        stats.documents_processed
    );
    println!("   - Samples created: {}", stats.samples_created);
    println!("   - Average confidence: {:.2}", stats.average_confidence);
}

/// Test event-driven communication flow
/// This demonstrates: State changes → Storage updates → Alerts → Audit logs
#[tokio::test]
async fn test_event_driven_communication_flow() {
    println!("📡 Testing Event-Driven Communication Flow");

    // Set up event simulation system
    let mut event_log = Vec::new();
    let mut system_state = SystemState::new();

    // Simulate sample state changes triggering events
    let sample_events = vec![
        SampleEvent::Created {
            sample_id: "S001".to_string(),
            initial_state: "Pending".to_string(),
        },
        SampleEvent::StateChanged {
            sample_id: "S001".to_string(),
            from: "Pending".to_string(),
            to: "Validated".to_string(),
        },
        SampleEvent::LocationChanged {
            sample_id: "S001".to_string(),
            from: "Reception".to_string(),
            to: "Storage_A1".to_string(),
        },
        SampleEvent::StateChanged {
            sample_id: "S001".to_string(),
            from: "Validated".to_string(),
            to: "InStorage".to_string(),
        },
        SampleEvent::StateChanged {
            sample_id: "S001".to_string(),
            from: "InStorage".to_string(),
            to: "InSequencing".to_string(),
        },
    ];

    println!("🔄 Processing sample lifecycle events...");

    for event in sample_events {
        // Process each event and trigger dependent updates
        let event_result = process_sample_event(&event, &mut system_state).await;
        event_log.push((event.clone(), event_result));

        // Simulate event propagation to dependent systems
        match &event {
            SampleEvent::StateChanged {
                sample_id,
                from,
                to,
            } => {
                println!("📝 Sample {} state: {} → {}", sample_id, from, to);

                // Trigger storage updates
                if to == "InStorage" {
                    trigger_storage_update(sample_id).await;
                }

                // Trigger capacity monitoring
                if to == "InStorage" || from == "InStorage" {
                    check_capacity_thresholds().await;
                }

                // Update audit trail
                update_audit_trail(sample_id, &format!("State changed from {} to {}", from, to))
                    .await;
            }
            SampleEvent::LocationChanged {
                sample_id,
                from,
                to,
            } => {
                println!("📍 Sample {} location: {} → {}", sample_id, from, to);
                update_audit_trail(sample_id, &format!("Moved from {} to {}", from, to)).await;
            }
            SampleEvent::Created {
                sample_id,
                initial_state,
            } => {
                println!(
                    "🆕 Sample {} created with state: {}",
                    sample_id, initial_state
                );
                update_audit_trail(sample_id, "Sample created").await;
            }
        }
    }

    // Verify event processing results
    assert_eq!(event_log.len(), 5);
    assert!(event_log.iter().all(|(_, result)| result.success));

    // Verify system state updates
    assert_eq!(
        system_state.sample_states.get("S001"),
        Some(&"InSequencing".to_string())
    );
    assert_eq!(
        system_state.sample_locations.get("S001"),
        Some(&"Storage_A1".to_string())
    );

    println!("✅ Event-driven communication flow completed:");
    println!("   - Events processed: {}", event_log.len());
    println!(
        "   - Final sample state: {:?}",
        system_state.sample_states.get("S001")
    );
    println!(
        "   - Final sample location: {:?}",
        system_state.sample_locations.get("S001")
    );
}

/// Test document processing flow with different formats
/// This demonstrates: PDF/DOCX ingestion → Text extraction → RAG processing → Data structuring
#[tokio::test]
async fn test_document_processing_flow() {
    println!("📚 Testing Document Processing Flow");

    let mut processor = SampleProcessingBuilder::new()
        .with_rag(true)
        .with_confidence_threshold(0.7)
        .build();

    let registry = ServiceRegistry::new();
    processor
        .initialize(&registry)
        .await
        .expect("Failed to initialize processor");

    // Test different document types and their processing flows
    let test_documents = vec![
        (
            create_lab_report_content(),
            "lab_report.pdf",
            "Laboratory Report",
        ),
        (
            create_clinical_notes_content(),
            "clinical_notes.docx",
            "Clinical Notes",
        ),
        (
            create_sample_manifest_content(),
            "sample_manifest.txt",
            "Sample Manifest",
        ),
    ];

    let mut processing_results = Vec::new();

    for (content, filename, doc_type) in test_documents {
        println!("📄 Processing {}: {}", doc_type, filename);

        let result = processor
            .process_document(&content, filename)
            .await
            .expect("Failed to process document");

        // Verify document processing stages
        assert_eq!(result.stage, ProcessingStage::ValidationComplete);
        assert!(result.confidence >= 0.7);

        // Verify structured data extraction
        assert!(!result.extracted_data.is_null());

        // Document-specific validations
        match doc_type {
            "Laboratory Report" => {
                // Should extract test results and patient information
                assert!(result.extracted_data.get("sample_type").is_some());
                assert!(result.extracted_data.get("patient_id").is_some());
            }
            "Clinical Notes" => {
                // Should extract clinical observations and recommendations
                assert!(result.extracted_data.get("patient_id").is_some());
            }
            "Sample Manifest" => {
                // Should extract batch information and sample list
                assert!(result.extracted_data.get("sample_type").is_some());
            }
            _ => {}
        }

        processing_results.push((doc_type, result));

        // Simulate confidence-based workflow routing
        let last_result = processing_results.last().unwrap();
        if last_result.1.confidence >= 0.9 {
            println!("   ✅ High confidence - Auto-validated");
        } else if last_result.1.confidence >= 0.7 {
            println!("   ⚠️  Medium confidence - Review recommended");
        } else {
            println!("   ❌ Low confidence - Manual review required");
        }
    }

    // Verify overall processing statistics
    let stats = processor.get_stats();
    assert_eq!(stats.documents_processed, 3);
    assert!(stats.average_confidence >= 0.7);

    println!("✅ Document processing flow completed:");
    println!("   - Documents processed: {}", processing_results.len());
    println!("   - Average confidence: {:.2}", stats.average_confidence);
}

/// Test complete integrated workflow combining all data flows
#[tokio::test]
async fn test_integrated_laboratory_workflow() {
    println!("🔬 Testing Complete Integrated Laboratory Workflow");

    // Set up complete laboratory system
    let mut registry = HybridLine::custom()
        .with_database(DatabaseComponentBuilder::new().for_testing())
        .with_storage(StorageComponentBuilder::new().mock())
        .build()
        .await
        .expect("Failed to set up laboratory system");

    // Initialize processing pipeline
    let mut processor = SampleProcessingBuilder::new()
        .with_rag(true)
        .with_confidence_threshold(0.8)
        .build();

    processor
        .initialize(&registry)
        .await
        .expect("Failed to initialize processor");

    println!("🏥 Simulating complete laboratory workflow...");

    // Step 1: Sample submission
    let submission_doc = create_lab_submission_form();
    let submission_result = processor
        .process_document(&submission_doc, "submission.pdf")
        .await
        .expect("Failed to process submission");

    assert_eq!(submission_result.stage, ProcessingStage::ValidationComplete);
    let sample_barcode = submission_result.metadata.get("barcode").unwrap();

    // Step 2: Sample validation and storage assignment
    let storage_assignment = assign_storage_location(&submission_result).await;
    assert!(storage_assignment.success);

    // Step 3: Template processing for batch samples
    let batch_template = create_batch_template();
    let batch_result = processor
        .process_document(&batch_template, "batch.csv")
        .await
        .expect("Failed to process batch template");

    assert_eq!(batch_result.stage, ProcessingStage::ValidationComplete);

    // Step 4: Event simulation for sample lifecycle
    let lifecycle_events = simulate_sample_lifecycle(sample_barcode).await;
    assert!(!lifecycle_events.is_empty());

    // Step 5: Final system health check
    let system_health = registry
        .health_check_all()
        .await
        .expect("Failed to check system health");
    assert!(system_health.values().all(|&healthy| healthy));

    // Verify final statistics
    let final_stats = processor.get_stats();

    println!("✅ Integrated workflow completed successfully:");
    println!("   - Sample barcode: {}", sample_barcode);
    println!("   - Storage location: {}", storage_assignment.location);
    println!(
        "   - Documents processed: {}",
        final_stats.documents_processed
    );
    println!("   - Samples created: {}", final_stats.samples_created);
    println!("   - Lifecycle events: {}", lifecycle_events.len());
    println!("   - System components healthy: {}", system_health.len());

    registry
        .shutdown_all()
        .await
        .expect("Failed to shutdown system");
}

// Helper functions and types for testing

#[derive(Debug, Clone)]
enum SampleEvent {
    Created {
        sample_id: String,
        initial_state: String,
    },
    StateChanged {
        sample_id: String,
        from: String,
        to: String,
    },
    LocationChanged {
        sample_id: String,
        from: String,
        to: String,
    },
}

#[derive(Debug, Default)]
struct SystemState {
    sample_states: HashMap<String, String>,
    sample_locations: HashMap<String, String>,
}

impl SystemState {
    fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug)]
struct EventResult {
    success: bool,
    message: String,
}

#[derive(Debug)]
struct StorageAssignment {
    success: bool,
    location: String,
    temperature_zone: String,
}

#[derive(Debug)]
struct CapacityInfo {
    utilization_percent: f64,
    available_slots: u32,
    total_slots: u32,
}

// Helper functions for testing workflows

fn validate_temperature_compatibility(storage_temp: i32, required_temp: i32) -> bool {
    storage_temp <= required_temp
}

fn simulate_storage_capacity_check(zone_name: &str) -> CapacityInfo {
    use fastrand;
    let utilization = match zone_name {
        "freezer_minus80" => 85.5,
        "freezer_minus20" => 72.3,
        "refrigerator_4c" => 91.2,
        "room_temp" => 45.8,
        _ => fastrand::f64() * 100.0,
    };

    let total_slots = 1000;
    let used_slots = (total_slots as f64 * utilization / 100.0) as u32;

    CapacityInfo {
        utilization_percent: utilization,
        available_slots: total_slots - used_slots,
        total_slots,
    }
}

async fn process_sample_event(event: &SampleEvent, state: &mut SystemState) -> EventResult {
    match event {
        SampleEvent::Created {
            sample_id,
            initial_state,
        } => {
            state
                .sample_states
                .insert(sample_id.clone(), initial_state.clone());
            EventResult {
                success: true,
                message: format!("Sample {} created", sample_id),
            }
        }
        SampleEvent::StateChanged { sample_id, to, .. } => {
            state.sample_states.insert(sample_id.clone(), to.clone());
            EventResult {
                success: true,
                message: format!("Sample {} state updated to {}", sample_id, to),
            }
        }
        SampleEvent::LocationChanged { sample_id, to, .. } => {
            state.sample_locations.insert(sample_id.clone(), to.clone());
            EventResult {
                success: true,
                message: format!("Sample {} location updated to {}", sample_id, to),
            }
        }
    }
}

async fn trigger_storage_update(sample_id: &str) {
    println!("🗄️  Storage update triggered for sample {}", sample_id);
}

async fn check_capacity_thresholds() {
    println!("📊 Checking storage capacity thresholds");
}

async fn update_audit_trail(sample_id: &str, action: &str) {
    println!("📝 Audit: Sample {} - {}", sample_id, action);
}

async fn assign_storage_location(processing_result: &ProcessingResult) -> StorageAssignment {
    // Simulate storage assignment based on sample requirements
    let sample_type = processing_result
        .extracted_data
        .get("sample_type")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    let (location, temp_zone) = match sample_type {
        "blood" => ("Freezer_A_Rack_1_Slot_15", "-20C"),
        "tissue" => ("Freezer_B_Rack_2_Slot_8", "-80C"),
        "urine" => ("Refrigerator_C_Shelf_3", "4C"),
        _ => ("Room_Temp_Cabinet_D", "RT"),
    };

    StorageAssignment {
        success: true,
        location: location.to_string(),
        temperature_zone: temp_zone.to_string(),
    }
}

async fn simulate_sample_lifecycle(sample_barcode: &str) -> Vec<SampleEvent> {
    vec![
        SampleEvent::StateChanged {
            sample_id: sample_barcode.to_string(),
            from: "Pending".to_string(),
            to: "Validated".to_string(),
        },
        SampleEvent::LocationChanged {
            sample_id: sample_barcode.to_string(),
            from: "Reception".to_string(),
            to: "Storage_A1".to_string(),
        },
        SampleEvent::StateChanged {
            sample_id: sample_barcode.to_string(),
            from: "Validated".to_string(),
            to: "InStorage".to_string(),
        },
    ]
}

// Test data creation helpers

fn create_lab_report_content() -> Vec<u8> {
    r#"
    Laboratory Analysis Report
    ==========================
    
    Patient ID: P12345
    Sample ID: S001
    Collection Date: 2024-01-15
    Sample Type: Blood
    
    Test Results:
    - Glucose: 95 mg/dL (Normal)
    - Cholesterol: 180 mg/dL (Normal)
    - Hemoglobin: 14.2 g/dL (Normal)
    
    Quality Control: PASSED
    Technician: Lab Tech 001
    Review Date: 2024-01-16
    "#
    .trim()
    .as_bytes()
    .to_vec()
}

fn create_clinical_notes_content() -> Vec<u8> {
    r#"
    Clinical Notes
    ==============
    
    Patient: P12346
    Date: 2024-01-15
    Physician: Dr. Johnson
    
    Chief Complaint: Routine check-up
    
    Sample Collection:
    - Blood sample collected for routine screening
    - Patient fasting for 12 hours
    - Sample stored at 4°C immediately after collection
    
    Recommendations:
    - Process within 24 hours
    - Standard laboratory panel requested
    "#
    .trim()
    .as_bytes()
    .to_vec()
}

fn create_sample_manifest_content() -> Vec<u8> {
    r#"
    Sample Manifest - Batch BATCH_001
    ==================================
    
    Date: 2024-01-15
    Laboratory: Central Lab
    
    Samples in this batch:
    1. S001 - Blood - P12345 - 4°C storage
    2. S002 - Urine - P12346 - RT storage  
    3. S003 - Tissue - P12347 - -80°C storage
    
    Total samples: 3
    Processing priority: Standard
    Expected completion: 2024-01-17
    "#
    .trim()
    .as_bytes()
    .to_vec()
}

fn create_lab_submission_form() -> Vec<u8> {
    r#"
    Laboratory Sample Submission
    ============================
    
    Sample Type: Blood
    Patient ID: P99999
    Collection Date: 2024-01-15
    Volume: 5ml
    Temperature: 4°C
    Priority: Standard
    
    Submitted by: Dr. Smith
    Laboratory: Central Lab
    Processing requested: Full panel
    "#
    .trim()
    .as_bytes()
    .to_vec()
}

fn create_batch_template() -> Vec<u8> {
    r#"
Sample_ID,Sample_Type,Patient_ID,Collection_Date,Volume,Temperature
B001,Blood,P11111,2024-01-15,5ml,4C
B002,Serum,P11112,2024-01-15,3ml,-20C
B003,Plasma,P11113,2024-01-15,4ml,-20C
    "#
    .trim()
    .as_bytes()
    .to_vec()
}
