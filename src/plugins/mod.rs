use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

/// Plugin trait that all plugins must implement
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Plugin identifier
    fn name(&self) -> &'static str;

    /// Plugin version
    fn version(&self) -> &'static str;

    /// Plugin dependencies (other plugin names)
    fn dependencies(&self) -> Vec<&'static str> {
        vec![]
    }

    /// Initialize the plugin
    async fn initialize(&mut self, context: &PluginContext) -> Result<(), PluginError>;

    /// Plugin-specific configuration schema
    fn config_schema(&self) -> serde_json::Value;

    /// Handle plugin-specific events
    async fn handle_event(&self, _event: &PluginEvent) -> Result<(), PluginError> {
        Ok(()) // Default: ignore events
    }
}

/// Context provided to plugins during initialization
pub struct PluginContext {
    pub database_pool: Option<sqlx::PgPool>,
    pub storage_path: std::path::PathBuf,
    pub config: serde_json::Value,
    pub event_bus: Arc<crate::events::EventBus>,
}

/// Plugin-specific events
#[derive(Debug, Clone)]
pub enum PluginEvent {
    SystemStart,
    SystemShutdown,
    ConfigChange(String),
    Custom(String, serde_json::Value),
}

/// Plugin manager for loading and managing plugins
pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
    context: PluginContext,
}

impl PluginManager {
    pub fn new(context: PluginContext) -> Self {
        Self {
            plugins: HashMap::new(),
            context,
        }
    }

    /// Register a plugin
    pub async fn register_plugin(
        &mut self,
        mut plugin: Box<dyn Plugin>,
    ) -> Result<(), PluginError> {
        let name = plugin.name().to_string();

        // Check dependencies
        for dep in plugin.dependencies() {
            if !self.plugins.contains_key(dep) {
                return Err(PluginError::MissingDependency(dep.to_string()));
            }
        }

        // Initialize plugin
        plugin.initialize(&self.context).await?;

        // Store plugin
        self.plugins.insert(name, plugin);

        Ok(())
    }

    /// Broadcast event to all plugins
    pub async fn broadcast_event(&self, event: PluginEvent) -> Result<(), PluginError> {
        for plugin in self.plugins.values() {
            plugin.handle_event(&event).await?;
        }
        Ok(())
    }

    /// Get plugin by name
    pub fn get_plugin(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins.get(name).map(|p| p.as_ref())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("Missing dependency: {0}")]
    MissingDependency(String),
    #[error("Initialization failed: {0}")]
    InitializationFailed(String),
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

/// Example plugin implementations
pub mod examples {
    use super::*;

    /// Notification plugin example
    pub struct NotificationPlugin {
        webhook_url: Option<String>,
    }

    #[async_trait]
    impl Plugin for NotificationPlugin {
        fn name(&self) -> &'static str {
            "notifications"
        }
        fn version(&self) -> &'static str {
            "1.0.0"
        }

        async fn initialize(&mut self, context: &PluginContext) -> Result<(), PluginError> {
            // Extract webhook URL from config
            self.webhook_url = context
                .config
                .get("webhook_url")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            Ok(())
        }

        fn config_schema(&self) -> serde_json::Value {
            serde_json::json!({
                "type": "object",
                "properties": {
                    "webhook_url": { "type": "string" }
                }
            })
        }

        async fn handle_event(&self, event: &PluginEvent) -> Result<(), PluginError> {
            match event {
                PluginEvent::Custom(event_type, _data) if event_type == "sample_created" => {
                    // Send notification
                    if let Some(url) = &self.webhook_url {
                        println!("📧 Sending notification to: {}", url);
                        // Implementation would send HTTP request
                    }
                }
                _ => {}
            }
            Ok(())
        }
    }
}

/// Builder for easier plugin registration
pub struct PluginRegistryBuilder {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginRegistryBuilder {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    pub fn with_plugin(mut self, plugin: Box<dyn Plugin>) -> Self {
        self.plugins.push(plugin);
        self
    }

    pub async fn build(self, context: PluginContext) -> Result<PluginManager, PluginError> {
        let mut manager = PluginManager::new(context);

        for plugin in self.plugins {
            manager.register_plugin(plugin).await?;
        }

        Ok(manager)
    }
}
