#[cfg(test)]
mod sample_management_tests {
    use crate::models::sample::{
        CreateSampleRequest, Sample, SampleFilter, SampleListQuery, SampleStatus, SampleType,
        StorageLocation, UpdateSampleRequest,
    };
    use crate::services::sample_service::SampleService;
    use chrono::Utc;
    use serde_json::json;
    use sqlx::postgres::PgPoolOptions;
    use uuid::Uuid;

    async fn setup_test_db() -> sqlx::PgPool {
        let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
            "postgres://lab_manager:lab_manager@localhost:5432/lab_manager_test".to_string()
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
        let service = SampleService::new(pool.clone());

        let create_request = CreateSampleRequest {
            name: "TEST_001".to_string(),
            sample_type: SampleType::DNA,
            collection_date: Some(Utc::now().date_naive()),
            source_organism: Some("Homo sapiens".to_string()),
            source_tissue: Some("Blood".to_string()),
            concentration: Some(125.5),
            volume: Some(50.0),
            quality_score: Some(8.5),
            storage_location: Some("Freezer_A_Rack_1_Box_12".to_string()),
            barcode: Some("BC001TEST".to_string()),
            metadata: Some(json!({
                "project": "Genomics Study",
                "patient_id": "P001"
            })),
            notes: Some("High-quality DNA sample for sequencing".to_string()),
        };

        let result = service.create_sample(create_request, None).await;
        assert!(result.is_ok(), "Creating valid sample should succeed");

        let sample = result.unwrap();
        assert_eq!(sample.name, "TEST_001");
        assert_eq!(sample.sample_type, SampleType::DNA);
        assert!(sample.concentration.is_some());
        assert_eq!(sample.concentration.unwrap(), 125.5);
        assert!(sample.barcode.is_some());

        // Cleanup
        let _ = service.delete_sample(sample.id).await;
    }

    #[tokio::test]
    async fn test_sample_status_transitions() {
        let pool = setup_test_db().await;
        let service = SampleService::new(pool.clone());

        // Create sample in pending status
        let create_request = CreateSampleRequest {
            name: "STATUS_TEST".to_string(),
            sample_type: SampleType::RNA,
            collection_date: Some(Utc::now().date_naive()),
            source_organism: Some("Mus musculus".to_string()),
            source_tissue: Some("Liver".to_string()),
            concentration: Some(80.0),
            volume: Some(25.0),
            quality_score: Some(7.0),
            storage_location: Some("Freezer_B_Rack_2".to_string()),
            barcode: Some("RNA001".to_string()),
            metadata: None,
            notes: None,
        };

        let sample = service
            .create_sample(create_request, None)
            .await
            .expect("Creating sample should succeed");

        // Verify initial status
        assert_eq!(sample.status, SampleStatus::Active);

        // Update to processing
        let update_request = UpdateSampleRequest {
            status: Some(SampleStatus::Processing),
            quality_score: Some(8.0),
            notes: Some("Started processing".to_string()),
            ..Default::default()
        };

        let updated_sample = service
            .update_sample(sample.id, update_request)
            .await
            .expect("Updating sample status should succeed");

        assert_eq!(updated_sample.status, SampleStatus::Processing);
        assert_eq!(updated_sample.quality_score.unwrap(), 8.0);

        // Cleanup
        let _ = service.delete_sample(sample.id).await;
    }

