pub mod test_utils;

#[cfg(test)]
pub mod unit {
    pub mod test_auth_handlers;
    pub mod test_auth_service;
    pub mod test_validation;
}

#[cfg(test)]
pub mod integration {
    pub mod test_auth_flow;
    pub mod test_database_operations;
}

#[cfg(test)]
pub mod end_to_end {
    pub mod test_complete_auth_workflows;
}

#[cfg(test)]
pub mod security {
    pub mod test_auth_security;
    pub mod test_jwt_security;
}

#[cfg(test)]
pub mod performance {
    pub mod test_auth_performance;
}

// Re-export test utilities
pub use test_utils::*;

// Import the test macro
#[macro_export]
macro_rules! test_with_auth_db {
    ($test_name:ident, $test_body:expr) => {
        #[tokio::test]
        async fn $test_name() {
            let mut test_db = crate::test_utils::TestDatabase::new().await;
            let result = std::panic::AssertUnwindSafe($test_body(&mut test_db))
                .catch_unwind()
                .await;
            test_db.cleanup().await;
            if let Err(panic) = result {
                std::panic::resume_unwind(panic);
            }
        }
    };
} 
