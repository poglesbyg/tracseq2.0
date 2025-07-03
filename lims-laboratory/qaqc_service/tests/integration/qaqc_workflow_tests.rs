use crate::test_utils::*;
use qaqc_service::{
    models::*,
    handlers::*,
    services::*,
    create_app,
};
use axum_test::TestServer;
use serde_json::json;
use uuid::Uuid;

/// Integration tests for complete quality control workflows
#[tokio::test]
async fn test_complete_qc_validation_lifecycle() {
    let mut test_db = TestDatabase::new().await;
    let app = create_app().await;
    let client = QaqcTestClient::new(app);

    // Phase 1: Create QC rule set
    let rule_set_request = QaqcFactory::create_valid_rule_set_request();
    let rule_set_name = rule_set_request.name.clone();
    
    let response = client.post_json("/api/qaqc/rulesets", &rule_set_request).await;
    QaqcAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let rule_set_data: serde_json::Value = response.json();
    QaqcAssertions::assert_rule_set_data(&rule_set_data, &rule_set_name);
    
    let rule_set_id = Uuid::parse_str(rule_set_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_rule_set(rule_set_id);

    // Phase 2: Create individual QC rules
    let qc_rules = vec![
        QaqcFactory::create_concentration_rule(),
        QaqcFactory::create_purity_rule(),
        QaqcFactory::create_integrity_rule(),
        QaqcFactory::create_volume_rule(),
    ];

    let mut rule_ids = Vec::new();
    for rule_request in qc_rules {
        let response = client.post_json(&format!("/api/qaqc/rulesets/{}/rules", rule_set_id), &rule_request).await;
        QaqcAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
        
        let rule_data: serde_json::Value = response.json();
        let rule_id = Uuid::parse_str(rule_data["data"]["id"].as_str().unwrap()).unwrap();
        rule_ids.push(rule_id);
        test_db.track_rule(rule_id);
    }

    // Phase 3: Validate rule set logic
    let validation_request = json!({
        "rule_set_id": rule_set_id,
        "check_conflicts": true,
        "check_completeness": true
    });
    
    let response = client.post_json("/api/qaqc/rulesets/validate", &validation_request).await;
    QaqcAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
    
    let validation_data: serde_json::Value = response.json();
    QaqcAssertions::assert_rule_set_validation(&validation_data);
    assert_eq!(validation_data["data"]["valid"], true);

    // Phase 4: Run QC validation on sample data
    let sample_data = QaqcFactory::create_sample_data_for_validation();
    let qc_request = QcValidationRequest {
        rule_set_id,
        sample_data: sample_data.clone(),
        validation_level: ValidationLevel::Strict,
        options: Some(QcValidationOptions {
            fail_fast: false,
            generate_report: true,
            include_warnings: true,
            detailed_results: true,
        }),
    };
    
    let response = client.post_json("/api/qaqc/validate", &qc_request).await;
    QaqcAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
    
    let qc_data: serde_json::Value = response.json();
    QaqcAssertions::assert_qc_validation_results(&qc_data);
    
    let validation_id = Uuid::parse_str(qc_data["data"]["validation_id"].as_str().unwrap()).unwrap();
    test_db.track_validation(validation_id);

    // Phase 5: Retrieve detailed QC report
    let response = client.get(&format!("/api/qaqc/validations/{}/report", validation_id)).await;
    QaqcAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
    
    let report_data: serde_json::Value = response.json();
    QaqcAssertions::assert_qc_report(&report_data);
    assert!(report_data["data"]["report_content"].is_string());

    // Phase 6: Test bulk validation
    let bulk_samples = QaqcFactory::create_bulk_sample_data(25);
    let bulk_request = BulkQcValidationRequest {
        rule_set_id,
        samples: bulk_samples,
        validation_level: ValidationLevel::Standard,
        batch_size: 10,
        options: Some(QcValidationOptions {
            fail_fast: false,
            generate_report: true,
            include_warnings: true,
            detailed_results: false, // Reduce detail for bulk processing
        }),
    };
    
    let response = client.post_json("/api/qaqc/validate/bulk", &bulk_request).await;
    QaqcAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let bulk_data: serde_json::Value = response.json();
    QaqcAssertions::assert_bulk_validation(&bulk_data, 25);
    
    let batch_id = bulk_data["data"]["batch_id"].as_str().unwrap();
    
    // Monitor bulk validation progress
    let mut attempts = 0;
    let max_attempts = 30;
    
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        
        let response = client.get(&format!("/api/qaqc/validate/bulk/{}/status", batch_id)).await;
        let status_data: serde_json::Value = response.json();
        
        let status = status_data["data"]["status"].as_str().unwrap();
        let processed = status_data["data"]["processed_count"].as_u64().unwrap();
        
        if status == "Completed" || processed == 25 {
            assert_eq!(status, "Completed");
            assert_eq!(processed, 25);
            break;
        }
        
        attempts += 1;
        if attempts >= max_attempts {
            panic!("Bulk validation did not complete within timeout");
        }
    }

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_advanced_qc_rule_configurations() {
    let mut test_db = TestDatabase::new().await;
    let app = create_app().await;
    let client = QaqcTestClient::new(app);

    // Create advanced rule set with complex conditions
    let advanced_rule_set = CreateRuleSetRequest {
        name: "Advanced QC Rules".to_string(),
        description: Some("Complex quality control rules with conditional logic".to_string()),
        category: RuleCategory::SampleQuality,
        version: "2.0.0".to_string(),
        is_active: true,
        rules: vec![
            // Multi-condition rule
            CreateQcRuleRequest {
                name: "DNA Purity Assessment".to_string(),
                rule_type: RuleType::Composite,
                conditions: vec![
                    QcCondition {
                        parameter: "concentration".to_string(),
                        operator: ComparisonOperator::GreaterThan,
                        value: json!(10.0),
                        unit: Some("ng/µL".to_string()),
                    },
                    QcCondition {
                        parameter: "purity_260_280".to_string(),
                        operator: ComparisonOperator::Between,
                        value: json!([1.7, 2.0]),
                        unit: Some("ratio".to_string()),
                    },
                    QcCondition {
                        parameter: "purity_260_230".to_string(),
                        operator: ComparisonOperator::Between,
                        value: json!([1.8, 2.2]),
                        unit: Some("ratio".to_string()),
                    },
                ],
                severity: RuleSeverity::Critical,
                action: RuleAction::Reject,
                logic_operator: LogicOperator::And,
                custom_message: Some("DNA purity requirements not met".to_string()),
            },
            // Conditional rule based on sample type
            CreateQcRuleRequest {
                name: "Sample Type Specific Volume".to_string(),
                rule_type: RuleType::Conditional,
                conditions: vec![
                    QcCondition {
                        parameter: "sample_type".to_string(),
                        operator: ComparisonOperator::Equals,
                        value: json!("Plasma"),
                        unit: None,
                    },
                    QcCondition {
                        parameter: "volume".to_string(),
                        operator: ComparisonOperator::GreaterThan,
                        value: json!(200.0),
                        unit: Some("µL".to_string()),
                    },
                ],
                severity: RuleSeverity::Warning,
                action: RuleAction::Flag,
                logic_operator: LogicOperator::And,
                custom_message: Some("Plasma samples require minimum 200µL volume".to_string()),
            },
            // Statistical rule
            CreateQcRuleRequest {
                name: "Batch Consistency Check".to_string(),
                rule_type: RuleType::Statistical,
                conditions: vec![
                    QcCondition {
                        parameter: "batch_cv".to_string(),
                        operator: ComparisonOperator::LessThan,
                        value: json!(15.0),
                        unit: Some("%".to_string()),
                    },
                ],
                severity: RuleSeverity::Warning,
                action: RuleAction::Review,
                logic_operator: LogicOperator::And,
                custom_message: Some("Batch coefficient of variation exceeds 15%".to_string()),
            },
        ],
    };
    
    let response = client.post_json("/api/qaqc/rulesets", &advanced_rule_set).await;
    let rule_set_data: serde_json::Value = response.json();
    let rule_set_id = Uuid::parse_str(rule_set_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_rule_set(rule_set_id);

    // Test complex sample data against advanced rules
    let complex_samples = vec![
        // Sample that should pass all rules
        json!({
            "sample_id": "SAM-PASS-001",
            "sample_type": "Plasma",
            "concentration": 25.5,
            "purity_260_280": 1.85,
            "purity_260_230": 2.0,
            "volume": 250.0,
            "batch_id": "BATCH-001",
            "batch_cv": 8.5
        }),
        // Sample that should fail DNA purity
        json!({
            "sample_id": "SAM-FAIL-001",
            "sample_type": "DNA",
            "concentration": 15.0,
            "purity_260_280": 1.5, // Below threshold
            "purity_260_230": 1.6, // Below threshold
            "volume": 100.0,
            "batch_id": "BATCH-001",
            "batch_cv": 8.5
        }),
        // Sample that should trigger volume warning
        json!({
            "sample_id": "SAM-WARN-001",
            "sample_type": "Plasma",
            "concentration": 20.0,
            "purity_260_280": 1.8,
            "purity_260_230": 1.9,
            "volume": 150.0, // Below required volume for plasma
            "batch_id": "BATCH-001",
            "batch_cv": 8.5
        }),
        // Sample that should trigger statistical warning
        json!({
            "sample_id": "SAM-STAT-001",
            "sample_type": "DNA",
            "concentration": 30.0,
            "purity_260_280": 1.9,
            "purity_260_230": 2.1,
            "volume": 200.0,
            "batch_id": "BATCH-002",
            "batch_cv": 18.0 // High CV
        }),
    ];

    let mut validation_results = Vec::new();
    for sample in complex_samples {
        let qc_request = QcValidationRequest {
            rule_set_id,
            sample_data: sample.clone(),
            validation_level: ValidationLevel::Comprehensive,
            options: Some(QcValidationOptions {
                fail_fast: false,
                generate_report: true,
                include_warnings: true,
                detailed_results: true,
            }),
        };
        
        let response = client.post_json("/api/qaqc/validate", &qc_request).await;
        let qc_data: serde_json::Value = response.json();
        
        let sample_id = sample["sample_id"].as_str().unwrap();
        let overall_status = qc_data["data"]["overall_status"].as_str().unwrap();
        
        validation_results.push((sample_id.to_string(), overall_status.to_string()));
        
        let validation_id = Uuid::parse_str(qc_data["data"]["validation_id"].as_str().unwrap()).unwrap();
        test_db.track_validation(validation_id);
    }

    // Verify expected results
    assert_eq!(validation_results[0].1, "Pass"); // SAM-PASS-001
    assert_eq!(validation_results[1].1, "Fail"); // SAM-FAIL-001
    assert_eq!(validation_results[2].1, "Warning"); // SAM-WARN-001
    assert_eq!(validation_results[3].1, "Warning"); // SAM-STAT-001

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_qc_trend_analysis_and_monitoring() {
    let mut test_db = TestDatabase::new().await;
    let app = create_app().await;
    let client = QaqcTestClient::new(app);

    // Create trend monitoring rule set
    let trend_rule_set = QaqcFactory::create_trend_monitoring_rule_set();
    let response = client.post_json("/api/qaqc/rulesets", &trend_rule_set).await;
    let rule_set_data: serde_json::Value = response.json();
    let rule_set_id = Uuid::parse_str(rule_set_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_rule_set(rule_set_id);

    // Generate time-series data for trend analysis
    let time_series_data = QaqcTestDataGenerator::generate_time_series_data(30); // 30 days of data
    
    let trend_request = TrendAnalysisRequest {
        rule_set_id,
        time_series_data: time_series_data.clone(),
        analysis_period: AnalysisPeriod::Month,
        parameters: vec![
            "concentration".to_string(),
            "purity_260_280".to_string(),
            "volume".to_string(),
        ],
        options: Some(TrendAnalysisOptions {
            detect_outliers: true,
            calculate_control_limits: true,
            generate_charts: true,
            include_predictions: true,
        }),
    };
    
    let response = client.post_json("/api/qaqc/trends/analyze", &trend_request).await;
    QaqcAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let trend_data: serde_json::Value = response.json();
    QaqcAssertions::assert_trend_analysis(&trend_data);
    
    let analysis_id = Uuid::parse_str(trend_data["data"]["analysis_id"].as_str().unwrap()).unwrap();
    test_db.track_analysis(analysis_id);

    // Wait for analysis completion
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    
    // Retrieve trend analysis results
    let response = client.get(&format!("/api/qaqc/trends/{}/results", analysis_id)).await;
    QaqcAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
    
    let results_data: serde_json::Value = response.json();
    QaqcAssertions::assert_trend_results(&results_data);
    
    // Verify trend analysis components
    assert!(results_data["data"]["statistical_summary"].is_object());
    assert!(results_data["data"]["outliers"].is_array());
    assert!(results_data["data"]["control_limits"].is_object());
    assert!(results_data["data"]["predictions"].is_object());

    // Test real-time monitoring setup
    let monitor_request = CreateMonitorRequest {
        name: "Real-time QC Monitor".to_string(),
        rule_set_id,
        parameters: vec!["concentration".to_string(), "purity_260_280".to_string()],
        thresholds: MonitorThresholds {
            warning_threshold: 2.0,  // 2 standard deviations
            critical_threshold: 3.0, // 3 standard deviations
        },
        notification_settings: Some(NotificationSettings {
            email_alerts: true,
            slack_alerts: true,
            dashboard_alerts: true,
            alert_frequency: AlertFrequency::Immediate,
        }),
        is_active: true,
    };
    
    let response = client.post_json("/api/qaqc/monitors", &monitor_request).await;
    QaqcAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let monitor_data: serde_json::Value = response.json();
    let monitor_id = Uuid::parse_str(monitor_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_monitor(monitor_id);

    // Simulate real-time data that should trigger alerts
    let alert_triggering_data = json!({
        "sample_id": "SAM-ALERT-001",
        "concentration": 5.0, // Significantly low
        "purity_260_280": 1.2, // Significantly low
        "timestamp": chrono::Utc::now(),
        "batch_id": "BATCH-ALERT-001"
    });
    
    let response = client.post_json(&format!("/api/qaqc/monitors/{}/check", monitor_id), &alert_triggering_data).await;
    QaqcAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
    
    let alert_data: serde_json::Value = response.json();
    QaqcAssertions::assert_alert_triggered(&alert_data);
    assert_eq!(alert_data["data"]["alert_level"], "Critical");

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_qc_performance_and_scalability() {
    let mut test_db = TestDatabase::new().await;
    let app = create_app().await;
    let client = QaqcTestClient::new(app);

    // Create performance test rule set
    let perf_rule_set = QaqcFactory::create_performance_rule_set();
    let response = client.post_json("/api/qaqc/rulesets", &perf_rule_set).await;
    let rule_set_data: serde_json::Value = response.json();
    let rule_set_id = Uuid::parse_str(rule_set_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_rule_set(rule_set_id);

    // Test large batch processing performance
    let large_batch_size = 1000;
    let large_batch_samples = QaqcFactory::create_bulk_sample_data(large_batch_size);
    
    let start_time = std::time::Instant::now();
    let bulk_request = BulkQcValidationRequest {
        rule_set_id,
        samples: large_batch_samples,
        validation_level: ValidationLevel::Standard,
        batch_size: 100, // Process in smaller chunks
        options: Some(QcValidationOptions {
            fail_fast: false,
            generate_report: false, // Skip report generation for performance
            include_warnings: true,
            detailed_results: false,
        }),
    };
    
    let response = client.post_json("/api/qaqc/validate/bulk", &bulk_request).await;
    let processing_time = start_time.elapsed();
    
    QaqcAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let bulk_data: serde_json::Value = response.json();
    let batch_id = bulk_data["data"]["batch_id"].as_str().unwrap();
    
    // Monitor completion time
    let start_monitoring = std::time::Instant::now();
    let mut completion_time = None;
    
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        let response = client.get(&format!("/api/qaqc/validate/bulk/{}/status", batch_id)).await;
        let status_data: serde_json::Value = response.json();
        
        let status = status_data["data"]["status"].as_str().unwrap();
        if status == "Completed" {
            completion_time = Some(start_monitoring.elapsed());
            break;
        }
        
        if start_monitoring.elapsed().as_secs() > 120 {
            panic!("Large batch processing took too long");
        }
    }
    
    let total_time = completion_time.unwrap();
    let throughput = large_batch_size as f64 / total_time.as_secs_f64();
    
    // Performance assertions
    assert!(processing_time.as_millis() < 5000, "Batch initiation should be fast");
    assert!(total_time.as_secs() < 60, "Large batch should complete within 60 seconds");
    assert!(throughput > 15.0, "Should process at least 15 samples per second");

    // Test concurrent validation performance
    let concurrent_count = 20;
    let concurrent_results = QaqcPerformanceUtils::concurrent_validation_test(
        &client,
        rule_set_id,
        concurrent_count,
    ).await;
    
    let successful_validations = concurrent_results.iter()
        .filter(|&status| *status == axum::http::StatusCode::OK)
        .count();
    
    assert!(successful_validations >= (concurrent_count * 90 / 100), "At least 90% of concurrent validations should succeed");

    // Test rule complexity impact
    let complex_rule_set = QaqcFactory::create_complex_rule_set();
    let response = client.post_json("/api/qaqc/rulesets", &complex_rule_set).await;
    let complex_rule_set_data: serde_json::Value = response.json();
    let complex_rule_set_id = Uuid::parse_str(complex_rule_set_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_rule_set(complex_rule_set_id);

    let complex_validation_time = QaqcPerformanceUtils::measure_validation_time(
        &client,
        complex_rule_set_id,
        QaqcFactory::create_sample_data_for_validation(),
    ).await;
    
    assert!(complex_validation_time.as_millis() < 2000, "Complex rule validation should complete within 2 seconds");

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_qc_integration_with_laboratory_workflow() {
    let mut test_db = TestDatabase::new().await;
    let app = create_app().await;
    let client = QaqcTestClient::new(app);

    // Create laboratory workflow QC rule set
    let workflow_rule_set = QaqcFactory::create_laboratory_workflow_rule_set();
    let response = client.post_json("/api/qaqc/rulesets", &workflow_rule_set).await;
    let rule_set_data: serde_json::Value = response.json();
    let rule_set_id = Uuid::parse_str(rule_set_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_rule_set(rule_set_id);

    // Simulate complete laboratory workflow with QC checkpoints
    let workflow_stages = vec![
        ("Sample Reception", QaqcFactory::create_reception_sample_data()),
        ("DNA Extraction", QaqcFactory::create_extraction_sample_data()),
        ("Quality Assessment", QaqcFactory::create_quality_assessment_data()),
        ("Library Preparation", QaqcFactory::create_library_prep_data()),
        ("Sequencing Ready", QaqcFactory::create_sequencing_ready_data()),
    ];

    let mut workflow_results = Vec::new();
    
    for (stage_name, stage_data) in workflow_stages {
        let qc_request = QcValidationRequest {
            rule_set_id,
            sample_data: stage_data,
            validation_level: ValidationLevel::Standard,
            options: Some(QcValidationOptions {
                fail_fast: false,
                generate_report: true,
                include_warnings: true,
                detailed_results: true,
            }),
        };
        
        let response = client.post_json("/api/qaqc/validate", &qc_request).await;
        QaqcAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
        
        let qc_data: serde_json::Value = response.json();
        let validation_id = Uuid::parse_str(qc_data["data"]["validation_id"].as_str().unwrap()).unwrap();
        test_db.track_validation(validation_id);
        
        let stage_result = WorkflowStageResult {
            stage: stage_name.to_string(),
            validation_id,
            status: qc_data["data"]["overall_status"].as_str().unwrap().to_string(),
            issues_count: qc_data["data"]["issues_count"].as_u64().unwrap() as usize,
        };
        
        workflow_results.push(stage_result);
    }

    // Verify workflow progression
    assert_eq!(workflow_results[0].status, "Pass"); // Sample Reception
    assert_eq!(workflow_results[1].status, "Pass"); // DNA Extraction
    assert_eq!(workflow_results[2].status, "Pass"); // Quality Assessment
    assert_eq!(workflow_results[3].status, "Pass"); // Library Preparation
    assert_eq!(workflow_results[4].status, "Pass"); // Sequencing Ready

    // Generate comprehensive workflow report
    let workflow_report_request = WorkflowReportRequest {
        workflow_id: Uuid::new_v4(),
        validation_ids: workflow_results.iter().map(|r| r.validation_id).collect(),
        include_recommendations: true,
        include_trend_analysis: true,
    };
    
    let response = client.post_json("/api/qaqc/workflow/report", &workflow_report_request).await;
    QaqcAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let workflow_report_data: serde_json::Value = response.json();
    QaqcAssertions::assert_workflow_report(&workflow_report_data);
    
    let report_id = Uuid::parse_str(workflow_report_data["data"]["report_id"].as_str().unwrap()).unwrap();
    test_db.track_report(report_id);

    // Download workflow report
    let response = client.get(&format!("/api/qaqc/reports/{}/download", report_id)).await;
    QaqcAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
    
    let report_content = response.bytes();
    assert!(report_content.len() > 5000, "Workflow report should be comprehensive");

    test_db.cleanup().await;
}

#[derive(Debug)]
struct WorkflowStageResult {
    stage: String,
    validation_id: Uuid,
    status: String,
    issues_count: usize,
}

#[derive(Debug)]
struct WorkflowReportRequest {
    workflow_id: Uuid,
    validation_ids: Vec<Uuid>,
    include_recommendations: bool,
    include_trend_analysis: bool,
}