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

        let port = env::var("PORT")
            .unwrap_or_else(|_| "8088".to_string())
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid PORT value: {}", e))?;

        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://postgres:postgres@localhost/lab_manager".to_string()
        });

        // Validate database URL
        if database_url.is_empty() {
            return Err(anyhow::anyhow!("DATABASE_URL cannot be empty"));
        }

        let max_file_size_mb = env::var("MAX_FILE_SIZE_MB")
            .unwrap_or_else(|_| "100".to_string())
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid MAX_FILE_SIZE_MB value: {}", e))?;

        let max_versions_per_spreadsheet = env::var("MAX_VERSIONS_PER_SPREADSHEET")
            .unwrap_or_else(|_| "50".to_string())
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid MAX_VERSIONS_PER_SPREADSHEET value: {}", e))?;

        let enable_auto_versioning = env::var("ENABLE_AUTO_VERSIONING")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid ENABLE_AUTO_VERSIONING value: {}", e))?;

        let retention_days = env::var("RETENTION_DAYS")
            .unwrap_or_else(|_| "365".to_string())
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid RETENTION_DAYS value: {}", e))?;

        let diff_algorithm = match env::var("DIFF_ALGORITHM")
            .unwrap_or_else(|_| "structural_aware".to_string())
            .as_str()
        {
            "cell_by_cell" => DiffAlgorithm::CellByCell,
            "semantic" => DiffAlgorithm::Semantic,
            "structural_aware" => DiffAlgorithm::StructuralAware,
            other => return Err(anyhow::anyhow!("Invalid DIFF_ALGORITHM value: {}", other)),
        };

        let conflict_strategy = match env::var("CONFLICT_STRATEGY")
            .unwrap_or_else(|_| "manual_review".to_string())
            .as_str()
        {
            "latest_wins" => ConflictStrategy::LatestWins,
            "auto_merge" => ConflictStrategy::AutoMerge,
            "custom_rules" => ConflictStrategy::CustomRules,
            "manual_review" => ConflictStrategy::ManualReview,
            other => return Err(anyhow::anyhow!("Invalid CONFLICT_STRATEGY value: {}", other)),
        };

        let auto_resolve_threshold = env::var("AUTO_RESOLVE_THRESHOLD")
            .unwrap_or_else(|_| "0.95".to_string())
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid AUTO_RESOLVE_THRESHOLD value: {}", e))?;

        let require_manual_approval = env::var("REQUIRE_MANUAL_APPROVAL")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid REQUIRE_MANUAL_APPROVAL value: {}", e))?;

        Ok(Config {
            port,
            database_url,
            max_file_size_mb,
            max_versions_per_spreadsheet,
            enable_auto_versioning,
            retention_days,
            diff_algorithm,
            conflict_resolution: ConflictResolution {
                strategy: conflict_strategy,
                auto_resolve_threshold,
                require_manual_approval,
            },
        })
    }
}
