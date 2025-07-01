//! Unit tests for cache behavior

use dashboard_service::{AppState, DashboardData};
use crate::test_utils::*;
use std::time::Duration;

#[tokio::test]
async fn test_cache_insertion_and_retrieval() {
    let cache = CacheTestUtils::create_test_cache();
    let data = TestDataGenerator::sample_dashboard_data();
    
    // Insert data
    cache.insert("test_key".to_string(), data.clone()).await;
    
    // Retrieve data
    let retrieved = cache.get(&"test_key".to_string()).await;
    assert!(retrieved.is_some());
    
    let retrieved_data = retrieved.unwrap();
    assert_eq!(retrieved_data.timestamp, data.timestamp);
    assert_eq!(retrieved_data.ttl_seconds, data.ttl_seconds);
}

#[tokio::test]
async fn test_cache_expiration() {
    let cache = moka::future::Cache::builder()
        .time_to_live(Duration::from_millis(100))
        .build();
    
    let data = TestDataGenerator::sample_dashboard_data();
    
    // Insert data
    cache.insert("expiring_key".to_string(), data).await;
    
    // Verify it exists
    assert!(cache.get(&"expiring_key".to_string()).await.is_some());
    
    // Wait for expiration
    tokio::time::sleep(Duration::from_millis(150)).await;
    
    // Verify it's expired
    assert!(cache.get(&"expiring_key".to_string()).await.is_none());
}

#[tokio::test]
async fn test_cache_capacity_limit() {
    let cache = moka::future::Cache::builder()
        .max_capacity(2)
        .build();
    
    // Insert more than capacity
    for i in 0..5 {
        let data = DashboardData {
            timestamp: chrono::Utc::now(),
            data: serde_json::json!({"index": i}),
            ttl_seconds: 300,
        };
        cache.insert(format!("key_{}", i), data).await;
    }
    
    // Cache should only contain the most recent entries
    cache.run_pending_tasks().await;
    
    let count = cache.entry_count();
    assert!(count <= 2, "Cache has {} entries, expected <= 2", count);
}

#[tokio::test]
async fn test_cache_update() {
    let cache = CacheTestUtils::create_test_cache();
    
    let original_data = DashboardData {
        timestamp: chrono::Utc::now(),
        data: serde_json::json!({"version": 1}),
        ttl_seconds: 300,
    };
    
    let updated_data = DashboardData {
        timestamp: chrono::Utc::now(),
        data: serde_json::json!({"version": 2}),
        ttl_seconds: 300,
    };
    
    // Insert original
    cache.insert("update_key".to_string(), original_data).await;
    
    // Update with new data
    cache.insert("update_key".to_string(), updated_data.clone()).await;
    
    // Retrieve and verify update
    let retrieved = cache.get(&"update_key".to_string()).await.unwrap();
    assert_eq!(retrieved.data["version"], 2);
}

#[tokio::test]
async fn test_cache_invalidation() {
    let cache = CacheTestUtils::create_test_cache();
    CacheTestUtils::populate_cache(&cache).await;
    
    // Verify data exists
    assert!(cache.get(&"test_key_1".to_string()).await.is_some());
    assert!(cache.get(&"test_key_2".to_string()).await.is_some());
    
    // Invalidate specific key
    cache.invalidate(&"test_key_1".to_string()).await;
    
    // Verify invalidation
    assert!(cache.get(&"test_key_1".to_string()).await.is_none());
    assert!(cache.get(&"test_key_2".to_string()).await.is_some());
}

#[tokio::test]
async fn test_cache_clear() {
    let cache = CacheTestUtils::create_test_cache();
    CacheTestUtils::populate_cache(&cache).await;
    
    // Verify data exists
    assert!(cache.entry_count() > 0);
    
    // Clear cache
    cache.invalidate_all();
    cache.run_pending_tasks().await;
    
    // Verify cache is empty
    assert_eq!(cache.entry_count(), 0);
}

