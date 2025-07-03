use crate::{
    database::Database,
    error::{ServiceError, ServiceResult},
    models::*,
};
use std::{collections::HashMap, sync::Arc};
use tracing::info;
use uuid::Uuid;

#[derive(Debug)]
pub struct DiffEngine {
    database: Arc<Database>,
}

impl DiffEngine {
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    /// Generate a comprehensive diff between two versions
    pub async fn generate_diff(
        &self,
        request: DiffRequest,
    ) -> ServiceResult<DiffResponse> {
        info!("Generating diff between versions {} and {}", 
              request.from_version_id, request.to_version_id);

        // Get version metadata
        let from_version = self.get_version(request.from_version_id).await?;
        let to_version = self.get_version(request.to_version_id).await?;

        // Get version data
        let from_data = self.get_version_data(request.from_version_id).await?;
        let to_data = self.get_version_data(request.to_version_id).await?;

        // Check if diff already exists
        if let Ok(existing_diffs) = self.get_existing_diff(request.from_version_id, request.to_version_id).await {
            if !existing_diffs.is_empty() {
                info!("Using existing diff for versions {} -> {}", 
                      request.from_version_id, request.to_version_id);
                return Ok(DiffResponse {
                    from_version,
                    to_version,
                    diffs: existing_diffs.clone(),
                    summary: self.calculate_diff_summary(&existing_diffs),
                    generated_at: chrono::Utc::now(),
                });
            }
        }

        // Generate new diff
        let diffs = self.compute_diff(&from_data, &to_data, &request.diff_options.unwrap_or_default()).await?;

        // Store the diff
        self.store_diff(request.from_version_id, request.to_version_id, &diffs).await?;

        let summary = self.calculate_diff_summary(&diffs);

        Ok(DiffResponse {
            from_version,
            to_version,
            diffs,
            summary,
            generated_at: chrono::Utc::now(),
        })
    }

    /// Compute structural diff between spreadsheet versions
    async fn compute_diff(
        &self,
        from_data: &[VersionData],
        to_data: &[VersionData],
        options: &DiffOptions,
    ) -> ServiceResult<Vec<VersionDiff>> {
        let mut diffs = Vec::new();

        // Create lookup maps for efficient comparison
        let from_map = self.create_cell_map(from_data);
        let to_map = self.create_cell_map(to_data);

        // Find all unique cell positions
        let mut all_positions = std::collections::HashSet::new();
        for key in from_map.keys().chain(to_map.keys()) {
            all_positions.insert(key.clone());
        }

        // Compare each cell position
        for position in all_positions {
            let from_cell = from_map.get(&position);
            let to_cell = to_map.get(&position);

            match (from_cell, to_cell) {
                (Some(from), Some(to)) => {
                    // Cell exists in both versions - check for changes
                    if self.cells_differ(from, to, options) {
                        diffs.push(VersionDiff {
                            id: Uuid::new_v4(),
                            from_version_id: Uuid::new_v4(), // Will be set when storing
                            to_version_id: Uuid::new_v4(),   // Will be set when storing
                            diff_type: "cell_change".to_string(),
                            sheet_name: Some(from.sheet_name.clone()),
                            row_index: Some(from.row_index),
                            column_index: Some(from.column_index),
                            column_name: from.column_name.clone(),
                            old_value: from.cell_value.clone(),
                            new_value: to.cell_value.clone(),
                            change_metadata: serde_json::json!({
                                "old_data_type": from.data_type,
                                "new_data_type": to.data_type,
                                "position": format!("{}:{}:{}", from.sheet_name, from.row_index, from.column_index)
                            }),
                            created_at: chrono::Utc::now(),
                        });
                    }
                }
                (Some(from), None) => {
                    // Cell was deleted
                    diffs.push(VersionDiff {
                        id: Uuid::new_v4(),
                        from_version_id: Uuid::new_v4(),
                        to_version_id: Uuid::new_v4(),
                        diff_type: "cell_deleted".to_string(),
                        sheet_name: Some(from.sheet_name.clone()),
                        row_index: Some(from.row_index),
                        column_index: Some(from.column_index),
                        column_name: from.column_name.clone(),
                        old_value: from.cell_value.clone(),
                        new_value: None,
                        change_metadata: serde_json::json!({
                            "deleted_data_type": from.data_type,
                            "position": format!("{}:{}:{}", from.sheet_name, from.row_index, from.column_index)
                        }),
                        created_at: chrono::Utc::now(),
                    });
                }
                (None, Some(to)) => {
                    // Cell was added
                    diffs.push(VersionDiff {
                        id: Uuid::new_v4(),
                        from_version_id: Uuid::new_v4(),
                        to_version_id: Uuid::new_v4(),
                        diff_type: "cell_added".to_string(),
                        sheet_name: Some(to.sheet_name.clone()),
                        row_index: Some(to.row_index),
                        column_index: Some(to.column_index),
                        column_name: to.column_name.clone(),
                        old_value: None,
                        new_value: to.cell_value.clone(),
                        change_metadata: serde_json::json!({
                            "added_data_type": to.data_type,
                            "position": format!("{}:{}:{}", to.sheet_name, to.row_index, to.column_index)
                        }),
                        created_at: chrono::Utc::now(),
                    });
                }
                (None, None) => unreachable!(),
            }
        }

        // Detect structural changes (rows/columns added/removed)
        let structural_diffs = self.detect_structural_changes(from_data, to_data).await?;
        diffs.extend(structural_diffs);

        info!("Generated {} diffs", diffs.len());
        Ok(diffs)
    }

