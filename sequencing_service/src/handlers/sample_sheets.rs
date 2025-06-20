use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde_json::json;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::{
    error::{Result, SequencingError},
    models::*,
    AppState,
};

/// Create a new sample sheet
pub async fn create_sample_sheet(
    State(state): State<AppState>,
    Json(request): Json<CreateSampleSheetRequest>,
) -> Result<Json<serde_json::Value>> {
    // Validate the sample sheet format
    validate_sample_sheet_format(&request.samples)?;

    let sheet_id = Uuid::new_v4();
    
    let sample_sheet = sqlx::query_as::<_, SampleSheet>(
        r#"
        INSERT INTO sample_sheets (
            id, name, platform, run_parameters, samples_data,
            created_at, created_by, status
        ) VALUES ($1, $2, $3, $4, $5, NOW(), $6, $7)
        RETURNING *
        "#
    )
    .bind(sheet_id)
    .bind(&request.name)
    .bind(&request.platform)
    .bind(&request.run_parameters)
    .bind(&request.samples)
    .bind(request.created_by.as_deref())
    .bind(SampleSheetStatus::Draft)
    .fetch_one(&state.db_pool.pool)
    .await?;

    // Create individual sample entries for easier querying
    for (index, sample) in request.samples.as_array().unwrap_or(&vec![]).iter().enumerate() {
        sqlx::query(
            r#"
            INSERT INTO sample_sheet_entries (
                id, sample_sheet_id, sample_id, well_position, index_sequence,
                sample_name, sample_plate, sample_well, i7_index_id, i5_index_id,
                sample_project, description, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NOW())
            "#
        )
        .bind(Uuid::new_v4())
        .bind(sheet_id)
        .bind(sample.get("Sample_ID").and_then(|v| v.as_str()).map(|s| s.parse::<Uuid>().ok()).flatten())
        .bind(sample.get("Well").and_then(|v| v.as_str()))
        .bind(sample.get("Index").and_then(|v| v.as_str()))
        .bind(sample.get("Sample_Name").and_then(|v| v.as_str()))
        .bind(sample.get("Sample_Plate").and_then(|v| v.as_str()))
        .bind(sample.get("Sample_Well").and_then(|v| v.as_str()))
        .bind(sample.get("I7_Index_ID").and_then(|v| v.as_str()))
        .bind(sample.get("I5_Index_ID").and_then(|v| v.as_str()))
        .bind(sample.get("Sample_Project").and_then(|v| v.as_str()))
        .bind(sample.get("Description").and_then(|v| v.as_str()))
        .execute(&state.db_pool.pool)
        .await?;
    }

    Ok(Json(json!({
        "success": true,
        "data": sample_sheet,
        "message": "Sample sheet created successfully"
    })))
}

/// Get sample sheet by ID
pub async fn get_sample_sheet(
    State(state): State<AppState>,
    Path(sheet_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let sample_sheet = sqlx::query_as::<_, SampleSheet>(
        "SELECT * FROM sample_sheets WHERE id = $1"
    )
    .bind(sheet_id)
    .fetch_optional(&state.db_pool.pool)
    .await?
    .ok_or(SequencingError::SampleSheetNotFound { sheet_id })?;

    // Get associated sample entries
    let entries = sqlx::query_as::<_, SampleSheetEntry>(
        "SELECT * FROM sample_sheet_entries WHERE sample_sheet_id = $1 ORDER BY well_position"
    )
    .bind(sheet_id)
    .fetch_all(&state.db_pool.pool)
    .await?;

    // Get validation results if any
    let validation_results = sqlx::query_as::<_, SampleSheetValidation>(
        "SELECT * FROM sample_sheet_validations WHERE sample_sheet_id = $1 ORDER BY validated_at DESC LIMIT 1"
    )
    .bind(sheet_id)
    .fetch_optional(&state.db_pool.pool)
    .await?;

    Ok(Json(json!({
        "success": true,
        "data": {
            "sample_sheet": sample_sheet,
            "entries": entries,
            "validation": validation_results,
            "entry_count": entries.len()
        }
    })))
}

/// List sample sheets with filtering
pub async fn list_sample_sheets(
    State(state): State<AppState>,
    Query(query): Query<SampleSheetListQuery>,
) -> Result<Json<serde_json::Value>> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).min(100);
    let offset = (page - 1) * page_size;

    let mut where_conditions = Vec::new();
    let mut param_count = 0;

    if let Some(platform) = &query.platform {
        param_count += 1;
        where_conditions.push(format!("platform = ${}", param_count));
    }

    if let Some(status) = &query.status {
        param_count += 1;
        where_conditions.push(format!("status = ${}", param_count));
    }

    if let Some(created_by) = &query.created_by {
        param_count += 1;
        where_conditions.push(format!("created_by = ${}", param_count));
    }

    let where_clause = if where_conditions.is_empty() {
        "".to_string()
    } else {
        format!("WHERE {}", where_conditions.join(" AND "))
    };

    // Get total count
    let total_count: i64 = sqlx::query_scalar(&format!(
        "SELECT COUNT(*) FROM sample_sheets {}", where_clause
    ))
    .fetch_one(&state.db_pool.pool)
    .await?;

    // Get sample sheets
    let sample_sheets = sqlx::query_as::<_, SampleSheet>(&format!(
        "SELECT * FROM sample_sheets {} ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
        where_clause,
        param_count + 1,
        param_count + 2
    ))
    .bind(page_size)
    .bind(offset)
    .fetch_all(&state.db_pool.pool)
    .await?;

    let total_pages = (total_count + page_size - 1) / page_size;

    Ok(Json(json!({
        "success": true,
        "data": {
            "sample_sheets": sample_sheets,
            "pagination": {
                "page": page,
                "page_size": page_size,
                "total_count": total_count,
                "total_pages": total_pages
            }
        }
    })))
}

