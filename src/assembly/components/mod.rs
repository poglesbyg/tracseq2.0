/// Modular component implementations for the IKEA-like architecture
///
/// This module contains different component implementations that can be mixed and matched
/// to create different application configurations. Each component implements the core
/// traits defined in the parent traits module.
pub mod database;
pub mod storage;

// Re-export key types for easier usage
pub use database::{DatabaseComponent, DatabaseComponentBuilder};
pub use storage::{StorageBackend, StorageComponent, StorageComponentBuilder};
