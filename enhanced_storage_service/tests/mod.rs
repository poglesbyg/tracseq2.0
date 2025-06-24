// Enhanced Storage Service Test Suite
// Comprehensive testing for all functionality including IoT, Blockchain, Analytics

pub mod fixtures;
pub mod test_utils;

// Unit test modules
pub mod unit {
    pub mod test_admin_handlers;
    pub mod test_ai_handlers;
    pub mod test_analytics_handlers;
    pub mod test_automation_handlers;
    pub mod test_blockchain_handlers;
    pub mod test_compliance_handlers;
    pub mod test_digital_twin_handlers;
    pub mod test_energy_handlers;
    pub mod test_health_handlers;
    pub mod test_integration_handlers;
    pub mod test_iot_handlers;
    pub mod test_mobile_handlers;
    pub mod test_storage_handlers;
}

// Integration test modules
pub mod integration {
    pub mod test_analytics_workflows;
    pub mod test_blockchain_workflows;
    pub mod test_cross_service_integration;
    pub mod test_database_operations;
    pub mod test_iot_workflows;
    pub mod test_service_communication;
}

// End-to-end test modules
pub mod end_to_end {
    pub mod test_complete_workflows;
    pub mod test_compliance_workflows;
    pub mod test_iot_monitoring;
    pub mod test_sample_lifecycle;
}

// Performance test modules
pub mod performance {
    pub mod test_concurrent_access;
    pub mod test_database_performance;
    pub mod test_load_scenarios;
}

// Security test modules
pub mod security {
    pub mod test_authentication;
    pub mod test_authorization;
    pub mod test_input_validation;
}
