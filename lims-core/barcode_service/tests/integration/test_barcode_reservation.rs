//! Integration tests for barcode reservation functionality

use barcode_service::{
    service::BarcodeService,
    error::{BarcodeError, Result},
};
use crate::test_utils::*;

test_with_barcode_db!(
    test_basic_reservation_flow,
    BarcodeConfigFactory::create_default(),
    |service: &BarcodeService, test_db: &mut TestDatabase| async move {
        let barcode = service.generate_barcode(
            Some("DNA"),
            Some(1),
            None,
            None,
        ).await.expect("Should generate barcode");
        
        test_db.track_barcode(barcode.clone());
        
        // Initial state - not reserved
        let status = service.get_barcode_status(&barcode).await.unwrap().unwrap();
        assert!(!status.is_reserved);
        assert!(status.reserved_by.is_none());
        assert!(status.reserved_at.is_none());
        
        // Reserve the barcode
        service.reserve_barcode(&barcode, "user123", Some("DNA extraction"))
            .await
            .expect("Should reserve barcode");
        
        // Check reservation state
        let status = service.get_barcode_status(&barcode).await.unwrap().unwrap();
        assert!(status.is_reserved);
        assert_eq!(status.reserved_by, Some("user123".to_string()));
        assert!(status.reserved_at.is_some());
        
        // Release the barcode
        service.release_barcode(&barcode, "user123")
            .await
            .expect("Should release barcode");
        
        // Check released state
        let status = service.get_barcode_status(&barcode).await.unwrap().unwrap();
        assert!(!status.is_reserved);
        assert!(status.reserved_by.is_none());
    }
);

test_with_barcode_db!(
    test_concurrent_reservation_attempts,
    BarcodeConfigFactory::create_default(),
    |service: &BarcodeService, test_db: &mut TestDatabase| async move {
        use futures::future::join_all;
        
        let barcode = service.generate_barcode(
            Some("RNA"),
            Some(2),
            None,
            None,
        ).await.expect("Should generate barcode");
        
        test_db.track_barcode(barcode.clone());
        
        // Create multiple concurrent reservation attempts
        let service_ref = &service;
        let barcode_ref = &barcode;
        let tasks: Vec<_> = (0..10)
            .map(|i| async move {
                service_ref.reserve_barcode(
                    barcode_ref,
                    &format!("user_{}", i),
                    Some(&format!("Purpose {}", i))
                ).await
            })
            .collect();
        
        let results = join_all(tasks).await;
        
        // Only one should succeed
        let successes = results.iter().filter(|r| r.is_ok()).count();
        assert_eq!(successes, 1, "Only one reservation should succeed");
        
        // The rest should fail with AlreadyReserved error
        let failures = results.iter().filter(|r| r.is_err()).count();
        assert_eq!(failures, 9, "Nine reservations should fail");
        
        for result in results.iter().filter(|r| r.is_err()) {
            assert!(matches!(
                result,
                Err(BarcodeError::BarcodeAlreadyReserved(_))
            ));
        }
    }
);

test_with_barcode_db!(
    test_reservation_with_metadata,
    BarcodeConfigFactory::create_default(),
    |service: &BarcodeService, test_db: &mut TestDatabase| async move {
        let barcode = service.generate_barcode(
            Some("PROTEIN"),
            Some(3),
            None,
            None,
        ).await.expect("Should generate barcode");
        
        test_db.track_barcode(barcode.clone());
        
        // Reserve with purpose
        let purpose = "Western blot analysis - Batch A123";
        service.reserve_barcode(&barcode, "lab_tech_001", Some(purpose))
            .await
            .expect("Should reserve with metadata");
        
        // Check metadata is stored
        let status = service.get_barcode_status(&barcode).await.unwrap().unwrap();
        assert!(status.metadata.is_some());
        
        let metadata = status.metadata.unwrap();
        if let Some(reservation_purpose) = metadata.get("reservation_purpose") {
            assert_eq!(reservation_purpose, purpose);
        }
    }
);

test_with_barcode_db!(
    test_release_non_reserved_barcode,
    BarcodeConfigFactory::create_default(),
    |service: &BarcodeService, test_db: &mut TestDatabase| async move {
        let barcode = service.generate_barcode(
            Some("CELL"),
            Some(4),
            None,
            None,
        ).await.expect("Should generate barcode");
        
        test_db.track_barcode(barcode.clone());
        
        // Try to release non-reserved barcode
        let result = service.release_barcode(&barcode, "user").await;
        assert!(result.is_err());
        assert!(matches!(result, Err(BarcodeError::BarcodeNotReserved(_))));
    }
);

