use crate::{
    database::Database,
    error::{ServiceError, ServiceResult},
    models::*,
};
use async_trait::async_trait;
use calamine::{open_workbook_auto_from_rs, Reader};
use sha2::{Digest, Sha256};
use std::{io::Cursor, sync::Arc};
use tracing::{info, warn, error};
use uuid::Uuid;

#[derive(Debug)]
pub struct VersioningService {
    database: Arc<Database>,
}

impl VersioningService {
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    /// Create a new version from spreadsheet data
    pub async fn create_version(
        &self,
        request: CreateVersionRequest,
    ) -> ServiceResult<SpreadsheetVersion> {
        info!("Creating new version for spreadsheet {}", request.spreadsheet_id);

        // Calculate file hash
        let file_hash = self.calculate_file_hash(&request.file_data);

        // Check if version already exists with same hash
        if let Ok(existing) = self.find_version_by_hash(&request.spreadsheet_id, &file_hash).await {
            warn!("Version with same hash already exists: {}", existing.id);
            return Err(ServiceError::VersionAlreadyExists {
                spreadsheet_id: request.spreadsheet_id.to_string(),
                version_number: existing.version_number,
            });
        }

        // Get next version number
        let version_number = self.get_next_version_number(&request.spreadsheet_id).await?;

        // Parse spreadsheet data
        let parsed_data = self.parse_spreadsheet_data(&request.file_data, &request.file_type).await?;

        let mut tx = self.database.pool.begin().await?;

        // Create version record
        let version_id = Uuid::new_v4();
        let version = sqlx::query_as::<_, SpreadsheetVersion>(
            r#"
            INSERT INTO spreadsheet_versions (
                id, spreadsheet_id, version_number, version_tag, parent_version_id,
                name, filename, original_filename, file_type, file_size, file_hash,
                changes_summary, change_count, created_by, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            RETURNING *
            "#,
        )
        .bind(version_id)
        .bind(request.spreadsheet_id)
        .bind(version_number)
        .bind(request.version_tag)
        .bind(request.parent_version_id)
        .bind(&request.name)
        .bind(&request.filename)
        .bind(&request.original_filename)
        .bind(&request.file_type)
        .bind(request.file_data.len() as i64)
        .bind(&file_hash)
        .bind(request.changes_summary)
        .bind(parsed_data.total_cells as i32)
        .bind(None::<Uuid>) // created_by - would be set from auth context
        .bind(request.metadata.unwrap_or_else(|| serde_json::json!({})))
        .fetch_one(&mut *tx)
        .await?;

        // Store version data
        self.store_version_data(&mut *tx, version_id, &parsed_data).await?;

        // Generate diff if parent version exists
        if let Some(parent_version_id) = request.parent_version_id {
            if let Err(e) = self.generate_diff(&mut *tx, parent_version_id, version_id).await {
                warn!("Failed to generate diff for version {}: {}", version_id, e);
                // Don't fail the transaction, just log the warning
            }
        }

        tx.commit().await?;

        info!("Successfully created version {} for spreadsheet {}", 
              version_number, request.spreadsheet_id);
        Ok(version)
    }

    /// Get a specific version
    pub async fn get_version(&self, version_id: Uuid) -> ServiceResult<SpreadsheetVersion> {
        let version = sqlx::query_as::<_, SpreadsheetVersion>(
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
        })?;

        Ok(version)
    }

    /// List versions for a spreadsheet
    pub async fn list_versions(
        &self,
        spreadsheet_id: Uuid,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> ServiceResult<VersionListResponse> {
        let limit = limit.unwrap_or(50).min(100);
        let offset = offset.unwrap_or(0);

        let versions = sqlx::query_as::<_, SpreadsheetVersion>(
            r#"
            SELECT * FROM spreadsheet_versions 
            WHERE spreadsheet_id = $1 
            ORDER BY version_number DESC 
            LIMIT $2 OFFSET $3
            "#
        )
        .bind(spreadsheet_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.database.pool)
        .await?;

        let total_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM spreadsheet_versions WHERE spreadsheet_id = $1"
        )
        .bind(spreadsheet_id)
        .fetch_one(&self.database.pool)
        .await?;

        Ok(VersionListResponse {
            versions,
            total_count: total_count as usize,
            page: (offset / limit) as usize,
            per_page: limit as usize,
        })
    }

