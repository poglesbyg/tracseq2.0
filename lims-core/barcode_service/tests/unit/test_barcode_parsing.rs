//! Unit tests for barcode parsing functionality

use barcode_service::{
    service::BarcodeService,
    models::BarcodeInfo,
};
use crate::test_utils::*;

#[tokio::test]
async fn test_parse_standard_barcode() {
    let config = BarcodeConfigFactory::create_default();
    let (service, mut test_db) = create_test_barcode_service(config).await;
    
    let barcode = "TST-DNA-20240115-L001-1234567";
    let info = service.parse_barcode(barcode);
    
    assert_eq!(info.full_barcode, barcode);
    assert_eq!(info.prefix, Some("TST".to_string()));
    assert_eq!(info.sample_type, Some("DNA".to_string()));
    assert_eq!(info.date_component, Some("20240115".to_string()));
    assert_eq!(info.location_component, Some(1));
    assert_eq!(info.sequence_component, Some("1234567".to_string()));
    assert!(info.is_valid);
    
    test_db.cleanup().await;
}

#[tokio::test]
async fn test_parse_minimal_barcode() {
    let config = BarcodeConfigFactory::create_minimal();
    let (service, mut test_db) = create_test_barcode_service(config).await;
    
    let barcode = "MIN12345";
    let info = service.parse_barcode(barcode);
    
    assert_eq!(info.full_barcode, barcode);
    assert_eq!(info.prefix, Some("MIN12345".to_string())); // No separator, all treated as prefix
    assert!(info.sample_type.is_none());
    assert!(info.date_component.is_none());
    assert!(info.location_component.is_none());
    assert!(info.is_valid);
    
    test_db.cleanup().await;
}

#[tokio::test]
async fn test_parse_complex_barcode() {
    let config = BarcodeConfigFactory::create_complex();
    let (service, mut test_db) = create_test_barcode_service(config).await;
    
    let barcode = "COMPLEX_RNA_20240115_L999_2345678";
    let info = service.parse_barcode(barcode);
    
    assert_eq!(info.full_barcode, barcode);
    assert_eq!(info.prefix, Some("COMPLEX".to_string()));
    assert_eq!(info.sample_type, Some("RNA".to_string()));
    assert_eq!(info.date_component, Some("20240115".to_string()));
    assert_eq!(info.location_component, Some(999));
    assert_eq!(info.sequence_component, Some("2345678".to_string()));
    assert!(info.is_valid);
    
    test_db.cleanup().await;
}

#[tokio::test]
async fn test_parse_barcode_without_date() {
    let mut config = BarcodeConfigFactory::create_default();
    config.include_date = false;
    let (service, mut test_db) = create_test_barcode_service(config).await;
    
    let barcode = "TST-DNA-L001-1234567";
    let info = service.parse_barcode(barcode);
    
    assert_eq!(info.prefix, Some("TST".to_string()));
    assert_eq!(info.sample_type, Some("DNA".to_string()));
    assert!(info.date_component.is_none()); // No 8-digit date component
    assert_eq!(info.location_component, Some(1));
    assert!(info.is_valid);
    
    test_db.cleanup().await;
}

#[tokio::test]
async fn test_parse_barcode_without_location() {
    let config = BarcodeConfigFactory::create_default();
    let (service, mut test_db) = create_test_barcode_service(config).await;
    
    let barcode = "TST-DNA-20240115-1234567";
    let info = service.parse_barcode(barcode);
    
    assert_eq!(info.prefix, Some("TST".to_string()));
    assert_eq!(info.sample_type, Some("DNA".to_string()));
    assert_eq!(info.date_component, Some("20240115".to_string()));
    assert!(info.location_component.is_none()); // No L-prefixed location
    assert_eq!(info.sequence_component, Some("1234567".to_string()));
    assert!(info.is_valid);
    
    test_db.cleanup().await;
}

#[tokio::test]
async fn test_parse_invalid_barcode() {
    let config = BarcodeConfigFactory::create_default();
    let (service, mut test_db) = create_test_barcode_service(config).await;
    
    let barcode = "invalid@barcode#123";
    let info = service.parse_barcode(barcode);
    
    assert_eq!(info.full_barcode, barcode);
    assert!(!info.is_valid); // Should be marked as invalid
    
    test_db.cleanup().await;
}

