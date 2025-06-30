pub mod test_utils;

#[cfg(test)]
pub mod unit {
    pub mod test_saga_patterns;
    pub mod test_transaction_coordinator;
    pub mod test_compensation_logic;
}

#[cfg(test)]
pub mod integration {
    pub mod test_distributed_transactions;
    pub mod test_saga_persistence;
    pub mod test_event_integration;
}

#[cfg(test)]
pub mod distributed {
    pub mod test_cross_service_transactions;
    pub mod test_saga_recovery;
    pub mod test_timeout_handling;
}

#[cfg(test)]
pub mod performance {
    pub mod test_concurrent_transactions;
    pub mod test_saga_throughput;
}

// Re-export test utilities
pub use test_utils::*;

// Test macro for transaction cleanup
#[macro_export]
macro_rules! test_with_transaction_cleanup {
    ($test_name:ident, $test_body:expr) => {
        #[tokio::test]
        async fn $test_name() {
            let mut test_env = crate::test_utils::TestTransactionEnvironment::new().await;
            let result = std::panic::AssertUnwindSafe($test_body(&mut test_env))
                .catch_unwind()
                .await;
            test_env.cleanup().await;
            if let Err(panic) = result {
                std::panic::resume_unwind(panic);
            }
        }
    };
} 