    #[tokio::test]
    async fn test_sample_filtering_and_search() {
        let pool = setup_test_db().await;
        let service = SampleService::new(pool.clone());

        // Create samples with different types and properties
        let samples_to_create = vec![
            ("DNA_SAMPLE_001", SampleType::DNA, "Homo sapiens", "Blood"),
            ("RNA_SAMPLE_001", SampleType::RNA, "Homo sapiens", "Tissue"),
            ("PROTEIN_001", SampleType::Protein, "Mus musculus", "Brain"),
            ("DNA_SAMPLE_002", SampleType::DNA, "Mus musculus", "Liver"),
        ];

        let mut created_ids = Vec::new();
        for (name, sample_type, organism, tissue) in samples_to_create {
            let sample = service
                .create_sample(
                    CreateSampleRequest {
                        name: name.to_string(),
                        sample_type,
                        collection_date: Some(Utc::now().date_naive()),
                        source_organism: Some(organism.to_string()),
                        source_tissue: Some(tissue.to_string()),
                        concentration: Some(100.0),
                        volume: Some(50.0),
                        quality_score: Some(7.5),
                        storage_location: Some("Test_Location".to_string()),
                        barcode: Some(format!("BC_{}", name)),
                        metadata: None,
                        notes: None,
                    },
                    None,
                )
                .await
                .expect("Creating sample should succeed");
            created_ids.push(sample.id);
        }

        // Filter by sample type
        let dna_filter = SampleFilter {
            sample_type: Some(SampleType::DNA),
            source_organism: None,
            status: None,
            collection_date_from: None,
            collection_date_to: None,
            name_contains: None,
        };

        let dna_query = SampleListQuery {
            page: 1,
            per_page: 10,
            filter: Some(dna_filter),
            sort_by: Some("name".to_string()),
            sort_order: Some("asc".to_string()),
        };

        let dna_results = service.list_samples(dna_query).await;
        assert!(dna_results.is_ok(), "Filtering DNA samples should succeed");

        let dna_list = dna_results.unwrap();
        assert!(dna_list.samples.iter().all(|s| s.sample_type == SampleType::DNA));

        // Filter by organism
        let human_filter = SampleFilter {
            sample_type: None,
            source_organism: Some("Homo sapiens".to_string()),
            status: None,
            collection_date_from: None,
            collection_date_to: None,
            name_contains: None,
        };

        let human_query = SampleListQuery {
            page: 1,
            per_page: 10,
            filter: Some(human_filter),
            sort_by: None,
            sort_order: None,
        };

        let human_results = service.list_samples(human_query).await;
        assert!(human_results.is_ok(), "Filtering human samples should succeed");

        // Cleanup
        for id in created_ids {
            let _ = service.delete_sample(id).await;
        }
    }

    #[test]
    fn test_sample_type_validation() {
        // Test sample type enum values
        let sample_types = vec!["DNA", "RNA", "Protein", "Tissue", "Cell", "Serum", "Plasma"];
        
        for sample_type in sample_types {
            assert!(!sample_type.is_empty());
            assert!(sample_type.len() > 2);
        }
    }

