#[cfg(test)]
mod sample_management_tests {
    use crate::sample_submission::{CreateSample, Sample, SampleStatus, SampleSubmissionManager};
    use crate::services::sample_service::SampleService;
    use chrono::Utc;
    use serde_json::json;
    use sqlx::postgres::PgPoolOptions;
    use uuid::Uuid;

    async fn setup_test_db() -> sqlx::PgPool {
        let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/lab_manager".to_string()
        });

        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database")
    }

    #[tokio::test]
    async fn test_create_sample_with_validation() {
        let pool = setup_test_db().await;
        let manager = SampleSubmissionManager::new(pool.clone());
        let service = SampleService::new(manager);

        let create_request = CreateSample {
            name: "TEST_001".to_string(),
            barcode: "BC001TEST".to_string(),
            location: "Freezer_A_Rack_1_Box_12".to_string(),
            metadata: Some(json!({
                "project": "Genomics Study",
                "patient_id": "P001",
                "sample_type": "DNA",
                "concentration": 125.5,
                "volume": 50.0,
                "quality_score": 8.5
            })),
        };

        let result = service.create_sample(create_request).await;
        assert!(result.is_ok(), "Creating valid sample should succeed");

        let sample = result.unwrap();
        assert_eq!(sample.name, "TEST_001");
        assert_eq!(sample.barcode, "BC001TEST");
        assert_eq!(sample.location, "Freezer_A_Rack_1_Box_12");
        assert_eq!(sample.status, SampleStatus::Pending);

        // Check metadata
        assert!(sample.metadata.is_object());
        assert_eq!(sample.metadata["project"], "Genomics Study");
        assert_eq!(sample.metadata["concentration"], 125.5);
    }

    #[tokio::test]
    async fn test_sample_status_transitions() {
        let pool = setup_test_db().await;
        let manager = SampleSubmissionManager::new(pool.clone());
        let service = SampleService::new(manager);

        // Create sample in pending status
        let create_request = CreateSample {
            name: "STATUS_TEST".to_string(),
            barcode: "RNA001".to_string(),
            location: "Freezer_B_Rack_2".to_string(),
            metadata: Some(json!({
                "sample_type": "RNA",
                "source_organism": "Mus musculus",
                "source_tissue": "Liver",
                "concentration": 80.0,
                "volume": 25.0,
                "quality_score": 7.0
            })),
        };

        let sample = service
            .create_sample(create_request)
            .await
            .expect("Creating sample should succeed");

        // Verify initial status
        assert_eq!(sample.status, SampleStatus::Pending);

        // Validate the sample (moves to validated status)
        let validated_sample = service
            .validate_sample(sample.id)
            .await
            .expect("Validating sample should succeed");

        assert_eq!(validated_sample.status, SampleStatus::Validated);
        assert_eq!(validated_sample.id, sample.id);
    }

    #[tokio::test]
    async fn test_sample_retrieval() {
        let pool = setup_test_db().await;
        let manager = SampleSubmissionManager::new(pool.clone());
        let service = SampleService::new(manager);

        // Create a sample
        let create_request = CreateSample {
            name: "RETRIEVAL_TEST".to_string(),
            barcode: "BC_RETRIEVAL_001".to_string(),
            location: "Test_Location".to_string(),
            metadata: Some(json!({
                "sample_type": "DNA",
                "source_organism": "Homo sapiens",
                "concentration": 100.0
            })),
        };

        let created_sample = service
            .create_sample(create_request)
            .await
            .expect("Creating sample should succeed");

        // Retrieve the sample by ID
        let retrieved_sample = service
            .get_sample(created_sample.id)
            .await
            .expect("Retrieving sample should succeed");

        assert_eq!(retrieved_sample.id, created_sample.id);
        assert_eq!(retrieved_sample.name, "RETRIEVAL_TEST");
        assert_eq!(retrieved_sample.barcode, "BC_RETRIEVAL_001");
    }

    #[tokio::test]
    async fn test_list_samples() {
        let pool = setup_test_db().await;
        let manager = SampleSubmissionManager::new(pool.clone());
        let service = SampleService::new(manager);

        // Create multiple samples
        let samples_to_create = vec![
            ("DNA_SAMPLE_001", "BC_DNA_001", "Location_A"),
            ("RNA_SAMPLE_001", "BC_RNA_001", "Location_B"),
            ("PROTEIN_001", "BC_PROT_001", "Location_C"),
        ];

        for (name, barcode, location) in samples_to_create {
            let create_request = CreateSample {
                name: name.to_string(),
                barcode: barcode.to_string(),
                location: location.to_string(),
                metadata: Some(json!({
                    "sample_type": "DNA",
                    "concentration": 100.0
                })),
            };

            service
                .create_sample(create_request)
                .await
                .expect("Creating sample should succeed");
        }

        // List all samples
        let samples_list = service
            .list_samples()
            .await
            .expect("Listing samples should succeed");

        assert!(samples_list.len() >= 3, "Should have at least 3 samples");
    }

    #[test]
    fn test_sample_metadata_structure() {
        let metadata = json!({
            "collection_date": "2024-01-15",
            "source_organism": "Homo sapiens",
            "sample_type": "DNA",
            "quality_metrics": {
                "concentration": 125.5,
                "purity": 1.8,
                "integrity": 8.5
            },
            "storage_conditions": {
                "temperature": -80.0,
                "location": "Freezer_A_Rack_1"
            }
        });

        assert!(metadata.is_object());
        assert_eq!(metadata["source_organism"], "Homo sapiens");
        assert_eq!(metadata["sample_type"], "DNA");
        assert_eq!(metadata["quality_metrics"]["concentration"], 125.5);
        assert_eq!(metadata["storage_conditions"]["temperature"], -80.0);
    }

    #[test]
    fn test_sample_barcode_format() {
        let valid_barcodes = vec!["DNA001", "RNA_001_A1", "PROT-2024-001", "BC123456789"];

        for barcode in valid_barcodes {
            assert!(!barcode.is_empty());
            assert!(barcode.len() >= 6);
            // Basic format validation
            assert!(barcode
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-'));
        }
    }

    #[test]
    fn test_quality_score_validation() {
        let valid_scores = vec![0.0, 5.5, 7.8, 9.0, 10.0];
        let invalid_scores = vec![-1.0, 11.0, 15.5];

        for score in valid_scores {
            assert!(
                score >= 0.0 && score <= 10.0,
                "Score {} should be valid",
                score
            );
        }

        for score in invalid_scores {
            assert!(
                score < 0.0 || score > 10.0,
                "Score {} should be invalid",
                score
            );
        }
    }

    #[test]
    fn test_concentration_units() {
        // Test concentration values and units
        let concentrations = vec![
            (50.0, "ng/μL"),
            (125.5, "ng/μL"),
            (0.85, "μg/μL"),
            (1200.0, "ng/μL"),
        ];

        for (value, unit) in concentrations {
            assert!(value > 0.0, "Concentration should be positive");
            assert!(!unit.is_empty(), "Unit should not be empty");
            assert!(
                unit.contains("ng") || unit.contains("μg"),
                "Should contain valid units"
            );
        }
    }

    #[test]
    fn test_sample_status_values() {
        // Test that sample status enum has expected values
        let pending = SampleStatus::Pending;
        let validated = SampleStatus::Validated;
        let in_storage = SampleStatus::InStorage;
        let in_sequencing = SampleStatus::InSequencing;
        let completed = SampleStatus::Completed;

        // Basic enum validation
        assert_ne!(pending, validated);
        assert_ne!(validated, in_storage);
        assert_ne!(in_storage, in_sequencing);
        assert_ne!(in_sequencing, completed);
    }

    #[test]
    fn test_storage_location_hierarchy() {
        let storage_locations = vec![
            "Freezer_A_Rack_1_Box_1_Position_A1",
            "Fridge_B_Shelf_2_Tray_3",
            "Room_Temp_Cabinet_C_Drawer_1",
            "LN2_Tank_1_Rack_A_Box_5",
        ];

        for location in storage_locations {
            assert!(!location.is_empty());
            assert!(
                location.contains("_"),
                "Location should have hierarchy separators"
            );

            let parts: Vec<&str> = location.split('_').collect();
            assert!(
                parts.len() >= 3,
                "Location should have at least 3 hierarchy levels"
            );
        }
    }

    #[test]
    fn test_sample_volume_tracking() {
        let initial_volume = 100.0;
        let volumes = vec![
            (10.0, 90.0), // Used 10μL, 90μL remaining
            (25.0, 65.0), // Used 25μL, 65μL remaining
            (30.0, 35.0), // Used 30μL, 35μL remaining
        ];

        let mut current_volume = initial_volume;

        for (used, expected_remaining) in volumes {
            current_volume -= used;
            assert_eq!(current_volume, expected_remaining);
            assert!(current_volume >= 0.0, "Volume should not be negative");
        }
    }

    #[test]
    fn test_laboratory_workflow_steps() {
        let workflow_steps = vec![
            ("Collection", "Sample collected from source"),
            ("Processing", "Sample being processed/extracted"),
            ("QC", "Quality control assessment"),
            ("Storage", "Long-term storage"),
            ("Analysis", "Sample used for analysis"),
        ];

        for (step, description) in workflow_steps {
            assert!(!step.is_empty());
            assert!(!description.is_empty());
            assert!(description.len() > 10, "Description should be meaningful");
        }
    }

    #[test]
    fn test_sample_relationship_tracking() {
        let parent_id = Uuid::new_v4();
        let child_samples = vec![
            (Uuid::new_v4(), "Aliquot 1"),
            (Uuid::new_v4(), "Aliquot 2"),
            (Uuid::new_v4(), "Extraction A"),
            (Uuid::new_v4(), "Extraction B"),
        ];

        // Test parent-child relationships
        for (child_id, relationship_type) in child_samples {
            assert_ne!(
                parent_id, child_id,
                "Parent and child should have different IDs"
            );
            assert!(!relationship_type.is_empty());

            // Test relationship metadata
            let relationship_metadata = json!({
                "parent_id": parent_id,
                "child_id": child_id,
                "relationship_type": relationship_type,
                "created_date": "2024-01-15"
            });

            assert_eq!(relationship_metadata["parent_id"], json!(parent_id));
            assert_eq!(
                relationship_metadata["relationship_type"],
                relationship_type
            );
        }
    }

    #[tokio::test]
    async fn test_sample_barcode_uniqueness() {
        let pool = setup_test_db().await;
        let manager = SampleSubmissionManager::new(pool.clone());
        let service = SampleService::new(manager);

        let barcode = "UNIQUE_BC_001";

        // Create first sample with barcode
        let first_sample = service
            .create_sample(CreateSample {
                name: "FIRST_SAMPLE".to_string(),
                barcode: barcode.to_string(),
                location: "Test_Location".to_string(),
                metadata: Some(json!({"sample_type": "DNA"})),
            })
            .await
            .expect("Creating first sample should succeed");

        // Try to create second sample with same barcode
        let duplicate_result = service
            .create_sample(CreateSample {
                name: "SECOND_SAMPLE".to_string(),
                barcode: barcode.to_string(),
                location: "Test_Location_2".to_string(),
                metadata: Some(json!({"sample_type": "RNA"})),
            })
            .await;

        // Should fail due to unique constraint
        assert!(
            duplicate_result.is_err(),
            "Creating sample with duplicate barcode should fail"
        );
    }

    #[tokio::test]
    async fn test_sample_quality_control() {
        let pool = setup_test_db().await;
        let manager = SampleSubmissionManager::new(pool.clone());
        let service = SampleService::new(manager);

        // Create sample with quality metrics
        let create_request = CreateSample {
            name: "QC_TEST_SAMPLE".to_string(),
            barcode: "QC_BC_001".to_string(),
            location: "QC_Storage".to_string(),
            metadata: Some(json!({
                "sample_type": "DNA",
                "source_organism": "Homo sapiens",
                "source_tissue": "Blood",
                "concentration": 150.0,
                "volume": 100.0,
                "quality_score": 9.2,
                "purity_260_280": 1.8,
                "purity_260_230": 2.1,
                "integrity_number": 8.5,
                "qc_passed": true
            })),
        };

        let sample = service
            .create_sample(create_request)
            .await
            .expect("Creating QC sample should succeed");

        // Verify quality metrics in metadata
        assert!(sample.metadata.is_object());
        assert_eq!(sample.metadata["quality_score"], 9.2);
        assert_eq!(sample.metadata["concentration"], 150.0);
        assert_eq!(sample.metadata["volume"], 100.0);
        assert_eq!(sample.metadata["qc_passed"], true);
        assert_eq!(sample.metadata["purity_260_280"], 1.8);
    }

    #[test]
    fn test_sample_validation_rules() {
        // Test basic validation rules for sample creation
        let valid_sample = CreateSample {
            name: "VALID_SAMPLE".to_string(),
            barcode: "VALID_BC_001".to_string(),
            location: "Valid_Location".to_string(),
            metadata: Some(json!({"sample_type": "DNA"})),
        };

        // Name validation
        assert!(!valid_sample.name.is_empty());
        assert!(valid_sample.name.len() >= 3);

        // Barcode validation
        assert!(!valid_sample.barcode.is_empty());
        assert!(valid_sample.barcode.len() >= 6);

        // Location validation
        assert!(!valid_sample.location.is_empty());
        assert!(valid_sample.location.len() >= 3);
    }

    #[test]
    fn test_sample_lifecycle_metadata() {
        // Test metadata for tracking sample lifecycle
        let lifecycle_metadata = json!({
            "collection_date": "2024-01-15T10:30:00Z",
            "processing_steps": [
                {
                    "step": "extraction",
                    "date": "2024-01-16T09:00:00Z",
                    "technician": "tech001"
                },
                {
                    "step": "quality_control",
                    "date": "2024-01-16T14:30:00Z",
                    "technician": "qc_tech002"
                }
            ],
            "status_history": [
                {"status": "pending", "timestamp": "2024-01-15T10:30:00Z"},
                {"status": "validated", "timestamp": "2024-01-16T15:00:00Z"}
            ]
        });

        assert!(lifecycle_metadata.is_object());
        assert!(lifecycle_metadata["processing_steps"].is_array());
        assert!(lifecycle_metadata["status_history"].is_array());

        let processing_steps = lifecycle_metadata["processing_steps"].as_array().unwrap();
        assert_eq!(processing_steps.len(), 2);
        assert_eq!(processing_steps[0]["step"], "extraction");
        assert_eq!(processing_steps[1]["step"], "quality_control");
    }
}
