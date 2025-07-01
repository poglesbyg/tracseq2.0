//! Integration tests for complete barcode service workflows

use barcode_service::{
    service::BarcodeService,
    error::{BarcodeError, Result},
};
use crate::test_utils::*;
use std::collections::HashSet;

test_with_barcode_db!(
    test_complete_barcode_lifecycle,
    BarcodeConfigFactory::create_default(),
    |service: &BarcodeService, test_db: &mut TestDatabase| async move {
        // 1. Generate a new barcode
        let barcode = service.generate_barcode(
            Some("DNA"),
            Some(1),
            Some("DNA Extraction Protocol"),
            None,
        ).await.expect("Should generate barcode");
        
        test_db.track_barcode(barcode.clone());
        
        // 2. Verify it's stored and unique
        assert!(!service.check_barcode_unique(&barcode).await.unwrap());
        
        // 3. Get barcode status
        let status = service.get_barcode_status(&barcode).await.unwrap();
        assert!(status.is_some());
        let stored = status.unwrap();
        assert_eq!(stored.barcode, barcode);
        assert!(!stored.is_reserved);
        
        // 4. Reserve the barcode
        service.reserve_barcode(&barcode, "test_user", Some("Testing purposes"))
            .await
            .expect("Should reserve barcode");
        
        // 5. Verify reservation
        let status = service.get_barcode_status(&barcode).await.unwrap().unwrap();
        assert!(status.is_reserved);
        assert_eq!(status.reserved_by, Some("test_user".to_string()));
        
        // 6. Try to reserve again (should fail)
        let result = service.reserve_barcode(&barcode, "another_user", None).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(BarcodeError::BarcodeAlreadyReserved(_))));
        
        // 7. Release the barcode
        service.release_barcode(&barcode, "test_user")
            .await
            .expect("Should release barcode");
        
        // 8. Verify release
        let status = service.get_barcode_status(&barcode).await.unwrap().unwrap();
        assert!(!status.is_reserved);
        assert!(status.reserved_by.is_none());
    }
);

test_with_barcode_db!(
    test_bulk_barcode_generation,
    BarcodeConfigFactory::create_default(),
    |service: &BarcodeService, test_db: &mut TestDatabase| async move {
        let mut generated_barcodes = HashSet::new();
        let batch_size = 50;
        
        // Generate multiple barcodes
        for i in 0..batch_size {
            let sample_type = TestDataGenerator::sample_types()[i % 7];
            let location_id = (i % 10) as i32 + 1;
            
            let barcode = service.generate_barcode(
                Some(sample_type),
                Some(location_id),
                None,
                None,
            ).await.expect(&format!("Should generate barcode {}", i));
            
            test_db.track_barcode(barcode.clone());
            
            // Verify uniqueness within batch
            assert!(
                generated_barcodes.insert(barcode.clone()),
                "Duplicate barcode generated: {}",
                barcode
            );
        }
        
        // Verify all are stored
        for barcode in &generated_barcodes {
            assert!(!service.check_barcode_unique(barcode).await.unwrap());
        }
        
        assert_eq!(generated_barcodes.len(), batch_size);
    }
);

test_with_barcode_db!(
    test_barcode_statistics,
    BarcodeConfigFactory::create_default(),
    |service: &BarcodeService, test_db: &mut TestDatabase| async move {
        // Get initial stats
        let initial_stats = service.get_stats().await.expect("Should get stats");
        let initial_count = initial_stats.total_generated;
        
        // Generate some barcodes
        let count = 10;
        for i in 0..count {
            let barcode = service.generate_barcode(
                Some(TestDataGenerator::sample_types()[i % 7]),
                Some((i % 5) as i32),
                None,
                None,
            ).await.expect("Should generate barcode");
            
            test_db.track_barcode(barcode);
        }
        
        // Reserve some barcodes
        let reserved_count = 3;
        for i in 0..reserved_count {
            let barcode = service.generate_barcode(
                Some("DNA"),
                Some(1),
                None,
                None,
            ).await.expect("Should generate barcode");
            
            test_db.track_barcode(barcode.clone());
            
            service.reserve_barcode(&barcode, &format!("user_{}", i), None)
                .await
                .expect("Should reserve barcode");
        }
        
        // Get updated stats
        let stats = service.get_stats().await.expect("Should get stats");
        
        // Verify counts
        assert_eq!(stats.total_generated, initial_count + count + reserved_count);
        assert!(stats.total_reserved >= reserved_count);
        assert!(stats.total_unique_prefixes > 0);
        assert!(stats.most_recent_barcode.is_some());
        assert!(stats.generation_rate_per_day >= 0.0);
    }
);

