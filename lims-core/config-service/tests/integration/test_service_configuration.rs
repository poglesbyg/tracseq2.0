//! Integration tests for service configuration management

use config_service::{AppState, ConfigEntry, ServiceConfigResponse};
use crate::test_utils::*;
use axum::http::StatusCode;
use std::collections::HashMap;

#[tokio::test]
async fn test_complete_service_configuration_lifecycle() {
    let app = create_test_app().await;
    
    // 1. Create initial configurations for a service
    let service_name = "test-microservice";
    let environment = "development";
    
    let configs = vec![
        ("database.host", json!("localhost")),
        ("database.port", json!(5432)),
        ("database.name", json!("testdb")),
        ("cache.enabled", json!(true)),
        ("cache.ttl", json!(300)),
        ("features.auth", json!(false)),
        ("features.logging", json!(true)),
    ];
    
    // Create all configs
    for (key, value) in &configs {
        let request = RequestFactory::create_config_request(service_name, key, value.clone());
        
        let response = app
            .post("/api/configs")
            .json(&request)
            .send()
            .await
            .expect("Failed to send request");
        
        assert_eq!(response.status(), StatusCode::CREATED);
        
        let created_config: ConfigEntry = response.json().await.unwrap();
        assert_eq!(created_config.key, *key);
        assert_eq!(created_config.value, *value);
    }
    
    // 2. Retrieve full service configuration
    let response = app
        .get(&format!("/api/services/{}/config/{}", service_name, environment))
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let service_config: ServiceConfigResponse = response.json().await.unwrap();
    assert_eq!(service_config.service_name, service_name);
    assert_eq!(service_config.environment, environment);
    assert_eq!(service_config.config.len(), configs.len());
    
    // Verify all configs are present
    for (key, expected_value) in &configs {
        let actual_value = service_config.config.get(*key).unwrap();
        assert_eq!(actual_value, expected_value);
    }
    
    // 3. Update specific configurations
    let update_request = RequestFactory::update_config_request(json!("production-db"));
    
    let response = app
        .put(&format!("/api/configs/{}/{}", service_name, "database.name"))
        .json(&update_request)
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let updated_config: ConfigEntry = response.json().await.unwrap();
    assert_eq!(updated_config.value, json!("production-db"));
    assert_eq!(updated_config.version, 2); // Version should increment
    
    // 4. Delete a configuration
    let response = app
        .delete(&format!("/api/configs/{}/{}", service_name, "features.auth"))
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    
    // 5. Verify final state
    let response = app
        .get(&format!("/api/services/{}/config/{}", service_name, environment))
        .send()
        .await
        .expect("Failed to send request");
    
    let final_config: ServiceConfigResponse = response.json().await.unwrap();
    assert_eq!(final_config.config.len(), configs.len() - 1); // One deleted
    assert!(!final_config.config.contains_key("features.auth"));
    assert_eq!(final_config.config.get("database.name").unwrap(), &json!("production-db"));
}

#[tokio::test]
async fn test_environment_specific_configurations() {
    let app = create_test_app().await;
    let service_name = "multi-env-service";
    
    // Create configs for different environments
    let environments = vec![
        ("development", "localhost", 5432, false),
        ("staging", "staging-db.example.com", 5432, true),
        ("production", "prod-db.example.com", 5433, true),
    ];
    
    for (env, host, port, ssl) in &environments {
        // Create with specific environment
        let mut request = RequestFactory::create_config_request(
            service_name,
            "database.host",
            json!(host)
        );
        request.environment = env.to_string();
        
        let response = app
            .post("/api/configs")
            .json(&request)
            .send()
            .await
            .expect("Failed to send request");
        
        assert_eq!(response.status(), StatusCode::CREATED);
        
        // Port config
        let mut port_request = RequestFactory::create_config_request(
            service_name,
            "database.port",
            json!(port)
        );
        port_request.environment = env.to_string();
        
        app.post("/api/configs")
            .json(&port_request)
            .send()
            .await
            .expect("Failed to send request");
        
        // SSL config
        let mut ssl_request = RequestFactory::create_config_request(
            service_name,
            "database.ssl",
            json!(ssl)
        );
        ssl_request.environment = env.to_string();
        
        app.post("/api/configs")
            .json(&ssl_request)
            .send()
            .await
            .expect("Failed to send request");
    }
    
    // Verify each environment has correct configs
    for (env, expected_host, expected_port, expected_ssl) in &environments {
        let response = app
            .get(&format!("/api/services/{}/config/{}", service_name, env))
            .send()
            .await
            .expect("Failed to send request");
        
        assert_eq!(response.status(), StatusCode::OK);
        
        let env_config: ServiceConfigResponse = response.json().await.unwrap();
        assert_eq!(env_config.environment, *env);
        assert_eq!(env_config.config.get("database.host").unwrap(), &json!(expected_host));
        assert_eq!(env_config.config.get("database.port").unwrap(), &json!(expected_port));
        assert_eq!(env_config.config.get("database.ssl").unwrap(), &json!(expected_ssl));
    }
}

