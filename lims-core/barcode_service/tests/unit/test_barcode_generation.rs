//! Unit tests for barcode generation functionality

use barcode_service::{
    service::BarcodeService,
    error::{BarcodeError, Result},
};
use crate::test_utils::*;

test_with_barcode_db!(
    test_generate_basic_barcode,
    BarcodeConfigFactory::create_default(),
    |service: &BarcodeService, test_db: &mut TestDatabase| async move {
        let barcode = service.generate_barcode(
            Some("DNA"),
            Some(1),
            None,
            None,
        ).await.expect("Should generate barcode");

        test_db.track_barcode(barcode.clone());

        // Verify barcode format
        BarcodeAssertions::assert_barcode_format(&barcode, &BarcodeConfigFactory::create_default());
        
        // Verify it starts with prefix
        assert!(barcode.starts_with("TST"));
        
        // Verify it contains sample type
        assert!(barcode.contains("DNA"));
    }
);

test_with_barcode_db!(
    test_generate_barcode_with_template,
    BarcodeConfigFactory::create_default(),
    |service: &BarcodeService, test_db: &mut TestDatabase| async move {
        let barcode = service.generate_barcode(
            Some("RNA"),
            Some(10),
            Some("RNA Sequencing Prep"),
            None,
        ).await.expect("Should generate barcode with template");

        test_db.track_barcode(barcode.clone());

        // Verify barcode contains enhanced sample type
        assert!(barcode.contains("RNA"));
        
        // Parse and verify components
        let info = service.parse_barcode(&barcode);
        assert!(info.is_valid);
    }
);

test_with_barcode_db!(
    test_generate_barcode_uniqueness,
    BarcodeConfigFactory::create_default(),
    |service: &BarcodeService, test_db: &mut TestDatabase| async move {
        let mut barcodes = Vec::new();
        
        // Generate multiple barcodes with same parameters
        for _ in 0..10 {
            let barcode = service.generate_barcode(
                Some("DNA"),
                Some(1),
                None,
                None,
            ).await.expect("Should generate unique barcode");
            
            test_db.track_barcode(barcode.clone());
            barcodes.push(barcode);
        }

        // Verify all barcodes are unique
        for i in 0..barcodes.len() {
            for j in (i + 1)..barcodes.len() {
                BarcodeAssertions::assert_barcode_unique(&barcodes[i], &barcodes[j]);
            }
        }
    }
);

test_with_barcode_db!(
    test_generate_minimal_barcode,
    BarcodeConfigFactory::create_minimal(),
    |service: &BarcodeService, test_db: &mut TestDatabase| async move {
        let barcode = service.generate_barcode(
            None,
            None,
            None,
            None,
        ).await.expect("Should generate minimal barcode");

        test_db.track_barcode(barcode.clone());

        // Verify minimal configuration
        assert!(barcode.starts_with("MIN"));
        assert!(barcode.len() >= 5);
        assert!(!barcode.contains("-")); // No separator in minimal config
    }
);

test_with_barcode_db!(
    test_generate_complex_barcode,
    BarcodeConfigFactory::create_complex(),
    |service: &BarcodeService, test_db: &mut TestDatabase| async move {
        let barcode = service.generate_barcode(
            Some("PROTEIN"),
            Some(999),
            Some("Protein Purification"),
            None,
        ).await.expect("Should generate complex barcode");

        test_db.track_barcode(barcode.clone());

        // Verify complex configuration
        assert!(barcode.starts_with("COMPLEX"));
        assert!(barcode.len() >= 20);
        assert!(barcode.contains("_")); // Underscore separator
        assert!(barcode.contains("PROTEIN"));
        assert!(barcode.contains("L999")); // Location component
    }
);

test_with_barcode_db!(
    test_generate_barcode_with_custom_prefix,
    BarcodeConfigFactory::create_default(),
    |service: &BarcodeService, test_db: &mut TestDatabase| async move {
        let barcode = service.generate_barcode(
            Some("CELL"),
            Some(42),
            None,
            Some("CUSTOM"),
        ).await.expect("Should generate barcode with custom prefix");

        test_db.track_barcode(barcode.clone());

        // Verify custom prefix is used
        assert!(barcode.starts_with("CUSTOM"));
        assert!(!barcode.starts_with("TST")); // Default prefix not used
    }
);

test_with_barcode_db!(
    test_barcode_minimum_length_padding,
    BarcodeConfigFactory::create_default(),
    |service: &BarcodeService, test_db: &mut TestDatabase| async move {
        // Generate a barcode that would be shorter than minimum
        let barcode = service.generate_barcode(
            None,
            None,
            None,
            Some("X"), // Very short prefix
        ).await.expect("Should pad barcode to minimum length");

        test_db.track_barcode(barcode.clone());

        // Verify minimum length is maintained
        assert!(barcode.len() >= 10);
    }
);

#[tokio::test]
async fn test_barcode_date_component() {
    let config = BarcodeConfigFactory::create_default();
    let (service, mut test_db) = create_test_barcode_service(config).await;
    
    let barcode = service.generate_barcode(
        Some("DNA"),
        Some(1),
        None,
        None,
    ).await.expect("Should generate barcode with date");

    test_db.track_barcode(barcode.clone());

    // Parse barcode and check for date component
    let info = service.parse_barcode(&barcode);
    assert!(info.date_component.is_some());
    
    let date_component = info.date_component.unwrap();
    assert_eq!(date_component.len(), 8); // YYYYMMDD format
    
    // Verify it's a valid date string
    assert!(date_component.chars().all(|c| c.is_numeric()));
    
    test_db.cleanup().await;
}

#[tokio::test]
async fn test_barcode_location_component() {
    let config = BarcodeConfigFactory::create_default();
    let (service, mut test_db) = create_test_barcode_service(config).await;
    
    let location_id = 123;
    let barcode = service.generate_barcode(
        Some("RNA"),
        Some(location_id),
        None,
        None,
    ).await.expect("Should generate barcode with location");

    test_db.track_barcode(barcode.clone());

    // Parse barcode and verify location component
    let info = service.parse_barcode(&barcode);
    assert_eq!(info.location_component, Some(location_id));
    
    // Verify format in barcode string
    assert!(barcode.contains(&format!("L{:03}", location_id)));
    
    test_db.cleanup().await;
}

#[tokio::test]
async fn test_all_sample_types() {
    let config = BarcodeConfigFactory::create_default();
    let (service, mut test_db) = create_test_barcode_service(config).await;
    
    for sample_type in TestDataGenerator::sample_types() {
        let barcode = service.generate_barcode(
            Some(sample_type),
            Some(1),
            None,
            None,
        ).await.expect(&format!("Should generate barcode for {}", sample_type));

        test_db.track_barcode(barcode.clone());

        // Verify sample type is included
        assert!(
            barcode.contains(sample_type),
            "Barcode {} should contain sample type {}",
            barcode,
            sample_type
        );
    }
    
    test_db.cleanup().await;
}

// Performance test
#[tokio::test]
async fn test_barcode_generation_performance() {
    let config = BarcodeConfigFactory::create_default();
    let (service, mut test_db) = create_test_barcode_service(config).await;
    
    let duration = PerformanceTestUtils::measure_generation_time(&service, 100).await;
    
    // Assert reasonable performance (adjust threshold as needed)
    assert!(
        duration.as_secs() < 10,
        "Generating 100 barcodes took {:?}, which is too slow",
        duration
    );
    
    println!("Generated 100 barcodes in {:?}", duration);
    
    test_db.cleanup().await;
}