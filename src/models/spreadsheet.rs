use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
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
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpreadsheetSearchResult {
    pub records: Vec<SpreadsheetRecord>,
    pub total_count: i64,
    pub dataset_info: Option<SpreadsheetDataset>,
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

        if let Some(dataset_id) = query.dataset_id {
            count_query_builder = count_query_builder.bind(dataset_id);
            param_count += 1;
        }

        if let Some(search_term) = &query.search_term {
            count_query_builder = count_query_builder.bind(search_term);
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

        Ok(SpreadsheetSearchResult {
            records,
            total_count,
            dataset_info,
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
}
