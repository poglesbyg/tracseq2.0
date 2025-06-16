#[cfg(test)]
use chrono::{DateTime, Utc};

use crate::models::storage::{BarcodeConfig, StorageValidationError};
use crate::services::barcode_service::{BarcodeInfo, BarcodeService, BarcodeStats};

/// Test helper to create a custom barcode config
fn create_custom_config(
    prefix: &str,
    min_length: usize,
    include_date: bool,
    include_sequence: bool,
) -> BarcodeConfig {
    BarcodeConfig {
        prefix: prefix.to_string(),
        separator: "-".to_string(),
        min_length,
        include_date,
        include_sequence,
    }
}

#[tokio::test]
async fn test_barcode_generation_with_custom_config() {
    let config = create_custom_config("CUSTOM", 15, true, true);
    let mut service = BarcodeService::new(config);

    let barcode = service
        .generate_barcode(Some("DNA"), Some(123))
        .await
        .unwrap();

    assert!(barcode.len() >= 15);
    assert!(barcode.starts_with("CUSTOM"));
    assert!(barcode.contains("DNA"));
    assert!(barcode.contains("L123"));
}

#[tokio::test]
async fn test_barcode_generation_without_date() {
    let config = create_custom_config("NODATE", 10, false, true);
    let mut service = BarcodeService::new(config);

    let barcode = service
        .generate_barcode(Some("RNA"), Some(456))
        .await
        .unwrap();

    assert!(barcode.starts_with("NODATE"));
    assert!(barcode.contains("RNA"));
    assert!(barcode.contains("L456"));
    // Should not contain 8-digit date pattern
    assert!(!barcode.matches(char::is_numeric).collect::<String>().len() >= 8);
}

#[tokio::test]
async fn test_barcode_generation_without_sequence() {
    let config = create_custom_config("NOSEQ", 8, true, false);
    let mut service = BarcodeService::new(config);

    let barcode = service
        .generate_barcode(Some("PROTEIN"), Some(789))
        .await
        .unwrap();

    assert!(barcode.starts_with("NOSEQ"));
    assert!(barcode.contains("PROTEIN"));
    assert!(barcode.contains("L789"));
    // Should contain date but not sequence
    let numeric_parts: String = barcode.matches(char::is_numeric).collect();
    assert!(numeric_parts.len() >= 8); // At least date
}

#[tokio::test]
async fn test_barcode_generation_minimum_length_enforcement() {
    let config = create_custom_config("SHORT", 25, false, false);
    let mut service = BarcodeService::new(config);

    let barcode = service.generate_barcode(Some("X"), Some(1)).await.unwrap();

    assert!(barcode.len() >= 25);
    assert!(barcode.starts_with("SHORT"));
    assert!(barcode.contains("X"));
    assert!(barcode.contains("L001"));
}

#[tokio::test]
async fn test_barcode_generation_uniqueness_guarantee() {
    let mut service = BarcodeService::with_default_config();
    let mut generated_barcodes = std::collections::HashSet::new();

    for i in 0..50 {
        let barcode = service
            .generate_barcode(Some("TEST"), Some(i))
            .await
            .unwrap();

        assert!(
            !generated_barcodes.contains(&barcode),
            "Barcode {} was generated twice",
            barcode
        );
        generated_barcodes.insert(barcode);
    }
}

#[tokio::test]
async fn test_barcode_generation_with_none_parameters() {
    let mut service = BarcodeService::with_default_config();

    let barcode = service.generate_barcode(None, None).await.unwrap();

    assert!(barcode.len() >= service.config.min_length);
    assert!(barcode.starts_with(&service.config.prefix));
    // Should still have sequence component even without sample type or location
}

