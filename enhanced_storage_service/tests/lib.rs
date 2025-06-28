// Enhanced Storage Service Test Suite
// Comprehensive testing for all functionality including IoT, Blockchain, Analytics

pub mod test_utils;
pub mod fixtures;

// Unit test modules
pub mod unit {
    pub mod test_storage_handlers;
    pub mod test_iot_handlers;
    pub mod test_health_handlers;
}

// Integration test modules
pub mod integration {
    pub mod test_database_operations;
    pub mod storage_workflow_tests;
}

// End-to-end test modules
pub mod end_to_end {
    pub mod test_complete_workflows;
} 