test_with_barcode_db!(
    test_reserve_non_existent_barcode,
    BarcodeConfigFactory::create_default(),
    |service: &BarcodeService, _test_db: &mut TestDatabase| async move {
        let fake_barcode = "TST-FAKE-20240115-L999-9999999";
        
        let result = service.reserve_barcode(fake_barcode, "user", None).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(BarcodeError::BarcodeNotFound(_))));
    }
);

test_with_barcode_db!(
    test_multiple_reserve_release_cycles,
    BarcodeConfigFactory::create_default(),
    |service: &BarcodeService, test_db: &mut TestDatabase| async move {
        let barcode = service.generate_barcode(
            Some("TISSUE"),
            Some(5),
            None,
            None,
        ).await.expect("Should generate barcode");
        
        test_db.track_barcode(barcode.clone());
        
        // Multiple cycles of reserve and release
        for i in 0..5 {
            let user = format!("user_{}", i);
            let purpose = format!("Experiment {}", i);
            
            // Reserve
            service.reserve_barcode(&barcode, &user, Some(&purpose))
                .await
                .expect(&format!("Should reserve in cycle {}", i));
            
            // Verify reservation
            let status = service.get_barcode_status(&barcode).await.unwrap().unwrap();
            assert!(status.is_reserved);
            assert_eq!(status.reserved_by, Some(user.clone()));
            
            // Release
            service.release_barcode(&barcode, &user)
                .await
                .expect(&format!("Should release in cycle {}", i));
            
            // Verify release
            let status = service.get_barcode_status(&barcode).await.unwrap().unwrap();
            assert!(!status.is_reserved);
        }
    }
);

test_with_barcode_db!(
    test_reservation_tracking,
    BarcodeConfigFactory::create_default(),
    |service: &BarcodeService, test_db: &mut TestDatabase| async move {
        let mut reserved_barcodes = Vec::new();
        let mut free_barcodes = Vec::new();
        
        // Generate and reserve some barcodes
        for i in 0..10 {
            let barcode = service.generate_barcode(
                Some("PLASMA"),
                Some(i),
                None,
                None,
            ).await.expect("Should generate barcode");
            
            test_db.track_barcode(barcode.clone());
            
            if i % 2 == 0 {
                // Reserve even-indexed barcodes
                service.reserve_barcode(&barcode, &format!("user_{}", i), None)
                    .await
                    .expect("Should reserve barcode");
                reserved_barcodes.push(barcode);
            } else {
                free_barcodes.push(barcode);
            }
        }
        
        // Verify reserved barcodes
        for barcode in &reserved_barcodes {
            let status = service.get_barcode_status(barcode).await.unwrap().unwrap();
            assert!(status.is_reserved, "Barcode {} should be reserved", barcode);
        }
        
        // Verify free barcodes
        for barcode in &free_barcodes {
            let status = service.get_barcode_status(barcode).await.unwrap().unwrap();
            assert!(!status.is_reserved, "Barcode {} should be free", barcode);
        }
        
        // Get stats and verify reservation count
        let stats = service.get_stats().await.expect("Should get stats");
        assert!(stats.total_reserved >= reserved_barcodes.len() as i64);
    }
);

#[tokio::test]
async fn test_reservation_timestamp_tracking() {
    let config = BarcodeConfigFactory::create_default();
    let (service, mut test_db) = create_test_barcode_service(config).await;
    
    let barcode = service.generate_barcode(
        Some("SERUM"),
        Some(1),
        None,
        None,
    ).await.expect("Should generate barcode");
    
    test_db.track_barcode(barcode.clone());
    
    // Record time before reservation
    let before_reserve = chrono::Utc::now();
    
    // Reserve the barcode
    service.reserve_barcode(&barcode, "timetest_user", Some("Timing test"))
        .await
        .expect("Should reserve barcode");
    
    // Record time after reservation
    let after_reserve = chrono::Utc::now();
    
    // Get status and check timestamp
    let status = service.get_barcode_status(&barcode).await.unwrap().unwrap();
    assert!(status.reserved_at.is_some());
    
    let reserved_at = status.reserved_at.unwrap();
    assert!(
        reserved_at >= before_reserve && reserved_at <= after_reserve,
        "Reservation timestamp should be between test boundaries"
    );
    
    test_db.cleanup().await;
}