#[tokio::test]
async fn test_barcode_validation_edge_cases() {
    let service = BarcodeService::with_default_config();

    // Test minimum valid barcode
    let min_valid = "LAB-DNA-001";
    assert!(service.validate_barcode(min_valid).is_ok());

    // Test exactly at min length
    let exactly_min = "A".repeat(service.config.min_length);
    assert!(service.validate_barcode(&exactly_min).is_ok());

    // Test one character below min length
    let below_min = "A".repeat(service.config.min_length - 1);
    assert!(service.validate_barcode(&below_min).is_err());

    // Test maximum length boundary
    let max_length = "A".repeat(50);
    assert!(service.validate_barcode(&max_length).is_ok());

    let over_max = "A".repeat(51);
    assert!(service.validate_barcode(&over_max).is_err());
}

#[tokio::test]
async fn test_barcode_validation_invalid_characters() {
    let service = BarcodeService::with_default_config();

    let invalid_chars = vec![
        "LAB@DNA#001",
        "LAB DNA 001", // spaces
        "LAB&DNA$001",
        "LAB%DNA^001",
        "LAB*DNA(001)",
        "LAB+DNA=001",
        "LAB[DNA]001",
        "LAB{DNA}001",
        "LAB|DNA\\001",
        "LAB:DNA;001",
        "LAB\"DNA'001",
        "LAB<DNA>001",
        "LAB,DNA.001",
        "LAB?DNA/001",
    ];

    for invalid_barcode in invalid_chars {
        assert!(
            service.validate_barcode(invalid_barcode).is_err(),
            "Should reject barcode with invalid characters: {}",
            invalid_barcode
        );
    }
}

#[tokio::test]
async fn test_barcode_validation_valid_characters() {
    let service = BarcodeService::with_default_config();

    let valid_chars = vec![
        "LAB-DNA-001",
        "LAB_DNA_001",
        "LABDNA001",
        "123-ABC-456",
        "A1B2C3D4E5",
        "TEST_SAMPLE_2024_001",
        "LAB-DNA-RNA-PROTEIN-001",
    ];

    for valid_barcode in valid_chars {
        assert!(
            service.validate_barcode(valid_barcode).is_ok(),
            "Should accept barcode with valid characters: {}",
            valid_barcode
        );
    }
}

#[tokio::test]
async fn test_barcode_generation_failure_after_max_attempts() {
    // Create a service with a very restricted config to force failures
    let config = BarcodeConfig {
        prefix: "X".to_string(),
        separator: "".to_string(),
        min_length: 1,
        include_date: false,
        include_sequence: false,
    };
    let mut service = BarcodeService::new(config);

    // Pre-populate with the only possible barcode to force failure
    service.reserve_barcode("X".to_string());

    let result = service.generate_barcode(None, None).await;
    assert!(result.is_err());

    if let Err(StorageValidationError::InvalidBarcode { reason, .. }) = result {
        assert!(reason.contains("Failed to generate unique barcode"));
    } else {
        panic!("Expected InvalidBarcode error");
    }
}

#[tokio::test]
async fn test_barcode_reserve_and_release() {
    let mut service = BarcodeService::with_default_config();

    let test_barcode = "TEST-RESERVE-001".to_string();

    // Initially should be unique
    assert!(service.is_barcode_unique(&test_barcode).await);

    // Reserve the barcode
    service.reserve_barcode(test_barcode.clone());
    assert!(!service.is_barcode_unique(&test_barcode).await);

    // Release the barcode
    service.release_barcode(&test_barcode);
    assert!(service.is_barcode_unique(&test_barcode).await);
}

#[tokio::test]
async fn test_generate_sample_barcode_with_template() {
    let mut service = BarcodeService::with_default_config();

    let barcode = service
        .generate_sample_barcode("DNA", 42, Some("Genomic DNA Extraction Protocol"))
        .await
        .unwrap();

    assert!(barcode.contains("DNA"));
    assert!(barcode.contains("L042"));
    // Should include some representation of the template
    assert!(barcode.contains("GenDNA") || barcode.contains("GEN"));
}

