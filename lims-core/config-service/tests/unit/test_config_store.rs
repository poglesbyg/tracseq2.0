//! Unit tests for configuration store operations

use config_service::{AppState, ConfigEntry};
use crate::test_utils::*;
use std::collections::HashMap;

config_test!(test_create_and_retrieve_config, |state: AppState| async move {
    let config = ConfigFactory::create_test_config(
        "test-service",
        "test-key",
        serde_json::json!("test-value")
    );
    
    // Insert config
    {
        let mut store = state.config_store.write().await;
        store.insert("test-service-test-key".to_string(), config.clone());
    }
    
    // Retrieve and verify
    {
        let store = state.config_store.read().await;
        let retrieved = store.get("test-service-test-key");
        assert!(retrieved.is_some());
        ConfigAssertions::assert_config_equal(retrieved.unwrap(), &config);
    }
});

config_test!(test_update_config_version, |state: AppState| async move {
    let mut config = ConfigFactory::create_test_config(
        "update-service",
        "update-key",
        serde_json::json!(100)
    );
    
    let original_version = config.version;
    let original_updated_at = config.updated_at;
    
    // Insert original
    {
        let mut store = state.config_store.write().await;
        store.insert("update-service-update-key".to_string(), config.clone());
    }
    
    // Simulate update
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    
    // Update config
    {
        let mut store = state.config_store.write().await;
        if let Some(stored_config) = store.get_mut("update-service-update-key") {
            stored_config.value = serde_json::json!(200);
            stored_config.version += 1;
            stored_config.updated_at = chrono::Utc::now();
            config = stored_config.clone();
        }
    }
    
    assert_eq!(config.version, original_version + 1);
    assert!(config.updated_at > original_updated_at);
    assert_eq!(config.value, serde_json::json!(200));
});

config_test!(test_delete_config, |state: AppState| async move {
    let config = ConfigFactory::create_test_config(
        "delete-service",
        "delete-key",
        serde_json::json!("to-be-deleted")
    );
    
    // Insert config
    {
        let mut store = state.config_store.write().await;
        store.insert("delete-service-delete-key".to_string(), config);
    }
    
    // Verify it exists
    {
        let store = state.config_store.read().await;
        assert!(store.contains_key("delete-service-delete-key"));
    }
    
    // Delete config
    {
        let mut store = state.config_store.write().await;
        let removed = store.remove("delete-service-delete-key");
        assert!(removed.is_some());
    }
    
    // Verify it's gone
    {
        let store = state.config_store.read().await;
        assert!(!store.contains_key("delete-service-delete-key"));
    }
});

config_test!(test_bulk_insert_configs, |state: AppState| async move {
    let configs = ConfigFactory::create_auth_service_configs();
    
    // Bulk insert
    {
        let mut store = state.config_store.write().await;
        for config in &configs {
            let key = format!("{}-{}", config.service_name, config.key);
            store.insert(key, config.clone());
        }
    }
    
    // Verify all inserted
    {
        let store = state.config_store.read().await;
        assert_eq!(store.len(), configs.len());
        
        for config in &configs {
            let key = format!("{}-{}", config.service_name, config.key);
            assert!(store.contains_key(&key));
        }
    }
});

config_test!(test_filter_configs_by_environment, |state: AppState| async move {
    let configs = vec![
        ConfigFactory::create_config_with_environment("service1", "key1", "dev"),
        ConfigFactory::create_config_with_environment("service1", "key2", "prod"),
        ConfigFactory::create_config_with_environment("service2", "key1", "dev"),
        ConfigFactory::create_config_with_environment("service2", "key2", "staging"),
    ];
    
    // Insert all configs
    {
        let mut store = state.config_store.write().await;
        for config in &configs {
            let key = format!("{}-{}", config.service_name, config.key);
            store.insert(key, config.clone());
        }
    }
    
    // Filter by environment
    {
        let store = state.config_store.read().await;
        let dev_configs: Vec<&ConfigEntry> = store.values()
            .filter(|c| c.environment == "dev")
            .collect();
        
        assert_eq!(dev_configs.len(), 2);
        assert!(dev_configs.iter().all(|c| c.environment == "dev"));
    }
});