test_with_barcode_db!(
    test_concurrent_barcode_generation,
    BarcodeConfigFactory::create_default(),
    |service: &BarcodeService, test_db: &mut TestDatabase| async move {
        use futures::future::join_all;
        
        let concurrent_tasks = 20;
        let service_ref = &service;
        
        // Create multiple concurrent generation tasks
        let tasks: Vec<_> = (0..concurrent_tasks)
            .map(|i| async move {
                service_ref.generate_barcode(
                    Some("DNA"),
                    Some((i % 5) as i32),
                    None,
                    None,
                ).await
            })
            .collect();
        
        // Execute all tasks concurrently
        let results = join_all(tasks).await;
        
        // Collect successful barcodes
        let mut barcodes = HashSet::new();
        for result in results {
            if let Ok(barcode) = result {
                test_db.track_barcode(barcode.clone());
                barcodes.insert(barcode);
            }
        }
        
        // All generated barcodes should be unique
        assert_eq!(barcodes.len(), concurrent_tasks);
    }
);

test_with_barcode_db!(
    test_health_check,
    BarcodeConfigFactory::create_default(),
    |service: &BarcodeService, _test_db: &mut TestDatabase| async move {
        // Health check should return successfully
        let result = service.health_check().await;
        assert!(result.is_ok());
        assert!(result.unwrap() >= 0);
    }
);

test_with_barcode_db!(
    test_invalid_operations,
    BarcodeConfigFactory::create_default(),
    |service: &BarcodeService, _test_db: &mut TestDatabase| async move {
        let non_existent_barcode = "NON-EXISTENT-BARCODE-12345";
        
        // Try to reserve non-existent barcode
        let result = service.reserve_barcode(non_existent_barcode, "user", None).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(BarcodeError::BarcodeNotFound(_))));
        
        // Try to release non-reserved barcode
        let barcode = service.generate_barcode(Some("DNA"), None, None, None)
            .await
            .expect("Should generate barcode");
        
        let result = service.release_barcode(&barcode, "user").await;
        assert!(result.is_err());
        assert!(matches!(result, Err(BarcodeError::BarcodeNotReserved(_))));
        
        // Get status of non-existent barcode
        let status = service.get_barcode_status(non_existent_barcode).await;
        assert!(status.is_ok());
        assert!(status.unwrap().is_none());
    }
);

test_with_barcode_db!(
    test_different_configurations,
    BarcodeConfigFactory::create_minimal(),
    |service: &BarcodeService, test_db: &mut TestDatabase| async move {
        // Test with minimal config
        let minimal_barcode = service.generate_barcode(None, None, None, None)
            .await
            .expect("Should generate minimal barcode");
        
        test_db.track_barcode(minimal_barcode.clone());
        assert!(minimal_barcode.starts_with("MIN"));
        assert!(minimal_barcode.len() >= 5);
        
        // Test with complex config
        let complex_config = BarcodeConfigFactory::create_complex();
        let (complex_service, _) = create_test_barcode_service(complex_config).await;
        
        let complex_barcode = complex_service.generate_barcode(
            Some("PROTEIN"),
            Some(999),
            Some("Protein Analysis"),
            None,
        ).await.expect("Should generate complex barcode");
        
        test_db.track_barcode(complex_barcode.clone());
        assert!(complex_barcode.starts_with("COMPLEX"));
        assert!(complex_barcode.contains("_"));
        assert!(complex_barcode.len() >= 20);
    }
);

#[tokio::test]
async fn test_cross_service_barcode_validation() {
    // Create services with different configurations
    let default_config = BarcodeConfigFactory::create_default();
    let (default_service, mut test_db) = create_test_barcode_service(default_config).await;
    
    let minimal_config = BarcodeConfigFactory::create_minimal();
    let (minimal_service, _) = create_test_barcode_service(minimal_config).await;
    
    // Generate barcode with default service
    let default_barcode = default_service.generate_barcode(
        Some("DNA"),
        Some(1),
        None,
        None,
    ).await.expect("Should generate default barcode");
    
    test_db.track_barcode(default_barcode.clone());
    
    // Try to validate with minimal service (different pattern)
    let result = minimal_service.validate_barcode_format(&default_barcode);
    // Should fail because patterns are different
    assert!(result.is_err());
    
    test_db.cleanup().await;
}