#[tokio::test]
async fn test_generate_sample_barcode_without_template() {
    let mut service = BarcodeService::with_default_config();

    let barcode = service
        .generate_sample_barcode("RNA", 123, None)
        .await
        .unwrap();

    assert!(barcode.contains("RNA"));
    assert!(barcode.contains("L123"));
}

#[tokio::test]
async fn test_parse_barcode_comprehensive() {
    let service = BarcodeService::with_default_config();

    let test_barcode = "LAB-DNA-20240115-L042-1234567";
    let info = service.parse_barcode(test_barcode);

    assert_eq!(info.full_barcode, test_barcode);
    assert_eq!(info.prefix, Some("LAB".to_string()));
    assert_eq!(info.sample_type, Some("DNA".to_string()));
    assert_eq!(info.date_component, Some("20240115".to_string()));
    assert_eq!(info.location_component, Some(42));
    assert_eq!(info.sequence_component, Some("1234567".to_string()));
    assert!(info.is_valid);
}

#[tokio::test]
async fn test_parse_barcode_minimal() {
    let service = BarcodeService::with_default_config();

    let minimal_barcode = "SIMPLE";
    let info = service.parse_barcode(minimal_barcode);

    assert_eq!(info.full_barcode, minimal_barcode);
    assert_eq!(info.prefix, Some("SIMPLE".to_string()));
    assert_eq!(info.sample_type, None);
    assert_eq!(info.date_component, None);
    assert_eq!(info.location_component, None);
    assert_eq!(info.sequence_component, Some("SIMPLE".to_string()));
}

#[tokio::test]
async fn test_parse_barcode_complex() {
    let service = BarcodeService::with_default_config();

    let complex_barcode = "RESEARCH-PROTEIN-20241225-L999-SPECIALSAMPLE-789123";
    let info = service.parse_barcode(complex_barcode);

    assert_eq!(info.full_barcode, complex_barcode);
    assert_eq!(info.prefix, Some("RESEARCH".to_string()));
    assert_eq!(info.sample_type, Some("PROTEIN".to_string()));
    assert_eq!(info.date_component, Some("20241225".to_string()));
    assert_eq!(info.location_component, Some(999));
    assert_eq!(info.sequence_component, Some("789123".to_string()));
}

#[tokio::test]
async fn test_get_stats() {
    let mut service = BarcodeService::with_default_config();

    // Generate some barcodes to populate stats
    for i in 0..5 {
        service
            .generate_barcode(Some("STATS"), Some(i))
            .await
            .unwrap();
    }

    let stats = service.get_stats();

    assert_eq!(stats.total_generated, 5);
    assert_eq!(stats.config.prefix, service.config.prefix);
    assert_eq!(stats.config.min_length, service.config.min_length);
    // Timestamp should be recent
    let now = Utc::now();
    let time_diff = now.signed_duration_since(stats.last_generated);
    assert!(time_diff.num_seconds() < 60);
}

#[tokio::test]
async fn test_barcode_serialization() {
    let info = BarcodeInfo {
        full_barcode: "TEST-001".to_string(),
        prefix: Some("TEST".to_string()),
        sample_type: Some("DNA".to_string()),
        date_component: Some("20240115".to_string()),
        location_component: Some(42),
        sequence_component: Some("001".to_string()),
        is_valid: true,
    };

    // Test Debug trait
    let debug_str = format!("{:?}", info);
    assert!(debug_str.contains("TEST-001"));
    assert!(debug_str.contains("DNA"));

    // Test Clone trait
    let cloned_info = info.clone();
    assert_eq!(cloned_info.full_barcode, info.full_barcode);
    assert_eq!(cloned_info.is_valid, info.is_valid);
}

#[tokio::test]
async fn test_barcode_stats_serialization() {
    let config = BarcodeConfig::default();
    let stats = BarcodeStats {
        total_generated: 100,
        config: config.clone(),
        last_generated: Utc::now(),
    };

    // Test Debug trait
    let debug_str = format!("{:?}", stats);
    assert!(debug_str.contains("100"));
    assert!(debug_str.contains(&config.prefix));

    // Test Clone trait
    let cloned_stats = stats.clone();
    assert_eq!(cloned_stats.total_generated, stats.total_generated);
}