#[tokio::test]
async fn test_parse_location_component_formats() {
    let config = BarcodeConfigFactory::create_default();
    let (service, mut test_db) = create_test_barcode_service(config).await;
    
    // Test different location formats
    let test_cases = vec![
        ("TST-DNA-20240115-L001-1234567", Some(1)),
        ("TST-DNA-20240115-L010-1234567", Some(10)),
        ("TST-DNA-20240115-L100-1234567", Some(100)),
        ("TST-DNA-20240115-L999-1234567", Some(999)),
        ("TST-DNA-20240115-L0001-1234567", Some(1)), // Extra digit
        ("TST-DNA-20240115-X001-1234567", None),     // Wrong prefix
        ("TST-DNA-20240115-L-1234567", None),        // No number
        ("TST-DNA-20240115-LABC-1234567", None),     // Letters instead of numbers
    ];
    
    for (barcode, expected_location) in test_cases {
        let info = service.parse_barcode(barcode);
        assert_eq!(
            info.location_component, expected_location,
            "Failed to parse location from barcode: {}",
            barcode
        );
    }
    
    test_db.cleanup().await;
}

#[tokio::test]
async fn test_parse_date_component_validation() {
    let config = BarcodeConfigFactory::create_default();
    let (service, mut test_db) = create_test_barcode_service(config).await;
    
    // Test different date-like patterns
    let test_cases = vec![
        ("TST-DNA-20240115-L001-1234567", Some("20240115")), // Valid date
        ("TST-DNA-12345678-L001-1234567", Some("12345678")), // 8 digits
        ("TST-DNA-2024-01-15-L001-1234567", None),          // Wrong format (dashes)
        ("TST-DNA-ABCD1234-L001-1234567", None),            // Contains letters
        ("TST-DNA-1234567-L001-1234567", None),             // Too short (7 digits)
        ("TST-DNA-123456789-L001-1234567", None),           // Too long (9 digits)
    ];
    
    for (barcode, expected_date) in test_cases {
        let info = service.parse_barcode(barcode);
        assert_eq!(
            info.date_component, 
            expected_date.map(|s| s.to_string()),
            "Failed to parse date from barcode: {}",
            barcode
        );
    }
    
    test_db.cleanup().await;
}

#[tokio::test]
async fn test_parse_empty_and_short_barcodes() {
    let config = BarcodeConfigFactory::create_default();
    let (service, mut test_db) = create_test_barcode_service(config).await;
    
    // Empty barcode
    let info = service.parse_barcode("");
    assert_eq!(info.full_barcode, "");
    assert!(!info.is_valid);
    assert!(info.prefix.is_none());
    
    // Single component
    let info = service.parse_barcode("TST");
    assert_eq!(info.prefix, Some("TST".to_string()));
    assert!(info.sample_type.is_none());
    assert!(!info.is_valid); // Too short
    
    // Two components
    let info = service.parse_barcode("TST-DNA");
    assert_eq!(info.prefix, Some("TST".to_string()));
    assert_eq!(info.sample_type, Some("DNA".to_string()));
    assert!(!info.is_valid); // Still too short
    
    test_db.cleanup().await;
}

#[tokio::test]
async fn test_parse_generated_barcodes() {
    let config = BarcodeConfigFactory::create_default();
    let (service, mut test_db) = create_test_barcode_service(config.clone()).await;
    
    // Generate a barcode and then parse it
    let generated = service.generate_barcode(
        Some("RNA"),
        Some(42),
        Some("RNA Extraction Protocol"),
        None,
    ).await.expect("Should generate barcode");
    
    test_db.track_barcode(generated.clone());
    
    // Parse the generated barcode
    let info = service.parse_barcode(&generated);
    
    // Verify all expected components are present
    assert_eq!(info.full_barcode, generated);
    assert_eq!(info.prefix, Some(config.prefix));
    assert!(info.sample_type.is_some());
    assert!(info.sample_type.unwrap().contains("RNA"));
    assert_eq!(info.location_component, Some(42));
    assert!(info.is_valid);
    
    if config.include_date {
        assert!(info.date_component.is_some());
    }
    
    if config.include_sequence {
        assert!(info.sequence_component.is_some());
    }
    
    test_db.cleanup().await;
}

#[tokio::test]
async fn test_parse_barcode_case_sensitivity() {
    let config = BarcodeConfigFactory::create_default();
    let (service, mut test_db) = create_test_barcode_service(config).await;
    
    // Uppercase (valid)
    let info = service.parse_barcode("TST-DNA-20240115-L001-1234567");
    assert!(info.is_valid);
    
    // Lowercase (invalid per default pattern)
    let info = service.parse_barcode("tst-dna-20240115-l001-1234567");
    assert!(!info.is_valid);
    
    // Mixed case (invalid)
    let info = service.parse_barcode("Tst-Dna-20240115-L001-1234567");
    assert!(!info.is_valid);
    
    test_db.cleanup().await;
}