use async_trait::async_trait;
use calamine::{open_workbook_auto_from_rs, Reader};
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

    pub async fn get_available_filters(
        &self,
        dataset_id: Option<Uuid>,
    ) -> Result<crate::models::spreadsheet::AvailableFilters, sqlx::Error> {
        self.manager.get_available_filters(dataset_id).await
    }

    pub async fn analyze_dataset(
        &self,
        dataset_id: Uuid,
    ) -> Result<crate::models::spreadsheet::DatasetAnalysis, sqlx::Error> {
        self.manager.analyze_dataset(dataset_id).await
    }

    pub async fn analyze_column(
        &self,
        dataset_id: Uuid,
        column_name: &str,
    ) -> Result<crate::models::spreadsheet::ColumnAnalysis, sqlx::Error> {
        self.manager.analyze_column(dataset_id, column_name).await
    }

    /// Enhanced data processing with metadata extraction for pools, samples, and projects
    pub async fn process_upload_with_metadata(
        &self,
        filename: String,
        original_filename: String,
        file_data: Vec<u8>,
        file_type: String,
        sheet_name: Option<String>,
        uploaded_by: Option<String>,
        pool_column: Option<String>,
        sample_column: Option<String>,
        project_column: Option<String>,
    ) -> Result<SpreadsheetDataset, Box<dyn std::error::Error + Send + Sync>> {
        // Enhanced metadata to include pool/sample/project column mappings
        let mut metadata = json!({
            "processing_started_at": chrono::Utc::now(),
            "file_size_bytes": file_data.len()
        });

        if let Some(pool_col) = &pool_column {
            metadata["pool_column"] = json!(pool_col);
        }
        if let Some(sample_col) = &sample_column {
            metadata["sample_column"] = json!(sample_col);
        }
        if let Some(project_col) = &project_column {
            metadata["project_column"] = json!(project_col);
        }

        // Create initial dataset record
        let create_dataset = CreateSpreadsheetDataset {
            filename: filename.clone(),
            original_filename,
            file_type: file_type.clone(),
            file_size: file_data.len() as i64,
            sheet_name: sheet_name.clone(),
            column_headers: Vec::new(), // Will be updated after parsing
            uploaded_by,
            metadata: Some(metadata),
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
                // Create records for each row with enhanced search text
                let mut records = Vec::new();
                for (index, row) in data.rows.iter().enumerate() {
                    // Enhanced search text to include pool/sample/project data
                    let enhanced_search_text = self.generate_enhanced_search_text(
                        row,
                        pool_column.as_deref(),
                        sample_column.as_deref(),
                        project_column.as_deref(),
                    );

                    let mut enhanced_row_data = row.clone();
                    enhanced_row_data.insert("_search_enhanced".to_string(), enhanced_search_text);

                    records.push(CreateSpreadsheetRecord {
                        dataset_id: initial_dataset.id,
                        row_number: (index + 1) as i32, // 1-based indexing
                        row_data: json!(enhanced_row_data),
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

    fn generate_enhanced_search_text(
        &self,
        row_data: &HashMap<String, String>,
        pool_column: Option<&str>,
        sample_column: Option<&str>,
        project_column: Option<&str>,
    ) -> String {
        let mut search_parts = Vec::new();

        // Add all values for general search
        for value in row_data.values() {
            if !value.is_empty() {
                search_parts.push(value.clone());
            }
        }

        // Add enhanced searchable terms for pools/samples/projects
        if let Some(pool_col) = pool_column {
            if let Some(pool_value) = row_data.get(pool_col) {
                search_parts.push(format!("pool:{}", pool_value));
            }
        }

        if let Some(sample_col) = sample_column {
            if let Some(sample_value) = row_data.get(sample_col) {
                search_parts.push(format!("sample:{}", sample_value));
            }
        }

        if let Some(project_col) = project_column {
            if let Some(project_value) = row_data.get(project_col) {
                search_parts.push(format!("project:{}", project_value));
            }
        }

        search_parts.join(" ")
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

    /// Get available sheet names from Excel data
    pub fn get_excel_sheet_names(
        &self,
        data: &[u8],
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let cursor = Cursor::new(data);
        let mut workbook = open_workbook_auto_from_rs(cursor)?;
        Ok(workbook.sheet_names().to_vec())
    }

    /// Parse all sheets from Excel data
    pub fn parse_all_excel_sheets(
        &self,
        data: &[u8],
    ) -> Result<Vec<(String, ParsedSpreadsheetData)>, Box<dyn std::error::Error + Send + Sync>>
    {
        let cursor = Cursor::new(data);
        let mut workbook = open_workbook_auto_from_rs(cursor)?;
        let sheet_names = workbook.sheet_names().to_vec();
        let mut results = Vec::new();

        for sheet_name in sheet_names {
            let sheet = workbook
                .worksheet_range(&sheet_name)
                .ok_or(format!("Sheet '{}' not found", sheet_name))?
                .map_err(|e| format!("Error reading sheet: {}", e))?;

            let mut headers = Vec::new();
            let mut rows = Vec::new();

            // Get dimensions
            let (row_count, col_count) = sheet.get_size();

            if row_count == 0 {
                results.push((
                    sheet_name,
                    ParsedSpreadsheetData {
                        headers: Vec::new(),
                        rows: Vec::new(),
                        total_rows: 0,
                        total_columns: 0,
                    },
                ));
                continue;
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

            let parsed_data = ParsedSpreadsheetData {
                headers,
                rows: rows.clone(),
                total_rows: rows.len(),
                total_columns: col_count,
            };

            results.push((sheet_name, parsed_data));
        }

        Ok(results)
    }

    /// Process and store uploaded spreadsheet data with multiple sheets support
    pub async fn process_upload_multiple_sheets(
        &self,
        filename: String,
        original_filename: String,
        file_data: Vec<u8>,
        file_type: String,
        selected_sheets: Option<Vec<String>>,
        uploaded_by: Option<String>,
    ) -> Result<Vec<SpreadsheetDataset>, Box<dyn std::error::Error + Send + Sync>> {
        match file_type.to_lowercase().as_str() {
            "csv" => {
                // For CSV, process as single sheet
                let dataset = self
                    .process_upload(
                        filename,
                        original_filename,
                        file_data,
                        file_type,
                        None,
                        uploaded_by,
                    )
                    .await?;
                Ok(vec![dataset])
            }
            "xlsx" | "xls" => {
                let all_sheets = self.parse_all_excel_sheets(&file_data)?;
                let mut datasets = Vec::new();

                // Filter sheets based on selection
                let sheets_to_process: Vec<_> = if let Some(selected) = selected_sheets {
                    all_sheets
                        .into_iter()
                        .filter(|(name, _)| selected.contains(name))
                        .collect()
                } else {
                    all_sheets
                };

                let process_multiple = sheets_to_process.len() > 1;

                for (sheet_name, parsed_data) in sheets_to_process {
                    // Create dataset name with sheet suffix if multiple sheets
                    let dataset_filename = if process_multiple {
                        format!("{}_{}", filename, sheet_name.replace(" ", "_"))
                    } else {
                        filename.clone()
                    };

                    let create_dataset = CreateSpreadsheetDataset {
                        filename: dataset_filename,
                        original_filename: format!("{}_{}", original_filename, sheet_name),
                        file_type: file_type.clone(),
                        file_size: file_data.len() as i64,
                        sheet_name: Some(sheet_name.clone()),
                        column_headers: parsed_data.headers.clone(),
                        uploaded_by: uploaded_by.clone(),
                        metadata: Some(json!({
                            "processing_started_at": chrono::Utc::now(),
                            "file_size_bytes": file_data.len(),
                            "sheet_name": sheet_name,
                            "total_rows": parsed_data.total_rows,
                            "total_columns": parsed_data.total_columns,
                        })),
                    };

                    let dataset =
                        self.manager
                            .create_dataset(create_dataset)
                            .await
                            .map_err(|e| {
                                format!(
                                    "Failed to create dataset for sheet '{}': {}",
                                    sheet_name, e
                                )
                            })?;

                    // Store the parsed records
                    let mut records_to_insert = Vec::new();
                    for (row_index, row_data) in parsed_data.rows.iter().enumerate() {
                        let record = CreateSpreadsheetRecord {
                            dataset_id: dataset.id,
                            row_number: (row_index + 1) as i32, // 1-based indexing
                            row_data: json!(row_data),
                        };
                        records_to_insert.push(record);
                    }

                    if !records_to_insert.is_empty() {
                        self.manager
                            .bulk_create_records(records_to_insert)
                            .await
                            .map_err(|e| {
                                format!(
                                    "Failed to insert records for sheet '{}': {}",
                                    sheet_name, e
                                )
                            })?;
                    }

                    // Update dataset status
                    self.manager
                        .update_dataset_status(
                            dataset.id,
                            crate::models::spreadsheet::UploadStatus::Completed,
                            None,
                            Some(parsed_data.total_rows as i32),
                            Some(parsed_data.total_columns as i32),
                        )
                        .await
                        .map_err(|e| {
                            format!(
                                "Failed to update dataset status for sheet '{}': {}",
                                sheet_name, e
                            )
                        })?;

                    datasets.push(dataset);
                }

                Ok(datasets)
            }
            _ => Err(format!("Unsupported file type: {}", file_type).into()),
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
