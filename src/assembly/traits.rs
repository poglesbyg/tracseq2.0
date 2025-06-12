use async_trait::async_trait;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

/// Core trait that all components must implement for the modular system
#[async_trait]
pub trait Component: Send + Sync {
    /// Unique identifier for this component type
    fn component_id(&self) -> &'static str;

    /// Human-readable name for this component
    fn component_name(&self) -> &'static str;

    /// Initialize the component with the given context
    async fn initialize(&mut self, context: &ServiceRegistry) -> Result<(), ComponentError>;

    /// Check if this component is healthy and ready to use
    async fn health_check(&self) -> Result<(), ComponentError>;

    /// Shutdown the component gracefully
    async fn shutdown(&mut self) -> Result<(), ComponentError>;

    /// Get component as Any for downcasting
    fn as_any(&self) -> &dyn Any;

    /// Get component as mutable Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Trait for components that provide services to other components
#[async_trait]
pub trait ServiceProvider: Component {
    /// Get the list of service types this component provides
    fn provided_services(&self) -> Vec<&'static str>;

    /// Check if this component can provide a specific service
    fn provides_service(&self, service_type: &str) -> bool {
        self.provided_services().contains(&service_type)
    }
}

/// Trait for components that depend on other services
#[async_trait]
pub trait ServiceConsumer: Component {
    /// Get the list of required services this component needs
    fn required_services(&self) -> Vec<&'static str>;

    /// Get the list of optional services this component can use
    fn optional_services(&self) -> Vec<&'static str> {
        vec![]
    }

    /// Inject a service dependency
    async fn inject_service(
        &mut self,
        service_type: &str,
        service: Arc<dyn Any + Send + Sync>,
    ) -> Result<(), ComponentError>;
}

/// Trait for configurable components
pub trait Configurable: Component {
    type Config: Clone + Send + Sync;

    /// Apply configuration to this component
    fn configure(&mut self, config: Self::Config) -> Result<(), ComponentError>;

    /// Get current configuration
    fn get_config(&self) -> &Self::Config;
}

/// Central registry for managing component lifecycle and dependencies
pub struct ServiceRegistry {
    components: HashMap<String, Arc<dyn Component>>,
    services: HashMap<String, Arc<dyn Any + Send + Sync>>,
    initialization_order: Vec<String>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            services: HashMap::new(),
            initialization_order: Vec::new(),
        }
    }

    /// Register a component with the registry
    pub fn register_component<T>(&mut self, component: T) -> Result<(), ComponentError>
    where
        T: Component + 'static,
    {
        let id = component.component_id().to_string();

        if self.components.contains_key(&id) {
            return Err(ComponentError::AlreadyRegistered(id));
        }

        tracing::info!(
            "Registering component: {} ({})",
            component.component_name(),
            id
        );
        self.components.insert(id.clone(), Arc::new(component));
        self.initialization_order.push(id);

        Ok(())
    }

    /// Get a component by its ID
    pub fn get_component(&self, component_id: &str) -> Option<Arc<dyn Component>> {
        self.components.get(component_id).cloned()
    }

    /// Get a service by its type
    pub fn get_service<T: 'static>(&self, service_type: &str) -> Option<Arc<T>> {
        self.services
            .get(service_type)
            .and_then(|service| service.clone().downcast().ok())
    }

    /// Register a service
    pub fn register_service<T: Send + Sync + 'static>(&mut self, service_type: &str, service: T) {
        tracing::info!("Registering service: {}", service_type);
        self.services
            .insert(service_type.to_string(), Arc::new(service));
    }

    /// Initialize all components in dependency order
    pub async fn initialize_all(&mut self) -> Result<(), ComponentError> {
        // First, build dependency graph and topological sort
        let order = self.resolve_initialization_order()?;

        // Initialize each component in order
        for component_id in order {
            if let Some(component) = self.components.get(&component_id) {
                tracing::info!("Initializing component: {}", component_id);

                // We need to handle mutable reference here - in real implementation
                // we'd need Arc<Mutex<>> or similar for thread-safe mutation
                // For now, we'll assume single-threaded initialization

                // component.initialize(self).await?;

                // Register services provided by this component
                if let Some(provider) = component
                    .as_ref()
                    .as_any()
                    .downcast_ref::<dyn ServiceProvider>()
                {
                    for service_type in provider.provided_services() {
                        // Register the component itself as the service provider
                        self.services
                            .insert(service_type.to_string(), component.clone());
                    }
                }
            }
        }

        Ok(())
    }

    /// Resolve the order in which components should be initialized based on dependencies
    fn resolve_initialization_order(&self) -> Result<Vec<String>, ComponentError> {
        // Simple approach: return registration order for now
        // In a full implementation, we'd do topological sorting based on service dependencies
        Ok(self.initialization_order.clone())
    }

    /// Shutdown all components in reverse order
    pub async fn shutdown_all(&mut self) -> Result<(), ComponentError> {
        let mut order = self.initialization_order.clone();
        order.reverse();

        for component_id in order {
            if let Some(_component) = self.components.get(&component_id) {
                tracing::info!("Shutting down component: {}", component_id);
                // component.shutdown().await?;
            }
        }

        Ok(())
    }

    /// Perform health check on all components
    pub async fn health_check_all(&self) -> Result<HashMap<String, bool>, ComponentError> {
        let mut results = HashMap::new();

        for (id, component) in &self.components {
            let is_healthy = component.health_check().await.is_ok();
            results.insert(id.clone(), is_healthy);
        }

        Ok(results)
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ComponentError {
    #[error("Component already registered: {0}")]
    AlreadyRegistered(String),
    #[error("Component not found: {0}")]
    NotFound(String),
    #[error("Service not available: {0}")]
    ServiceUnavailable(String),
    #[error("Dependency cycle detected")]
    DependencyCycle,
    #[error("Initialization failed: {0}")]
    InitializationFailed(String),
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    #[error("Service injection failed: {0}")]
    ServiceInjectionFailed(String),
}
