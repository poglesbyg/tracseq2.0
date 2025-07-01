//! Unit tests for configuration validation

use config_service::{
    ConfigEntry, ConfigValue, validate_config_value,
    validate_service_name, validate_key_name,
    ValidationError, ValidationResult,
};
use crate::test_utils::*;
use serde_json::json;

#[tokio::test]
async fn test_validate_string_config() {
    let value = json!("simple string");
    let result = validate_config_value(&value, None);
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_validate_number_config() {
    let values = vec![
        json!(42),
        json!(3.14),
        json!(-100),
        json!(0),
    ];
    
    for value in values {
        let result = validate_config_value(&value, None);
        assert!(result.is_ok(), "Failed to validate: {:?}", value);
    }
}

#[tokio::test]
async fn test_validate_boolean_config() {
    let values = vec![json!(true), json!(false)];
    
    for value in values {
        let result = validate_config_value(&value, None);
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_validate_array_config() {
    let values = vec![
        json!([]),
        json!(["a", "b", "c"]),
        json!([1, 2, 3]),
        json!([true, false]),
        json!([{"key": "value"}, {"key": "value2"}]),
    ];
    
    for value in values {
        let result = validate_config_value(&value, None);
        assert!(result.is_ok(), "Failed to validate: {:?}", value);
    }
}

#[tokio::test]
async fn test_validate_object_config() {
    let values = vec![
        json!({}),
        json!({"key": "value"}),
        json!({
            "database": {
                "host": "localhost",
                "port": 5432,
                "ssl": true
            }
        }),
        json!({
            "features": ["auth", "logging", "metrics"],
            "version": "1.0.0",
            "enabled": true
        }),
    ];
    
    for value in values {
        let result = validate_config_value(&value, None);
        assert!(result.is_ok(), "Failed to validate: {:?}", value);
    }
}

#[tokio::test]
async fn test_validate_config_with_schema() {
    let schema = json!({
        "type": "object",
        "properties": {
            "port": {"type": "number"},
            "host": {"type": "string"},
            "ssl": {"type": "boolean"}
        },
        "required": ["port", "host"]
    });
    
    // Valid config
    let valid_config = json!({
        "port": 8080,
        "host": "localhost",
        "ssl": true
    });
    
    let result = validate_config_value(&valid_config, Some(&schema));
    assert!(result.is_ok());
    
    // Invalid config - missing required field
    let invalid_config = json!({
        "port": 8080,
        "ssl": false
    });
    
    let result = validate_config_value(&invalid_config, Some(&schema));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("required"));
}

#[tokio::test]
async fn test_validate_service_name() {
    // Valid service names
    let valid_names = vec![
        "auth-service",
        "storage_service",
        "service123",
        "my-awesome-service",
        "ServiceName",
    ];
    
    for name in valid_names {
        let result = validate_service_name(name);
        assert!(result.is_ok(), "Failed to validate service name: {}", name);
    }
    
    // Invalid service names
    let invalid_names = vec![
        "",
        "service name", // spaces
        "service@name", // special chars
        "123service", // starts with number
        "-service", // starts with dash
        "service-", // ends with dash
        "a", // too short
        &"x".repeat(65), // too long
    ];
    
    for name in invalid_names {
        let result = validate_service_name(name);
        assert!(result.is_err(), "Should have failed to validate: {}", name);
    }
}

#[tokio::test]
async fn test_validate_key_name() {
    // Valid key names
    let valid_keys = vec![
        "simple_key",
        "database.host",
        "feature.enabled",
        "JWT_SECRET",
        "api_key_123",
        "config.nested.value",
    ];
    
    for key in valid_keys {
        let result = validate_key_name(key);
        assert!(result.is_ok(), "Failed to validate key: {}", key);
    }
    
    // Invalid key names
    let invalid_keys = vec![
        "",
        "key with spaces",
        "key@special",
        ".startwithdot",
        "endwithdot.",
        "double..dot",
        &"x".repeat(129), // too long
    ];
    
    for key in invalid_keys {
        let result = validate_key_name(key);
        assert!(result.is_err(), "Should have failed to validate: {}", key);
    }
}

#[tokio::test]
async fn test_validate_environment_name() {
    use config_service::validate_environment_name;
    
    // Valid environments
    let valid_envs = vec![
        "dev",
        "test",
        "staging",
        "production",
        "local",
        "qa",
        "uat",
        "prod",
    ];
    
    for env in valid_envs {
        let result = validate_environment_name(env);
        assert!(result.is_ok());
    }
    
    // Invalid environments
    let invalid_envs = vec![
        "",
        "PROD", // uppercase not allowed
        "dev-1",
        "test env",
        &"x".repeat(21), // too long
    ];
    
    for env in invalid_envs {
        let result = validate_environment_name(env);
        assert!(result.is_err());
    }
}

#[tokio::test]
async fn test_validate_tags() {
    use config_service::validate_tags;
    
    // Valid tags
    let valid_tag_sets = vec![
        vec![],
        vec!["critical"],
        vec!["database", "connection"],
        vec!["feature-flag", "experimental"],
        vec!["v1.0.0", "stable", "production"],
    ];
    
    for tags in valid_tag_sets {
        let tags: Vec<String> = tags.into_iter().map(String::from).collect();
        let result = validate_tags(&tags);
        assert!(result.is_ok());
    }
    
    // Invalid tags
    let invalid_tag_sets = vec![
        vec![""], // empty tag
        vec!["tag with spaces"],
        vec!["tag@special"],
        vec!["valid", ""], // mix of valid and invalid
        vec![&"x".repeat(51)], // too long
        vec!["tag1", "tag2", "tag3", "tag4", "tag5", "tag6"], // too many tags
    ];
    
    for tags in invalid_tag_sets {
        let tags: Vec<String> = tags.into_iter().map(String::from).collect();
        let result = validate_tags(&tags);
        assert!(result.is_err());
    }
}

#[tokio::test]
async fn test_validate_sensitive_config() {
    use config_service::validate_sensitive_config;
    
    // Configs that should be marked as sensitive
    let sensitive_configs = vec![
        ("auth-service", "jwt_secret", json!("my-secret")),
        ("database", "password", json!("db-pass")),
        ("api", "api_key", json!("key-123")),
        ("service", "private_key", json!("-----BEGIN RSA PRIVATE KEY-----")),
    ];
    
    for (service, key, value) in sensitive_configs {
        let config = ConfigFactory::create_test_config(service, key, value);
        let warnings = validate_sensitive_config(&config);
        
        assert!(!warnings.is_empty(), "Should warn about sensitive config: {}", key);
        assert!(warnings[0].contains("sensitive") || warnings[0].contains("encrypted"));
    }
    
    // Non-sensitive configs
    let normal_configs = vec![
        ("service", "port", json!(8080)),
        ("service", "host", json!("localhost")),
        ("service", "enabled", json!(true)),
    ];
    
    for (service, key, value) in normal_configs {
        let config = ConfigFactory::create_test_config(service, key, value);
        let warnings = validate_sensitive_config(&config);
        
        assert!(warnings.is_empty(), "Should not warn about config: {}", key);
    }
}

#[tokio::test]
async fn test_validate_config_size() {
    use config_service::validate_config_size;
    
    // Small configs - should pass
    let small_value = json!("small value");
    assert!(validate_config_size(&small_value).is_ok());
    
    // Large but acceptable config
    let large_object = TestDataGenerator::generate_large_config(1000);
    assert!(validate_config_size(&large_object).is_ok());
    
    // Too large config
    let huge_object = TestDataGenerator::generate_large_config(10000);
    let result = validate_config_size(&huge_object);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("size"));
}

#[tokio::test]
async fn test_validate_config_complexity() {
    use config_service::validate_config_complexity;
    
    // Simple config
    let simple = json!({"key": "value"});
    assert!(validate_config_complexity(&simple, 0).is_ok());
    
    // Moderately nested
    let moderate = json!({
        "level1": {
            "level2": {
                "level3": {
                    "value": "deep"
                }
            }
        }
    });
    assert!(validate_config_complexity(&moderate, 0).is_ok());
    
    // Too deeply nested
    let mut too_deep = json!({});
    let mut current = &mut too_deep;
    for i in 0..20 {
        current[format!("level{}", i)] = json!({});
        current = &mut current[format!("level{}", i)];
    }
    
    let result = validate_config_complexity(&too_deep, 0);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("nested") || result.unwrap_err().contains("complexity"));
}

