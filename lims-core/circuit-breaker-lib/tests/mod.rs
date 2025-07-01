#[macro_use]
pub mod test_utils;

#[cfg(test)]
pub mod unit {
    pub mod test_circuit_states;
    pub mod test_circuit_operations;
    pub mod test_http_client;
    pub mod test_registry;
}

#[cfg(test)]
pub mod integration {
    pub mod test_fault_tolerance;
    pub mod test_concurrent_operations;
}

// Re-export test utilities
pub use test_utils::*;