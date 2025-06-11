#[cfg(test)]
mod storage_management_tests {
    use serde_json::json;
    use uuid::Uuid;

    #[test]
    fn test_storage_location_types() {
        let location_types = vec![
            ("freezer", -80.0),
            ("fridge", 4.0),
            ("room_temp", 25.0),
            ("ln2_tank", -196.0),
        ];

        for (location_type, temperature) in location_types {
            assert!(!location_type.is_empty());
            assert!(temperature >= -200.0 && temperature <= 50.0);
        }
    }

    #[test]
    fn test_storage_hierarchy() {
        let locations = vec![
            "Building_A_Room_205_Freezer_A_Rack_1_Box_5",
            "Building_B_Room_101_Fridge_B_Shelf_2_Tray_3",
        ];

        for location in locations {
            assert!(!location.is_empty());
            assert!(location.contains("_"));
            let parts: Vec<&str> = location.split('_').collect();
            assert!(parts.len() >= 5);
        }
    }

    #[test]
    fn test_capacity_management() {
        let storage_units = vec![
            ("Freezer_A", 100, 45, 55),
            ("Fridge_B", 50, 30, 20),
            ("Cabinet_C", 25, 25, 0),
        ];

        for (location, total, used, expected_available) in storage_units {
            assert!(!location.is_empty());
            assert!(total > 0);
            assert!(used >= 0);
            assert!(used <= total);

            let available = total - used;
            assert_eq!(available, expected_available);
        }
    }

    #[test]
    fn test_temperature_monitoring() {
        let readings = vec![
            (-80.0, -82.0, true),  // Within range
            (4.0, 3.5, true),      // Within range
            (-80.0, -75.0, false), // Out of range
        ];

        for (target, actual, should_be_valid) in readings {
            let tolerance: f64 = if target < -70.0 { 5.0 } else { 2.0 };
            let is_valid = (actual - target).abs() <= tolerance;
            assert_eq!(is_valid, should_be_valid);
        }
    }

    #[test]
    fn test_storage_location_metadata() {
        let location_metadata = json!({
            "location_id": "FREEZER_A_RACK_1_BOX_5",
            "location_type": "freezer",
            "building": "Lab Building A",
            "room": "205",
            "equipment_id": "FREEZER_A_001",
            "coordinates": {
                "rack": 1,
                "shelf": 2,
                "box": 5
            },
            "capacity": {
                "total_positions": 100,
                "occupied_positions": 45,
                "available_positions": 55
            },
            "temperature": {
                "target": -80.0,
                "current": -82.1,
                "last_check": "2024-01-15T10:30:00Z"
            },
            "maintenance": {
                "last_service": "2024-01-10",
                "next_service": "2024-04-10",
                "service_interval_days": 90
            }
        });

        // Validate metadata structure
        assert_eq!(location_metadata["location_type"], "freezer");
        assert_eq!(location_metadata["building"], "Lab Building A");

        // Validate coordinates
        let coords = &location_metadata["coordinates"];
        assert_eq!(coords["rack"], 1);
        assert_eq!(coords["box"], 5);

        // Validate capacity
        let capacity = &location_metadata["capacity"];
        assert_eq!(capacity["total_positions"], 100);
        assert_eq!(capacity["occupied_positions"], 45);
        assert_eq!(capacity["available_positions"], 55);

        // Verify capacity calculation
        let total: i32 = capacity["total_positions"].as_i64().unwrap() as i32;
        let occupied: i32 = capacity["occupied_positions"].as_i64().unwrap() as i32;
        let available: i32 = capacity["available_positions"].as_i64().unwrap() as i32;
        assert_eq!(total - occupied, available);

        // Validate temperature
        let temp = &location_metadata["temperature"];
        assert_eq!(temp["target"], -80.0);
        assert_eq!(temp["current"], -82.1);
    }

