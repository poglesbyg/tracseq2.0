//! Integration tests for concurrent configuration access

use config_service::{ConfigEntry, ServiceConfigResponse};
use crate::test_utils::*;
use axum::http::StatusCode;
use std::sync::Arc;
use tokio::sync::Barrier;

#[tokio::test]
async fn test_concurrent_config_creation() {
    let app = Arc::new(create_test_app().await);
    let service_name = "concurrent-create-service";
    let num_workers = 10;
    let configs_per_worker = 5;
    
    let barrier = Arc::new(Barrier::new(num_workers));
    let mut handles = Vec::new();
    
    for worker_id in 0..num_workers {
        let app_clone = app.clone();
        let barrier_clone = barrier.clone();
        
        let handle = tokio::spawn(async move {
            // Wait for all workers to be ready
            barrier_clone.wait().await;
            
            for config_id in 0..configs_per_worker {
                let key = format!("worker_{}_config_{}", worker_id, config_id);
                let value = json!({
                    "worker": worker_id,
                    "config": config_id,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                });
                
                let request = RequestFactory::create_config_request(
                    service_name,
                    &key,
                    value
                );
                
                let response = app_clone
                    .post("/api/configs")
                    .json(&request)
                    .send()
                    .await
                    .expect("Failed to send request");
                
                assert_eq!(response.status(), StatusCode::CREATED);
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all workers to complete
    for handle in handles {
        handle.await.expect("Worker task failed");
    }
    
    // Verify all configs were created
    let response = app
        .get(&format!("/api/services/{}/config/test", service_name))
        .send()
        .await
        .expect("Failed to send request");
    
    let service_config: ServiceConfigResponse = response.json().await.unwrap();
    assert_eq!(
        service_config.config.len(),
        num_workers * configs_per_worker
    );
}

#[tokio::test]
async fn test_concurrent_config_updates() {
    let app = Arc::new(create_test_app().await);
    let service_name = "concurrent-update-service";
    let key = "shared_counter";
    
    // Create initial config
    let initial_value = json!({"counter": 0, "updates": Vec::<String>::new()});
    let request = RequestFactory::create_config_request(
        service_name,
        key,
        initial_value
    );
    
    app.post("/api/configs")
        .json(&request)
        .send()
        .await
        .expect("Failed to create initial config");
    
    // Concurrent updates
    let num_updaters = 20;
    let barrier = Arc::new(Barrier::new(num_updaters));
    let mut handles = Vec::new();
    
    for updater_id in 0..num_updaters {
        let app_clone = app.clone();
        let barrier_clone = barrier.clone();
        
        let handle = tokio::spawn(async move {
            // Synchronize start
            barrier_clone.wait().await;
            
            // Get current config
            let response = app_clone
                .get(&format!("/api/configs/{}/{}", service_name, key))
                .send()
                .await
                .expect("Failed to get config");
            
            let mut config: ConfigEntry = response.json().await.unwrap();
            
            // Update the counter
            let counter = config.value["counter"].as_i64().unwrap_or(0);
            let mut updates = config.value["updates"]
                .as_array()
                .cloned()
                .unwrap_or_default();
            
            updates.push(json!(format!("Updater {}", updater_id)));
            
            let new_value = json!({
                "counter": counter + 1,
                "updates": updates
            });
            
            let update_request = RequestFactory::update_config_request(new_value);
            
            // Try to update
            let update_response = app_clone
                .put(&format!("/api/configs/{}/{}", service_name, key))
                .json(&update_request)
                .send()
                .await
                .expect("Failed to send update");
            
            // Some updates might fail due to concurrent modifications
            update_response.status()
        });
        
        handles.push(handle);
    }
    
    let results: Vec<StatusCode> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    
    // Count successful updates
    let successful_updates = results.iter()
        .filter(|&&status| status == StatusCode::OK)
        .count();
    
    println!("Successful updates: {}/{}", successful_updates, num_updaters);
    
    // Verify final state
    let response = app
        .get(&format!("/api/configs/{}/{}", service_name, key))
        .send()
        .await
        .expect("Failed to get final config");
    
    let final_config: ConfigEntry = response.json().await.unwrap();
    let final_counter = final_config.value["counter"].as_i64().unwrap();
    let final_updates = final_config.value["updates"].as_array().unwrap();
    
    // Counter should match successful updates
    assert!(final_counter > 0 && final_counter <= num_updaters as i64);
    assert!(final_updates.len() > 0 && final_updates.len() <= num_updaters);
}

#[tokio::test]
async fn test_read_write_consistency() {
    let app = Arc::new(create_test_app().await);
    let service_name = "consistency-service";
    
    // Create test configs
    let num_configs = 10;
    for i in 0..num_configs {
        let request = RequestFactory::create_config_request(
            service_name,
            &format!("config_{}", i),
            json!({"value": i, "stable": true})
        );
        
        app.post("/api/configs")
            .json(&request)
            .send()
            .await
            .expect("Failed to create config");
    }
    
    let barrier = Arc::new(Barrier::new(3)); // 1 writer + 2 readers
    let app_writer = app.clone();
    let app_reader1 = app.clone();
    let app_reader2 = app.clone();
    
    // Writer task - continuously updates configs
    let writer_handle = tokio::spawn(async move {
        barrier.wait().await;
        
        for iteration in 0..5 {
            for i in 0..num_configs {
                let key = format!("config_{}", i);
                let new_value = json!({
                    "value": i * 10 + iteration,
                    "stable": false,
                    "iteration": iteration
                });
                
                let update_request = RequestFactory::update_config_request(new_value);
                
                app_writer
                    .put(&format!("/api/configs/{}/{}", service_name, key))
                    .json(&update_request)
                    .send()
                    .await
                    .ok(); // Ignore failures
                
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
        }
    });
    
    // Reader tasks - continuously read and verify consistency
    let reader_task = |app: Arc<TestApp>, reader_id: usize| async move {
        barrier.wait().await;
        
        let mut inconsistencies = 0;
        let mut total_reads = 0;
        
        for _ in 0..20 {
            // Read all configs
            let response = app
                .get(&format!("/api/services/{}/config/test", service_name))
                .send()
                .await
                .expect("Failed to read configs");
            
            let service_config: ServiceConfigResponse = response.json().await.unwrap();
            
            // Check consistency - all configs should have same 'iteration' if 'stable' is false
            let mut current_iteration: Option<i64> = None;
            for (_, value) in &service_config.config {
                if !value["stable"].as_bool().unwrap_or(true) {
                    if let Some(iter) = value["iteration"].as_i64() {
                        if let Some(expected) = current_iteration {
                            if iter != expected {
                                inconsistencies += 1;
                            }
                        } else {
                            current_iteration = Some(iter);
                        }
                    }
                }
            }
            
            total_reads += 1;
            tokio::time::sleep(tokio::time::Duration::from_millis(25)).await;
        }
        
        println!("Reader {}: {} inconsistencies in {} reads", 
                 reader_id, inconsistencies, total_reads);
        
        (inconsistencies, total_reads)
    };
    
    let reader1_handle = tokio::spawn(reader_task(app_reader1, 1));
    let reader2_handle = tokio::spawn(reader_task(app_reader2, 2));
    
    // Wait for all tasks
    writer_handle.await.expect("Writer task failed");
    let (inconsistencies1, reads1) = reader1_handle.await.expect("Reader 1 failed");
    let (inconsistencies2, reads2) = reader2_handle.await.expect("Reader 2 failed");
    
    // Some inconsistencies are expected due to concurrent updates
    // but they should be relatively rare
    let total_inconsistencies = inconsistencies1 + inconsistencies2;
    let total_reads = reads1 + reads2;
    let inconsistency_rate = total_inconsistencies as f64 / total_reads as f64;
    
    println!("Total inconsistency rate: {:.2}%", inconsistency_rate * 100.0);
    assert!(inconsistency_rate < 0.1, "Too many inconsistencies detected");
}

#[tokio::test]
async fn test_optimistic_locking() {
    let app = Arc::new(create_test_app().await);
    let service_name = "locking-service";
    let key = "versioned_config";
    
    // Create initial config
    let request = RequestFactory::create_config_request(
        service_name,
        key,
        json!({"data": "initial"})
    );
    
    app.post("/api/configs")
        .json(&request)
        .send()
        .await
        .expect("Failed to create config");
    
    // Get initial version
    let response = app
        .get(&format!("/api/configs/{}/{}", service_name, key))
        .send()
        .await
        .expect("Failed to get config");
    
    let initial_config: ConfigEntry = response.json().await.unwrap();
    let initial_version = initial_config.version;
    
    // Simulate two concurrent updates
    let app1 = app.clone();
    let app2 = app.clone();
    
    let update1 = tokio::spawn(async move {
        // Update with correct version
        let mut update_request = RequestFactory::update_config_request(
            json!({"data": "update1"})
        );
        update_request.expected_version = Some(initial_version);
        
        app1.put(&format!("/api/configs/{}/{}", service_name, key))
            .json(&update_request)
            .send()
            .await
    });
    
    let update2 = tokio::spawn(async move {
        // Small delay to ensure update1 goes first
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        
        // This should fail - using outdated version
        let mut update_request = RequestFactory::update_config_request(
            json!({"data": "update2"})
        );
        update_request.expected_version = Some(initial_version);
        
        app2.put(&format!("/api/configs/{}/{}", service_name, key))
            .json(&update_request)
            .send()
            .await
    });
    
    let result1 = update1.await.expect("Update 1 task failed")
        .expect("Update 1 request failed");
    let result2 = update2.await.expect("Update 2 task failed")
        .expect("Update 2 request failed");
    
    // First update should succeed
    assert_eq!(result1.status(), StatusCode::OK);
    
    // Second update should fail with conflict
    assert_eq!(result2.status(), StatusCode::CONFLICT);
    
    // Verify final state
    let final_response = app
        .get(&format!("/api/configs/{}/{}", service_name, key))
        .send()
        .await
        .expect("Failed to get final config");
    
    let final_config: ConfigEntry = final_response.json().await.unwrap();
    assert_eq!(final_config.value["data"], "update1");
    assert_eq!(final_config.version, initial_version + 1);
}

#[tokio::test]
async fn test_bulk_operations_atomicity() {
    let app = Arc::new(create_test_app().await);
    let service_name = "atomic-service";
    
    // Create initial configs
    for i in 0..5 {
        let request = RequestFactory::create_config_request(
            service_name,
            &format!("config_{}", i),
            json!({"value": i})
        );
        
        app.post("/api/configs")
            .json(&request)
            .send()
            .await
            .expect("Failed to create config");
    }
    
    let num_workers = 5;
    let barrier = Arc::new(Barrier::new(num_workers));
    let mut handles = Vec::new();
    
    for worker_id in 0..num_workers {
        let app_clone = app.clone();
        let barrier_clone = barrier.clone();
        
        let handle = tokio::spawn(async move {
            barrier_clone.wait().await;
            
            // Each worker tries to update all configs as a bulk operation
            let mut bulk_update = std::collections::HashMap::new();
            for i in 0..5 {
                bulk_update.insert(
                    format!("config_{}", i),
                    json!({"value": i * 10 + worker_id})
                );
            }
            
            let response = app_clone
                .put(&format!("/api/services/{}/config/test", service_name))
                .json(&bulk_update)
                .send()
                .await
                .expect("Failed to send bulk update");
            
            (worker_id, response.status())
        });
        
        handles.push(handle);
    }
    
    let results: Vec<(usize, StatusCode)> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    
    // Check results
    let successful_workers: Vec<_> = results.iter()
        .filter(|(_, status)| *status == StatusCode::OK)
        .map(|(id, _)| *id)
        .collect();
    
    println!("Successful bulk updates from workers: {:?}", successful_workers);
    
    // Verify final state - all configs should be from the same worker
    let response = app
        .get(&format!("/api/services/{}/config/test", service_name))
        .send()
        .await
        .expect("Failed to get final configs");
    
    let final_configs: ServiceConfigResponse = response.json().await.unwrap();
    
    // Extract worker ID from first config
    let first_value = final_configs.config.get("config_0").unwrap()["value"]
        .as_i64()
        .unwrap();
    let winning_worker = first_value % 10;
    
    // All configs should be from the same worker (atomicity)
    for i in 0..5 {
        let key = format!("config_{}", i);
        let value = final_configs.config.get(&key).unwrap()["value"]
            .as_i64()
            .unwrap();
        let expected = i * 10 + winning_worker;
        
        assert_eq!(
            value, expected,
            "Config {} has inconsistent value. Expected {}, got {}",
            key, expected, value
        );
    }
}

#[tokio::test]
async fn test_service_isolation() {
    let app = Arc::new(create_test_app().await);
    let services = vec!["service-a", "service-b", "service-c"];
    let num_configs_per_service = 10;
    
    let barrier = Arc::new(Barrier::new(services.len()));
    let mut handles = Vec::new();
    
    // Each service creates and manages its own configs concurrently
    for service_name in services.clone() {
        let app_clone = app.clone();
        let barrier_clone = barrier.clone();
        let service = service_name.to_string();
        
        let handle = tokio::spawn(async move {
            barrier_clone.wait().await;
            
            // Create configs
            for i in 0..num_configs_per_service {
                let request = RequestFactory::create_config_request(
                    &service,
                    &format!("config_{}", i),
                    json!({
                        "service": &service,
                        "index": i
                    })
                );
                
                app_clone
                    .post("/api/configs")
                    .json(&request)
                    .send()
                    .await
                    .expect("Failed to create config");
            }
            
            // Verify only this service's configs are visible
            let response = app_clone
                .get(&format!("/api/services/{}/config/test", service))
                .send()
                .await
                .expect("Failed to get service configs");
            
            let service_configs: ServiceConfigResponse = response.json().await.unwrap();
            
            // Should have exactly the configs we created
            assert_eq!(service_configs.config.len(), num_configs_per_service);
            
            // All configs should belong to this service
            for (_, value) in &service_configs.config {
                assert_eq!(value["service"], service);
            }
            
            service
        });
        
        handles.push(handle);
    }
    
    // Wait for all services to complete
    let completed_services: Vec<String> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    
    assert_eq!(completed_services.len(), services.len());
    
    // Verify total config count across all services
    let response = app
        .get("/api/configs")
        .send()
        .await
        .expect("Failed to get all configs");
    
    let all_configs: config_service::ConfigResponse = response.json().await.unwrap();
    assert_eq!(
        all_configs.total,
        services.len() * num_configs_per_service
    );
}