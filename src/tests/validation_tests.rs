#[cfg(test)]
mod validation_tests {
    use crate::models::user::{
        ChangePasswordRequest, ConfirmResetPasswordRequest, CreateUserRequest, LoginRequest,
        ResetPasswordRequest, UpdateUserRequest, UserRole,
    };
    use validator::Validate;

    #[test]
    fn test_email_validation_comprehensive() {
        let test_cases = vec![
            // Valid emails
            ("user@example.com", true),
            ("test.email@domain.co.uk", true),
            ("user+tag@example.org", true),
            ("firstname.lastname@company.com", true),
            ("user123@test-domain.com", true),
            // Invalid emails
            ("", false),
            ("plainaddress", false),
            ("@missingdomain.com", false),
            ("missing@.com", false),
            ("spaces in@email.com", false),
            ("double@@domain.com", false),
        ];

        for (email, should_be_valid) in test_cases {
            let login_request = LoginRequest {
                email: email.to_string(),
                password: "validpassword123".to_string(),
            };

            let result = login_request.validate();
            if should_be_valid {
                assert!(result.is_ok(), "Email '{}' should be valid", email);
            } else {
                assert!(result.is_err(), "Email '{}' should be invalid", email);
            }
        }
    }

    #[test]
    fn test_password_strength_validation() {
        let password_cases = vec![
            // Valid passwords
            ("password123", true),
            ("MySecurePass2024", true),
            ("complex!Password1", true),
            ("12345678", true), // Minimum length
            // Invalid passwords
            ("", false),
            ("short", false),
            ("1234567", false), // One char too short
        ];

        for (password, should_be_valid) in password_cases {
            let create_request = CreateUserRequest {
                email: "test@example.com".to_string(),
                password: password.to_string(),
                first_name: "Test".to_string(),
                last_name: "User".to_string(),
                role: UserRole::LabTechnician,
                lab_affiliation: None,
                department: None,
                position: None,
                phone: None,
                office_location: None,
            };

            let result = create_request.validate();
            if should_be_valid {
                assert!(result.is_ok(), "Password '{}' should be valid", password);
            } else {
                assert!(result.is_err(), "Password '{}' should be invalid", password);
            }
        }
    }

    #[test]
    fn test_name_validation() {
        let name_cases = vec![
            // Valid names
            ("John", true),
            ("Mary-Jane", true),
            ("José", true),
            ("O'Connor", true),
            ("Van Der Berg", true),
            ("李", true), // Unicode support
            // Invalid names
            ("", false),
        ];

        for (name, should_be_valid) in name_cases {
            let create_request = CreateUserRequest {
                email: "test@example.com".to_string(),
                password: "validpassword123".to_string(),
                first_name: name.to_string(),
                last_name: "TestLast".to_string(),
                role: UserRole::LabTechnician,
                lab_affiliation: None,
                department: None,
                position: None,
                phone: None,
                office_location: None,
            };

            let result = create_request.validate();
            if should_be_valid {
                assert!(result.is_ok(), "Name '{}' should be valid", name);
            } else {
                assert!(result.is_err(), "Name '{}' should be invalid", name);
            }
        }
    }

    #[test]
    fn test_laboratory_field_validation() {
        // Test laboratory-specific optional fields
        let create_request = CreateUserRequest {
            email: "lab.user@example.com".to_string(),
            password: "validpassword123".to_string(),
            first_name: "Lab".to_string(),
            last_name: "User".to_string(),
            role: UserRole::LabTechnician,
            lab_affiliation: Some("Genomics Research Lab".to_string()),
            department: Some("Molecular Biology".to_string()),
            position: Some("Senior Lab Technician".to_string()),
            phone: Some("+1-555-123-4567".to_string()),
            office_location: Some("Building A, Room 205".to_string()),
        };

        let result = create_request.validate();
        assert!(result.is_ok(), "Laboratory fields should be valid");
    }

    #[test]
    fn test_update_request_validation() {
        // Test that update request allows partial updates
        let minimal_update = UpdateUserRequest {
            email: Some("newemail@example.com".to_string()),
            first_name: None,
            last_name: None,
            role: None,
            status: None,
            lab_affiliation: None,
            department: None,
            position: None,
            phone: None,
            office_location: None,
        };

        assert!(minimal_update.validate().is_ok());

        // Test invalid email in update
        let invalid_email_update = UpdateUserRequest {
            email: Some("invalid-email".to_string()),
            first_name: None,
            last_name: None,
            role: None,
            status: None,
            lab_affiliation: None,
            department: None,
            position: None,
            phone: None,
            office_location: None,
        };

        assert!(invalid_email_update.validate().is_err());
    }

    #[test]
    fn test_password_change_validation() {
        // Valid password change
        let valid_change = ChangePasswordRequest {
            current_password: "oldpassword".to_string(),
            new_password: "newvalidpassword123".to_string(),
        };
        assert!(valid_change.validate().is_ok());

        // Empty current password
        let empty_current = ChangePasswordRequest {
            current_password: "".to_string(),
            new_password: "newvalidpassword123".to_string(),
        };
        assert!(empty_current.validate().is_err());

        // New password too short
        let short_new = ChangePasswordRequest {
            current_password: "oldpassword".to_string(),
            new_password: "short".to_string(),
        };
        assert!(short_new.validate().is_err());
    }

