use crate::test_utils::*;
use spreadsheet_versioning_service::{
    models::*,
    handlers::*,
    services::*,
    create_app,
};
use axum_test::TestServer;
use serde_json::json;
use uuid::Uuid;

/// Integration tests for complete spreadsheet version control workflows
#[tokio::test]
async fn test_complete_spreadsheet_lifecycle() {
    let mut test_db = TestDatabase::new().await;
    let app = create_app().await;
    let client = SpreadsheetTestClient::new(app);

    // Phase 1: Create initial spreadsheet
    let spreadsheet_request = SpreadsheetFactory::create_valid_spreadsheet_request();
    let spreadsheet_name = spreadsheet_request.name.clone();
    
    let response = client.post_json("/api/spreadsheets", &spreadsheet_request).await;
    SpreadsheetAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let spreadsheet_data: serde_json::Value = response.json();
    SpreadsheetAssertions::assert_spreadsheet_data(&spreadsheet_data, &spreadsheet_name);
    
    let spreadsheet_id = Uuid::parse_str(spreadsheet_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_spreadsheet(spreadsheet_id);

    // Phase 2: Initialize with sample data
    let initial_data = SpreadsheetFactory::create_sample_data();
    let data_request = json!({
        "spreadsheet_id": spreadsheet_id,
        "data": initial_data,
        "sheet_name": "Sheet1",
        "author_id": Uuid::new_v4(),
        "commit_message": "Initial data import"
    });
    
    let response = client.post_json("/api/spreadsheets/data", &data_request).await;
    SpreadsheetAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let data_response: serde_json::Value = response.json();
    SpreadsheetAssertions::assert_data_update(&data_response);
    
    let version_1 = Uuid::parse_str(data_response["data"]["version_id"].as_str().unwrap()).unwrap();
    test_db.track_version(version_1);

    // Phase 3: Create multiple versions with different changes
    let changes = vec![
        ("Add new samples", SpreadsheetFactory::create_additional_rows(5)),
        ("Update concentrations", SpreadsheetFactory::create_column_updates("concentration", 10)),
        ("Add QC results", SpreadsheetFactory::create_new_columns(vec!["qc_status", "qc_notes"])),
        ("Fix validation errors", SpreadsheetFactory::create_cell_corrections(3)),
    ];

    let mut version_ids = vec![version_1];
    
    for (commit_message, change_data) in changes {
        let change_request = json!({
            "spreadsheet_id": spreadsheet_id,
            "changes": change_data,
            "author_id": Uuid::new_v4(),
            "commit_message": commit_message,
            "parent_version": version_ids.last().unwrap()
        });
        
        let response = client.post_json("/api/spreadsheets/changes", &change_request).await;
        SpreadsheetAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
        
        let change_response: serde_json::Value = response.json();
        let version_id = Uuid::parse_str(change_response["data"]["version_id"].as_str().unwrap()).unwrap();
        version_ids.push(version_id);
        test_db.track_version(version_id);
    }

    // Phase 4: Query version history
    let response = client.get(&format!("/api/spreadsheets/{}/history", spreadsheet_id)).await;
    SpreadsheetAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
    
    let history_data: serde_json::Value = response.json();
    SpreadsheetAssertions::assert_version_history(&history_data, 5); // Initial + 4 changes
    
    let versions = history_data["data"]["versions"].as_array().unwrap();
    assert_eq!(versions.len(), 5);

    // Phase 5: Compare versions
    let compare_request = json!({
        "from_version": version_ids[0],
        "to_version": version_ids.last().unwrap(),
        "include_detailed_diff": true
    });
    
    let response = client.post_json("/api/spreadsheets/compare", &compare_request).await;
    SpreadsheetAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
    
    let diff_data: serde_json::Value = response.json();
    SpreadsheetAssertions::assert_version_diff(&diff_data);
    
    // Verify diff contains expected changes
    assert!(diff_data["data"]["added_rows"].as_u64().unwrap() >= 5);
    assert!(diff_data["data"]["modified_cells"].as_u64().unwrap() >= 10);
    assert!(diff_data["data"]["added_columns"].as_u64().unwrap() >= 2);

    // Phase 6: Revert to previous version
    let revert_request = json!({
        "spreadsheet_id": spreadsheet_id,
        "target_version": version_ids[2], // Revert to version 3
        "author_id": Uuid::new_v4(),
        "commit_message": "Revert to version with QC results only"
    });
    
    let response = client.post_json("/api/spreadsheets/revert", &revert_request).await;
    SpreadsheetAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let revert_response: serde_json::Value = response.json();
    let revert_version_id = Uuid::parse_str(revert_response["data"]["version_id"].as_str().unwrap()).unwrap();
    test_db.track_version(revert_version_id);

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_collaborative_editing_workflow() {
    let mut test_db = TestDatabase::new().await;
    let app = create_app().await;
    let client = SpreadsheetTestClient::new(app);

    // Create shared spreadsheet
    let shared_spreadsheet = SpreadsheetFactory::create_collaborative_spreadsheet();
    let response = client.post_json("/api/spreadsheets", &shared_spreadsheet).await;
    let spreadsheet_data: serde_json::Value = response.json();
    let spreadsheet_id = Uuid::parse_str(spreadsheet_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_spreadsheet(spreadsheet_id);

    // Create multiple users
    let users = vec![
        ("Alice", Uuid::new_v4()),
        ("Bob", Uuid::new_v4()),
        ("Charlie", Uuid::new_v4()),
    ];

    // Grant permissions to users
    for (username, user_id) in &users {
        let permission_request = json!({
            "spreadsheet_id": spreadsheet_id,
            "user_id": user_id,
            "permission_level": "Editor",
            "granted_by": users[0].1 // Alice grants permissions
        });
        
        let response = client.post_json("/api/spreadsheets/permissions", &permission_request).await;
        SpreadsheetAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    }

    // Initialize spreadsheet with base data
    let base_data = SpreadsheetFactory::create_laboratory_data();
    let init_request = json!({
        "spreadsheet_id": spreadsheet_id,
        "data": base_data,
        "sheet_name": "Laboratory_Data",
        "author_id": users[0].1,
        "commit_message": "Initial laboratory data setup"
    });
    
    let response = client.post_json("/api/spreadsheets/data", &init_request).await;
    let init_response: serde_json::Value = response.json();
    let base_version = Uuid::parse_str(init_response["data"]["version_id"].as_str().unwrap()).unwrap();
    test_db.track_version(base_version);

    // Simulate concurrent editing by multiple users
    let mut concurrent_changes = Vec::new();
    
    // Alice adds sample information
    concurrent_changes.push((
        users[0].1,
        "Alice",
        "Add sample metadata",
        SpreadsheetFactory::create_sample_metadata_changes(),
    ));
    
    // Bob updates analysis results
    concurrent_changes.push((
        users[1].1,
        "Bob", 
        "Update analysis results",
        SpreadsheetFactory::create_analysis_results_changes(),
    ));
    
    // Charlie adds quality control data
    concurrent_changes.push((
        users[2].1,
        "Charlie",
        "Add QC data",
        SpreadsheetFactory::create_qc_data_changes(),
    ));

    // Apply changes concurrently
    let mut change_futures = Vec::new();
    for (user_id, username, message, changes) in concurrent_changes {
        let change_request = json!({
            "spreadsheet_id": spreadsheet_id,
            "changes": changes,
            "author_id": user_id,
            "commit_message": message,
            "parent_version": base_version
        });
        
        let future = client.post_json("/api/spreadsheets/changes", &change_request);
        change_futures.push(future);
    }

    // Wait for all changes to complete
    let mut change_results = Vec::new();
    for future in change_futures {
        let response = future.await;
        let result: serde_json::Value = response.json();
        change_results.push(result);
    }

    // Verify conflict resolution
    let mut successful_changes = 0;
    let mut conflicted_changes = 0;
    
    for result in &change_results {
        match result["data"]["status"].as_str().unwrap() {
            "Applied" => successful_changes += 1,
            "Conflicted" => conflicted_changes += 1,
            _ => {}
        }
    }

    // At least one change should succeed, others may conflict
    assert!(successful_changes >= 1, "At least one concurrent change should succeed");
    
    // Handle conflicts by creating merge proposals
    for result in &change_results {
        if result["data"]["status"] == "Conflicted" {
            let conflict_id = result["data"]["conflict_id"].as_str().unwrap();
            let resolution_request = json!({
                "conflict_id": conflict_id,
                "resolution_strategy": "AutoMerge",
                "resolver_id": users[0].1, // Alice resolves conflicts
                "notes": "Auto-merge non-overlapping changes"
            });
            
            let response = client.post_json("/api/spreadsheets/conflicts/resolve", &resolution_request).await;
            SpreadsheetAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
            
            let resolution_data: serde_json::Value = response.json();
            if let Some(version_id) = resolution_data["data"]["merged_version_id"].as_str() {
                let merged_version = Uuid::parse_str(version_id).unwrap();
                test_db.track_version(merged_version);
            }
        }
    }

    // Test real-time collaboration features
    let collaboration_request = json!({
        "spreadsheet_id": spreadsheet_id,
        "user_id": users[1].1,
        "enable_real_time_sync": true,
        "cursor_tracking": true
    });
    
    let response = client.post_json("/api/spreadsheets/collaboration/join", &collaboration_request).await;
    SpreadsheetAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
    
    let collaboration_data: serde_json::Value = response.json();
    SpreadsheetAssertions::assert_collaboration_session(&collaboration_data);

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_advanced_version_control_features() {
    let mut test_db = TestDatabase::new().await;
    let app = create_app().await;
    let client = SpreadsheetTestClient::new(app);

    // Create spreadsheet with branching support
    let branching_spreadsheet = CreateSpreadsheetRequest {
        name: "Branching Test Spreadsheet".to_string(),
        description: Some("Test spreadsheet for branching and merging".to_string()),
        category: SpreadsheetCategory::DataAnalysis,
        template_id: None,
        owner_id: Uuid::new_v4(),
        collaboration_settings: Some(CollaborationSettings {
            allow_branching: true,
            allow_forking: true,
            require_review: true,
            auto_merge_threshold: 0.95,
        }),
        schema: Some(SpreadsheetFactory::create_laboratory_schema()),
    };
    
    let response = client.post_json("/api/spreadsheets", &branching_spreadsheet).await;
    let spreadsheet_data: serde_json::Value = response.json();
    let spreadsheet_id = Uuid::parse_str(spreadsheet_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_spreadsheet(spreadsheet_id);

    // Create main branch with initial data
    let main_data = SpreadsheetFactory::create_comprehensive_data();
    let main_init_request = json!({
        "spreadsheet_id": spreadsheet_id,
        "data": main_data,
        "branch_name": "main",
        "author_id": Uuid::new_v4(),
        "commit_message": "Initial main branch setup"
    });
    
    let response = client.post_json("/api/spreadsheets/branches/commit", &main_init_request).await;
    let main_response: serde_json::Value = response.json();
    let main_version = Uuid::parse_str(main_response["data"]["version_id"].as_str().unwrap()).unwrap();
    test_db.track_version(main_version);

    // Create feature branches
    let branches = vec![
        ("feature/sample-tracking", "Enhanced sample tracking features"),
        ("feature/analysis-pipeline", "New analysis pipeline integration"),
        ("hotfix/data-validation", "Critical data validation fixes"),
    ];

    let mut branch_versions = Vec::new();
    
    for (branch_name, description) in branches {
        let branch_request = json!({
            "spreadsheet_id": spreadsheet_id,
            "branch_name": branch_name,
            "source_version": main_version,
            "description": description,
            "author_id": Uuid::new_v4()
        });
        
        let response = client.post_json("/api/spreadsheets/branches", &branch_request).await;
        SpreadsheetAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
        
        let branch_data: serde_json::Value = response.json();
        let branch_id = branch_data["data"]["branch_id"].as_str().unwrap();
        
        // Make changes in each branch
        let branch_changes = match branch_name {
            "feature/sample-tracking" => SpreadsheetFactory::create_sample_tracking_changes(),
            "feature/analysis-pipeline" => SpreadsheetFactory::create_analysis_pipeline_changes(),
            "hotfix/data-validation" => SpreadsheetFactory::create_validation_fixes(),
            _ => json!({}),
        };
        
        let commit_request = json!({
            "spreadsheet_id": spreadsheet_id,
            "branch_name": branch_name,
            "changes": branch_changes,
            "author_id": Uuid::new_v4(),
            "commit_message": format!("Implement {}", description)
        });
        
        let response = client.post_json("/api/spreadsheets/branches/commit", &commit_request).await;
        let commit_response: serde_json::Value = response.json();
        let branch_version = Uuid::parse_str(commit_response["data"]["version_id"].as_str().unwrap()).unwrap();
        
        branch_versions.push((branch_name.to_string(), branch_version));
        test_db.track_version(branch_version);
    }

    // Create merge requests
    for (branch_name, branch_version) in &branch_versions {
        let merge_request = json!({
            "spreadsheet_id": spreadsheet_id,
            "source_branch": branch_name,
            "target_branch": "main",
            "source_version": branch_version,
            "target_version": main_version,
            "title": format!("Merge {} into main", branch_name),
            "description": format!("Merge request for {}", branch_name),
            "author_id": Uuid::new_v4()
        });
        
        let response = client.post_json("/api/spreadsheets/merge-requests", &merge_request).await;
        SpreadsheetAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
        
        let mr_data: serde_json::Value = response.json();
        SpreadsheetAssertions::assert_merge_request(&mr_data);
        
        let mr_id = Uuid::parse_str(mr_data["data"]["id"].as_str().unwrap()).unwrap();
        test_db.track_merge_request(mr_id);
    }

    // Test merge conflict detection and resolution
    let hotfix_branch = &branch_versions[2]; // hotfix branch
    let merge_analysis_request = json!({
        "spreadsheet_id": spreadsheet_id,
        "source_version": hotfix_branch.1,
        "target_version": main_version,
        "analyze_conflicts": true,
        "suggest_resolution": true
    });
    
    let response = client.post_json("/api/spreadsheets/merge/analyze", &merge_analysis_request).await;
    SpreadsheetAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
    
    let analysis_data: serde_json::Value = response.json();
    SpreadsheetAssertions::assert_merge_analysis(&analysis_data);

    // Perform automatic merge of hotfix (assuming no conflicts)
    let auto_merge_request = json!({
        "spreadsheet_id": spreadsheet_id,
        "source_version": hotfix_branch.1,
        "target_version": main_version,
        "merge_strategy": "AutoMerge",
        "author_id": Uuid::new_v4(),
        "commit_message": "Auto-merge hotfix into main"
    });
    
    let response = client.post_json("/api/spreadsheets/merge", &auto_merge_request).await;
    SpreadsheetAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let merge_response: serde_json::Value = response.json();
    let merged_version = Uuid::parse_str(merge_response["data"]["merged_version_id"].as_str().unwrap()).unwrap();
    test_db.track_version(merged_version);

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_data_validation_and_schema_evolution() {
    let mut test_db = TestDatabase::new().await;
    let app = create_app().await;
    let client = SpreadsheetTestClient::new(app);

    // Create spreadsheet with strict schema
    let schema_spreadsheet = CreateSpreadsheetRequest {
        name: "Schema Evolution Test".to_string(),
        description: Some("Test spreadsheet for schema validation and evolution".to_string()),
        category: SpreadsheetCategory::QualityControl,
        template_id: None,
        owner_id: Uuid::new_v4(),
        collaboration_settings: Some(CollaborationSettings {
            allow_branching: false,
            allow_forking: false,
            require_review: false,
            auto_merge_threshold: 1.0,
        }),
        schema: Some(SpreadsheetFactory::create_strict_schema()),
    };
    
    let response = client.post_json("/api/spreadsheets", &schema_spreadsheet).await;
    let spreadsheet_data: serde_json::Value = response.json();
    let spreadsheet_id = Uuid::parse_str(spreadsheet_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_spreadsheet(spreadsheet_id);

    // Test valid data insertion
    let valid_data = SpreadsheetFactory::create_schema_compliant_data();
    let valid_request = json!({
        "spreadsheet_id": spreadsheet_id,
        "data": valid_data,
        "validate_schema": true,
        "author_id": Uuid::new_v4(),
        "commit_message": "Add schema-compliant data"
    });
    
    let response = client.post_json("/api/spreadsheets/data", &valid_request).await;
    SpreadsheetAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let valid_response: serde_json::Value = response.json();
    let valid_version = Uuid::parse_str(valid_response["data"]["version_id"].as_str().unwrap()).unwrap();
    test_db.track_version(valid_version);

    // Test invalid data rejection
    let invalid_data = SpreadsheetFactory::create_schema_violating_data();
    let invalid_request = json!({
        "spreadsheet_id": spreadsheet_id,
        "data": invalid_data,
        "validate_schema": true,
        "author_id": Uuid::new_v4(),
        "commit_message": "Attempt to add invalid data"
    });
    
    let response = client.post_json("/api/spreadsheets/data", &invalid_request).await;
    SpreadsheetAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::BAD_REQUEST);
    
    let error_response: serde_json::Value = response.json();
    SpreadsheetAssertions::assert_validation_errors(&error_response);

    // Test schema evolution
    let schema_evolution_request = json!({
        "spreadsheet_id": spreadsheet_id,
        "schema_updates": {
            "add_columns": [
                {
                    "name": "quality_score",
                    "type": "number",
                    "constraints": {
                        "min": 0.0,
                        "max": 100.0
                    }
                },
                {
                    "name": "notes",
                    "type": "text",
                    "constraints": {
                        "max_length": 500
                    }
                }
            ],
            "modify_columns": [
                {
                    "name": "concentration",
                    "new_constraints": {
                        "min": 0.0,
                        "max": 1000.0,
                        "required": true
                    }
                }
            ]
        },
        "migration_strategy": "AddDefaults",
        "author_id": Uuid::new_v4(),
        "commit_message": "Evolve schema to support quality scoring"
    });
    
    let response = client.post_json("/api/spreadsheets/schema/evolve", &schema_evolution_request).await;
    SpreadsheetAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let evolution_response: serde_json::Value = response.json();
    SpreadsheetAssertions::assert_schema_evolution(&evolution_response);
    
    let evolved_version = Uuid::parse_str(evolution_response["data"]["version_id"].as_str().unwrap()).unwrap();
    test_db.track_version(evolved_version);

    // Test data validation with evolved schema
    let evolved_data = SpreadsheetFactory::create_evolved_schema_data();
    let evolved_request = json!({
        "spreadsheet_id": spreadsheet_id,
        "data": evolved_data,
        "validate_schema": true,
        "author_id": Uuid::new_v4(),
        "commit_message": "Add data with new schema fields"
    });
    
    let response = client.post_json("/api/spreadsheets/data", &evolved_request).await;
    SpreadsheetAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);

    // Test rollback to previous schema version
    let rollback_request = json!({
        "spreadsheet_id": spreadsheet_id,
        "target_schema_version": valid_version,
        "migration_strategy": "RemoveColumns",
        "author_id": Uuid::new_v4(),
        "commit_message": "Rollback schema to previous version"
    });
    
    let response = client.post_json("/api/spreadsheets/schema/rollback", &rollback_request).await;
    SpreadsheetAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let rollback_response: serde_json::Value = response.json();
    let rollback_version = Uuid::parse_str(rollback_response["data"]["version_id"].as_str().unwrap()).unwrap();
    test_db.track_version(rollback_version);

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_performance_and_large_dataset_handling() {
    let mut test_db = TestDatabase::new().await;
    let app = create_app().await;
    let client = SpreadsheetTestClient::new(app);

    // Create performance test spreadsheet
    let performance_spreadsheet = SpreadsheetFactory::create_performance_test_spreadsheet();
    let response = client.post_json("/api/spreadsheets", &performance_spreadsheet).await;
    let spreadsheet_data: serde_json::Value = response.json();
    let spreadsheet_id = Uuid::parse_str(spreadsheet_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_spreadsheet(spreadsheet_id);

    // Test large dataset insertion
    let large_dataset_size = 10000;
    let large_dataset = SpreadsheetFactory::create_large_dataset(large_dataset_size);
    
    let start_time = std::time::Instant::now();
    let large_data_request = json!({
        "spreadsheet_id": spreadsheet_id,
        "data": large_dataset,
        "author_id": Uuid::new_v4(),
        "commit_message": format!("Add large dataset with {} rows", large_dataset_size),
        "batch_size": 1000 // Process in batches
    });
    
    let response = client.post_json("/api/spreadsheets/data/bulk", &large_data_request).await;
    let insertion_time = start_time.elapsed();
    
    SpreadsheetAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let bulk_response: serde_json::Value = response.json();
    let bulk_version = Uuid::parse_str(bulk_response["data"]["version_id"].as_str().unwrap()).unwrap();
    test_db.track_version(bulk_version);

    // Performance assertions
    assert!(insertion_time.as_secs() < 60, "Large dataset insertion should complete within 60 seconds");
    
    let rows_per_second = large_dataset_size as f64 / insertion_time.as_secs_f64();
    assert!(rows_per_second >= 100.0, "Should process at least 100 rows per second");

    // Test concurrent operations on large dataset
    let concurrent_operations = vec![
        ("Filter operation", SpreadsheetFactory::create_filter_operation()),
        ("Sort operation", SpreadsheetFactory::create_sort_operation()),
        ("Aggregate operation", SpreadsheetFactory::create_aggregate_operation()),
        ("Search operation", SpreadsheetFactory::create_search_operation()),
    ];

    let mut operation_futures = Vec::new();
    for (operation_name, operation_data) in concurrent_operations {
        let operation_request = json!({
            "spreadsheet_id": spreadsheet_id,
            "version_id": bulk_version,
            "operation": operation_data,
            "operation_name": operation_name
        });
        
        let future = client.post_json("/api/spreadsheets/operations", &operation_request);
        operation_futures.push((operation_name, future));
    }

    // Wait for all operations to complete
    let mut operation_times = Vec::new();
    for (operation_name, future) in operation_futures {
        let operation_start = std::time::Instant::now();
        let response = future.await;
        let operation_time = operation_start.elapsed();
        
        SpreadsheetAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
        operation_times.push((operation_name, operation_time));
    }

    // Verify operation performance
    for (operation_name, operation_time) in operation_times {
        assert!(operation_time.as_secs() < 30, "{} should complete within 30 seconds", operation_name);
    }

    // Test memory-efficient version diff for large datasets
    let diff_start = std::time::Instant::now();
    let diff_request = json!({
        "spreadsheet_id": spreadsheet_id,
        "from_version": bulk_version,
        "to_version": bulk_version, // Same version (no changes)
        "streaming_diff": true,
        "chunk_size": 1000
    });
    
    let response = client.post_json("/api/spreadsheets/diff/streaming", &diff_request).await;
    let diff_time = diff_start.elapsed();
    
    SpreadsheetAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
    assert!(diff_time.as_secs() < 15, "Streaming diff should complete within 15 seconds");

    // Test incremental changes on large dataset
    let incremental_changes = SpreadsheetFactory::create_incremental_changes(100);
    let incremental_request = json!({
        "spreadsheet_id": spreadsheet_id,
        "changes": incremental_changes,
        "parent_version": bulk_version,
        "author_id": Uuid::new_v4(),
        "commit_message": "Incremental updates to large dataset",
        "optimize_for_size": true
    });
    
    let response = client.post_json("/api/spreadsheets/changes", &incremental_request).await;
    SpreadsheetAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let incremental_response: serde_json::Value = response.json();
    let incremental_version = Uuid::parse_str(incremental_response["data"]["version_id"].as_str().unwrap()).unwrap();
    test_db.track_version(incremental_version);

    // Test export performance
    let export_formats = vec!["CSV", "XLSX", "JSON"];
    for format in export_formats {
        let export_start = std::time::Instant::now();
        let export_request = json!({
            "spreadsheet_id": spreadsheet_id,
            "version_id": incremental_version,
            "format": format,
            "streaming": true
        });
        
        let response = client.post_json("/api/spreadsheets/export", &export_request).await;
        let export_time = export_start.elapsed();
        
        SpreadsheetAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
        assert!(export_time.as_secs() < 45, "{} export should complete within 45 seconds", format);
        
        let export_data = response.bytes();
        assert!(export_data.len() > 100000, "{} export should produce substantial output", format);
    }

    test_db.cleanup().await;
}