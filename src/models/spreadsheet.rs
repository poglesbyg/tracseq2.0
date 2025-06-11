use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct SpreadsheetDataset {
    pub id: Uuid,
    pub filename: String,
    pub original_filename: String,
    pub file_type: String,
    pub file_size: i64,
    pub sheet_name: Option<String>,
    pub total_rows: i32,
    pub total_columns: i32,
    pub column_headers: Vec<String>,
    pub upload_status: UploadStatus,
    pub error_message: Option<String>,
    pub uploaded_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "snake_case")]
pub enum UploadStatus {
    Processing,
    Completed,
    Failed,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct SpreadsheetRecord {
    pub id: Uuid,
    pub dataset_id: Uuid,
    pub row_number: i32,
    pub row_data: serde_json::Value,
    pub search_text: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct CreateSpreadsheetDataset {
    pub filename: String,
    pub original_filename: String,
    pub file_type: String,
    pub file_size: i64,
    pub sheet_name: Option<String>,
    pub column_headers: Vec<String>,
    pub uploaded_by: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct CreateSpreadsheetRecord {
    pub dataset_id: Uuid,
    pub row_number: i32,
    pub row_data: serde_json::Value,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct SpreadsheetSearchQuery {
    pub search_term: Option<String>,
    pub dataset_id: Option<Uuid>,
    pub column_filters: Option<HashMap<String, String>>,
    pub pool_filter: Option<String>,
    pub sample_filter: Option<String>,
    pub project_filter: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpreadsheetSearchResult {
    pub records: Vec<SpreadsheetRecord>,
    pub total_count: i64,
    pub dataset_info: Option<SpreadsheetDataset>,
    pub available_filters: Option<AvailableFilters>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AvailableFilters {
    pub pools: Vec<String>,
    pub samples: Vec<String>,
    pub projects: Vec<String>,
    pub all_columns: Vec<String>,
    pub column_values: HashMap<String, Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ColumnAnalysis {
    pub column_name: String,
    pub unique_values: Vec<String>,
    pub value_count: usize,
    pub data_type: String,
    pub suggested_filter_type: FilterType,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FilterType {
    Dropdown, // For columns with limited unique values
    Text,     // For text search
    Numeric,  // For numeric ranges
    Date,     // For date ranges
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatasetAnalysis {
    pub dataset_id: Uuid,
    pub column_analysis: Vec<ColumnAnalysis>,
    pub detected_pools: Vec<String>,
    pub detected_samples: Vec<String>,
    pub detected_projects: Vec<String>,
    pub total_unique_values: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParsedSpreadsheetData {
    pub headers: Vec<String>,
    pub rows: Vec<HashMap<String, String>>,
    pub total_rows: usize,
    pub total_columns: usize,
}

#[derive(Clone)]
pub struct SpreadsheetDataManager {
    pool: PgPool,
}

impl SpreadsheetDataManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_dataset(
        &self,
        dataset: CreateSpreadsheetDataset,
    ) -> Result<SpreadsheetDataset, sqlx::Error> {
        sqlx::query_as::<_, SpreadsheetDataset>(
            r#"
            INSERT INTO spreadsheet_datasets (filename, original_filename, file_type, file_size, sheet_name, column_headers, uploaded_by, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, filename, original_filename, file_type, file_size, sheet_name, total_rows, total_columns, column_headers, upload_status, error_message, uploaded_by, created_at, updated_at, metadata
            "#,
        )
        .bind(&dataset.filename)
        .bind(&dataset.original_filename)
        .bind(&dataset.file_type)
        .bind(dataset.file_size)
        .bind(dataset.sheet_name.as_deref())
        .bind(&dataset.column_headers)
        .bind(dataset.uploaded_by.as_deref())
        .bind(dataset.metadata.unwrap_or(serde_json::json!({})))
        .fetch_one(&self.pool)
        .await
    }

    pub async fn update_dataset_status(
        &self,
        dataset_id: Uuid,
        status: UploadStatus,
        error_message: Option<String>,
        total_rows: Option<i32>,
        total_columns: Option<i32>,
    ) -> Result<SpreadsheetDataset, sqlx::Error> {
        sqlx::query_as::<_, SpreadsheetDataset>(
            r#"
            UPDATE spreadsheet_datasets 
            SET upload_status = $2, error_message = $3, total_rows = COALESCE($4, total_rows), total_columns = COALESCE($5, total_columns), updated_at = NOW()
            WHERE id = $1
            RETURNING id, filename, original_filename, file_type, file_size, sheet_name, total_rows, total_columns, column_headers, upload_status, error_message, uploaded_by, created_at, updated_at, metadata
            "#,
        )
        .bind(dataset_id)
        .bind(status)
        .bind(error_message)
        .bind(total_rows)
        .bind(total_columns)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn create_record(
        &self,
        record: CreateSpreadsheetRecord,
    ) -> Result<SpreadsheetRecord, sqlx::Error> {
        // Generate search text from row data
        let search_text = self.generate_search_text(&record.row_data);

        sqlx::query_as::<_, SpreadsheetRecord>(
            r#"
            INSERT INTO spreadsheet_records (dataset_id, row_number, row_data, search_text)
            VALUES ($1, $2, $3, $4)
            RETURNING id, dataset_id, row_number, row_data, search_text, created_at
            "#,
        )
        .bind(record.dataset_id)
        .bind(record.row_number)
        .bind(&record.row_data)
        .bind(&search_text)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn bulk_create_records(
        &self,
        records: Vec<CreateSpreadsheetRecord>,
    ) -> Result<u64, sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        let mut affected_rows = 0u64;

        for record in records {
            let search_text = self.generate_search_text(&record.row_data);

            let result = sqlx::query(
                r#"
                INSERT INTO spreadsheet_records (dataset_id, row_number, row_data, search_text)
                VALUES ($1, $2, $3, $4)
                "#,
            )
            .bind(record.dataset_id)
            .bind(record.row_number)
            .bind(&record.row_data)
            .bind(&search_text)
            .execute(&mut *tx)
            .await?;

            affected_rows += result.rows_affected();
        }

        tx.commit().await?;
        Ok(affected_rows)
    }

    pub async fn search_records(
        &self,
        query: SpreadsheetSearchQuery,
    ) -> Result<SpreadsheetSearchResult, sqlx::Error> {
        let limit = query.limit.unwrap_or(50).min(1000); // Cap at 1000 results
        let offset = query.offset.unwrap_or(0);

        // Build the WHERE clause dynamically
        let mut where_conditions = Vec::new();
        let mut param_count = 1;

        if query.dataset_id.is_some() {
            where_conditions.push(format!("sr.dataset_id = ${}", param_count));
            param_count += 1;
        }

        if query.search_term.is_some() {
            where_conditions.push(format!(
                "to_tsvector('english', sr.search_text) @@ plainto_tsquery('english', ${})",
                param_count
            ));
            param_count += 1;
        }

        // Add pool/sample/project filters with smart column detection
        if query.pool_filter.is_some() {
            where_conditions.push(format!(
                "(sr.row_data ->> 'Pool' ILIKE ${} OR sr.row_data ->> 'Pool_ID' ILIKE ${} OR sr.row_data ->> 'PoolID' ILIKE ${})",
                param_count, param_count, param_count
            ));
            param_count += 1;
        }

        if query.sample_filter.is_some() {
            where_conditions.push(format!(
                "(sr.row_data ->> 'Sample' ILIKE ${} OR sr.row_data ->> 'Sample_ID' ILIKE ${} OR sr.row_data ->> 'SampleID' ILIKE ${} OR sr.row_data ->> 'Sample_Name' ILIKE ${})",
                param_count, param_count, param_count, param_count
            ));
            param_count += 1;
        }

        if let Some(project_filter) = &query.project_filter {
            where_conditions.push(format!(
                "(sr.row_data ->> 'Project' ILIKE ${} OR sr.row_data ->> 'Project_ID' ILIKE ${} OR sr.row_data ->> 'ProjectID' ILIKE ${} OR sr.row_data ->> 'Project_Name' ILIKE ${})",
                param_count, param_count, param_count, param_count
            ));
            param_count += 1;
        }

        // Add column filters
        if let Some(filters) = &query.column_filters {
            for (column, value) in filters {
                where_conditions.push(format!(
                    "sr.row_data ->> '{}' ILIKE ${}",
                    column, param_count
                ));
                param_count += 1;
            }
        }

        let where_clause = if where_conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_conditions.join(" AND "))
        };

        // Build the main query
        let records_query = format!(
            r#"
            SELECT sr.id, sr.dataset_id, sr.row_number, sr.row_data, sr.search_text, sr.created_at
            FROM spreadsheet_records sr
            {}
            ORDER BY sr.dataset_id, sr.row_number
            LIMIT ${} OFFSET ${}
            "#,
            where_clause,
            param_count,
            param_count + 1
        );

        // Build the count query
        let count_query = format!(
            r#"
            SELECT COUNT(*) as total
            FROM spreadsheet_records sr
            {}
            "#,
            where_clause
        );

        // Execute count query
        let mut count_query_builder = sqlx::query_scalar::<_, i64>(&count_query);
        param_count = 1;

        // Prepare filter patterns outside the binding logic to handle lifetimes
        let pool_filter_pattern = query.pool_filter.as_ref().map(|f| format!("%{}%", f));
        let sample_filter_pattern = query.sample_filter.as_ref().map(|f| format!("%{}%", f));
        let project_filter_pattern = query.project_filter.as_ref().map(|f| format!("%{}%", f));

        if let Some(dataset_id) = query.dataset_id {
            count_query_builder = count_query_builder.bind(dataset_id);
            param_count += 1;
        }

        if let Some(search_term) = &query.search_term {
            count_query_builder = count_query_builder.bind(search_term);
            param_count += 1;
        }

        if let Some(ref filter_pattern) = pool_filter_pattern {
            count_query_builder = count_query_builder.bind(filter_pattern);
            param_count += 1;
        }

        if let Some(ref filter_pattern) = sample_filter_pattern {
            count_query_builder = count_query_builder.bind(filter_pattern);
            param_count += 1;
        }

        if let Some(ref filter_pattern) = project_filter_pattern {
            count_query_builder = count_query_builder.bind(filter_pattern);
            param_count += 1;
        }

        if let Some(filters) = &query.column_filters {
            for (_, value) in filters {
                count_query_builder = count_query_builder.bind(format!("%{}%", value));
            }
        }

        let total_count = count_query_builder.fetch_one(&self.pool).await?;

        // Execute records query
        let mut records_query_builder = sqlx::query_as::<_, SpreadsheetRecord>(&records_query);
        param_count = 1;

        if let Some(dataset_id) = query.dataset_id {
            records_query_builder = records_query_builder.bind(dataset_id);
            param_count += 1;
        }

        if let Some(search_term) = &query.search_term {
            records_query_builder = records_query_builder.bind(search_term);
            param_count += 1;
        }

        if let Some(ref filter_pattern) = pool_filter_pattern {
            records_query_builder = records_query_builder.bind(filter_pattern);
            param_count += 1;
        }

        if let Some(ref filter_pattern) = sample_filter_pattern {
            records_query_builder = records_query_builder.bind(filter_pattern);
            param_count += 1;
        }

        if let Some(ref filter_pattern) = project_filter_pattern {
            records_query_builder = records_query_builder.bind(filter_pattern);
            param_count += 1;
        }

        if let Some(filters) = &query.column_filters {
            for (_, value) in filters {
                records_query_builder = records_query_builder.bind(format!("%{}%", value));
                param_count += 1;
            }
        }

        records_query_builder = records_query_builder.bind(limit).bind(offset);

        let records = records_query_builder.fetch_all(&self.pool).await?;

        // Get dataset info if we have a specific dataset_id
        let dataset_info = if let Some(dataset_id) = query.dataset_id {
            self.get_dataset(dataset_id).await.ok()
        } else {
            None
        };

        // Get available filters if requested
        let available_filters = if query.dataset_id.is_some() {
            self.get_available_filters(query.dataset_id).await.ok()
        } else {
            None
        };

        Ok(SpreadsheetSearchResult {
            records,
            total_count,
            dataset_info,
            available_filters,
        })
    }

    pub async fn get_dataset(&self, dataset_id: Uuid) -> Result<SpreadsheetDataset, sqlx::Error> {
        sqlx::query_as::<_, SpreadsheetDataset>(
            r#"
            SELECT id, filename, original_filename, file_type, file_size, sheet_name, total_rows, total_columns, column_headers, upload_status, error_message, uploaded_by, created_at, updated_at, metadata
            FROM spreadsheet_datasets
            WHERE id = $1
            "#,
        )
        .bind(dataset_id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn list_datasets(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<SpreadsheetDataset>, sqlx::Error> {
        let limit = limit.unwrap_or(50).min(1000);
        let offset = offset.unwrap_or(0);

        sqlx::query_as::<_, SpreadsheetDataset>(
            r#"
            SELECT id, filename, original_filename, file_type, file_size, sheet_name, total_rows, total_columns, column_headers, upload_status, error_message, uploaded_by, created_at, updated_at, metadata
            FROM spreadsheet_datasets
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn delete_dataset(&self, dataset_id: Uuid) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM spreadsheet_datasets
            WHERE id = $1
            "#,
        )
        .bind(dataset_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    fn generate_search_text(&self, row_data: &serde_json::Value) -> String {
        match row_data.as_object() {
            Some(obj) => obj
                .values()
                .filter_map(|v| v.as_str())
                .collect::<Vec<&str>>()
                .join(" "),
            None => String::new(),
        }
    }

    pub async fn get_available_filters(
        &self,
        dataset_id: Option<Uuid>,
    ) -> Result<AvailableFilters, sqlx::Error> {
        let where_clause = if let Some(id) = dataset_id {
            format!("WHERE dataset_id = '{}'", id)
        } else {
            String::new()
        };

        // Get all unique values for pool, sample, and project detection
        let query = format!(
            r#"
            SELECT DISTINCT
                row_data ->> 'Pool' as pool1,
                row_data ->> 'Pool_ID' as pool2,
                row_data ->> 'PoolID' as pool3,
                row_data ->> 'Sample' as sample1,
                row_data ->> 'Sample_ID' as sample2,
                row_data ->> 'SampleID' as sample3,
                row_data ->> 'Sample_Name' as sample4,
                row_data ->> 'Project' as project1,
                row_data ->> 'Project_ID' as project2,
                row_data ->> 'ProjectID' as project3,
                row_data ->> 'Project_Name' as project4
            FROM spreadsheet_records
            {}
            "#,
            where_clause
        );

        let rows = sqlx::query(&query).fetch_all(&self.pool).await?;

        let mut pools = std::collections::HashSet::new();
        let mut samples = std::collections::HashSet::new();
        let mut projects = std::collections::HashSet::new();

        for row in rows {
            // Collect pool values
            for i in 0..3 {
                if let Ok(value) = row.try_get::<Option<String>, _>(i) {
                    if let Some(v) = value {
                        if !v.is_empty() && v != "null" {
                            pools.insert(v);
                        }
                    }
                }
            }

            // Collect sample values
            for i in 3..7 {
                if let Ok(value) = row.try_get::<Option<String>, _>(i) {
                    if let Some(v) = value {
                        if !v.is_empty() && v != "null" {
                            samples.insert(v);
                        }
                    }
                }
            }

            // Collect project values
            for i in 7..11 {
                if let Ok(value) = row.try_get::<Option<String>, _>(i) {
                    if let Some(v) = value {
                        if !v.is_empty() && v != "null" {
                            projects.insert(v);
                        }
                    }
                }
            }
        }

        // Get all column names from dataset(s)
        let columns_query = if let Some(id) = dataset_id {
            format!(
                "SELECT DISTINCT column_headers FROM spreadsheet_datasets WHERE id = '{}'",
                id
            )
        } else {
            "SELECT DISTINCT column_headers FROM spreadsheet_datasets".to_string()
        };

        let column_rows = sqlx::query(&columns_query).fetch_all(&self.pool).await?;
        let mut all_columns = std::collections::HashSet::new();

        for row in column_rows {
            if let Ok(headers) = row.try_get::<Vec<String>, _>("column_headers") {
                for header in headers {
                    all_columns.insert(header);
                }
            }
        }

        Ok(AvailableFilters {
            pools: pools.into_iter().collect(),
            samples: samples.into_iter().collect(),
            projects: projects.into_iter().collect(),
            all_columns: all_columns.into_iter().collect(),
            column_values: HashMap::new(), // TODO: Implement column value discovery
        })
    }

    pub async fn analyze_dataset(&self, dataset_id: Uuid) -> Result<DatasetAnalysis, sqlx::Error> {
        // Get the dataset info
        let dataset = self.get_dataset(dataset_id).await?;

        // Analyze each column
        let mut column_analysis = Vec::new();
        for column in &dataset.column_headers {
            let analysis = self.analyze_column(dataset_id, column).await?;
            column_analysis.push(analysis);
        }

        // Get available filters for this dataset
        let filters = self.get_available_filters(Some(dataset_id)).await?;

        Ok(DatasetAnalysis {
            dataset_id,
            column_analysis,
            detected_pools: filters.pools,
            detected_samples: filters.samples,
            detected_projects: filters.projects,
            total_unique_values: filters.all_columns.len(),
        })
    }

    pub async fn analyze_column(
        &self,
        dataset_id: Uuid,
        column_name: &str,
    ) -> Result<ColumnAnalysis, sqlx::Error> {
        // Get unique values for this column
        let query = format!(
            r#"
            SELECT 
                row_data ->> '{}' as value,
                COUNT(*) as count
            FROM spreadsheet_records 
            WHERE dataset_id = $1 
            AND row_data ->> '{}' IS NOT NULL 
            AND row_data ->> '{}' != ''
            GROUP BY row_data ->> '{}'
            ORDER BY count DESC
            LIMIT 100
            "#,
            column_name, column_name, column_name, column_name
        );

        let rows = sqlx::query(&query)
            .bind(dataset_id)
            .fetch_all(&self.pool)
            .await?;

        let mut unique_values = Vec::new();
        let mut total_count = 0;

        for row in rows {
            if let Ok(value) = row.try_get::<Option<String>, _>("value") {
                if let Some(v) = value {
                    if !v.is_empty() {
                        unique_values.push(v);
                    }
                }
            }
            if let Ok(count) = row.try_get::<i64, _>("count") {
                total_count += count;
            }
        }

        // Calculate data type and filter type before moving unique_values
        let data_type = self.detect_data_type(&unique_values);
        let suggested_filter_type = if unique_values.len() <= 10 {
            FilterType::Dropdown
        } else if self.is_numeric_column(&unique_values) {
            FilterType::Numeric
        } else if self.is_date_column(&unique_values) {
            FilterType::Date
        } else {
            FilterType::Text
        };

        let limited_values: Vec<String> = unique_values.into_iter().take(50).collect(); // Limit to 50 for UI

        Ok(ColumnAnalysis {
            column_name: column_name.to_string(),
            unique_values: limited_values,
            value_count: total_count as usize,
            data_type,
            suggested_filter_type,
        })
    }

    fn is_numeric_column(&self, values: &[String]) -> bool {
        if values.is_empty() {
            return false;
        }

        let numeric_count = values.iter().filter(|v| v.parse::<f64>().is_ok()).count();

        numeric_count as f64 / values.len() as f64 > 0.8 // 80% numeric
    }

    fn is_date_column(&self, values: &[String]) -> bool {
        if values.is_empty() {
            return false;
        }

        let date_patterns = [
            "%Y-%m-%d",
            "%m/%d/%Y",
            "%d/%m/%Y",
            "%Y/%m/%d",
            "%Y-%m-%d %H:%M:%S",
            "%m/%d/%Y %H:%M:%S",
        ];

        let date_count = values
            .iter()
            .filter(|v| {
                date_patterns.iter().any(|pattern| {
                    chrono::NaiveDate::parse_from_str(v, pattern).is_ok()
                        || chrono::NaiveDateTime::parse_from_str(v, pattern).is_ok()
                })
            })
            .count();

        date_count as f64 / values.len() as f64 > 0.8 // 80% dates
    }

    fn detect_data_type(&self, values: &[String]) -> String {
        if self.is_numeric_column(values) {
            "numeric".to_string()
        } else if self.is_date_column(values) {
            "date".to_string()
        } else {
            "text".to_string()
        }
    }
}