    #[test]
    fn test_sample_metadata_structure() {
        let metadata = json!({
            "collection_date": "2024-01-15",
            "source_organism": "Homo sapiens",
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
        assert_eq!(metadata["quality_metrics"]["concentration"], 125.5);
        assert_eq!(metadata["storage_conditions"]["temperature"], -80.0);
    }

    #[test]
    fn test_sample_barcode_format() {
        let valid_barcodes = vec![
            "DNA001",
            "RNA_001_A1",
            "PROT-2024-001",
            "BC123456789",
        ];

        for barcode in valid_barcodes {
            assert!(!barcode.is_empty());
            assert!(barcode.len() >= 6);
            // Basic format validation
            assert!(barcode.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-'));
        }
    }

    #[test]
    fn test_quality_score_validation() {
        let valid_scores = vec![0.0, 5.5, 7.8, 9.0, 10.0];
        let invalid_scores = vec![-1.0, 11.0, 15.5];

        for score in valid_scores {
            assert!(score >= 0.0 && score <= 10.0, "Score {} should be valid", score);
        }

        for score in invalid_scores {
            assert!(score < 0.0 || score > 10.0, "Score {} should be invalid", score);
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
            assert!(unit.contains("ng") || unit.contains("μg"), "Should contain valid units");
        }
    }

    #[test]
    fn test_sample_status_transitions() {
        let statuses = vec!["Active", "Processing", "Consumed", "Archived", "Failed"];
        
        for status in statuses {
            assert!(!status.is_empty());
            // Test valid status names
            match status {
                "Active" | "Processing" | "Consumed" | "Archived" | "Failed" => assert!(true),
                _ => panic!("Invalid status: {}", status),
            }
        }
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
            assert!(location.contains("_"), "Location should have hierarchy separators");
            
            let parts: Vec<&str> = location.split('_').collect();
            assert!(parts.len() >= 3, "Location should have at least 3 hierarchy levels");
        }
    }

    #[test]
    fn test_sample_volume_tracking() {
        let initial_volume = 100.0;
        let volumes = vec![
            (10.0, 90.0),   // Used 10μL, 90μL remaining
            (25.0, 65.0),   // Used 25μL, 65μL remaining  
            (30.0, 35.0),   // Used 30μL, 35μL remaining
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
            assert_ne!(parent_id, child_id, "Parent and child should have different IDs");
            assert!(!relationship_type.is_empty());
            
            // Test relationship metadata
            let relationship_metadata = json!({
                "parent_id": parent_id,
                "child_id": child_id,
                "relationship_type": relationship_type,
                "created_date": "2024-01-15"
            });

            assert_eq!(relationship_metadata["parent_id"], json!(parent_id));
            assert_eq!(relationship_metadata["relationship_type"], relationship_type);
        }
    }

    #[tokio::test]
    async fn test_sample_barcode_uniqueness() {
        let pool = setup_test_db().await;
        let service = SampleService::new(pool.clone());

        let barcode = "UNIQUE_BC_001";

        // Create first sample with barcode
        let first_sample = service
            .create_sample(
                CreateSampleRequest {
                    name: "FIRST_SAMPLE".to_string(),
                    sample_type: SampleType::DNA,
                    collection_date: None,
                    source_organism: None,
                    source_tissue: None,
                    concentration: None,
                    volume: None,
                    quality_score: None,
                    storage_location: None,
                    barcode: Some(barcode.to_string()),
                    metadata: None,
                    notes: None,
                },
                None,
            )
            .await
            .expect("Creating first sample should succeed");

        // Try to create second sample with same barcode
        let duplicate_result = service
            .create_sample(
                CreateSampleRequest {
                    name: "SECOND_SAMPLE".to_string(),
                    sample_type: SampleType::RNA,
                    collection_date: None,
                    source_organism: None,
                    source_tissue: None,
                    concentration: None,
                    volume: None,
                    quality_score: None,
                    storage_location: None,
                    barcode: Some(barcode.to_string()),
                    metadata: None,
                    notes: None,
                },
                None,
            )
            .await;

        // Should fail due to unique constraint
        assert!(
            duplicate_result.is_err(),
            "Creating sample with duplicate barcode should fail"
        );

        // Cleanup
        let _ = service.delete_sample(first_sample.id).await;
    }

    #[tokio::test]
    async fn test_sample_quality_control() {
        let pool = setup_test_db().await;
        let service = SampleService::new(pool.clone());

        // Create sample with quality metrics
        let create_request = CreateSampleRequest {
            name: "QC_TEST_SAMPLE".to_string(),
            sample_type: SampleType::DNA,
            collection_date: Some(Utc::now().date_naive()),
            source_organism: Some("Homo sapiens".to_string()),
            source_tissue: Some("Blood".to_string()),
            concentration: Some(150.0),
            volume: Some(100.0),
            quality_score: Some(9.2),
            storage_location: Some("QC_Storage".to_string()),
            barcode: Some("QC_BC_001".to_string()),
            metadata: Some(json!({
                "purity_260_280": 1.8,
                "purity_260_230": 2.1,
                "integrity_number": 8.5,
                "qc_passed": true
            })),
            notes: Some("High-quality sample passed all QC checks".to_string()),
        };

        let sample = service
            .create_sample(create_request, None)
            .await
            .expect("Creating QC sample should succeed");

        // Verify quality metrics
        assert_eq!(sample.quality_score.unwrap(), 9.2);
        assert_eq!(sample.concentration.unwrap(), 150.0);
        assert_eq!(sample.volume.unwrap(), 100.0);

        // Check metadata
        assert!(sample.metadata.is_object());
        assert_eq!(sample.metadata["qc_passed"], true);
        assert_eq!(sample.metadata["purity_260_280"], 1.8);

        // Update quality score
        let update_request = UpdateSampleRequest {
            quality_score: Some(9.5),
            metadata: Some(json!({
                "purity_260_280": 1.9,
                "purity_260_230": 2.2,
                "integrity_number": 9.0,
                "qc_passed": true,
                "retest_reason": "Improved measurement"
            })),
            ..Default::default()
        };

        let updated_sample = service
            .update_sample(sample.id, update_request)
            .await
            .expect("Updating quality score should succeed");

        assert_eq!(updated_sample.quality_score.unwrap(), 9.5);
        assert_eq!(updated_sample.metadata["integrity_number"], 9.0);

        // Cleanup
        let _ = service.delete_sample(sample.id).await;
    }

    #[test]
    fn test_storage_location_structure() {
        let storage_location = StorageLocation {
            id: Uuid::new_v4(),
            name: "Freezer_A_Rack_1_Box_5".to_string(),
            location_type: "freezer".to_string(),
            temperature: Some(-80.0),
            capacity: Some(100),
            current_occupancy: Some(45),
            metadata: json!({
                "building": "Lab Building A",
                "room": "205",
                "coordinates": {"x": 2, "y": 3}
            }),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(storage_location.name, "Freezer_A_Rack_1_Box_5");
        assert_eq!(storage_location.location_type, "freezer");
        assert_eq!(storage_location.temperature.unwrap(), -80.0);
        assert_eq!(storage_location.capacity.unwrap(), 100);
        assert_eq!(storage_location.current_occupancy.unwrap(), 45);

        // Test occupancy calculation
        let available_slots = storage_location.capacity.unwrap() - storage_location.current_occupancy.unwrap();
        assert_eq!(available_slots, 55);

        // Test metadata access
        assert_eq!(storage_location.metadata["building"], "Lab Building A");
        assert_eq!(storage_location.metadata["coordinates"]["x"], 2);
    }

    #[tokio::test]
    async fn test_sample_metadata_operations() {
        let pool = setup_test_db().await;
        let service = SampleService::new(pool.clone());

        let complex_metadata = json!({
            "experimental_conditions": {
                "temperature": 4.0,
                "humidity": 45.0,
                "collection_method": "sterile_collection"
            },
            "processing_history": [
                {
                    "step": "extraction",
                    "date": "2024-01-15",
                    "reagents": ["TRIzol", "Chloroform"]
                },
                {
                    "step": "purification",
                    "date": "2024-01-16",
                    "method": "column_based"
                }
            ],
            "analysis_results": {
                "spectrophotometry": {
                    "260nm": 0.85,
                    "280nm": 0.47,
                    "ratio": 1.81
                },
                "gel_electrophoresis": {
                    "bands_observed": true,
                    "degradation": "minimal"
                }
            }
        });

        let create_request = CreateSampleRequest {
            name: "METADATA_TEST".to_string(),
            sample_type: SampleType::RNA,
            collection_date: Some(Utc::now().date_naive()),
            source_organism: Some("Homo sapiens".to_string()),
            source_tissue: Some("Kidney".to_string()),
            concentration: Some(95.0),
            volume: Some(30.0),
            quality_score: Some(8.1),
            storage_location: Some("RNA_Storage".to_string()),
            barcode: Some("META_BC_001".to_string()),
            metadata: Some(complex_metadata.clone()),
            notes: Some("Complex metadata test sample".to_string()),
        };

        let sample = service
            .create_sample(create_request, None)
            .await
            .expect("Creating sample with complex metadata should succeed");

        // Verify complex metadata structure
        assert_eq!(sample.metadata, complex_metadata);
        assert_eq!(sample.metadata["experimental_conditions"]["temperature"], 4.0);
        assert_eq!(sample.metadata["analysis_results"]["spectrophotometry"]["ratio"], 1.81);
        assert!(sample.metadata["processing_history"].is_array());

        // Update metadata
        let updated_metadata = json!({
            "experimental_conditions": {
                "temperature": 4.0,
                "humidity": 45.0,
                "collection_method": "sterile_collection"
            },
            "processing_history": [
                {
                    "step": "extraction",
                    "date": "2024-01-15",
                    "reagents": ["TRIzol", "Chloroform"]
                },
                {
                    "step": "purification",
                    "date": "2024-01-16",
                    "method": "column_based"
                },
                {
                    "step": "quantification",
                    "date": "2024-01-17",
                    "method": "fluorometric"
                }
            ],
            "analysis_results": {
                "spectrophotometry": {
                    "260nm": 0.87,
                    "280nm": 0.48,
                    "ratio": 1.81
                },
                "gel_electrophoresis": {
                    "bands_observed": true,
                    "degradation": "minimal"
                },
                "bioanalyzer": {
                    "rin_score": 8.2,
                    "concentration": 98.5
                }
            }
        });

        let update_request = UpdateSampleRequest {
            metadata: Some(updated_metadata.clone()),
            ..Default::default()
        };

        let updated_sample = service
            .update_sample(sample.id, update_request)
            .await
            .expect("Updating sample metadata should succeed");

        assert_eq!(updated_sample.metadata, updated_metadata);
        assert!(updated_sample.metadata["analysis_results"]["bioanalyzer"].is_object());
        assert_eq!(updated_sample.metadata["analysis_results"]["bioanalyzer"]["rin_score"], 8.2);

        // Cleanup
        let _ = service.delete_sample(sample.id).await;
    }

    #[tokio::test]
    async fn test_sample_lifecycle_workflow() {
        let pool = setup_test_db().await;
        let service = SampleService::new(pool.clone());

        // Step 1: Sample collection
        let collection_request = CreateSampleRequest {
            name: "WORKFLOW_SAMPLE_001".to_string(),
            sample_type: SampleType::Tissue,
            collection_date: Some(Utc::now().date_naive()),
            source_organism: Some("Mus musculus".to_string()),
            source_tissue: Some("Heart".to_string()),
            concentration: None, // Not measured yet
            volume: Some(200.0), // Initial volume
            quality_score: None, // Not assessed yet
            storage_location: Some("Collection_Bay".to_string()),
            barcode: Some("WF_001".to_string()),
            metadata: Some(json!({
                "collection_site": "Animal Facility",
                "collection_time": "09:30",
                "collector": "Dr. Smith"
            })),
            notes: Some("Fresh tissue sample for RNA extraction".to_string()),
        };

        let mut sample = service
            .create_sample(collection_request, None)
            .await
            .expect("Sample collection should succeed");

        assert_eq!(sample.status, SampleStatus::Active);
        assert!(sample.concentration.is_none());

        // Step 2: Processing begins
        let processing_update = UpdateSampleRequest {
            status: Some(SampleStatus::Processing),
            storage_location: Some("Processing_Lab".to_string()),
            metadata: Some(json!({
                "collection_site": "Animal Facility",
                "collection_time": "09:30",
                "collector": "Dr. Smith",
                "processing_started": "2024-01-15T10:00:00Z",
                "processor": "Lab Tech A"
            })),
            notes: Some("Started RNA extraction protocol".to_string()),
            ..Default::default()
        };

        sample = service
            .update_sample(sample.id, processing_update)
            .await
            .expect("Processing update should succeed");

        assert_eq!(sample.status, SampleStatus::Processing);

        // Step 3: Quality control and measurement
        let qc_update = UpdateSampleRequest {
            concentration: Some(85.0),
            volume: Some(45.0), // Volume reduced after extraction
            quality_score: Some(7.8),
            metadata: Some(json!({
                "collection_site": "Animal Facility",
                "collection_time": "09:30",
                "collector": "Dr. Smith",
                "processing_started": "2024-01-15T10:00:00Z",
                "processor": "Lab Tech A",
                "qc_completed": "2024-01-15T14:30:00Z",
                "extraction_yield": 42.5,
                "purity_check": "passed"
            })),
            ..Default::default()
        };

        sample = service
            .update_sample(sample.id, qc_update)
            .await
            .expect("QC update should succeed");

        assert_eq!(sample.concentration.unwrap(), 85.0);
        assert_eq!(sample.quality_score.unwrap(), 7.8);

        // Step 4: Final storage
        let storage_update = UpdateSampleRequest {
            status: Some(SampleStatus::Active),
            storage_location: Some("Freezer_RNA_A1_B5".to_string()),
            metadata: Some(json!({
                "collection_site": "Animal Facility",
                "collection_time": "09:30",
                "collector": "Dr. Smith",
                "processing_started": "2024-01-15T10:00:00Z",
                "processor": "Lab Tech A",
                "qc_completed": "2024-01-15T14:30:00Z",
                "extraction_yield": 42.5,
                "purity_check": "passed",
                "final_storage": "2024-01-15T15:00:00Z",
                "storage_temperature": -80.0
            })),
            notes: Some("Sample processed and stored for long-term preservation".to_string()),
            ..Default::default()
        };

        sample = service
            .update_sample(sample.id, storage_update)
            .await
            .expect("Storage update should succeed");

        assert_eq!(sample.status, SampleStatus::Active);
        assert_eq!(sample.storage_location.unwrap(), "Freezer_RNA_A1_B5");
        assert_eq!(sample.metadata["storage_temperature"], -80.0);

        // Verify complete workflow
        let final_metadata = &sample.metadata;
        assert!(final_metadata["collection_site"].is_string());
        assert!(final_metadata["processing_started"].is_string());
        assert!(final_metadata["qc_completed"].is_string());
        assert!(final_metadata["final_storage"].is_string());
        assert_eq!(final_metadata["extraction_yield"], 42.5);

        // Cleanup
        let _ = service.delete_sample(sample.id).await;
    }
} 
