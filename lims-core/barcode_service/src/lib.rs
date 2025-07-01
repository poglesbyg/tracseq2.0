//! TracSeq Barcode Service Library
//! 
//! This library provides barcode generation, validation, and management
//! functionality for the TracSeq laboratory management system.

pub mod config;
pub mod database;
pub mod error;
pub mod handlers;
pub mod models;
pub mod service;

// Re-export commonly used types
pub use config::{BarcodeConfig, Config};
pub use database::DatabasePool;
pub use error::{BarcodeError, Result};
pub use models::*;
pub use service::BarcodeService;