    /// Detect structural changes like added/removed rows or columns
    async fn detect_structural_changes(
        &self,
        from_data: &[VersionData],
        to_data: &[VersionData],
    ) -> ServiceResult<Vec<VersionDiff>> {
        let mut diffs = Vec::new();

        // Analyze row structure
        let from_rows = self.get_row_structure(from_data);
        let to_rows = self.get_row_structure(to_data);

        for sheet_name in from_rows.keys().chain(to_rows.keys()).collect::<std::collections::HashSet<_>>() {
            let from_sheet_rows = from_rows.get(sheet_name).cloned().unwrap_or_default();
            let to_sheet_rows = to_rows.get(sheet_name).cloned().unwrap_or_default();

            // Find added rows
            for row_index in to_sheet_rows.difference(&from_sheet_rows) {
                diffs.push(VersionDiff {
                    id: Uuid::new_v4(),
                    from_version_id: Uuid::new_v4(),
                    to_version_id: Uuid::new_v4(),
                    diff_type: "row_added".to_string(),
                    sheet_name: Some(sheet_name.clone()),
                    row_index: Some(*row_index),
                    column_index: None,
                    column_name: None,
                    old_value: None,
                    new_value: Some(format!("Row {}", row_index)),
                    change_metadata: serde_json::json!({
                        "structural_change": true,
                        "change_type": "row_addition"
                    }),
                    created_at: chrono::Utc::now(),
                });
            }

            // Find deleted rows
            for row_index in from_sheet_rows.difference(&to_sheet_rows) {
                diffs.push(VersionDiff {
                    id: Uuid::new_v4(),
                    from_version_id: Uuid::new_v4(),
                    to_version_id: Uuid::new_v4(),
                    diff_type: "row_deleted".to_string(),
                    sheet_name: Some(sheet_name.clone()),
                    row_index: Some(*row_index),
                    column_index: None,
                    column_name: None,
                    old_value: Some(format!("Row {}", row_index)),
                    new_value: None,
                    change_metadata: serde_json::json!({
                        "structural_change": true,
                        "change_type": "row_deletion"
                    }),
                    created_at: chrono::Utc::now(),
                });
            }
        }

        // Similarly analyze column structure
        let from_columns = self.get_column_structure(from_data);
        let to_columns = self.get_column_structure(to_data);

        for sheet_name in from_columns.keys().chain(to_columns.keys()).collect::<std::collections::HashSet<_>>() {
            let from_sheet_columns = from_columns.get(sheet_name).cloned().unwrap_or_default();
            let to_sheet_columns = to_columns.get(sheet_name).cloned().unwrap_or_default();

            // Find added columns
            for col_index in to_sheet_columns.difference(&from_sheet_columns) {
                diffs.push(VersionDiff {
                    id: Uuid::new_v4(),
                    from_version_id: Uuid::new_v4(),
                    to_version_id: Uuid::new_v4(),
                    diff_type: "column_added".to_string(),
                    sheet_name: Some(sheet_name.clone()),
                    row_index: None,
                    column_index: Some(*col_index),
                    column_name: Some(format!("Column_{}", col_index + 1)),
                    old_value: None,
                    new_value: Some(format!("Column {}", col_index)),
                    change_metadata: serde_json::json!({
                        "structural_change": true,
                        "change_type": "column_addition"
                    }),
                    created_at: chrono::Utc::now(),
                });
            }

            // Find deleted columns
            for col_index in from_sheet_columns.difference(&to_sheet_columns) {
                diffs.push(VersionDiff {
                    id: Uuid::new_v4(),
                    from_version_id: Uuid::new_v4(),
                    to_version_id: Uuid::new_v4(),
                    diff_type: "column_deleted".to_string(),
                    sheet_name: Some(sheet_name.clone()),
                    row_index: None,
                    column_index: Some(*col_index),
                    column_name: Some(format!("Column_{}", col_index + 1)),
                    old_value: Some(format!("Column {}", col_index)),
                    new_value: None,
                    change_metadata: serde_json::json!({
                        "structural_change": true,
                        "change_type": "column_deletion"
                    }),
                    created_at: chrono::Utc::now(),
                });
            }
        }

        Ok(diffs)
    }

