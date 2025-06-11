use async_trait::async_trait;
use calamine::{open_workbook_auto_from_rs, Reader, Xls, Xlsx};
use serde_json::json;
use std::collections::HashMap;
use std::io::Cursor;
use uuid::Uuid;

use crate::{
    models::spreadsheet::{
        CreateSpreadsheetDataset, CreateSpreadsheetRecord, ParsedSpreadsheetData,
        SpreadsheetDataManager, SpreadsheetDataset, SpreadsheetSearchQuery,
        SpreadsheetSearchResult, UploadStatus,
    },
    services::{HealthCheck, HealthStatus, Service, ServiceConfig, ServiceHealth},
};

#[derive(Clone)]
pub struct SpreadsheetService {
    manager: SpreadsheetDataManager,
}

impl SpreadsheetService {
    pub fn new(manager: SpreadsheetDataManager) -> Self {
        Self { manager }
    }

    /// Parse CSV data from bytes
    pub fn parse_csv_data(
        &self,
        data: &[u8],
    ) -> Result<ParsedSpreadsheetData, Box<dyn std::error::Error + Send + Sync>> {
        let content = std::str::from_utf8(data)?;
        let mut reader = csv::Reader::from_reader(content.as_bytes());

        // Get headers
        let headers = reader
            .headers()?
            .iter()
            .map(|h| h.to_string())
            .collect::<Vec<_>>();
        let total_columns = headers.len();

        // Parse rows
        let mut rows = Vec::new();
        for result in reader.records() {
            let record = result?;
            let mut row_map = HashMap::new();

            for (i, field) in record.iter().enumerate() {
                if i < headers.len() {
                    row_map.insert(headers[i].clone(), field.to_string());
                }
            }
            rows.push(row_map);
        }

        let total_rows = rows.len();

        Ok(ParsedSpreadsheetData {
            headers,
            rows,
            total_rows,
            total_columns,
        })
    }

    /// Parse Excel data from bytes
    pub fn parse_excel_data(
        &self,
        data: &[u8],
        sheet_name: Option<&str>,
    ) -> Result<ParsedSpreadsheetData, Box<dyn std::error::Error + Send + Sync>> {
        let cursor = Cursor::new(data);
        let mut workbook = open_workbook_auto_from_rs(cursor)?;

        // Get the sheet name to use
        let target_sheet = match sheet_name {
            Some(name) => name.to_string(),
            None => {
                // Use the first sheet if no sheet name specified
                workbook
                    .sheet_names()
                    .first()
                    .ok_or("No sheets found in workbook")?
                    .clone()
            }
        };

        let sheet = workbook
            .worksheet_range(&target_sheet)
            .ok_or(format!("Sheet '{}' not found", target_sheet))?
            .map_err(|e| format!("Error reading sheet: {}", e))?;

        let mut headers = Vec::new();
        let mut rows = Vec::new();

        // Get dimensions
        let (row_count, col_count) = sheet.get_size();

        if row_count == 0 {
            return Ok(ParsedSpreadsheetData {
                headers: Vec::new(),
                rows: Vec::new(),
                total_rows: 0,
                total_columns: 0,
            });
        }

        // Extract headers from first row
        for col in 0..col_count {
            let header = sheet
                .get_value((0, col as u32))
                .map(|v| v.to_string())
                .unwrap_or_else(|| format!("Column_{}", col + 1));
            headers.push(header);
        }

        // Extract data rows (skip header row)
        for row in 1..row_count {
            let mut row_map = HashMap::new();

            for col in 0..col_count {
                let value = sheet
                    .get_value((row as u32, col as u32))
                    .map(|v| v.to_string())
                    .unwrap_or_default();

                if col < headers.len() {
                    row_map.insert(headers[col].clone(), value);
                }
            }
            rows.push(row_map);
        }

        let total_rows = rows.len();
        Ok(ParsedSpreadsheetData {
            headers,
            rows,
            total_rows,
            total_columns: col_count,
        })
    }

