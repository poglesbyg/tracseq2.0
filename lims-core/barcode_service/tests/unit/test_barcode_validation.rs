//! Unit tests for barcode validation functionality

use barcode_service::{
    service::BarcodeService,
    error::{BarcodeError, Result},
};
use crate::test_utils::*;

#[tokio::test]
async fn test_validate_valid_barcodes() {
    let config = BarcodeConfigFactory::create_default();
    let (service, mut test_db) = create_test_barcode_service(config).await;
    
    for valid_barcode in TestDataGenerator::valid_barcodes() {
        let result = service.validate_barcode_format(valid_barcode);
        assert!(
            result.is_ok(),
            "Barcode '{}' should be valid, but got error: {:?}",
            valid_barcode,
            result
        );
    }
    
    test_db.cleanup().await;
}

#[tokio::test]
async fn test_validate_invalid_barcodes() {
    let config = BarcodeConfigFactory::create_default();
    let (service, mut test_db) = create_test_barcode_service(config).await;
    
    // Test empty barcode
    let result = service.validate_barcode_format("");
    assert!(result.is_err());
    assert!(matches!(result, Err(BarcodeError::ValidationError(_))));
    
    // Test too short barcode
    let result = service.validate_barcode_format("ABC");
    assert!(result.is_err());
    assert!(matches!(result, Err(BarcodeError::ValidationError(_))));
    
    // Test invalid characters
    let result = service.validate_barcode_format("TST-@#$-123");
    assert!(result.is_err());
    assert!(matches!(result, Err(BarcodeError::ValidationError(_))));
    
    // Test lowercase (not allowed in default config)
    let result = service.validate_barcode_format("tst-dna-12345");
    assert!(result.is_err());
    assert!(matches!(result, Err(BarcodeError::ValidationError(_))));
    
    test_db.cleanup().await;
}

#[tokio::test]
async fn test_validate_minimum_length() {
    let config = BarcodeConfigFactory::create_default(); // min_length = 10
    let (service, mut test_db) = create_test_barcode_service(config).await;
    
    // Test exactly minimum length
    let barcode = "A".repeat(10);
    assert!(service.validate_barcode_format(&barcode).is_ok());
    
    // Test one less than minimum
    let barcode = "A".repeat(9);
    assert!(service.validate_barcode_format(&barcode).is_err());
    
    // Test well above minimum
    let barcode = "A".repeat(20);
    assert!(service.validate_barcode_format(&barcode).is_ok());
    
    test_db.cleanup().await;
}

#[tokio::test]
async fn test_validate_maximum_length() {
    let config = BarcodeConfigFactory::create_default();
    let (service, mut test_db) = create_test_barcode_service(config).await;
    
    // Test at maximum allowed length (50)
    let barcode = "A".repeat(50);
    assert!(service.validate_barcode_format(&barcode).is_ok());
    
    // Test exceeding maximum length
    let barcode = "A".repeat(51);
    let result = service.validate_barcode_format(&barcode);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("exceeds maximum length"));
    
    test_db.cleanup().await;
}

#[tokio::test]
async fn test_validate_special_characters() {
    let config = BarcodeConfigFactory::create_default();
    let (service, mut test_db) = create_test_barcode_service(config).await;
    
    let special_char_barcodes = vec![
        "TST!DNA!12345",      // Exclamation marks
        "TST@DNA@12345",      // At symbols
        "TST#DNA#12345",      // Hash symbols
        "TST$DNA$12345",      // Dollar signs
        "TST%DNA%12345",      // Percent signs
        "TST^DNA^12345",      // Caret symbols
        "TST&DNA&12345",      // Ampersands
        "TST*DNA*12345",      // Asterisks
        "TST(DNA)12345",      // Parentheses
        "TST[DNA]12345",      // Square brackets
        "TST{DNA}12345",      // Curly brackets
        "TST<DNA>12345",      // Angle brackets
        "TST DNA 12345",      // Spaces
        "TST\tDNA\t12345",    // Tabs
        "TST\nDNA\n12345",    // Newlines
    ];
    
    for barcode in special_char_barcodes {
        let result = service.validate_barcode_format(barcode);
        assert!(
            result.is_err(),
            "Barcode '{}' with special characters should be invalid",
            barcode
        );
    }
    
    test_db.cleanup().await;
}

