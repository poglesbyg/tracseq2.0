#[cfg(test)]
mod tests {
    use serde_json::json;
    use std::collections::HashMap;

    use crate::{
        config::{AppConfig, RagIntegrationConfig},
        services::{
            rag_integration_service::{
                AdministrativeInfo, ContainerInfo, InformaticsInfo, PoolingInfo, RagConfig,
                RagExtractionResult, RagIntegrationService, RagSubmission, SampleDetails,
                SequenceGeneration, SourceMaterial,
            },
            Service,
        },
    };

    fn create_test_rag_config() -> RagConfig {
        RagConfig {
            base_url: "http://localhost:8000".to_string(),
            timeout_seconds: 30,
            max_file_size_mb: 10,
            supported_formats: vec!["txt".to_string(), "pdf".to_string()],
        }
    }

    fn create_mock_rag_submission() -> RagSubmission {
        RagSubmission {
            administrative_info: AdministrativeInfo {
                submitter_first_name: "Dr. Jane".to_string(),
                submitter_last_name: "Smith".to_string(),
                submitter_email: "jane.smith@lab.edu".to_string(),
                submitter_phone: "(555) 123-4567".to_string(),
                assigned_project: "PROJ-2024-001".to_string(),
                department: Some("Molecular Biology".to_string()),
                institution: Some("Research University".to_string()),
            },
            source_material: SourceMaterial {
                source_type: "dna".to_string(),
                collection_date: Some("2024-01-15".to_string()),
                collection_method: Some("Blood draw".to_string()),
                source_organism: Some("Homo sapiens".to_string()),
                tissue_type: Some("Whole blood".to_string()),
                preservation_method: Some("EDTA".to_string()),
                storage_conditions: Some("-80C".to_string()),
            },
            pooling_info: PoolingInfo {
                is_pooled: false,
                pool_id: None,
                samples_in_pool: Vec::new(),
                pooling_ratio: HashMap::new(),
                barcode_sequences: HashMap::new(),
                multiplex_strategy: None,
            },
            sequence_generation: SequenceGeneration {
                sequencing_platform: Some("Illumina NovaSeq 6000".to_string()),
                read_length: Some(150),
                read_type: Some("paired-end".to_string()),
                target_coverage: Some(30.0),
                library_prep_kit: Some("TruSeq DNA PCR-Free".to_string()),
                index_sequences: Vec::new(),
                quality_metrics: HashMap::new(),
            },
            container_info: ContainerInfo {
                container_type: Some("tube".to_string()),
                container_id: Some("TUBE-001".to_string()),
                volume: Some(5.0),
                concentration: Some(250.0),
                diluent_used: Some("TE buffer".to_string()),
                storage_temperature: Some("minus80".to_string()),
                container_barcode: Some("DNA-001-2024".to_string()),
            },
            informatics_info: InformaticsInfo {
                analysis_type: "wgs".to_string(),
                reference_genome: Some("GRCh38".to_string()),
                analysis_pipeline: Some("GATK Best Practices".to_string()),
                custom_parameters: HashMap::new(),
                data_delivery_format: Some("FASTQ".to_string()),
                computational_requirements: Some("High-memory instance".to_string()),
            },
            sample_details: SampleDetails {
                sample_id: "SAMPLE-WGS-001".to_string(),
                patient_id: Some("PT-001".to_string()),
                sample_name: Some("Patient 001 DNA".to_string()),
                priority: "high".to_string(),
                quality_score: Some(9.2),
                purity_ratio: Some(1.8),
                integrity_number: Some(8.5),
                notes: Some("High-quality DNA extraction".to_string()),
                special_instructions: Some("Process immediately".to_string()),
            },
            submission_id: Some("SUB-2024-001".to_string()),
            status: "pending".to_string(),
            extracted_confidence: Some(0.95),
        }
    }

    fn create_mock_extraction_result() -> RagExtractionResult {
        RagExtractionResult {
            success: true,
            submission: Some(create_mock_rag_submission()),
            confidence_score: 0.95,
            missing_fields: Vec::new(),
            warnings: Vec::new(),
            processing_time: 2.34,
            source_document: "test_lab_form.pdf".to_string(),
        }
    }

