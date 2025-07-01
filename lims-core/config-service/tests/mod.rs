#[macro_use]
pub mod test_utils;

#[cfg(test)]
pub mod unit {
    pub mod test_config_store;
    pub mod test_handlers;
    pub mod test_config_validation;
}

#[cfg(test)]
pub mod integration {
    pub mod test_service_configuration;
    pub mod test_concurrent_access;
}

// Re-export test utilities
pub use test_utils::*;