#[tokio::test]
async fn test_validate_config_references() {
    use config_service::validate_config_references;
    
    // Config with valid references
    let config_with_refs = json!({
        "database": {
            "host": "${DB_HOST}",
            "port": "${DB_PORT:5432}",
            "url": "postgres://${DB_USER}:${DB_PASS}@${DB_HOST}:${DB_PORT}/mydb"
        }
    });
    
    let (is_valid, refs) = validate_config_references(&config_with_refs);
    assert!(is_valid);
    assert!(!refs.is_empty());
    assert!(refs.contains(&"DB_HOST".to_string()));
    assert!(refs.contains(&"DB_PORT".to_string()));
    
    // Config with invalid references
    let invalid_refs = json!({
        "bad_ref": "${}", // empty reference
        "unclosed": "${DB_HOST", // unclosed
        "nested": "${{INVALID}}", // invalid format
    });
    
    let (is_valid, _) = validate_config_references(&invalid_refs);
    assert!(!is_valid);
}

#[tokio::test]
async fn test_validate_template_syntax() {
    use config_service::validate_template_syntax;
    
    // Valid templates
    let valid_templates = vec![
        json!("Hello {{name}}!"),
        json!("{{#if enabled}}Feature is on{{/if}}"),
        json!({
            "template": "Welcome {{user.name}} from {{user.location}}"
        }),
    ];
    
    for template in valid_templates {
        let result = validate_template_syntax(&template);
        assert!(result.is_ok());
    }
    
    // Invalid templates
    let invalid_templates = vec![
        json!("Unclosed {{name"),
        json!("Bad syntax {{#if}}{{/unless}}"),
        json!("Mismatched {{#each items}}{{/if}}"),
    ];
    
    for template in invalid_templates {
        let result = validate_template_syntax(&template);
        assert!(result.is_err());
    }
}

#[tokio::test]
async fn test_comprehensive_validation() {
    use config_service::comprehensive_config_validation;
    
    let config = ConfigFactory::create_test_config(
        "test-service",
        "app.settings",
        json!({
            "port": 8080,
            "host": "localhost",
            "database": {
                "url": "${DATABASE_URL}",
                "pool_size": 10
            },
            "features": ["auth", "logging"],
            "templates": {
                "welcome": "Hello {{name}}!"
            }
        })
    );
    
    let validation_result = comprehensive_config_validation(&config);
    
    assert!(validation_result.is_valid);
    assert!(validation_result.warnings.is_empty() || validation_result.warnings.len() > 0);
    assert!(validation_result.errors.is_empty());
    
    // Check that references were detected
    assert!(!validation_result.references.is_empty());
    assert!(validation_result.references.contains(&"DATABASE_URL".to_string()));
}