pub mod test_utils;

#[cfg(test)]
pub mod integration {
    pub mod test_cross_service_events;
}

// Re-export test utilities
pub use test_utils::*; 
