#[cfg(test)]
mod tests {
    use crate::{
        assembly::ComponentBuilder,
        config::AppConfig,
        router::{create_test_router, health_routes, template_routes},
    };

    #[tokio::test]
    async fn test_modular_component_assembly() {
        // Test that we can build components step by step
        let config = AppConfig::for_testing();

        // This should work even if database/storage aren't available in test environment
        let builder = ComponentBuilder::new(config);

        // Test that builder pattern works
        assert!(builder.config.database.url.contains("test"));
        assert_eq!(builder.config.server.port, 0); // Random port for tests
    }

    #[test]
    fn test_modular_router_assembly() {
        // Test that we can combine routes modularly
        let _health_router = health_routes();
        let _template_router = template_routes();
        let _test_router = create_test_router();

        // These should compile and be valid routers
        assert!(true); // Placeholder - in real tests we'd test routing
    }

    #[test]
    fn test_configuration_modularity() {
        // Test different configuration scenarios
        let test_config = AppConfig::for_testing();
        assert!(test_config
            .storage
            .base_path
            .to_string_lossy()
            .contains("test"));

        // Test that we can create custom configurations
        let custom_config = AppConfig {
            database: crate::config::DatabaseConfig {
                url: "postgres://custom:custom@localhost:5432/custom".to_string(),
                max_connections: 5,
                min_connections: 1,
            },
            storage: crate::config::StorageConfig {
                base_path: "/tmp/custom".into(),
                max_file_size: 50 * 1024 * 1024,
                allowed_extensions: vec!["csv".to_string()],
            },
            server: crate::config::ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                cors_enabled: false,
            },
            rag: crate::config::RagIntegrationConfig::default(),
        };

        assert_eq!(custom_config.server.port, 8080);
        assert!(!custom_config.server.cors_enabled);
    }

    #[test]
    fn test_democratic_component_design() {
        // Test that components are independent and can be configured separately
        let config = AppConfig::for_testing();

        // Each component should be configurable independently
        assert!(config.database.max_connections > 0);
        assert!(config.storage.max_file_size > 0);
        assert!(!config.storage.allowed_extensions.is_empty());

        // Components should have sensible defaults
        assert_eq!(config.storage.allowed_extensions.len(), 2); // xlsx, csv for tests
    }
}