config_test!(test_filter_configs_by_tags, |state: AppState| async move {
    let configs = vec![
        ConfigFactory::create_config_with_tags("service1", "key1", vec!["critical".to_string()]),
        ConfigFactory::create_config_with_tags("service1", "key2", vec!["optional".to_string()]),
        ConfigFactory::create_config_with_tags("service2", "key1", vec!["critical".to_string(), "encrypted".to_string()]),
    ];
    
    // Insert configs
    {
        let mut store = state.config_store.write().await;
        for config in &configs {
            let key = format!("{}-{}", config.service_name, config.key);
            store.insert(key, config.clone());
        }
    }
    
    // Filter by tags
    {
        let store = state.config_store.read().await;
        let critical_configs: Vec<&ConfigEntry> = store.values()
            .filter(|c| c.tags.contains(&"critical".to_string()))
            .collect();
        
        assert_eq!(critical_configs.len(), 2);
        
        let encrypted_configs: Vec<&ConfigEntry> = store.values()
            .filter(|c| c.tags.contains(&"encrypted".to_string()))
            .collect();
        
        assert_eq!(encrypted_configs.len(), 1);
    }
});

#[tokio::test]
async fn test_concurrent_read_write_safety() {
    let state = create_test_app_state();
    let mut handles = Vec::new();
    
    // Spawn multiple readers
    for i in 0..10 {
        let state_clone = state.clone();
        let handle = tokio::spawn(async move {
            let store = state_clone.config_store.read().await;
            store.len()
        });
        handles.push(handle);
    }
    
    // Spawn multiple writers
    for i in 0..5 {
        let state_clone = state.clone();
        let config = ConfigFactory::create_test_config(
            "concurrent-service",
            &format!("key-{}", i),
            serde_json::json!(i)
        );
        
        let handle = tokio::spawn(async move {
            let mut store = state_clone.config_store.write().await;
            store.insert(format!("concurrent-service-key-{}", i), config);
        });
        handles.push(handle);
    }
    
    // Wait for all operations to complete
    let results = futures::future::join_all(handles).await;
    
    // Verify all operations succeeded
    assert!(results.iter().all(|r| r.is_ok()));
    
    // Verify final state
    let store = state.config_store.read().await;
    assert_eq!(store.len(), 5);
}

#[tokio::test]
async fn test_complex_config_values() {
    let state = create_test_app_state();
    let complex_value = TestDataGenerator::generate_complex_config_value();
    
    let config = ConfigFactory::create_test_config(
        "complex-service",
        "complex-config",
        complex_value.clone()
    );
    
    // Store complex config
    {
        let mut store = state.config_store.write().await;
        store.insert("complex-service-complex-config".to_string(), config);
    }
    
    // Retrieve and verify structure
    {
        let store = state.config_store.read().await;
        let retrieved = store.get("complex-service-complex-config").unwrap();
        
        assert_eq!(retrieved.value, complex_value);
        
        // Verify nested values
        let db_config = &retrieved.value["database"];
        assert_eq!(db_config["host"], "localhost");
        assert_eq!(db_config["port"], 5432);
        
        let features = &retrieved.value["features"];
        assert_eq!(features["enable_cache"], true);
    }
}

#[tokio::test]
async fn test_encrypted_config_handling() {
    let state = create_test_app_state();
    let encrypted_value = TestDataGenerator::generate_encrypted_config_value();
    
    let mut config = ConfigFactory::create_test_config(
        "secure-service",
        "api-key",
        encrypted_value
    );
    config.encrypted = true;
    
    // Store encrypted config
    {
        let mut store = state.config_store.write().await;
        store.insert("secure-service-api-key".to_string(), config.clone());
    }
    
    // Verify encrypted flag is preserved
    {
        let store = state.config_store.read().await;
        let retrieved = store.get("secure-service-api-key").unwrap();
        
        assert!(retrieved.encrypted);
        assert_eq!(retrieved.value["encrypted"], true);
        assert_eq!(retrieved.value["algorithm"], "AES-256-GCM");
    }
}

#[tokio::test]
async fn test_performance_bulk_operations() {
    let state = create_test_app_state();
    let bulk_configs = TestDataGenerator::generate_bulk_configs("perf-service", 1000);
    
    // Measure bulk insert time
    let (insert_duration, _) = PerformanceTestUtils::measure_config_operation(|| async {
        let mut store = state.config_store.write().await;
        for (key, value) in &bulk_configs {
            let config = ConfigFactory::create_test_config("perf-service", key, value.clone());
            store.insert(format!("perf-service-{}", key), config);
        }
    }).await;
    
    println!("Bulk insert of 1000 configs took: {:?}", insert_duration);
    assert!(insert_duration.as_millis() < 100); // Should be fast
    
    // Measure bulk read time
    let (read_duration, count) = PerformanceTestUtils::measure_config_operation(|| async {
        let store = state.config_store.read().await;
        store.values()
            .filter(|c| c.service_name == "perf-service")
            .count()
    }).await;
    
    println!("Bulk read of {} configs took: {:?}", count, read_duration);
    assert_eq!(count, 1000);
    assert!(read_duration.as_millis() < 50); // Should be very fast
}