    #[test]
    fn test_password_reset_validation() {
        // Valid reset request
        let valid_reset = ResetPasswordRequest {
            email: "user@example.com".to_string(),
        };
        assert!(valid_reset.validate().is_ok());

        // Invalid email in reset
        let invalid_reset = ResetPasswordRequest {
            email: "not-an-email".to_string(),
        };
        assert!(invalid_reset.validate().is_err());

        // Valid confirm reset
        let valid_confirm = ConfirmResetPasswordRequest {
            token: "valid-reset-token".to_string(),
            new_password: "newvalidpassword123".to_string(),
        };
        assert!(valid_confirm.validate().is_ok());

        // Weak password in confirm reset
        let weak_confirm = ConfirmResetPasswordRequest {
            token: "valid-reset-token".to_string(),
            new_password: "weak".to_string(),
        };
        assert!(weak_confirm.validate().is_err());
    }

    #[test]
    fn test_validation_error_messages() {
        // Test that validation errors contain helpful messages
        let invalid_request = CreateUserRequest {
            email: "invalid-email".to_string(),
            password: "short".to_string(),
            first_name: "".to_string(),
            last_name: "".to_string(),
            role: UserRole::LabTechnician,
            lab_affiliation: None,
            department: None,
            position: None,
            phone: None,
            office_location: None,
        };

        let validation_result = invalid_request.validate();
        assert!(validation_result.is_err());

        if let Err(validation_errors) = validation_result {
            // Should have multiple validation errors
            assert!(!validation_errors.field_errors().is_empty());

            // Check that specific fields have errors
            assert!(validation_errors.field_errors().contains_key("email"));
            assert!(validation_errors.field_errors().contains_key("password"));
            assert!(validation_errors.field_errors().contains_key("first_name"));
            assert!(validation_errors.field_errors().contains_key("last_name"));
        }
    }

    #[test]
    fn test_role_validation() {
        // Test that all roles are valid in requests
        for role in UserRole::all_roles() {
            let create_request = CreateUserRequest {
                email: "test@example.com".to_string(),
                password: "validpassword123".to_string(),
                first_name: "Test".to_string(),
                last_name: "User".to_string(),
                role: role.clone(),
                lab_affiliation: None,
                department: None,
                position: None,
                phone: None,
                office_location: None,
            };

            let result = create_request.validate();
            assert!(
                result.is_ok(),
                "Role {:?} should be valid in create request",
                role
            );
        }
    }

    #[test]
    fn test_field_length_limits() {
        // Test maximum field lengths
        let long_string = "a".repeat(101); // Over 100 char limit

        // Test first name length limit
        let long_first_name = CreateUserRequest {
            email: "test@example.com".to_string(),
            password: "validpassword123".to_string(),
            first_name: long_string.clone(),
            last_name: "User".to_string(),
            role: UserRole::LabTechnician,
            lab_affiliation: None,
            department: None,
            position: None,
            phone: None,
            office_location: None,
        };
        assert!(long_first_name.validate().is_err());

        // Test last name length limit
        let long_last_name = CreateUserRequest {
            email: "test@example.com".to_string(),
            password: "validpassword123".to_string(),
            first_name: "Test".to_string(),
            last_name: long_string.clone(),
            role: UserRole::LabTechnician,
            lab_affiliation: None,
            department: None,
            position: None,
            phone: None,
            office_location: None,
        };
        assert!(long_last_name.validate().is_err());

        // Test that 100 characters is allowed (boundary condition)
        let exactly_100_chars = "a".repeat(100);
        let boundary_request = CreateUserRequest {
            email: "test@example.com".to_string(),
            password: "validpassword123".to_string(),
            first_name: exactly_100_chars.clone(),
            last_name: exactly_100_chars,
            role: UserRole::LabTechnician,
            lab_affiliation: None,
            department: None,
            position: None,
            phone: None,
            office_location: None,
        };
        assert!(boundary_request.validate().is_ok());
    }

    #[test]
    fn test_special_characters_in_names() {
        // Test names with special characters that should be valid
        let special_name_cases = vec![
            "O'Brian",
            "Mary-Jane",
            "José María",
            "李小明",
            "François",
            "Müller",
            "van der Berg",
            "MacPherson",
            "D'Angelo",
        ];

        for name in special_name_cases {
            let create_request = CreateUserRequest {
                email: "test@example.com".to_string(),
                password: "validpassword123".to_string(),
                first_name: name.to_string(),
                last_name: "TestLast".to_string(),
                role: UserRole::LabTechnician,
                lab_affiliation: None,
                department: None,
                position: None,
                phone: None,
                office_location: None,
            };

            let result = create_request.validate();
            assert!(
                result.is_ok(),
                "Special character name '{}' should be valid",
                name
            );
        }
    }
}
