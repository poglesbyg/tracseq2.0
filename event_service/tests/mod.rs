pub mod test_utils;

#[cfg(test)]
pub mod unit {
    pub mod test_event_types;
    pub mod test_event_filters;
    pub mod test_event_handlers;
}

#[cfg(test)]
pub mod integration {
    pub mod test_pub_sub_flow;
    pub mod test_cross_service_events;
    pub mod test_event_persistence;
}

#[cfg(test)]
pub mod performance {
    pub mod test_event_throughput;
    pub mod test_concurrent_subscribers;
}

#[cfg(test)]
pub mod end_to_end {
    pub mod test_complete_event_workflows;
}

// Re-export test utilities
pub use test_utils::*;

// Test macro for event cleanup
#[macro_export]
macro_rules! test_with_event_cleanup {
    ($test_name:ident, $test_body:expr) => {
        #[tokio::test]
        async fn $test_name() {
            let mut test_env = crate::test_utils::TestEventEnvironment::new().await;
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
