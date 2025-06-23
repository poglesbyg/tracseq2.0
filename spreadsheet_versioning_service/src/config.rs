use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub max_file_size_mb: usize,
    pub max_versions_per_spreadsheet: usize,
    pub enable_auto_versioning: bool,
    pub retention_days: u32,
    pub diff_algorithm: DiffAlgorithm,
    pub conflict_resolution: ConflictResolution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiffAlgorithm {
    CellByCell,
    StructuralAware,
    Semantic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolution {
    pub strategy: ConflictStrategy,
    pub auto_resolve_threshold: f64,
    pub require_manual_approval: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictStrategy {
    LatestWins,
    ManualReview,
    AutoMerge,
    CustomRules,
}

impl Config {
    pub fn load() -> Result<Self> {
        dotenvy::dotenv().ok();

        Ok(Config {
            port: env::var("PORT")
                .unwrap_or_else(|_| "8088".to_string())
                .parse()
                .unwrap_or(8088),

            database_url: env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgresql://postgres:password@localhost/tracseq_versioning".to_string()
            }),

            max_file_size_mb: env::var("MAX_FILE_SIZE_MB")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .unwrap_or(100),

            max_versions_per_spreadsheet: env::var("MAX_VERSIONS_PER_SPREADSHEET")
                .unwrap_or_else(|_| "50".to_string())
                .parse()
                .unwrap_or(50),

            enable_auto_versioning: env::var("ENABLE_AUTO_VERSIONING")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),

            retention_days: env::var("RETENTION_DAYS")
                .unwrap_or_else(|_| "365".to_string())
                .parse()
                .unwrap_or(365),

            diff_algorithm: match env::var("DIFF_ALGORITHM")
                .unwrap_or_else(|_| "structural_aware".to_string())
                .as_str()
            {
                "cell_by_cell" => DiffAlgorithm::CellByCell,
                "semantic" => DiffAlgorithm::Semantic,
                _ => DiffAlgorithm::StructuralAware,
            },

            conflict_resolution: ConflictResolution {
                strategy: match env::var("CONFLICT_STRATEGY")
                    .unwrap_or_else(|_| "manual_review".to_string())
                    .as_str()
                {
                    "latest_wins" => ConflictStrategy::LatestWins,
                    "auto_merge" => ConflictStrategy::AutoMerge,
                    "custom_rules" => ConflictStrategy::CustomRules,
                    _ => ConflictStrategy::ManualReview,
                },
                auto_resolve_threshold: env::var("AUTO_RESOLVE_THRESHOLD")
                    .unwrap_or_else(|_| "0.95".to_string())
                    .parse()
                    .unwrap_or(0.95),
                require_manual_approval: env::var("REQUIRE_MANUAL_APPROVAL")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
            },
        })
    }
}