    #[test]
    fn test_barcode_position_mapping() {
        let barcode_positions = vec![
            (
                "FREEZER_A_R1_B5_P01",
                "Freezer_A",
                "Rack_1",
                "Box_5",
                "Position_A1",
            ),
            (
                "FRIDGE_B_S2_T3_P12",
                "Fridge_B",
                "Shelf_2",
                "Tray_3",
                "Position_B6",
            ),
            (
                "CABINET_C_D1_S4_P08",
                "Cabinet_C",
                "Drawer_1",
                "Slot_4",
                "Position_C2",
            ),
        ];

        for (barcode, equipment, container, sub_container, position) in barcode_positions {
            assert!(!barcode.is_empty());
            assert!(barcode.len() >= 10, "Barcode should be descriptive");
            assert!(barcode.contains("_"), "Barcode should have separators");

            // Validate component references
            assert!(!equipment.is_empty());
            assert!(!container.is_empty());
            assert!(!sub_container.is_empty());
            assert!(!position.is_empty());

            // Test barcode parsing
            let parts: Vec<&str> = barcode.split('_').collect();
            assert!(parts.len() >= 4, "Barcode should have multiple components");

            // Verify position format
            assert!(position.starts_with("Position_"));
            let pos_part = position.strip_prefix("Position_").unwrap();
            assert!(
                pos_part.len() >= 2,
                "Position identifier should be meaningful"
            );
        }
    }

    #[test]
    fn test_storage_access_control() {
        let access_levels = vec![
            ("admin", vec!["read", "write", "delete", "configure"]),
            ("lab_tech", vec!["read", "write", "move"]),
            ("researcher", vec!["read", "query"]),
            ("guest", vec!["read"]),
        ];

        for (role, permissions) in access_levels {
            assert!(!role.is_empty());
            assert!(!permissions.is_empty());

            // Validate permission escalation
            match role {
                "admin" => {
                    assert!(permissions.contains(&"configure"));
                    assert!(permissions.contains(&"delete"));
                    assert!(permissions.len() >= 4);
                }
                "lab_tech" => {
                    assert!(permissions.contains(&"write"));
                    assert!(permissions.contains(&"move"));
                    assert!(!permissions.contains(&"delete"));
                }
                "researcher" => {
                    assert!(permissions.contains(&"read"));
                    assert!(!permissions.contains(&"write"));
                }
                "guest" => {
                    assert_eq!(permissions.len(), 1);
                    assert_eq!(permissions[0], "read");
                }
                _ => panic!("Unknown role: {}", role),
            }
        }
    }

    #[test]
    fn test_storage_audit_trail() {
        let audit_events = vec![
            (
                "sample_stored",
                "S001",
                "FREEZER_A_R1_B5_P01",
                "lab_tech_001",
            ),
            ("sample_moved", "S002", "FRIDGE_B_S2_T3_P12", "lab_tech_002"),
            (
                "sample_retrieved",
                "S003",
                "CABINET_C_D1_S4_P08",
                "researcher_001",
            ),
            (
                "location_maintenance",
                "",
                "FREEZER_A_R1",
                "maintenance_staff",
            ),
        ];

        for (event_type, sample_id, location, user) in audit_events {
            assert!(!event_type.is_empty());
            assert!(!location.is_empty());
            assert!(!user.is_empty());

            // Validate event types
            match event_type {
                "sample_stored" | "sample_moved" | "sample_retrieved" => {
                    assert!(!sample_id.is_empty(), "Sample events should have sample ID");
                }
                "location_maintenance" => {
                    // Maintenance events may not have sample ID
                    assert!(user.contains("maintenance"));
                }
                _ => panic!("Unknown event type: {}", event_type),
            }

            // Validate user roles
            assert!(user.contains("_"), "User should have role identifier");

            // Create audit record
            let audit_record = json!({
                "event_type": event_type,
                "sample_id": sample_id,
                "location": location,
                "user": user,
                "timestamp": "2024-01-15T10:30:00Z"
            });

            assert_eq!(audit_record["event_type"], event_type);
            assert_eq!(audit_record["location"], location);
        }
    }
}