#[tokio::test]
async fn test_concurrent_cache_access() {
    let cache = CacheTestUtils::create_test_cache();
    let cache_clone = cache.clone();
    
    // Spawn multiple tasks accessing cache concurrently
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let cache = cache_clone.clone();
            tokio::spawn(async move {
                let data = DashboardData {
                    timestamp: chrono::Utc::now(),
                    data: serde_json::json!({"task": i}),
                    ttl_seconds: 300,
                };
                cache.insert(format!("concurrent_{}", i), data).await;
                
                // Try to read
                cache.get(&format!("concurrent_{}", i)).await
            })
        })
        .collect();
    
    // Wait for all tasks
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_some());
    }
    
    // Verify all entries exist
    cache.run_pending_tasks().await;
    for i in 0..10 {
        assert!(cache.get(&format!("concurrent_{}", i)).await.is_some());
    }
}

#[tokio::test]
async fn test_cache_get_or_insert() {
    let cache = CacheTestUtils::create_test_cache();
    let key = "get_or_insert_key";
    
    // First access - should insert
    let data = cache.get_with(key.to_string(), async {
        TestDataGenerator::sample_dashboard_data()
    }).await;
    
    assert_eq!(data.data["widgets"].as_array().unwrap().len(), 2);
    
    // Second access - should get from cache
    let cached_data = cache.get(&key.to_string()).await;
    assert!(cached_data.is_some());
}

#[tokio::test]
async fn test_cache_performance() {
    let cache = CacheTestUtils::create_test_cache();
    
    // Populate cache
    let insert_start = std::time::Instant::now();
    for i in 0..100 {
        let data = DashboardData {
            timestamp: chrono::Utc::now(),
            data: serde_json::json!({"index": i}),
            ttl_seconds: 300,
        };
        cache.insert(format!("perf_key_{}", i), data).await;
    }
    let insert_duration = insert_start.elapsed();
    
    // Read from cache
    let read_start = std::time::Instant::now();
    for i in 0..100 {
        let _ = cache.get(&format!("perf_key_{}", i)).await;
    }
    let read_duration = read_start.elapsed();
    
    // Cache operations should be fast
    assert!(
        insert_duration.as_millis() < 100,
        "Cache insertions took {:?}, which is too slow",
        insert_duration
    );
    assert!(
        read_duration.as_millis() < 50,
        "Cache reads took {:?}, which is too slow",
        read_duration
    );
    
    println!("Cache performance - Insert: {:?}, Read: {:?}", insert_duration, read_duration);
}

test_with_mock_services!(
    test_cache_integration_with_app_state,
    |app_state: &AppState, _mock_server: &MockServer| async move {
        let key = "app_state_test";
        let data = TestDataGenerator::sample_dashboard_data();
        
        // Insert through app state cache
        app_state.cache.insert(key.to_string(), data.clone()).await;
        
        // Retrieve through app state cache
        let retrieved = app_state.cache.get(&key.to_string()).await;
        assert!(retrieved.is_some());
        
        let retrieved_data = retrieved.unwrap();
        assert_eq!(retrieved_data.timestamp, data.timestamp);
    }
);

#[tokio::test]
async fn test_cache_with_different_ttl() {
    let cache = CacheTestUtils::create_test_cache();
    
    // Insert data with different TTLs (though cache has global TTL)
    let short_ttl_data = DashboardData {
        timestamp: chrono::Utc::now(),
        data: serde_json::json!({"ttl": "short"}),
        ttl_seconds: 10,
    };
    
    let long_ttl_data = DashboardData {
        timestamp: chrono::Utc::now(),
        data: serde_json::json!({"ttl": "long"}),
        ttl_seconds: 600,
    };
    
    cache.insert("short_ttl".to_string(), short_ttl_data).await;
    cache.insert("long_ttl".to_string(), long_ttl_data).await;
    
    // Both should exist initially
    assert!(cache.get(&"short_ttl".to_string()).await.is_some());
    assert!(cache.get(&"long_ttl".to_string()).await.is_some());
}