    // Helper methods

    fn create_cell_map<'a>(&self, data: &'a [VersionData]) -> HashMap<String, &'a VersionData> {
        data.iter()
            .map(|cell| {
                let key = format!("{}:{}:{}", cell.sheet_name, cell.row_index, cell.column_index);
                (key, cell)
            })
            .collect()
    }

    fn cells_differ(&self, from: &VersionData, to: &VersionData, options: &DiffOptions) -> bool {
        let from_value = from.cell_value.as_deref().unwrap_or("");
        let to_value = to.cell_value.as_deref().unwrap_or("");

        let from_compare = if options.ignore_whitespace {
            from_value.trim()
        } else {
            from_value
        };

        let to_compare = if options.ignore_whitespace {
            to_value.trim()
        } else {
            to_value
        };

        if options.ignore_case {
            from_compare.to_lowercase() != to_compare.to_lowercase()
        } else {
            from_compare != to_compare
        }
    }

    fn get_row_structure(&self, data: &[VersionData]) -> HashMap<String, std::collections::HashSet<i32>> {
        let mut structure = HashMap::new();
        for cell in data {
            structure
                .entry(cell.sheet_name.clone())
                .or_insert_with(std::collections::HashSet::new)
                .insert(cell.row_index);
        }
        structure
    }

    fn get_column_structure(&self, data: &[VersionData]) -> HashMap<String, std::collections::HashSet<i32>> {
        let mut structure = HashMap::new();
        for cell in data {
            structure
                .entry(cell.sheet_name.clone())
                .or_insert_with(std::collections::HashSet::new)
                .insert(cell.column_index);
        }
        structure
    }

    fn calculate_diff_summary(&self, diffs: &[VersionDiff]) -> DiffSummary {
        let mut summary = DiffSummary {
            total_changes: diffs.len(),
            cell_changes: 0,
            row_changes: 0,
            column_changes: 0,
            sheet_changes: 0,
            structural_changes: 0,
        };

        for diff in diffs {
            match diff.diff_type.as_str() {
                "cell_change" | "cell_added" | "cell_deleted" => summary.cell_changes += 1,
                "row_added" | "row_deleted" => {
                    summary.row_changes += 1;
                    summary.structural_changes += 1;
                }
                "column_added" | "column_deleted" => {
                    summary.column_changes += 1;
                    summary.structural_changes += 1;
                }
                "sheet_added" | "sheet_deleted" => {
                    summary.sheet_changes += 1;
                    summary.structural_changes += 1;
                }
                _ => {}
            }
        }

        summary
    }

    async fn get_version(&self, version_id: Uuid) -> ServiceResult<SpreadsheetVersion> {
        sqlx::query_as::<_, SpreadsheetVersion>(
            "SELECT * FROM spreadsheet_versions WHERE id = $1"
        )
        .bind(version_id)
        .fetch_one(&self.database.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => ServiceError::VersionNotFound {
                version_id: version_id.to_string(),
            },
            _ => ServiceError::Database(e),
        })
    }

    async fn get_version_data(&self, version_id: Uuid) -> ServiceResult<Vec<VersionData>> {
        sqlx::query_as::<_, VersionData>(
            "SELECT * FROM version_data WHERE version_id = $1 ORDER BY sheet_index, row_index, column_index"
        )
        .bind(version_id)
        .fetch_all(&self.database.pool)
        .await
        .map_err(ServiceError::Database)
    }

    async fn get_existing_diff(
        &self,
        from_version_id: Uuid,
        to_version_id: Uuid,
    ) -> Result<Vec<VersionDiff>, sqlx::Error> {
        sqlx::query_as::<_, VersionDiff>(
            "SELECT * FROM version_diffs WHERE from_version_id = $1 AND to_version_id = $2"
        )
        .bind(from_version_id)
        .bind(to_version_id)
        .fetch_all(&self.database.pool)
        .await
    }

    async fn store_diff(
        &self,
        from_version_id: Uuid,
        to_version_id: Uuid,
        diffs: &[VersionDiff],
    ) -> ServiceResult<()> {
        let mut tx = self.database.pool.begin().await?;

        for diff in diffs {
            sqlx::query(
                r#"
                INSERT INTO version_diffs (
                    from_version_id, to_version_id, diff_type, sheet_name,
                    row_index, column_index, column_name, old_value, new_value,
                    change_metadata
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                "#,
            )
            .bind(from_version_id)
            .bind(to_version_id)
            .bind(&diff.diff_type)
            .bind(&diff.sheet_name)
            .bind(diff.row_index)
            .bind(diff.column_index)
            .bind(&diff.column_name)
            .bind(&diff.old_value)
            .bind(&diff.new_value)
            .bind(&diff.change_metadata)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }
} 