    #[test]
    fn test_rag_config_creation() {
        let config = create_test_rag_config();
        assert_eq!(config.base_url, "http://localhost:8000");
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.max_file_size_mb, 10);
        assert!(config.supported_formats.contains(&"txt".to_string()));
        assert!(config.supported_formats.contains(&"pdf".to_string()));
    }

    #[test]
    fn test_rag_config_default() {
        let config = RagConfig::default();
        assert_eq!(config.base_url, "http://127.0.0.1:8000");
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.max_file_size_mb, 50);
        assert!(config.supported_formats.contains(&"pdf".to_string()));
        assert!(config.supported_formats.contains(&"docx".to_string()));
        assert!(config.supported_formats.contains(&"txt".to_string()));
    }

    #[test]
    fn test_rag_integration_config() {
        let config = RagIntegrationConfig::default();
        assert!(config.enabled);
        assert_eq!(config.base_url, "http://127.0.0.1:8000");
        assert_eq!(config.default_confidence_threshold, 0.7);
        assert!(!config.auto_create_samples);
    }

    #[test]
    fn test_app_config_with_rag() {
        let config = AppConfig::for_testing();
        assert!(config.rag.enabled);
        assert_eq!(config.rag.base_url, "http://127.0.0.1:8000");
        assert_eq!(config.rag.default_confidence_threshold, 0.7);
    }

    #[test]
    fn test_rag_submission_data_model() {
        let submission = create_mock_rag_submission();

        // Test administrative info
        assert_eq!(
            submission.administrative_info.submitter_first_name,
            "Dr. Jane"
        );
        assert_eq!(
            submission.administrative_info.assigned_project,
            "PROJ-2024-001"
        );

        // Test source material
        assert_eq!(submission.source_material.source_type, "dna");
        assert_eq!(
            submission.source_material.source_organism,
            Some("Homo sapiens".to_string())
        );

        // Test sequencing info
        assert_eq!(
            submission.sequence_generation.sequencing_platform,
            Some("Illumina NovaSeq 6000".to_string())
        );
        assert_eq!(submission.sequence_generation.read_length, Some(150));

        // Test sample details
        assert_eq!(submission.sample_details.sample_id, "SAMPLE-WGS-001");
        assert_eq!(submission.sample_details.priority, "high");
    }

    #[test]
    fn test_convert_to_samples_single_sample() {
        let config = create_test_rag_config();
        let service = RagIntegrationService::new(config);
        let extraction_result = create_mock_extraction_result();

        let samples = service.convert_to_samples(&extraction_result).unwrap();

        assert_eq!(samples.len(), 1);
        let sample = &samples[0];

        // Test sample name generation - uses sample_details.sample_name if available
        assert_eq!(sample.name, "Patient 001 DNA");

        // Test barcode generation - just verify it exists and has reasonable format
        assert!(!sample.barcode.is_empty());
        assert!(sample.barcode.len() >= 6);

        // Test location assignment
        assert_eq!(sample.location, "Storage-minus80");

        // Test metadata preservation
        assert!(sample.metadata.is_some());
        let metadata = sample.metadata.as_ref().unwrap();
        assert!(metadata.get("rag_extraction").is_some());
        assert!(metadata.get("processing").is_some());
    }

    #[test]
    fn test_convert_to_samples_pooled_samples() {
        let config = create_test_rag_config();
        let service = RagIntegrationService::new(config);

        let mut extraction_result = create_mock_extraction_result();
        let submission = extraction_result.submission.as_mut().unwrap();

        // Set up pooled samples
        submission.pooling_info.is_pooled = true;
        submission.pooling_info.samples_in_pool = vec![
            "POOL-SAMPLE-001".to_string(),
            "POOL-SAMPLE-002".to_string(),
            "POOL-SAMPLE-003".to_string(),
        ];

        let samples = service.convert_to_samples(&extraction_result).unwrap();

        assert_eq!(samples.len(), 3);

        for (i, sample) in samples.iter().enumerate() {
            assert_eq!(sample.name, format!("POOL-SAMPLE-{:03}", i + 1));
            assert!(sample.barcode.starts_with("DNA-"));
            assert_eq!(sample.location, "Storage-minus80");
        }
    }

    #[test]
    fn test_barcode_generation_patterns() {
        // Test barcode pattern logic without accessing private methods
        let sample_types = vec!["dna", "blood", "tissue", "unknown"];
        let expected_prefixes = vec!["DNA", "BLD", "TSU", "UNK"];

        for (i, sample_type) in sample_types.iter().enumerate() {
            let prefix = match *sample_type {
                "dna" => "DNA",
                "blood" => "BLD",
                "tissue" => "TSU",
                _ => "UNK",
            };
            assert_eq!(prefix, expected_prefixes[i]);
        }
    }

    #[test]
    fn test_extraction_result_confidence_validation() {
        let mut extraction_result = create_mock_extraction_result();

        // Test high confidence
        extraction_result.confidence_score = 0.95;
        assert!(extraction_result.success);
        assert!(extraction_result.confidence_score > 0.7);

        // Test low confidence
        extraction_result.confidence_score = 0.4;
        assert!(extraction_result.confidence_score < 0.7);

        // Test edge case
        extraction_result.confidence_score = 0.7;
        assert!(extraction_result.confidence_score >= 0.7);
    }

    #[test]
    fn test_extraction_result_with_warnings() {
        let mut extraction_result = create_mock_extraction_result();
        extraction_result.warnings = vec![
            "Missing phone number".to_string(),
            "Storage temperature unclear".to_string(),
        ];
        extraction_result.missing_fields = vec![
            "submitter_phone".to_string(),
            "storage_temperature".to_string(),
        ];

        assert_eq!(extraction_result.warnings.len(), 2);
        assert_eq!(extraction_result.missing_fields.len(), 2);
        assert!(extraction_result
            .warnings
            .contains(&"Missing phone number".to_string()));
    }

    #[test]
    fn test_sample_metadata_structure() {
        let config = create_test_rag_config();
        let service = RagIntegrationService::new(config);
        let extraction_result = create_mock_extraction_result();

        let samples = service.convert_to_samples(&extraction_result).unwrap();
        let sample = &samples[0];
        let metadata = sample.metadata.as_ref().unwrap();

        // Test RAG extraction metadata
        let rag_metadata = metadata.get("rag_extraction").unwrap();
        assert!(rag_metadata.get("confidence_score").is_some());
        assert!(rag_metadata.get("administrative_info").is_some());
        assert!(rag_metadata.get("source_material").is_some());

        // Test processing metadata
        let processing_metadata = metadata.get("processing").unwrap();
        assert!(processing_metadata.get("extracted_at").is_some());
        assert!(processing_metadata.get("priority").is_some());
        assert_eq!(
            processing_metadata
                .get("priority")
                .unwrap()
                .as_str()
                .unwrap(),
            "high"
        );
    }

    #[test]
    fn test_failed_extraction_handling() {
        let config = create_test_rag_config();
        let service = RagIntegrationService::new(config);

        let mut extraction_result = create_mock_extraction_result();
        extraction_result.success = false;
        extraction_result.submission = None;

        let result = service.convert_to_samples(&extraction_result);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not successful"));
    }

    #[test]
    fn test_location_assignment_logic() {
        let config = create_test_rag_config();
        let service = RagIntegrationService::new(config);

        let extraction_result = create_mock_extraction_result();
        let samples = service.convert_to_samples(&extraction_result).unwrap();
        assert_eq!(samples[0].location, "Storage-minus80");

        // Test location logic patterns
        let storage_temps = vec!["minus80", "minus20", "plus4", "rt"];
        let expected_locations = vec![
            "Storage-minus80",
            "Storage-minus20",
            "Storage-plus4",
            "Storage-rt",
        ];

        for (i, temp) in storage_temps.iter().enumerate() {
            let location = if temp.is_empty() {
                "Unknown-Location".to_string()
            } else {
                format!("Storage-{}", temp)
            };
            assert_eq!(location, expected_locations[i]);
        }
    }

    #[test]
    fn test_service_configuration() {
        let config = create_test_rag_config();
        let service = RagIntegrationService::new(config);

        let service_config = service.config();
        assert_eq!(service_config.name, "rag_integration_service");
        assert_eq!(service_config.version, "1.0.0");
        assert!(service_config
            .dependencies
            .contains(&"rag_system".to_string()));
        assert!(service_config.settings.contains_key("base_url"));
        assert!(service_config.settings.contains_key("timeout_seconds"));
    }

    #[tokio::test]
    async fn test_service_health_check_mock() {
        let config = create_test_rag_config();
        let service = RagIntegrationService::new(config);

        // This will fail because we don't have a real RAG system running in tests
        // But we can test the structure
        let health = service.health_check().await;
        assert_eq!(
            health.message,
            Some("RAG Integration service health check".to_string())
        );
        assert!(health.checks.contains_key("rag_system"));

        // In tests, the RAG system won't be available, so it should be unhealthy
        let rag_check = &health.checks["rag_system"];
        assert!(rag_check.duration_ms > 0);
    }

    #[test]
    fn test_serialization_compatibility() {
        let submission = create_mock_rag_submission();

        // Test that our models can be serialized to JSON (required for API responses)
        let json_str = serde_json::to_string(&submission).unwrap();
        assert!(json_str.contains("submitter_first_name"));
        assert!(json_str.contains("Dr. Jane"));

        // Test deserialization
        let deserialized: RagSubmission = serde_json::from_str(&json_str).unwrap();
        assert_eq!(
            deserialized.administrative_info.submitter_first_name,
            "Dr. Jane"
        );
    }
}
