use crate::AppComponents;
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::{Column, Row, TypeInfo};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct ReportQuery {
    pub sql: String,
    pub export_format: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ReportResult {
    pub columns: Vec<String>,
    pub rows: Vec<HashMap<String, serde_json::Value>>,
    pub row_count: usize,
    pub execution_time_ms: u64,
    pub query: String,
}

#[derive(Debug, Serialize)]
pub struct DatabaseSchema {
    pub tables: Vec<TableInfo>,
}

#[derive(Debug, Serialize)]
pub struct TableInfo {
    pub name: String,
    pub columns: Vec<ColumnInfo>,
}

#[derive(Debug, Serialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub is_primary_key: bool,
}

#[derive(Debug, Serialize)]
pub struct ReportTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub sql: String,
    pub category: String,
}

/// Get database schema information
pub async fn get_schema(
    State(state): State<AppComponents>,
) -> Result<Json<DatabaseSchema>, (StatusCode, String)> {
    let tables_query = r#"
        SELECT 
            t.table_name,
            c.column_name,
            c.data_type,
            c.is_nullable,
            COALESCE(
                CASE WHEN tc.constraint_type = 'PRIMARY KEY' THEN true ELSE false END,
                false
            ) as is_primary_key
        FROM information_schema.tables t
        JOIN information_schema.columns c ON t.table_name = c.table_name
        LEFT JOIN information_schema.key_column_usage kcu ON c.column_name = kcu.column_name 
                                                           AND c.table_name = kcu.table_name
        LEFT JOIN information_schema.table_constraints tc ON kcu.constraint_name = tc.constraint_name
                                                           AND tc.constraint_type = 'PRIMARY KEY'
        WHERE t.table_schema = 'public' 
        AND t.table_type = 'BASE TABLE'
        ORDER BY t.table_name, c.ordinal_position
    "#;

    let rows = sqlx::query(tables_query)
        .fetch_all(&state.database.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut tables_map: HashMap<String, Vec<ColumnInfo>> = HashMap::new();

    for row in rows {
        let table_name: String = row.get("table_name");
        let column_name: String = row.get("column_name");
        let data_type: String = row.get("data_type");
        let is_nullable: String = row.get("is_nullable");
        let is_primary_key: bool = row.get("is_primary_key");

        let column_info = ColumnInfo {
            name: column_name,
            data_type,
            is_nullable: is_nullable == "YES",
            is_primary_key,
        };

        tables_map
            .entry(table_name)
            .or_insert_with(Vec::new)
            .push(column_info);
    }

    let tables = tables_map
        .into_iter()
        .map(|(name, columns)| TableInfo { name, columns })
        .collect();

    Ok(Json(DatabaseSchema { tables }))
}

/// Execute a SQL query and return results
pub async fn execute_report(
    State(state): State<AppComponents>,
    Json(query_request): Json<ReportQuery>,
) -> Result<Json<ReportResult>, (StatusCode, String)> {
    // Security: Validate the SQL query
    if !is_safe_query(&query_request.sql) {
        return Err((
            StatusCode::BAD_REQUEST,
            "Only SELECT queries are allowed for reports".to_string(),
        ));
    }

    let start_time = std::time::Instant::now();

    // Execute the query
    let rows = sqlx::query(&query_request.sql)
        .fetch_all(&state.database.pool)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("SQL Error: {}", e)))?;

    let execution_time = start_time.elapsed().as_millis() as u64;

    if rows.is_empty() {
        return Ok(Json(ReportResult {
            columns: vec![],
            rows: vec![],
            row_count: 0,
            execution_time_ms: execution_time,
            query: query_request.sql,
        }));
    }

    // Get column names from the first row
    let columns: Vec<String> = rows[0]
        .columns()
        .iter()
        .map(|col| col.name().to_string())
        .collect();

    // Convert rows to JSON
    let mut result_rows = Vec::new();
    for row in &rows {
        let mut row_map = HashMap::new();
        for (i, column) in row.columns().iter().enumerate() {
            let value = convert_sql_value_to_json(&row, i, column.type_info());
            row_map.insert(column.name().to_string(), value);
        }
        result_rows.push(row_map);
    }

    Ok(Json(ReportResult {
        columns,
        rows: result_rows,
        row_count: rows.len(),
        execution_time_ms: execution_time,
        query: query_request.sql,
    }))
}