    /// Process and store uploaded spreadsheet data
    pub async fn process_upload(
        &self,
        filename: String,
        original_filename: String,
        file_data: Vec<u8>,
        file_type: String,
        sheet_name: Option<String>,
        uploaded_by: Option<String>,
    ) -> Result<SpreadsheetDataset, Box<dyn std::error::Error + Send + Sync>> {
        // Create initial dataset record
        let create_dataset = CreateSpreadsheetDataset {
            filename: filename.clone(),
            original_filename,
            file_type: file_type.clone(),
            file_size: file_data.len() as i64,
            sheet_name: sheet_name.clone(),
            column_headers: Vec::new(), // Will be updated after parsing
            uploaded_by,
            metadata: Some(json!({
                "processing_started_at": chrono::Utc::now(),
                "file_size_bytes": file_data.len()
            })),
        };

        let initial_dataset = self
            .manager
            .create_dataset(create_dataset)
            .await
            .map_err(|e| format!("Failed to create dataset: {}", e))?;

        // Parse the file data
        let parsed_data = match file_type.to_lowercase().as_str() {
            "csv" => self.parse_csv_data(&file_data),
            "xlsx" | "xls" => self.parse_excel_data(&file_data, sheet_name.as_deref()),
            _ => return Err(format!("Unsupported file type: {}", file_type).into()),
        };

        match parsed_data {
            Ok(data) => {
                // Create records for each row
                let mut records = Vec::new();
                for (index, row) in data.rows.iter().enumerate() {
                    records.push(CreateSpreadsheetRecord {
                        dataset_id: initial_dataset.id,
                        row_number: (index + 1) as i32, // 1-based indexing
                        row_data: json!(row),
                    });
                }

                // Bulk insert records
                let _inserted_count = self
                    .manager
                    .bulk_create_records(records)
                    .await
                    .map_err(|e| format!("Failed to insert records: {}", e))?;

                // Update dataset with final counts and status
                let dataset = self
                    .manager
                    .update_dataset_status(
                        initial_dataset.id,
                        UploadStatus::Completed,
                        None,
                        Some(data.total_rows as i32),
                        Some(data.total_columns as i32),
                    )
                    .await
                    .map_err(|e| format!("Failed to update dataset status: {}", e))?;

                Ok(dataset)
            }
            Err(e) => {
                // Update dataset with error status
                let error_message = format!("Failed to parse file: {}", e);
                let _dataset = self
                    .manager
                    .update_dataset_status(
                        initial_dataset.id,
                        UploadStatus::Failed,
                        Some(error_message.clone()),
                        None,
                        None,
                    )
                    .await
                    .map_err(|e| format!("Failed to update dataset error status: {}", e))?;

                Err(error_message.into())
            }
        }
    }

    pub async fn search_data(
        &self,
        query: SpreadsheetSearchQuery,
    ) -> Result<SpreadsheetSearchResult, sqlx::Error> {
        self.manager.search_records(query).await
    }

    pub async fn get_dataset(&self, dataset_id: Uuid) -> Result<SpreadsheetDataset, sqlx::Error> {
        self.manager.get_dataset(dataset_id).await
    }

    pub async fn list_datasets(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<SpreadsheetDataset>, sqlx::Error> {
        self.manager.list_datasets(limit, offset).await
    }

    pub async fn delete_dataset(&self, dataset_id: Uuid) -> Result<u64, sqlx::Error> {
        self.manager.delete_dataset(dataset_id).await
    }

    /// Get supported file types
    pub fn supported_file_types(&self) -> Vec<&'static str> {
        vec!["csv", "xlsx", "xls"]
    }

    /// Validate file type
    pub fn is_supported_file_type(&self, file_type: &str) -> bool {
        self.supported_file_types()
            .contains(&file_type.to_lowercase().as_str())
    }

    /// Get file type from filename
    pub fn detect_file_type(&self, filename: &str) -> Option<String> {
        let extension = std::path::Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase());

        match extension.as_deref() {
            Some("csv") => Some("csv".to_string()),
            Some("xlsx") => Some("xlsx".to_string()),
            Some("xls") => Some("xls".to_string()),
            _ => None,
        }
    }
}

#[async_trait]
impl Service for SpreadsheetService {
    fn name(&self) -> &'static str {
        "spreadsheet_service"
    }

    async fn health_check(&self) -> ServiceHealth {
        let mut checks = HashMap::new();

        // Test database connectivity by listing datasets
        let start = std::time::Instant::now();
        let db_check = match self.manager.list_datasets(Some(1), Some(0)).await {
            Ok(_) => HealthCheck {
                status: HealthStatus::Healthy,
                duration_ms: start.elapsed().as_millis() as u64,
                details: Some("Database connection successful".to_string()),
            },
            Err(e) => HealthCheck {
                status: HealthStatus::Unhealthy,
                duration_ms: start.elapsed().as_millis() as u64,
                details: Some(format!("Database error: {}", e)),
            },
        };

        checks.insert("database".to_string(), db_check.clone());

        // Test CSV parsing capability
        let csv_test_data = "header1,header2\nvalue1,value2\n";
        let parse_check = match self.parse_csv_data(csv_test_data.as_bytes()) {
            Ok(_) => HealthCheck {
                status: HealthStatus::Healthy,
                duration_ms: 0,
                details: Some("CSV parsing functional".to_string()),
            },
            Err(e) => HealthCheck {
                status: HealthStatus::Unhealthy,
                duration_ms: 0,
                details: Some(format!("CSV parsing error: {}", e)),
            },
        };

        checks.insert("parsing".to_string(), parse_check.clone());

        // Overall status is unhealthy if any check fails
        let overall_status = if db_check.status == HealthStatus::Healthy
            && parse_check.status == HealthStatus::Healthy
        {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unhealthy
        };

        ServiceHealth {
            status: overall_status,
            message: Some("Spreadsheet service health check".to_string()),
            checks,
        }
    }

    fn config(&self) -> ServiceConfig {
        ServiceConfig {
            name: "spreadsheet_service".to_string(),
            version: "1.0.0".to_string(),
            dependencies: vec!["database".to_string()],
            settings: HashMap::from([
                ("supported_formats".to_string(), "csv,xlsx,xls".to_string()),
                ("max_file_size".to_string(), "100MB".to_string()),
                ("max_records_per_file".to_string(), "1000000".to_string()),
            ]),
        }
    }
}