    /// Update version metadata
    pub async fn update_version(
        &self,
        version_id: Uuid,
        request: UpdateVersionRequest,
    ) -> ServiceResult<SpreadsheetVersion> {
        let version = sqlx::query_as::<_, SpreadsheetVersion>(
            r#"
            UPDATE spreadsheet_versions 
            SET version_tag = COALESCE($2, version_tag),
                status = COALESCE($3, status),
                changes_summary = COALESCE($4, changes_summary),
                metadata = COALESCE($5, metadata),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(version_id)
        .bind(request.version_tag)
        .bind(request.status)
        .bind(request.changes_summary)
        .bind(request.metadata)
        .fetch_one(&self.database.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => ServiceError::VersionNotFound {
                version_id: version_id.to_string(),
            },
            _ => ServiceError::Database(e),
        })?;

        Ok(version)
    }

    /// Delete a version (soft delete)
    pub async fn delete_version(&self, version_id: Uuid) -> ServiceResult<()> {
        let affected = sqlx::query(
            "UPDATE spreadsheet_versions SET status = 'deleted' WHERE id = $1"
        )
        .bind(version_id)
        .execute(&self.database.pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(ServiceError::VersionNotFound {
                version_id: version_id.to_string(),
            });
        }

        Ok(())
    }

    /// Get version data (cells)
    pub async fn get_version_data(&self, version_id: Uuid) -> ServiceResult<Vec<VersionData>> {
        let data = sqlx::query_as::<_, VersionData>(
            r#"
            SELECT * FROM version_data 
            WHERE version_id = $1 
            ORDER BY sheet_index, row_index, column_index
            "#
        )
        .bind(version_id)
        .fetch_all(&self.database.pool)
        .await?;

        Ok(data)
    }

    // Private helper methods

    fn calculate_file_hash(&self, file_data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(file_data);
        format!("{:x}", hasher.finalize())
    }

    async fn find_version_by_hash(
        &self,
        spreadsheet_id: &Uuid,
        file_hash: &str,
    ) -> Result<SpreadsheetVersion, sqlx::Error> {
        sqlx::query_as::<_, SpreadsheetVersion>(
            "SELECT * FROM spreadsheet_versions WHERE spreadsheet_id = $1 AND file_hash = $2"
        )
        .bind(spreadsheet_id)
        .bind(file_hash)
        .fetch_one(&self.database.pool)
        .await
    }

    async fn get_next_version_number(&self, spreadsheet_id: &Uuid) -> ServiceResult<i32> {
        let max_version: Option<i32> = sqlx::query_scalar(
            "SELECT MAX(version_number) FROM spreadsheet_versions WHERE spreadsheet_id = $1"
        )
        .bind(spreadsheet_id)
        .fetch_one(&self.database.pool)
        .await?;

        Ok(max_version.unwrap_or(0) + 1)
    }

    async fn parse_spreadsheet_data(
        &self,
        file_data: &[u8],
        file_type: &str,
    ) -> ServiceResult<ParsedSpreadsheetData> {
        match file_type.to_lowercase().as_str() {
            "csv" => self.parse_csv_data(file_data),
            "xlsx" | "xls" => self.parse_excel_data(file_data),
            _ => Err(ServiceError::FileProcessing(format!(
                "Unsupported file type: {}",
                file_type
            ))),
        }
    }

    fn parse_csv_data(&self, file_data: &[u8]) -> ServiceResult<ParsedSpreadsheetData> {
        let content = std::str::from_utf8(file_data)
            .map_err(|e| ServiceError::FileProcessing(format!("Invalid UTF-8: {}", e)))?;

        let mut reader = csv::Reader::from_reader(content.as_bytes());
        let mut cells = Vec::new();
        let mut max_columns = 0;
        let mut total_rows = 0;

        // Process CSV data
        for (row_index, result) in reader.records().enumerate() {
            let record = result
                .map_err(|e| ServiceError::FileProcessing(format!("CSV parse error: {}", e)))?;
            
            total_rows = row_index + 1;
            max_columns = max_columns.max(record.len());

            for (col_index, field) in record.iter().enumerate() {
                if !field.is_empty() {
                    cells.push(CellData {
                        sheet_name: "Sheet1".to_string(),
                        sheet_index: 0,
                        row_index: row_index as i32,
                        column_index: col_index as i32,
                        column_name: Some(format!("Column_{}", col_index + 1)),
                        cell_value: Some(field.to_string()),
                        data_type: Some("text".to_string()),
                        formatted_value: Some(field.to_string()),
                        cell_formula: None,
                    });
                }
            }
        }

        Ok(ParsedSpreadsheetData {
            sheets: vec![ParsedSheetData {
                name: "Sheet1".to_string(),
                index: 0,
                cells: cells.clone(),
                row_count: total_rows,
                column_count: max_columns,
            }],
            total_cells: cells.len(),
            total_sheets: 1,
        })
    }