/// Get predefined report templates
pub async fn get_report_templates(
    State(_state): State<AppComponents>,
) -> Result<Json<Vec<ReportTemplate>>, (StatusCode, String)> {
    let templates = vec![
        ReportTemplate {
            id: "samples_by_status".to_string(),
            name: "Samples by Status".to_string(),
            description: "Count of samples grouped by status".to_string(),
            sql: "SELECT status, COUNT(*) as count FROM samples GROUP BY status ORDER BY count DESC".to_string(),
            category: "Samples".to_string(),
        },
        ReportTemplate {
            id: "recent_samples".to_string(),
            name: "Recent Samples".to_string(),
            description: "Samples created in the last 30 days".to_string(),
            sql: "SELECT name, barcode, location, status, created_at FROM samples WHERE created_at >= NOW() - INTERVAL '30 days' ORDER BY created_at DESC".to_string(),
            category: "Samples".to_string(),
        },
        ReportTemplate {
            id: "templates_usage".to_string(),
            name: "Template Usage".to_string(),
            description: "Number of samples per template".to_string(),
            sql: "SELECT t.name as template_name, COUNT(CASE WHEN s.metadata->>'template_name' = t.name THEN 1 END) as sample_count, t.created_at as template_created FROM templates t LEFT JOIN samples s ON s.metadata->>'template_name' = t.name GROUP BY t.id, t.name, t.created_at ORDER BY sample_count DESC".to_string(),
            category: "Templates".to_string(),
        },
        ReportTemplate {
            id: "sample_locations".to_string(),
            name: "Sample Storage Locations".to_string(),
            description: "Samples grouped by storage location".to_string(),
            sql: "SELECT location, COUNT(*) as sample_count, status FROM samples GROUP BY location, status ORDER BY location, status".to_string(),
            category: "Storage".to_string(),
        },
    ];

    Ok(Json(templates))
}

/// Security function to validate SQL queries
fn is_safe_query(sql: &str) -> bool {
    let sql_lower = sql.to_lowercase();

    // Must start with SELECT
    if !sql_lower.trim().starts_with("select") {
        return false;
    }

    // Forbidden keywords that could modify data
    let forbidden_keywords = [
        "insert", "update", "delete", "drop", "create", "alter", "truncate", "grant", "revoke",
        "exec", "execute", "call", "merge", "replace", "copy", "bulk", "load", "import",
    ];

    for keyword in &forbidden_keywords {
        if sql_lower.contains(keyword) {
            return false;
        }
    }

    // Additional security checks
    if sql_lower.contains("--") || sql_lower.contains("/*") || sql_lower.contains("*/") {
        return false; // No comments allowed
    }

    if sql_lower.contains(";") && sql_lower.matches(';').count() > 1 {
        return false; // No multiple statements
    }

    true
}

/// Convert SQL values to JSON values
fn convert_sql_value_to_json(
    row: &sqlx::postgres::PgRow,
    index: usize,
    type_info: &sqlx::postgres::PgTypeInfo,
) -> serde_json::Value {
    use sqlx::ValueRef;

    let value_ref = row.try_get_raw(index).unwrap();

    if value_ref.is_null() {
        return serde_json::Value::Null;
    }

    match type_info.name() {
        "BOOL" => serde_json::Value::Bool(row.try_get::<bool, _>(index).unwrap_or(false)),
        "INT2" | "INT4" => serde_json::Value::Number(serde_json::Number::from(
            row.try_get::<i32, _>(index).unwrap_or(0),
        )),
        "INT8" => serde_json::Value::Number(serde_json::Number::from(
            row.try_get::<i64, _>(index).unwrap_or(0),
        )),
        "TIMESTAMPTZ" | "TIMESTAMP" => {
            if let Ok(val) = row.try_get::<chrono::DateTime<chrono::Utc>, _>(index) {
                serde_json::Value::String(val.to_rfc3339())
            } else {
                serde_json::Value::Null
            }
        }
        "UUID" => {
            if let Ok(val) = row.try_get::<uuid::Uuid, _>(index) {
                serde_json::Value::String(val.to_string())
            } else {
                serde_json::Value::Null
            }
        }
        "JSONB" | "JSON" => row
            .try_get::<serde_json::Value, _>(index)
            .unwrap_or(serde_json::Value::Null),
        _ => {
            // Default to string for everything else
            if let Ok(val) = row.try_get::<String, _>(index) {
                serde_json::Value::String(val)
            } else {
                serde_json::Value::Null
            }
        }
    }
}
