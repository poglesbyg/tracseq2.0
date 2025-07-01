#[macro_use]
pub mod test_utils;

#[cfg(test)]
pub mod unit {
    pub mod test_dashboard_handlers;
    pub mod test_cache_behavior;
    pub mod test_service_aggregation;
}

#[cfg(test)]
pub mod integration {
    pub mod test_dashboard_api;
    pub mod test_service_integration;
}

// Re-export test utilities
pub use test_utils::*;