#[tokio::test]
async fn test_bulk_configuration_update() {
    let app = create_test_app().await;
    let service_name = "bulk-update-service";
    let environment = "test";
    
    // Create initial configs
    let initial_configs = TestDataGenerator::generate_service_configs(service_name, 5);
    for config in &initial_configs {
        let response = app
            .post("/api/configs")
            .json(config)
            .send()
            .await
            .expect("Failed to send request");
        
        assert_eq!(response.status(), StatusCode::CREATED);
    }
    
    // Prepare bulk update with mix of updates and new configs
    let mut bulk_update = HashMap::new();
    bulk_update.insert("existing_key_0".to_string(), json!("updated_value_0"));
    bulk_update.insert("existing_key_1".to_string(), json!("updated_value_1"));
    bulk_update.insert("new_key_1".to_string(), json!("new_value_1"));
    bulk_update.insert("new_key_2".to_string(), json!("new_value_2"));
    
    let response = app
        .put(&format!("/api/services/{}/config/{}", service_name, environment))
        .json(&bulk_update)
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let updated_config: ServiceConfigResponse = response.json().await.unwrap();
    
    // Should have original 5 + 2 new - but only if no deletions
    assert!(updated_config.config.len() >= 7);
    assert_eq!(updated_config.config.get("existing_key_0").unwrap(), &json!("updated_value_0"));
    assert_eq!(updated_config.config.get("new_key_1").unwrap(), &json!("new_value_1"));
}

#[tokio::test]
async fn test_config_inheritance_and_overrides() {
    let app = create_test_app().await;
    let service_name = "inheritance-service";
    
    // Create base configs (default environment)
    let base_configs = vec![
        ("app.name", json!("MyApp")),
        ("app.version", json!("1.0.0")),
        ("database.pool_size", json!(10)),
        ("cache.enabled", json!(true)),
    ];
    
    for (key, value) in &base_configs {
        let mut request = RequestFactory::create_config_request(service_name, key, value.clone());
        request.environment = "default".to_string();
        
        app.post("/api/configs")
            .json(&request)
            .send()
            .await
            .expect("Failed to send request");
    }
    
    // Create environment-specific overrides
    let dev_overrides = vec![
        ("database.pool_size", json!(5)), // Override
        ("debug.enabled", json!(true)), // New config
    ];
    
    for (key, value) in &dev_overrides {
        let mut request = RequestFactory::create_config_request(service_name, key, value.clone());
        request.environment = "development".to_string();
        
        app.post("/api/configs")
            .json(&request)
            .send()
            .await
            .expect("Failed to send request");
    }
    
    // Get merged configuration
    let response = app
        .get(&format!("/api/services/{}/config/development?include_defaults=true", service_name))
        .send()
        .await
        .expect("Failed to send request");
    
    let merged_config: ServiceConfigResponse = response.json().await.unwrap();
    
    // Should have all base configs plus overrides
    assert_eq!(merged_config.config.get("app.name").unwrap(), &json!("MyApp")); // From base
    assert_eq!(merged_config.config.get("database.pool_size").unwrap(), &json!(5)); // Overridden
    assert_eq!(merged_config.config.get("debug.enabled").unwrap(), &json!(true)); // New in dev
    assert_eq!(merged_config.config.get("cache.enabled").unwrap(), &json!(true)); // From base
}

