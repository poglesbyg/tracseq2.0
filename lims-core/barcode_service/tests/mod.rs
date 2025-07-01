#[macro_use]
pub mod test_utils;

#[cfg(test)]
pub mod unit {
    pub mod test_barcode_generation;
    pub mod test_barcode_validation;
    pub mod test_barcode_parsing;
}

#[cfg(test)]
pub mod integration {
    pub mod test_barcode_service_flow;
    pub mod test_barcode_reservation;
}

// Re-export test utilities
pub use test_utils::*;