#[tokio::test]
async fn test_validate_with_different_separators() {
    // Test with dash separator
    let config = BarcodeConfigFactory::create_default(); // Uses "-" separator
    let (service, mut test_db) = create_test_barcode_service(config).await;
    
    assert!(service.validate_barcode_format("TST-DNA-12345").is_ok());
    assert!(service.validate_barcode_format("TST_DNA_12345").is_err()); // Wrong separator
    
    test_db.cleanup().await;
    
    // Test with underscore separator
    let config = BarcodeConfigFactory::create_complex(); // Uses "_" separator
    let (service, mut test_db) = create_test_barcode_service(config).await;
    
    assert!(service.validate_barcode_format("COMPLEX_RNA_12345678901234567890").is_ok());
    assert!(service.validate_barcode_format("COMPLEX-RNA-12345678901234567890").is_err()); // Wrong separator
    
    test_db.cleanup().await;
}

#[tokio::test]
async fn test_validate_regex_patterns() {
    // Test custom validation pattern
    let mut config = BarcodeConfigFactory::create_default();
    config.validation_pattern = r"^TST-[A-Z]{3}-\d{8}-L\d{3}-\d{7}$".to_string();
    
    let (service, mut test_db) = create_test_barcode_service(config).await;
    
    // Should match pattern exactly
    assert!(service.validate_barcode_format("TST-DNA-20240115-L001-1234567").is_ok());
    
    // Should fail - wrong prefix
    assert!(service.validate_barcode_format("XYZ-DNA-20240115-L001-1234567").is_err());
    
    // Should fail - sample type too long
    assert!(service.validate_barcode_format("TST-DNAX-20240115-L001-1234567").is_err());
    
    // Should fail - date wrong format
    assert!(service.validate_barcode_format("TST-DNA-2024-01-15-L001-1234567").is_err());
    
    // Should fail - missing location prefix
    assert!(service.validate_barcode_format("TST-DNA-20240115-001-1234567").is_err());
    
    test_db.cleanup().await;
}

#[tokio::test]
async fn test_uniqueness_check() {
    let config = BarcodeConfigFactory::create_default();
    let (service, mut test_db) = create_test_barcode_service(config).await;
    
    let barcode = "TST-DNA-20240115-L001-1234567";
    
    // First check - should be unique
    assert!(service.check_barcode_unique(barcode).await.unwrap());
    
    // Generate and store the barcode
    let generated = service.generate_barcode(
        Some("DNA"),
        Some(1),
        None,
        Some("TST-DNA-20240115-L001-1234567"), // Force specific barcode
    ).await;
    
    // If generation succeeded, it should now not be unique
    if generated.is_ok() {
        assert!(!service.check_barcode_unique(barcode).await.unwrap());
    }
    
    test_db.cleanup().await;
}

// Performance test for validation
#[tokio::test]
async fn test_validation_performance() {
    let config = BarcodeConfigFactory::create_default();
    let (service, mut test_db) = create_test_barcode_service(config).await;
    
    let test_barcodes: Vec<&str> = TestDataGenerator::valid_barcodes()
        .into_iter()
        .chain(TestDataGenerator::invalid_barcodes())
        .collect();
    
    let duration = PerformanceTestUtils::measure_validation_time(&service, &test_barcodes);
    
    // Validation should be very fast
    assert!(
        duration.as_millis() < 100,
        "Validating {} barcodes took {:?}, which is too slow",
        test_barcodes.len(),
        duration
    );
    
    println!("Validated {} barcodes in {:?}", test_barcodes.len(), duration);
    
    test_db.cleanup().await;
}

#[tokio::test]
async fn test_validation_error_messages() {
    let config = BarcodeConfigFactory::create_default();
    let (service, mut test_db) = create_test_barcode_service(config).await;
    
    // Test empty barcode error
    let result = service.validate_barcode_format("");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    
    // Test minimum length error
    let result = service.validate_barcode_format("SHORT");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("at least"));
    
    // Test invalid characters error
    let result = service.validate_barcode_format("TST-DNA-12345!");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("invalid characters"));
    
    // Test maximum length error
    let result = service.validate_barcode_format(&"A".repeat(51));
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("exceeds maximum"));
    
    test_db.cleanup().await;
}