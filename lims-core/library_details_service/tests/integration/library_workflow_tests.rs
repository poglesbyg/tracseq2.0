use crate::test_utils::*;
use library_details_service::{
    models::*,
    handlers::*,
    services::*,
    create_app,
};
use axum_test::TestServer;
use serde_json::json;
use uuid::Uuid;

/// Integration tests for complete library preparation workflows
#[tokio::test]
async fn test_complete_library_preparation_lifecycle() {
    let mut test_db = TestDatabase::new().await;
    let app = create_app().await;
    let client = LibraryTestClient::new(app);

    // Phase 1: Create library preparation protocol
    let protocol_request = LibraryFactory::create_valid_protocol_request();
    let protocol_name = protocol_request.name.clone();
    
    let response = client.post_json("/api/library/protocols", &protocol_request).await;
    LibraryAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let protocol_data: serde_json::Value = response.json();
    LibraryAssertions::assert_protocol_data(&protocol_data, &protocol_name);
    
    let protocol_id = Uuid::parse_str(protocol_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_protocol(protocol_id);

    // Phase 2: Validate protocol steps
    let validation_request = json!({
        "protocol_id": protocol_id,
        "check_reagents": true,
        "check_equipment": true,
        "check_steps": true
    });
    
    let response = client.post_json("/api/library/protocols/validate", &validation_request).await;
    LibraryAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
    
    let validation_data: serde_json::Value = response.json();
    LibraryAssertions::assert_protocol_validation(&validation_data);
    assert_eq!(validation_data["data"]["valid"], true);

    // Phase 3: Create library preparation request
    let mut prep_request = LibraryFactory::create_preparation_request();
    prep_request.protocol_id = protocol_id;
    prep_request.samples = LibraryFactory::create_sample_inputs(8);
    
    let response = client.post_json("/api/library/preparations", &prep_request).await;
    LibraryAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let prep_data: serde_json::Value = response.json();
    LibraryAssertions::assert_preparation_data(&prep_data);
    
    let prep_id = Uuid::parse_str(prep_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_preparation(prep_id);

    // Phase 4: Execute library preparation steps
    let prep_steps = LibraryFactory::create_preparation_steps();
    for (step_index, step) in prep_steps.iter().enumerate() {
        let step_request = json!({
            "preparation_id": prep_id,
            "step_number": step_index + 1,
            "step_data": step,
            "operator_id": Uuid::new_v4(),
            "timestamp": chrono::Utc::now()
        });
        
        let response = client.post_json("/api/library/preparations/steps", &step_request).await;
        LibraryAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
        
        let step_data: serde_json::Value = response.json();
        LibraryAssertions::assert_step_execution(&step_data, step_index + 1);
        
        let step_id = Uuid::parse_str(step_data["data"]["id"].as_str().unwrap()).unwrap();
        test_db.track_step(step_id);
    }

    // Phase 5: Perform quality control checks
    let qc_request = LibraryQcRequest {
        preparation_id: prep_id,
        qc_type: QcType::PostPreparation,
        measurements: LibraryFactory::create_qc_measurements(),
        options: Some(QcTestingOptions {
            check_concentration: true,
            check_fragment_size: true,
            check_adapter_content: true,
            check_contamination: true,
        }),
    };
    
    let response = client.post_json("/api/library/qc", &qc_request).await;
    LibraryAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let qc_data: serde_json::Value = response.json();
    LibraryAssertions::assert_qc_results(&qc_data);
    
    let qc_id = Uuid::parse_str(qc_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_qc_result(qc_id);

    // Phase 6: Generate library metrics report
    let report_request = json!({
        "preparation_id": prep_id,
        "include_qc_results": true,
        "include_protocol_details": true,
        "include_cost_analysis": true,
        "format": "PDF"
    });
    
    let response = client.post_json("/api/library/reports", &report_request).await;
    LibraryAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let report_data: serde_json::Value = response.json();
    LibraryAssertions::assert_report_generation(&report_data);
    
    let report_id = Uuid::parse_str(report_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_report(report_id);

    // Phase 7: Download generated report
    let response = client.get(&format!("/api/library/reports/{}/download", report_id)).await;
    LibraryAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
    
    let report_content = response.bytes();
    LibraryFileUtils::assert_pdf_structure(&report_content);

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_multi_library_type_preparation() {
    let mut test_db = TestDatabase::new().await;
    let app = create_app().await;
    let client = LibraryTestClient::new(app);

    // Test different library types with specific protocols
    let library_types = vec![
        (LibraryType::WholeDNA, LibraryFactory::create_dna_protocol()),
        (LibraryType::RNASeq, LibraryFactory::create_rna_protocol()),
        (LibraryType::ChIPSeq, LibraryFactory::create_chip_protocol()),
        (LibraryType::ATACSeq, LibraryFactory::create_atac_protocol()),
        (LibraryType::Amplicon, LibraryFactory::create_amplicon_protocol()),
    ];

    let mut protocol_ids = Vec::new();
    let mut prep_ids = Vec::new();

    // Create protocols for each library type
    for (library_type, protocol_request) in library_types {
        let response = client.post_json("/api/library/protocols", &protocol_request).await;
        LibraryAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
        
        let protocol_data: serde_json::Value = response.json();
        let protocol_id = Uuid::parse_str(protocol_data["data"]["id"].as_str().unwrap()).unwrap();
        protocol_ids.push((library_type, protocol_id));
        test_db.track_protocol(protocol_id);

        // Create preparation for each library type
        let prep_request = CreatePreparationRequest {
            protocol_id,
            library_type,
            samples: LibraryFactory::create_sample_inputs_for_type(library_type, 4),
            expected_yield: LibraryFactory::get_expected_yield_for_type(library_type),
            priority: PreparationPriority::Normal,
            notes: Some(format!("Automated test preparation for {:?}", library_type)),
            operator_id: Uuid::new_v4(),
        };
        
        let response = client.post_json("/api/library/preparations", &prep_request).await;
        LibraryAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
        
        let prep_data: serde_json::Value = response.json();
        let prep_id = Uuid::parse_str(prep_data["data"]["id"].as_str().unwrap()).unwrap();
        prep_ids.push((library_type, prep_id));
        test_db.track_preparation(prep_id);
    }

    // Execute preparations with type-specific requirements
    for (library_type, prep_id) in prep_ids {
        let type_specific_steps = LibraryFactory::create_type_specific_steps(library_type);
        
        for (step_index, step) in type_specific_steps.iter().enumerate() {
            let step_request = json!({
                "preparation_id": prep_id,
                "step_number": step_index + 1,
                "step_data": step,
                "operator_id": Uuid::new_v4(),
                "timestamp": chrono::Utc::now()
            });
            
            let response = client.post_json("/api/library/preparations/steps", &step_request).await;
            LibraryAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
            
            let step_data: serde_json::Value = response.json();
            let step_id = Uuid::parse_str(step_data["data"]["id"].as_str().unwrap()).unwrap();
            test_db.track_step(step_id);
        }

        // Perform type-specific quality control
        let qc_request = LibraryQcRequest {
            preparation_id: prep_id,
            qc_type: QcType::TypeSpecific(library_type),
            measurements: LibraryFactory::create_type_specific_qc(library_type),
            options: Some(LibraryFactory::get_qc_options_for_type(library_type)),
        };
        
        let response = client.post_json("/api/library/qc", &qc_request).await;
        LibraryAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
        
        let qc_data: serde_json::Value = response.json();
        LibraryAssertions::assert_qc_results(&qc_data);
        
        let qc_id = Uuid::parse_str(qc_data["data"]["id"].as_str().unwrap()).unwrap();
        test_db.track_qc_result(qc_id);

        // Verify library type-specific metrics
        let specific_metrics = qc_data["data"]["type_specific_metrics"].as_object().unwrap();
        LibraryAssertions::assert_type_specific_metrics(specific_metrics, library_type);
    }

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_automated_library_optimization() {
    let mut test_db = TestDatabase::new().await;
    let app = create_app().await;
    let client = LibraryTestClient::new(app);

    // Create optimization-enabled protocol
    let optimization_protocol = CreateProtocolRequest {
        name: "Adaptive Library Preparation".to_string(),
        description: Some("Protocol with automated optimization capabilities".to_string()),
        library_type: LibraryType::WholeDNA,
        version: "2.0.0".to_string(),
        steps: LibraryFactory::create_optimizable_steps(),
        reagents: LibraryFactory::create_protocol_reagents(),
        equipment: LibraryFactory::create_protocol_equipment(),
        optimization_enabled: true,
        optimization_parameters: Some(OptimizationParameters {
            target_yield: 50.0, // ng/µL
            target_fragment_size: FragmentSizeRange { min: 200, max: 800 },
            max_iterations: 3,
            success_criteria: OptimizationCriteria {
                min_yield: 30.0,
                max_fragment_variance: 50,
                min_adapter_efficiency: 90.0,
            },
        }),
        is_active: true,
    };
    
    let response = client.post_json("/api/library/protocols", &optimization_protocol).await;
    let protocol_data: serde_json::Value = response.json();
    let protocol_id = Uuid::parse_str(protocol_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_protocol(protocol_id);

    // Create preparation with optimization
    let prep_request = CreatePreparationRequest {
        protocol_id,
        library_type: LibraryType::WholeDNA,
        samples: LibraryFactory::create_diverse_sample_inputs(6), // Varied input quality
        expected_yield: 45.0,
        priority: PreparationPriority::High,
        notes: Some("Optimization test with diverse sample quality".to_string()),
        operator_id: Uuid::new_v4(),
    };
    
    let response = client.post_json("/api/library/preparations", &prep_request).await;
    let prep_data: serde_json::Value = response.json();
    let prep_id = Uuid::parse_str(prep_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_preparation(prep_id);

    // Start optimization process
    let optimization_request = json!({
        "preparation_id": prep_id,
        "enable_real_time_adjustment": true,
        "feedback_sensitivity": "High",
        "optimization_mode": "YieldMaximization"
    });
    
    let response = client.post_json("/api/library/optimization/start", &optimization_request).await;
    LibraryAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let optimization_data: serde_json::Value = response.json();
    LibraryAssertions::assert_optimization_started(&optimization_data);
    
    let optimization_id = Uuid::parse_str(optimization_data["data"]["optimization_id"].as_str().unwrap()).unwrap();
    test_db.track_optimization(optimization_id);

    // Simulate iterative optimization steps
    let mut iteration = 1;
    let max_iterations = 3;
    
    while iteration <= max_iterations {
        // Execute current iteration
        let iteration_request = json!({
            "optimization_id": optimization_id,
            "iteration": iteration,
            "input_metrics": LibraryFactory::create_iteration_metrics(iteration),
            "environmental_conditions": LibraryFactory::create_environmental_conditions()
        });
        
        let response = client.post_json("/api/library/optimization/iterate", &iteration_request).await;
        LibraryAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
        
        let iteration_data: serde_json::Value = response.json();
        LibraryAssertions::assert_iteration_results(&iteration_data, iteration);
        
        // Check if optimization is complete
        let optimization_status = iteration_data["data"]["status"].as_str().unwrap();
        if optimization_status == "Optimized" {
            break;
        }
        
        iteration += 1;
    }

    // Get final optimization results
    let response = client.get(&format!("/api/library/optimization/{}/results", optimization_id)).await;
    LibraryAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
    
    let results_data: serde_json::Value = response.json();
    LibraryAssertions::assert_optimization_results(&results_data);
    
    // Verify optimization improvements
    let final_yield = results_data["data"]["final_metrics"]["yield"].as_f64().unwrap();
    let initial_yield = results_data["data"]["initial_metrics"]["yield"].as_f64().unwrap();
    let improvement = (final_yield - initial_yield) / initial_yield * 100.0;
    
    assert!(improvement >= 5.0, "Optimization should improve yield by at least 5%");

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_high_throughput_library_processing() {
    let mut test_db = TestDatabase::new().await;
    let app = create_app().await;
    let client = LibraryTestClient::new(app);

    // Create high-throughput protocol
    let htp_protocol = CreateProtocolRequest {
        name: "High-Throughput DNA Library Prep".to_string(),
        description: Some("96-well plate based library preparation".to_string()),
        library_type: LibraryType::WholeDNA,
        version: "HTP-1.0".to_string(),
        steps: LibraryFactory::create_high_throughput_steps(),
        reagents: LibraryFactory::create_htp_reagents(),
        equipment: LibraryFactory::create_htp_equipment(),
        optimization_enabled: false,
        optimization_parameters: None,
        is_active: true,
    };
    
    let response = client.post_json("/api/library/protocols", &htp_protocol).await;
    let protocol_data: serde_json::Value = response.json();
    let protocol_id = Uuid::parse_str(protocol_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_protocol(protocol_id);

    // Create batch preparation for 96 samples
    let batch_size = 96;
    let batch_request = CreateBatchPreparationRequest {
        protocol_id,
        library_type: LibraryType::WholeDNA,
        batch_name: "HTP-BATCH-001".to_string(),
        samples: LibraryFactory::create_plate_layout_samples(batch_size),
        plate_layout: Some(PlateLayout::Standard96Well),
        expected_yield: 40.0,
        priority: PreparationPriority::High,
        operator_id: Uuid::new_v4(),
    };
    
    let start_time = std::time::Instant::now();
    let response = client.post_json("/api/library/preparations/batch", &batch_request).await;
    let batch_creation_time = start_time.elapsed();
    
    LibraryAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let batch_data: serde_json::Value = response.json();
    LibraryAssertions::assert_batch_preparation(&batch_data, batch_size);
    
    let batch_id = batch_data["data"]["batch_id"].as_str().unwrap();
    let prep_ids: Vec<Uuid> = batch_data["data"]["preparation_ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|id| Uuid::parse_str(id.as_str().unwrap()).unwrap())
        .collect();

    for prep_id in &prep_ids {
        test_db.track_preparation(*prep_id);
    }

    // Performance assertions for batch creation
    assert!(batch_creation_time.as_secs() < 10, "Batch creation should complete within 10 seconds");

    // Execute batch processing
    let batch_execution_request = json!({
        "batch_id": batch_id,
        "execution_mode": "Parallel",
        "max_concurrent_preps": 8,
        "enable_real_time_monitoring": true
    });
    
    let response = client.post_json("/api/library/preparations/batch/execute", &batch_execution_request).await;
    LibraryAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::ACCEPTED);
    
    let execution_data: serde_json::Value = response.json();
    let execution_id = execution_data["data"]["execution_id"].as_str().unwrap();

    // Monitor batch execution progress
    let start_monitoring = std::time::Instant::now();
    let mut execution_complete = false;
    
    while !execution_complete && start_monitoring.elapsed().as_secs() < 300 {
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        
        let response = client.get(&format!("/api/library/preparations/batch/{}/status", execution_id)).await;
        let status_data: serde_json::Value = response.json();
        
        let status = status_data["data"]["status"].as_str().unwrap();
        let completed_count = status_data["data"]["completed_count"].as_u64().unwrap();
        let total_count = status_data["data"]["total_count"].as_u64().unwrap();
        
        if status == "Completed" && completed_count == total_count {
            execution_complete = true;
        }
        
        // Log progress
        println!("Batch execution progress: {}/{} completed", completed_count, total_count);
    }
    
    assert!(execution_complete, "Batch execution should complete within timeout");

    // Perform batch quality control
    let batch_qc_request = json!({
        "batch_id": batch_id,
        "qc_type": "BatchQC",
        "include_cross_contamination_check": true,
        "include_batch_effect_analysis": true,
        "include_outlier_detection": true
    });
    
    let response = client.post_json("/api/library/qc/batch", &batch_qc_request).await;
    LibraryAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let batch_qc_data: serde_json::Value = response.json();
    LibraryAssertions::assert_batch_qc_results(&batch_qc_data, batch_size);
    
    let batch_qc_id = Uuid::parse_str(batch_qc_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_qc_result(batch_qc_id);

    // Generate comprehensive batch report
    let batch_report_request = json!({
        "batch_id": batch_id,
        "include_individual_results": true,
        "include_statistical_analysis": true,
        "include_cost_breakdown": true,
        "include_efficiency_metrics": true,
        "format": "PDF"
    });
    
    let response = client.post_json("/api/library/reports/batch", &batch_report_request).await;
    LibraryAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let batch_report_data: serde_json::Value = response.json();
    let batch_report_id = Uuid::parse_str(batch_report_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_report(batch_report_id);

    // Verify throughput metrics
    let total_processing_time = start_monitoring.elapsed();
    let throughput = batch_size as f64 / total_processing_time.as_secs_f64() * 3600.0; // samples per hour
    
    assert!(throughput >= 50.0, "High-throughput processing should achieve at least 50 samples/hour");

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_library_cost_analysis_and_tracking() {
    let mut test_db = TestDatabase::new().await;
    let app = create_app().await;
    let client = LibraryTestClient::new(app);

    // Create cost-tracked protocol
    let cost_protocol = CreateProtocolRequest {
        name: "Cost-Optimized Library Prep".to_string(),
        description: Some("Protocol with detailed cost tracking".to_string()),
        library_type: LibraryType::RNASeq,
        version: "COST-1.0".to_string(),
        steps: LibraryFactory::create_cost_tracked_steps(),
        reagents: LibraryFactory::create_cost_tracked_reagents(),
        equipment: LibraryFactory::create_cost_tracked_equipment(),
        optimization_enabled: false,
        optimization_parameters: None,
        is_active: true,
    };
    
    let response = client.post_json("/api/library/protocols", &cost_protocol).await;
    let protocol_data: serde_json::Value = response.json();
    let protocol_id = Uuid::parse_str(protocol_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_protocol(protocol_id);

    // Create multiple preparations for cost analysis
    let prep_sizes = vec![8, 16, 32, 48, 96];
    let mut prep_ids = Vec::new();
    
    for sample_count in prep_sizes {
        let prep_request = CreatePreparationRequest {
            protocol_id,
            library_type: LibraryType::RNASeq,
            samples: LibraryFactory::create_sample_inputs(sample_count),
            expected_yield: 35.0,
            priority: PreparationPriority::Normal,
            notes: Some(format!("Cost analysis prep - {} samples", sample_count)),
            operator_id: Uuid::new_v4(),
        };
        
        let response = client.post_json("/api/library/preparations", &prep_request).await;
        let prep_data: serde_json::Value = response.json();
        let prep_id = Uuid::parse_str(prep_data["data"]["id"].as_str().unwrap()).unwrap();
        prep_ids.push((sample_count, prep_id));
        test_db.track_preparation(prep_id);
    }

    // Execute preparations and track costs
    let mut cost_data = Vec::new();
    
    for (sample_count, prep_id) in prep_ids {
        // Execute all steps with cost tracking
        let steps = LibraryFactory::create_cost_tracked_steps();
        let mut total_reagent_cost = 0.0;
        let mut total_labor_cost = 0.0;
        
        for (step_index, step) in steps.iter().enumerate() {
            let step_request = json!({
                "preparation_id": prep_id,
                "step_number": step_index + 1,
                "step_data": step,
                "operator_id": Uuid::new_v4(),
                "timestamp": chrono::Utc::now(),
                "cost_tracking": {
                    "track_reagent_usage": true,
                    "track_labor_time": true,
                    "track_equipment_usage": true
                }
            });
            
            let response = client.post_json("/api/library/preparations/steps", &step_request).await;
            let step_data: serde_json::Value = response.json();
            let step_id = Uuid::parse_str(step_data["data"]["id"].as_str().unwrap()).unwrap();
            test_db.track_step(step_id);
            
            // Accumulate costs
            if let Some(costs) = step_data["data"]["costs"].as_object() {
                total_reagent_cost += costs["reagents"].as_f64().unwrap_or(0.0);
                total_labor_cost += costs["labor"].as_f64().unwrap_or(0.0);
            }
        }
        
        // Get comprehensive cost analysis
        let cost_analysis_request = json!({
            "preparation_id": prep_id,
            "include_overhead": true,
            "include_depreciation": true,
            "include_waste_factor": true
        });
        
        let response = client.post_json("/api/library/cost-analysis", &cost_analysis_request).await;
        LibraryAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
        
        let analysis_data: serde_json::Value = response.json();
        LibraryAssertions::assert_cost_analysis(&analysis_data);
        
        let total_cost = analysis_data["data"]["total_cost"].as_f64().unwrap();
        let cost_per_sample = total_cost / sample_count as f64;
        
        cost_data.push(CostDataPoint {
            sample_count,
            total_cost,
            cost_per_sample,
            reagent_cost: total_reagent_cost,
            labor_cost: total_labor_cost,
        });
    }

    // Analyze cost scaling
    LibraryAssertions::assert_cost_scaling(&cost_data);
    
    // Verify economies of scale
    let cost_per_sample_8 = cost_data[0].cost_per_sample;
    let cost_per_sample_96 = cost_data[4].cost_per_sample;
    let cost_reduction = (cost_per_sample_8 - cost_per_sample_96) / cost_per_sample_8 * 100.0;
    
    assert!(cost_reduction >= 20.0, "Larger batches should show significant cost reduction");

    // Generate cost comparison report
    let cost_report_request = json!({
        "preparation_ids": prep_ids.iter().map(|(_, id)| id).collect::<Vec<_>>(),
        "analysis_type": "ScalingAnalysis",
        "include_charts": true,
        "include_recommendations": true,
        "format": "PDF"
    });
    
    let response = client.post_json("/api/library/reports/cost-analysis", &cost_report_request).await;
    LibraryAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let cost_report_data: serde_json::Value = response.json();
    let cost_report_id = Uuid::parse_str(cost_report_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_report(cost_report_id);

    test_db.cleanup().await;
}

#[derive(Debug)]
struct CostDataPoint {
    sample_count: usize,
    total_cost: f64,
    cost_per_sample: f64,
    reagent_cost: f64,
    labor_cost: f64,
}