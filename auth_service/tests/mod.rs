#[macro_use]
pub mod test_utils;

#[cfg(test)]
pub mod unit {
    pub mod test_auth_handlers;
    pub mod test_basic_auth;
}

#[cfg(test)]
pub mod integration {
    pub mod test_auth_flow;
}

#[cfg(test)]
pub mod security {
    pub mod test_auth_security;
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
