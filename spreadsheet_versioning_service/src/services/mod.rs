pub mod conflict_resolver;
pub mod diff_engine;
pub mod merge_engine;
pub mod versioning;

pub use diff_engine::DiffEngine;

// Services are exposed through AppState, not re-exported here
// pub use conflict_resolver::ConflictResolver;
// pub use merge_engine::MergeEngine;
// pub use versioning::VersioningService;