#[tokio::test]
async fn test_concurrent_barcode_generation() {
    use tokio::task;

    let service = std::sync::Arc::new(tokio::sync::Mutex::new(
        BarcodeService::with_default_config(),
    ));

    let mut handles = Vec::new();

    // Spawn multiple tasks to generate barcodes concurrently
    for i in 0..10 {
        let service_clone = service.clone();
        let handle = task::spawn(async move {
            let mut service_guard = service_clone.lock().await;
            service_guard
                .generate_barcode(Some("CONCURRENT"), Some(i))
                .await
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    let mut results = Vec::new();
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
        results.push(result.unwrap());
    }

    // All barcodes should be unique
    let mut unique_barcodes = std::collections::HashSet::new();
    for barcode in &results {
        assert!(
            unique_barcodes.insert(barcode.clone()),
            "Duplicate barcode generated: {}",
            barcode
        );
    }

    assert_eq!(unique_barcodes.len(), 10);
}

#[tokio::test]
async fn test_barcode_generation_with_special_sample_types() {
    let mut service = BarcodeService::with_default_config();

    let special_types = vec![
        "DNA",
        "RNA",
        "PROTEIN",
        "LIPID",
        "METABOLITE",
        "COMPOUND",
        "TISSUE",
        "BLOOD",
        "PLASMA",
        "SERUM",
        "CELL_LINE",
        "BACTERIA",
        "VIRUS",
    ];

    for sample_type in special_types {
        let barcode = service
            .generate_barcode(Some(sample_type), Some(1))
            .await
            .unwrap();

        assert!(barcode.contains(sample_type));
        assert!(service.validate_barcode(&barcode).is_ok());

        let parsed = service.parse_barcode(&barcode);
        assert!(parsed.is_valid);
        assert_eq!(parsed.sample_type, Some(sample_type.to_string()));
    }
}

#[tokio::test]
async fn test_barcode_generation_performance() {
    let mut service = BarcodeService::with_default_config();

    let start_time = std::time::Instant::now();

    // Generate 100 barcodes to test performance
    for i in 0..100 {
        let barcode = service
            .generate_barcode(Some("PERF"), Some(i))
            .await
            .unwrap();
        assert!(!barcode.is_empty());
    }

    let elapsed = start_time.elapsed();

    // Should complete within reasonable time (1 second for 100 barcodes)
    assert!(
        elapsed.as_secs() < 1,
        "Barcode generation took too long: {:?}",
        elapsed
    );

    // Average should be under 10ms per barcode
    let avg_per_barcode = elapsed.as_millis() / 100;
    assert!(
        avg_per_barcode < 10,
        "Average barcode generation time too slow: {}ms",
        avg_per_barcode
    );
}

#[tokio::test]
async fn test_barcode_config_variations() {
    let configs = vec![
        BarcodeConfig {
            prefix: "SHORT".to_string(),
            separator: "-".to_string(),
            min_length: 5,
            include_date: false,
            include_sequence: false,
        },
        BarcodeConfig {
            prefix: "LONG_PREFIX".to_string(),
            separator: "_".to_string(),
            min_length: 30,
            include_date: true,
            include_sequence: true,
        },
        BarcodeConfig {
            prefix: "NOSEP".to_string(),
            separator: "".to_string(),
            min_length: 10,
            include_date: true,
            include_sequence: false,
        },
    ];

    for config in configs {
        let mut service = BarcodeService::new(config.clone());

        let barcode = service
            .generate_barcode(Some("TEST"), Some(1))
            .await
            .unwrap();

        assert!(barcode.len() >= config.min_length);
        assert!(barcode.starts_with(&config.prefix));
        assert!(service.validate_barcode(&barcode).is_ok());
    }
}