/// Validate sample sheet
pub async fn validate_sample_sheet(
    State(state): State<AppState>,
    Path(sheet_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let sample_sheet = sqlx::query_as::<_, SampleSheet>(
        "SELECT * FROM sample_sheets WHERE id = $1"
    )
    .bind(sheet_id)
    .fetch_optional(&state.db_pool.pool)
    .await?
    .ok_or(SequencingError::SampleSheetNotFound { sheet_id })?;

    // Perform comprehensive validation
    let validation_result = perform_sample_sheet_validation(&state, &sample_sheet).await?;

    // Save validation results
    let validation_id = Uuid::new_v4();
    let validation_record = sqlx::query_as::<_, SampleSheetValidation>(
        r#"
        INSERT INTO sample_sheet_validations (
            id, sample_sheet_id, is_valid, errors, warnings,
            validated_at, validation_rules_version
        ) VALUES ($1, $2, $3, $4, $5, NOW(), $6)
        RETURNING *
        "#
    )
    .bind(validation_id)
    .bind(sheet_id)
    .bind(validation_result.is_valid)
    .bind(&validation_result.errors)
    .bind(&validation_result.warnings)
    .bind("v1.0")
    .fetch_one(&state.db_pool.pool)
    .await?;

    // Update sample sheet status based on validation
    let new_status = if validation_result.is_valid {
        SampleSheetStatus::Validated
    } else {
        SampleSheetStatus::Invalid
    };

    sqlx::query(
        "UPDATE sample_sheets SET status = $2, updated_at = NOW() WHERE id = $1"
    )
    .bind(sheet_id)
    .bind(&new_status)
    .execute(&state.db_pool.pool)
    .await?;

    Ok(Json(json!({
        "success": true,
        "data": {
            "validation": validation_record,
            "summary": {
                "is_valid": validation_result.is_valid,
                "error_count": validation_result.errors.as_array().unwrap_or(&vec![]).len(),
                "warning_count": validation_result.warnings.as_array().unwrap_or(&vec![]).len(),
                "new_status": new_status
            }
        },
        "message": if validation_result.is_valid {
            "Sample sheet is valid"
        } else {
            "Sample sheet validation failed"
        }
    })))
}

/// Generate sample sheet from job
pub async fn generate_from_job(
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
    Json(request): Json<GenerateSheetRequest>,
) -> Result<Json<serde_json::Value>> {
    // Get job details
    let job = sqlx::query_as::<_, SequencingJob>(
        "SELECT * FROM sequencing_jobs WHERE id = $1"
    )
    .bind(job_id)
    .fetch_optional(&state.db_pool.pool)
    .await?
    .ok_or(SequencingError::JobNotFound { job_id })?;

    // Get samples associated with the job (this would need to be implemented based on your sample-job relationship)
    let samples = get_job_samples(&state, job_id).await?;

    if samples.is_empty() {
        return Err(SequencingError::Validation {
            message: "No samples found for this job".to_string(),
        });
    }

    // Generate sample sheet data based on platform
    let sheet_data = generate_platform_specific_sheet(&job.platform, &samples, &request.template_options)?;

    // Create the sample sheet
    let sheet_id = Uuid::new_v4();
    let sample_sheet = sqlx::query_as::<_, SampleSheet>(
        r#"
        INSERT INTO sample_sheets (
            id, name, platform, run_parameters, samples_data,
            job_id, created_at, created_by, status
        ) VALUES ($1, $2, $3, $4, $5, $6, NOW(), $7, $8)
        RETURNING *
        "#
    )
    .bind(sheet_id)
    .bind(format!("SampleSheet_{}", job.job_name))
    .bind(&job.platform)
    .bind(&sheet_data.run_parameters)
    .bind(&sheet_data.samples)
    .bind(job_id)
    .bind(request.created_by.as_deref())
    .bind(SampleSheetStatus::Generated)
    .fetch_one(&state.db_pool.pool)
    .await?;

    Ok(Json(json!({
        "success": true,
        "data": {
            "sample_sheet": sample_sheet,
            "generated_from_job": job_id,
            "sample_count": samples.len()
        },
        "message": "Sample sheet generated successfully from job"
    })))
}

/// Update sample sheet
pub async fn update_sample_sheet(
    State(state): State<AppState>,
    Path(sheet_id): Path<Uuid>,
    Json(request): Json<UpdateSampleSheetRequest>,
) -> Result<Json<serde_json::Value>> {
    // Check if sample sheet exists
    let existing_sheet = sqlx::query("SELECT id FROM sample_sheets WHERE id = $1")
        .bind(sheet_id)
        .fetch_optional(&state.db_pool.pool)
        .await?
        .ok_or(SequencingError::SampleSheetNotFound { sheet_id })?;

    let sample_sheet = sqlx::query_as::<_, SampleSheet>(
        r#"
        UPDATE sample_sheets 
        SET 
            name = COALESCE($2, name),
            run_parameters = COALESCE($3, run_parameters),
            samples_data = COALESCE($4, samples_data),
            status = COALESCE($5, status),
            updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#
    )
    .bind(sheet_id)
    .bind(request.name.as_deref())
    .bind(request.run_parameters.as_ref())
    .bind(request.samples.as_ref())
    .bind(request.status.as_ref())
    .fetch_one(&state.db_pool.pool)
    .await?;

    // If samples were updated, update the entries
    if let Some(samples) = &request.samples {
        // Delete existing entries
        sqlx::query("DELETE FROM sample_sheet_entries WHERE sample_sheet_id = $1")
            .bind(sheet_id)
            .execute(&state.db_pool.pool)
            .await?;

        // Create new entries
        for sample in samples.as_array().unwrap_or(&vec![]) {
            sqlx::query(
                r#"
                INSERT INTO sample_sheet_entries (
                    id, sample_sheet_id, sample_id, well_position, index_sequence,
                    sample_name, sample_plate, sample_well, i7_index_id, i5_index_id,
                    sample_project, description, created_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NOW())
                "#
            )
            .bind(Uuid::new_v4())
            .bind(sheet_id)
            .bind(sample.get("Sample_ID").and_then(|v| v.as_str()).map(|s| s.parse::<Uuid>().ok()).flatten())
            .bind(sample.get("Well").and_then(|v| v.as_str()))
            .bind(sample.get("Index").and_then(|v| v.as_str()))
            .bind(sample.get("Sample_Name").and_then(|v| v.as_str()))
            .bind(sample.get("Sample_Plate").and_then(|v| v.as_str()))
            .bind(sample.get("Sample_Well").and_then(|v| v.as_str()))
            .bind(sample.get("I7_Index_ID").and_then(|v| v.as_str()))
            .bind(sample.get("I5_Index_ID").and_then(|v| v.as_str()))
            .bind(sample.get("Sample_Project").and_then(|v| v.as_str()))
            .bind(sample.get("Description").and_then(|v| v.as_str()))
            .execute(&state.db_pool.pool)
            .await?;
        }
    }

    Ok(Json(json!({
        "success": true,
        "data": sample_sheet,
        "message": "Sample sheet updated successfully"
    })))
}

/// Delete sample sheet
pub async fn delete_sample_sheet(
    State(state): State<AppState>,
    Path(sheet_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    // Check if sample sheet is being used in any active jobs
    let active_usage: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM sequencing_jobs 
        WHERE sample_sheet_id = $1 AND status IN ('running', 'queued', 'validated')
        "#
    )
    .bind(sheet_id)
    .fetch_one(&state.db_pool.pool)
    .await?;

    if active_usage > 0 {
        return Err(SequencingError::SampleSheetInUse { 
            sheet_id,
            active_jobs: active_usage as u32,
        });
    }

    let deleted_sheet = sqlx::query_as::<_, SampleSheet>(
        "DELETE FROM sample_sheets WHERE id = $1 RETURNING *"
    )
    .bind(sheet_id)
    .fetch_optional(&state.db_pool.pool)
    .await?
    .ok_or(SequencingError::SampleSheetNotFound { sheet_id })?;

    Ok(Json(json!({
        "success": true,
        "data": deleted_sheet,
        "message": "Sample sheet deleted successfully"
    })))
}

/// Export sample sheet in CSV format
pub async fn export_sample_sheet(
    State(state): State<AppState>,
    Path(sheet_id): Path<Uuid>,
    Query(query): Query<ExportQuery>,
) -> Result<Json<serde_json::Value>> {
    let sample_sheet = sqlx::query_as::<_, SampleSheet>(
        "SELECT * FROM sample_sheets WHERE id = $1"
    )
    .bind(sheet_id)
    .fetch_optional(&state.db_pool.pool)
    .await?
    .ok_or(SequencingError::SampleSheetNotFound { sheet_id })?;

    let format = query.format.as_deref().unwrap_or("csv");
    
    let exported_content = match format {
        "csv" => export_to_csv(&sample_sheet)?,
        "illumina" => export_to_illumina_format(&sample_sheet)?,
        "json" => serde_json::to_string_pretty(&sample_sheet.samples_data)
            .map_err(|_| SequencingError::ExportError {
                message: "Failed to serialize to JSON".to_string(),
            })?,
        _ => return Err(SequencingError::Validation {
            message: "Unsupported export format".to_string(),
        }),
    };

    Ok(Json(json!({
        "success": true,
        "data": {
            "content": exported_content,
            "format": format,
            "filename": format!("{}_{}.{}", sample_sheet.name, sheet_id, format)
        }
    })))
}

/// Helper functions
async fn get_job_samples(state: &AppState, job_id: Uuid) -> Result<Vec<serde_json::Value>> {
    // This is a simplified implementation - you'd need to implement based on your actual data model
    let samples = sqlx::query_as::<_, (Uuid, String, String)>(
        r#"
        SELECT s.id, s.name, s.barcode
        FROM samples s
        JOIN job_samples js ON s.id = js.sample_id
        WHERE js.job_id = $1
        "#
    )
    .bind(job_id)
    .fetch_all(&state.db_pool.pool)
    .await
    .unwrap_or_default();

    Ok(samples.into_iter().map(|(id, name, barcode)| {
        json!({
            "Sample_ID": id,
            "Sample_Name": name,
            "Sample_Plate": "Plate1",
            "Sample_Well": "A01",
            "I7_Index_ID": "N701",
            "index": "TAAGGCGA",
            "Sample_Project": "Project1",
            "Description": format!("Sample {}", name)
        })
    }).collect())
}

fn generate_platform_specific_sheet(
    platform: &str,
    samples: &[serde_json::Value],
    template_options: &serde_json::Value,
) -> Result<GeneratedSheetData> {
    let run_parameters = match platform {
        "illumina_novaseq" => json!({
            "Application": "NextSeq FASTQ Only",
            "Chemistry": "Version3",
            "Read1": 151,
            "Read2": 151,
            "Index1": 8,
            "Index2": 8
        }),
        "illumina_miseq" => json!({
            "Application": "FASTQ Only",
            "Chemistry": "V3",
            "Read1": 301,
            "Read2": 301,
            "Index1": 8,
            "Index2": 8
        }),
        _ => json!({
            "Application": "FASTQ Only",
            "Read1": 151,
            "Read2": 151
        }),
    };

    Ok(GeneratedSheetData {
        run_parameters,
        samples: json!(samples),
    })
}

async fn perform_sample_sheet_validation(
    state: &AppState,
    sample_sheet: &SampleSheet,
) -> Result<ValidationResult> {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Basic format validation
    if let Err(e) = validate_sample_sheet_format(&sample_sheet.samples_data) {
        errors.push(format!("Format validation failed: {}", e));
    }

    // Platform-specific validation
    validate_platform_compatibility(&sample_sheet.platform, &sample_sheet.samples_data, &mut errors, &mut warnings);

    // Index validation
    validate_index_sequences(&sample_sheet.samples_data, &mut errors, &mut warnings);

    // Sample name validation
    validate_sample_names(&sample_sheet.samples_data, &mut errors, &mut warnings);

    Ok(ValidationResult {
        is_valid: errors.is_empty(),
        errors: json!(errors),
        warnings: json!(warnings),
    })
}

fn validate_sample_sheet_format(samples_data: &serde_json::Value) -> Result<()> {
    if !samples_data.is_array() {
        return Err(SequencingError::Validation {
            message: "Samples data must be an array".to_string(),
        });
    }

    let samples = samples_data.as_array().unwrap();
    for (index, sample) in samples.iter().enumerate() {
        if !sample.is_object() {
            return Err(SequencingError::Validation {
                message: format!("Sample at index {} must be an object", index),
            });
        }

        // Check required fields
        let required_fields = ["Sample_Name", "Sample_ID"];
        for field in &required_fields {
            if sample.get(field).is_none() {
                return Err(SequencingError::Validation {
                    message: format!("Sample at index {} missing required field: {}", index, field),
                });
            }
        }
    }

    Ok(())
}

fn validate_platform_compatibility(
    platform: &str,
    samples_data: &serde_json::Value,
    errors: &mut Vec<String>,
    warnings: &mut Vec<String>,
) {
    // Platform-specific validation logic
    match platform {
        "illumina_novaseq" | "illumina_miseq" => {
            // Check for Illumina-specific fields
            if let Some(samples) = samples_data.as_array() {
                for sample in samples {
                    if sample.get("I7_Index_ID").is_none() {
                        warnings.push("Missing I7_Index_ID for Illumina platform".to_string());
                    }
                }
            }
        }
        _ => {
            warnings.push(format!("Unknown platform: {}", platform));
        }
    }
}

fn validate_index_sequences(
    samples_data: &serde_json::Value,
    errors: &mut Vec<String>,
    warnings: &mut Vec<String>,
) {
    if let Some(samples) = samples_data.as_array() {
        let mut seen_indices: HashMap<String, usize> = HashMap::new();
        
        for (index, sample) in samples.iter().enumerate() {
            if let Some(index_seq) = sample.get("index").and_then(|v| v.as_str()) {
                if let Some(&prev_index) = seen_indices.get(index_seq) {
                    errors.push(format!(
                        "Duplicate index sequence '{}' found at positions {} and {}",
                        index_seq, prev_index, index
                    ));
                } else {
                    seen_indices.insert(index_seq.to_string(), index);
                }

                // Validate index sequence format (DNA bases only)
                if !index_seq.chars().all(|c| matches!(c, 'A' | 'T' | 'G' | 'C' | 'N')) {
                    errors.push(format!(
                        "Invalid index sequence '{}' at position {} - must contain only A, T, G, C, N",
                        index_seq, index
                    ));
                }
            }
        }
    }
}

fn validate_sample_names(
    samples_data: &serde_json::Value,
    errors: &mut Vec<String>,
    warnings: &mut Vec<String>,
) {
    if let Some(samples) = samples_data.as_array() {
        let mut seen_names: HashMap<String, usize> = HashMap::new();
        
        for (index, sample) in samples.iter().enumerate() {
            if let Some(sample_name) = sample.get("Sample_Name").and_then(|v| v.as_str()) {
                if let Some(&prev_index) = seen_names.get(sample_name) {
                    warnings.push(format!(
                        "Duplicate sample name '{}' found at positions {} and {}",
                        sample_name, prev_index, index
                    ));
                } else {
                    seen_names.insert(sample_name.to_string(), index);
                }

                // Check for special characters that might cause issues
                if sample_name.contains(' ') {
                    warnings.push(format!(
                        "Sample name '{}' contains spaces - consider using underscores",
                        sample_name
                    ));
                }
            }
        }
    }
}

fn export_to_csv(sample_sheet: &SampleSheet) -> Result<String> {
    // Simplified CSV export - in a real implementation, you'd want more sophisticated formatting
    let mut csv_content = String::new();
    
    // Add header
    csv_content.push_str("Sample_Name,Sample_ID,Sample_Plate,Sample_Well,I7_Index_ID,index,Sample_Project,Description\n");
    
    if let Some(samples) = sample_sheet.samples_data.as_array() {
        for sample in samples {
            let row = format!(
                "{},{},{},{},{},{},{},{}\n",
                sample.get("Sample_Name").and_then(|v| v.as_str()).unwrap_or(""),
                sample.get("Sample_ID").and_then(|v| v.as_str()).unwrap_or(""),
                sample.get("Sample_Plate").and_then(|v| v.as_str()).unwrap_or(""),
                sample.get("Sample_Well").and_then(|v| v.as_str()).unwrap_or(""),
                sample.get("I7_Index_ID").and_then(|v| v.as_str()).unwrap_or(""),
                sample.get("index").and_then(|v| v.as_str()).unwrap_or(""),
                sample.get("Sample_Project").and_then(|v| v.as_str()).unwrap_or(""),
                sample.get("Description").and_then(|v| v.as_str()).unwrap_or("")
            );
            csv_content.push_str(&row);
        }
    }
    
    Ok(csv_content)
}

fn export_to_illumina_format(sample_sheet: &SampleSheet) -> Result<String> {
    // Illumina-specific format with headers and sections
    let mut content = String::new();
    
    content.push_str("[Header]\n");
    content.push_str(&format!("Date,{}\n", Utc::now().format("%Y-%m-%d")));
    content.push_str(&format!("Workflow,GenerateFASTQ\n"));
    content.push_str(&format!("Application,NextSeq FASTQ Only\n"));
    content.push_str("\n");
    
    content.push_str("[Reads]\n");
    content.push_str("151\n");
    content.push_str("151\n");
    content.push_str("\n");
    
    content.push_str("[Data]\n");
    content.push_str("Sample_ID,Sample_Name,Sample_Plate,Sample_Well,I7_Index_ID,index,Sample_Project,Description\n");
    
    if let Some(samples) = sample_sheet.samples_data.as_array() {
        for sample in samples {
            let row = format!(
                "{},{},{},{},{},{},{},{}\n",
                sample.get("Sample_ID").and_then(|v| v.as_str()).unwrap_or(""),
                sample.get("Sample_Name").and_then(|v| v.as_str()).unwrap_or(""),
                sample.get("Sample_Plate").and_then(|v| v.as_str()).unwrap_or(""),
                sample.get("Sample_Well").and_then(|v| v.as_str()).unwrap_or(""),
                sample.get("I7_Index_ID").and_then(|v| v.as_str()).unwrap_or(""),
                sample.get("index").and_then(|v| v.as_str()).unwrap_or(""),
                sample.get("Sample_Project").and_then(|v| v.as_str()).unwrap_or(""),
                sample.get("Description").and_then(|v| v.as_str()).unwrap_or("")
            );
            content.push_str(&row);
        }
    }
    
    Ok(content)
}

/// Request and response structures
#[derive(serde::Deserialize)]
pub struct CreateSampleSheetRequest {
    pub name: String,
    pub platform: String,
    pub run_parameters: serde_json::Value,
    pub samples: serde_json::Value,
    pub created_by: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct UpdateSampleSheetRequest {
    pub name: Option<String>,
    pub run_parameters: Option<serde_json::Value>,
    pub samples: Option<serde_json::Value>,
    pub status: Option<SampleSheetStatus>,
}

#[derive(serde::Deserialize)]
pub struct GenerateSheetRequest {
    pub template_options: serde_json::Value,
    pub created_by: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct SampleSheetListQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub platform: Option<String>,
    pub status: Option<SampleSheetStatus>,
    pub created_by: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct ExportQuery {
    pub format: Option<String>,
}

struct GeneratedSheetData {
    pub run_parameters: serde_json::Value,
    pub samples: serde_json::Value,
}

struct ValidationResult {
    pub is_valid: bool,
    pub errors: serde_json::Value,
    pub warnings: serde_json::Value,
}