    fn parse_excel_data(&self, file_data: &[u8]) -> ServiceResult<ParsedSpreadsheetData> {
        let cursor = Cursor::new(file_data);
        let mut workbook = open_workbook_auto_from_rs(cursor)
            .map_err(|e| ServiceError::FileProcessing(format!("Excel parse error: {}", e)))?;

        let sheet_names = workbook.sheet_names().to_vec();
        let mut sheets = Vec::new();
        let mut total_cells = 0;

        for (sheet_index, sheet_name) in sheet_names.iter().enumerate() {
            let sheet = workbook
                .worksheet_range(sheet_name)
                .ok_or_else(|| ServiceError::FileProcessing(format!("Sheet '{}' not found", sheet_name)))?
                .map_err(|e| ServiceError::FileProcessing(format!("Error reading sheet: {}", e)))?;

            let (row_count, col_count) = sheet.get_size();
            let mut cells = Vec::new();

            for row in 0..row_count {
                for col in 0..col_count {
                    if let Some(cell_value) = sheet.get_value((row as u32, col as u32)) {
                        let value_str = cell_value.to_string();
                        if !value_str.is_empty() {
                            cells.push(CellData {
                                sheet_name: sheet_name.clone(),
                                sheet_index: sheet_index as i32,
                                row_index: row as i32,
                                column_index: col as i32,
                                column_name: Some(format!("Column_{}", col + 1)),
                                cell_value: Some(value_str.clone()),
                                data_type: Some(self.detect_cell_type(&cell_value)),
                                formatted_value: Some(value_str),
                                cell_formula: None,
                            });
                        }
                    }
                }
            }

            total_cells += cells.len();
            sheets.push(ParsedSheetData {
                name: sheet_name.clone(),
                index: sheet_index as i32,
                cells,
                row_count,
                column_count: col_count,
            });
        }

        Ok(ParsedSpreadsheetData {
            sheets,
            total_cells,
            total_sheets: sheet_names.len(),
        })
    }

    fn detect_cell_type(&self, cell_value: &calamine::DataType) -> String {
        match cell_value {
            calamine::DataType::Int(_) => "integer".to_string(),
            calamine::DataType::Float(_) => "float".to_string(),
            calamine::DataType::String(_) => "text".to_string(),
            calamine::DataType::Bool(_) => "boolean".to_string(),
            calamine::DataType::DateTime(_) => "datetime".to_string(),
            calamine::DataType::DateTimeIso(_) => "datetime".to_string(),
            calamine::DataType::DurationIso(_) => "duration".to_string(),
            _ => "text".to_string(),
        }
    }

    async fn store_version_data(
        &self,
        tx: &mut sqlx::PgConnection,
        version_id: Uuid,
        data: &ParsedSpreadsheetData,
    ) -> ServiceResult<()> {
        for sheet in &data.sheets {
            for cell in &sheet.cells {
                sqlx::query(
                    r#"
                    INSERT INTO version_data (
                        version_id, sheet_name, sheet_index, row_index, column_index,
                        column_name, cell_value, data_type, formatted_value, cell_formula,
                        cell_metadata
                    ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                    "#,
                )
                .bind(version_id)
                .bind(&cell.sheet_name)
                .bind(cell.sheet_index)
                .bind(cell.row_index)
                .bind(cell.column_index)
                .bind(&cell.column_name)
                .bind(&cell.cell_value)
                .bind(&cell.data_type)
                .bind(&cell.formatted_value)
                .bind(&cell.cell_formula)
                .bind(serde_json::json!({}))
                .execute(&mut *tx)
                .await?;
            }
        }

        Ok(())
    }

    async fn generate_diff(
        &self,
        tx: &mut sqlx::PgConnection,
        from_version_id: Uuid,
        to_version_id: Uuid,
    ) -> ServiceResult<()> {
        sqlx::query(
            r#"
            INSERT INTO version_diffs (
                from_version_id, to_version_id, diff_type, 
                change_metadata
            ) VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(from_version_id)
        .bind(to_version_id)
        .bind("version_created")
        .bind(serde_json::json!({
            "diff_generated_at": chrono::Utc::now(),
            "auto_generated": true
        }))
        .execute(&mut *tx)
        .await?;

        Ok(())
    }
}

// Helper structs for parsing
#[derive(Debug, Clone)]
struct ParsedSpreadsheetData {
    sheets: Vec<ParsedSheetData>,
    total_cells: usize,
    total_sheets: usize,
}

#[derive(Debug, Clone)]
struct ParsedSheetData {
    name: String,
    index: i32,
    cells: Vec<CellData>,
    row_count: usize,
    column_count: usize,
}

#[derive(Debug, Clone)]
struct CellData {
    sheet_name: String,
    sheet_index: i32,
    row_index: i32,
    column_index: i32,
    column_name: Option<String>,
    cell_value: Option<String>,
    data_type: Option<String>,
    formatted_value: Option<String>,
    cell_formula: Option<String>,
} 