#[tokio::test]
async fn test_configuration_templates() {
    let app = create_test_app().await;
    let service_name = "template-service";
    
    // Create template configuration
    let template_config = TemplateBuilder::new()
        .with_template("connection_string", 
            "postgres://{{username}}:{{password}}@{{host}}:{{port}}/{{database}}")
        .with_template("welcome_message",
            "Welcome to {{app_name}} v{{version}}!")
        .with_variables(vec![
            ("username", "dbuser"),
            ("password", "secret123"),
            ("host", "localhost"),
            ("port", "5432"),
            ("database", "myapp"),
            ("app_name", "MyService"),
            ("version", "2.0.0"),
        ])
        .build();
    
    let request = RequestFactory::create_config_request(
        service_name,
        "templates",
        template_config
    );
    
    let response = app
        .post("/api/configs")
        .json(&request)
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::CREATED);
    
    // Request rendered configuration
    let response = app
        .get(&format!("/api/services/{}/config/test?render_templates=true", service_name))
        .send()
        .await
        .expect("Failed to send request");
    
    let rendered_config: ServiceConfigResponse = response.json().await.unwrap();
    let templates = rendered_config.config.get("templates").unwrap();
    
    // Verify templates are rendered correctly
    assert!(templates["connection_string"]
        .as_str()
        .unwrap()
        .contains("postgres://dbuser:secret123@localhost:5432/myapp"));
    
    assert!(templates["welcome_message"]
        .as_str()
        .unwrap()
        .contains("Welcome to MyService v2.0.0!"));
}

#[tokio::test]
async fn test_configuration_validation_rules() {
    let app = create_test_app().await;
    let service_name = "validated-service";
    
    // Try to create invalid configurations
    let invalid_configs = vec![
        ("", json!("value"), "empty key"),
        ("key with spaces", json!("value"), "spaces in key"),
        ("valid_key", json!(null), "null value"),
        ("port", json!("not_a_number"), "wrong type for port"),
        ("enabled", json!("yes"), "string instead of boolean"),
    ];
    
    for (key, value, reason) in invalid_configs {
        let request = RequestFactory::create_config_request(service_name, key, value);
        
        let response = app
            .post("/api/configs")
            .json(&request)
            .send()
            .await
            .expect("Failed to send request");
        
        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "Should reject config with {}", reason
        );
    }
    
    // Valid configurations should pass
    let valid_configs = vec![
        ("database.host", json!("localhost")),
        ("server.port", json!(8080)),
        ("features.enabled", json!(true)),
        ("nested.config.value", json!({"key": "value"})),
    ];
    
    for (key, value) in valid_configs {
        let request = RequestFactory::create_config_request(service_name, key, value);
        
        let response = app
            .post("/api/configs")
            .json(&request)
            .send()
            .await
            .expect("Failed to send request");
        
        assert_eq!(response.status(), StatusCode::CREATED);
    }
}

#[tokio::test]
async fn test_configuration_export_import() {
    let app = create_test_app().await;
    let source_service = "export-service";
    let target_service = "import-service";
    
    // Create configurations in source service
    let configs = TestDataGenerator::generate_complex_service_config();
    
    for (key, value) in configs {
        let request = RequestFactory::create_config_request(source_service, &key, value);
        
        app.post("/api/configs")
            .json(&request)
            .send()
            .await
            .expect("Failed to send request");
    }
    
    // Export configuration
    let response = app
        .get(&format!("/api/services/{}/export?format=json", source_service))
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let exported_data: serde_json::Value = response.json().await.unwrap();
    
    // Import to new service
    let import_request = json!({
        "service_name": target_service,
        "data": exported_data,
        "merge_strategy": "overwrite"
    });
    
    let response = app
        .post("/api/services/import")
        .json(&import_request)
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::OK);
    
    // Verify imported configs match source
    let source_config = app
        .get(&format!("/api/services/{}/config/test", source_service))
        .send()
        .await
        .expect("Failed to send request")
        .json::<ServiceConfigResponse>()
        .await
        .unwrap();
    
    let target_config = app
        .get(&format!("/api/services/{}/config/test", target_service))
        .send()
        .await
        .expect("Failed to send request")
        .json::<ServiceConfigResponse>()
        .await
        .unwrap();
    
    assert_eq!(source_config.config, target_config.config);
}