use sqlx::PgPool;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;

use uuid::Uuid;

use crate::models::*;

// Request/Response structs







#[derive(Debug, Serialize, Deserialize)]
pub struct LibraryPrepWithProtocol {
    #[serde(flatten)]
    pub preparation: LibraryPreparation,
    pub protocol: Option<LibraryPrepProtocol>,
}

// Manager struct for library prep database operations
#[derive(Clone)]
pub struct LibraryPrepManager {
    pool: PgPool,
}

impl LibraryPrepManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // Protocol operations
    pub async fn create_protocol(&self, request: CreateLibraryPrepProtocolRequest, created_by: Uuid) -> Result<LibraryPrepProtocol, sqlx::Error> {
        sqlx::query_as::<_, LibraryPrepProtocol>(
            r#"
            INSERT INTO library_prep_protocols (
                name, version, protocol_type, kit_name, kit_manufacturer,
                input_requirements, protocol_steps, reagents, equipment_required,
                estimated_duration_hours, created_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#,
        )
        .bind(&request.name)
        .bind(&request.version)
        .bind(&request.protocol_type)
        .bind(&request.kit_name)
        .bind(&request.kit_manufacturer)
        .bind(&request.input_requirements)
        .bind(&request.protocol_steps)
        .bind(&request.reagents)
        .bind(&request.equipment_required)
        .bind(&request.estimated_duration_hours)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_protocol(&self, protocol_id: Uuid) -> Result<Option<LibraryPrepProtocol>, sqlx::Error> {
        sqlx::query_as::<_, LibraryPrepProtocol>(
            "SELECT * FROM library_prep_protocols WHERE id = $1"
        )
        .bind(protocol_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list_protocols(&self, is_active: Option<bool>) -> Result<Vec<LibraryPrepProtocol>, sqlx::Error> {
        let query = if let Some(active) = is_active {
            "SELECT * FROM library_prep_protocols WHERE is_active = $1 ORDER BY name, version DESC"
        } else {
            "SELECT * FROM library_prep_protocols ORDER BY name, version DESC"
        };

        if let Some(active) = is_active {
            sqlx::query_as::<_, LibraryPrepProtocol>(query)
                .bind(active)
                .fetch_all(&self.pool)
                .await
        } else {
            sqlx::query_as::<_, LibraryPrepProtocol>(query)
                .fetch_all(&self.pool)
                .await
        }
    }

    pub async fn update_protocol(&self, protocol_id: Uuid, request: UpdateLibraryPrepProtocolRequest) -> Result<LibraryPrepProtocol, sqlx::Error> {
        sqlx::query_as::<_, LibraryPrepProtocol>(
            r#"
            UPDATE library_prep_protocols
            SET name = COALESCE($2, name),
                version = COALESCE($3, version),
                kit_name = COALESCE($4, kit_name),
                kit_manufacturer = COALESCE($5, kit_manufacturer),
                input_requirements = COALESCE($6, input_requirements),
                protocol_steps = COALESCE($7, protocol_steps),
                reagents = COALESCE($8, reagents),
                equipment_required = COALESCE($9, equipment_required),
                estimated_duration_hours = COALESCE($10, estimated_duration_hours),
                is_active = COALESCE($11, is_active),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(protocol_id)
        .bind(&request.name)
        .bind(&request.version)
        .bind(&request.kit_name)
        .bind(&request.kit_manufacturer)
        .bind(&request.input_requirements)
        .bind(&request.protocol_steps)
        .bind(&request.reagents)
        .bind(&request.equipment_required)
        .bind(&request.estimated_duration_hours)
        .bind(&request.is_active)
        .fetch_one(&self.pool)
        .await
    }

    // Library preparation operations
    pub async fn create_library_prep(&self, request: CreateLibraryPreparationRequest) -> Result<LibraryPreparation, sqlx::Error> {
        sqlx::query_as::<_, LibraryPreparation>(
            r#"
            INSERT INTO library_preparations (
                batch_id, project_id, protocol_id, operator_id,
                sample_ids, prep_date, input_metrics, reagent_lots,
                status, notes
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
        )
        .bind(&request.batch_id)
        .bind(&request.project_id)
        .bind(&request.protocol_id)
        .bind(&request.operator_id)
        .bind(&request.sample_ids)
        .bind(&request.prep_date)
        .bind(&request.input_metrics)
        .bind(&request.reagent_lots)
        .bind("in_progress")
        .bind(&request.notes)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_library_prep(&self, prep_id: Uuid) -> Result<Option<LibraryPreparation>, sqlx::Error> {
        sqlx::query_as::<_, LibraryPreparation>(
            "SELECT * FROM library_preparations WHERE id = $1"
        )
        .bind(prep_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn get_library_prep_with_protocol(&self, prep_id: Uuid) -> Result<Option<LibraryPrepWithProtocol>, sqlx::Error> {
        let prep = self.get_library_prep(prep_id).await?;
        
        if let Some(preparation) = prep {
            let protocol = self.get_protocol(preparation.protocol_id).await?;
            Ok(Some(LibraryPrepWithProtocol {
                preparation,
                protocol,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn list_library_preps(&self, filters: ListLibraryPreparationsQuery) -> Result<Vec<LibraryPrepWithProtocol>, sqlx::Error> {
        let mut query = String::from("SELECT * FROM library_preparations WHERE 1=1");
        let mut bindings = vec![];
        let mut param_count = 1;

        if let Some(project_id) = filters.project_id {
            query.push_str(&format!(" AND project_id = ${}", param_count));
            bindings.push(format!("{}", project_id));
            param_count += 1;
        }

        if let Some(protocol_id) = filters.protocol_id {
            query.push_str(&format!(" AND protocol_id = ${}", param_count));
            bindings.push(format!("{}", protocol_id));
            param_count += 1;
        }

        if let Some(status) = filters.status {
            query.push_str(&format!(" AND status = ${}", param_count));
            bindings.push(status);
            param_count += 1;
        }

        if let Some(batch_search) = filters.batch_search {
            query.push_str(&format!(" AND batch_id ILIKE ${}", param_count));
            bindings.push(format!("%{}%", batch_search));
            param_count += 1;
        }

        query.push_str(" ORDER BY created_at DESC LIMIT 100");

        let mut query_builder = sqlx::query_as::<_, LibraryPreparation>(&query);
        for binding in &bindings {
            query_builder = query_builder.bind(binding);
        }

        let preparations = query_builder.fetch_all(&self.pool).await?;
        
        // Fetch protocols for each preparation
        let mut results = Vec::new();
        for prep in preparations {
            let protocol = self.get_protocol(prep.protocol_id).await?;
            results.push(LibraryPrepWithProtocol {
                preparation: prep,
                protocol,
            });
        }
        
        Ok(results)
    }

    pub async fn update_library_prep(&self, prep_id: Uuid, request: UpdateLibraryPreparationRequest) -> Result<LibraryPreparation, sqlx::Error> {
        sqlx::query_as::<_, LibraryPreparation>(
            r#"
            UPDATE library_preparations
            SET status = COALESCE($2, status),
                input_metrics = COALESCE($3, input_metrics),
                output_metrics = COALESCE($4, output_metrics),
                reagent_lots = COALESCE($5, reagent_lots),
                notes = COALESCE($6, notes),
                qc_status = COALESCE($7, qc_status),
                qc_metrics = COALESCE($8, qc_metrics),
                completed_at = COALESCE($9, completed_at),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(prep_id)
        .bind(&request.status)
        .bind(&request.input_metrics)
        .bind(&request.output_metrics)
        .bind(&request.reagent_lots)
        .bind(&request.notes)
        .bind(&request.qc_status)
        .bind(&request.qc_metrics)
        .bind(&request.completed_at)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn complete_library_prep(&self, prep_id: Uuid, metrics: LibraryPrepMetrics) -> Result<LibraryPreparation, sqlx::Error> {
        let output_metrics = json!({
            "concentration_ngul": metrics.concentration_ngul,
            "volume_ul": metrics.volume_ul,
            "total_yield_ng": metrics.total_yield_ng,
            "fragment_size_bp": metrics.fragment_size_bp,
            "fragment_size_cv": metrics.fragment_size_cv,
            "quality_score": metrics.quality_score
        });

        sqlx::query_as::<_, LibraryPreparation>(
            r#"
            UPDATE library_preparations
            SET status = 'completed',
                completed_at = NOW(),
                output_metrics = $2,
                qc_status = 'passed',
                qc_metrics = $3,
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(prep_id)
        .bind(&output_metrics)
        .bind(&output_metrics)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_library_prep_stats(&self) -> Result<serde_json::Value, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct Stats {
            total_preps: Option<i64>,
            in_progress: Option<i64>,
            completed: Option<i64>,
            failed: Option<i64>,
            avg_yield: Option<f64>,
            success_rate: Option<f64>,
        }

        let stats = sqlx::query_as::<_, Stats>(
            r#"
            SELECT 
                COUNT(*) as total_preps,
                COUNT(*) FILTER (WHERE status = 'in_progress') as in_progress,
                COUNT(*) FILTER (WHERE status = 'completed') as completed,
                COUNT(*) FILTER (WHERE status = 'failed') as failed,
                AVG((output_metrics->>'total_yield_ng')::FLOAT) FILTER (WHERE status = 'completed') as avg_yield,
                CAST(COUNT(*) FILTER (WHERE status = 'completed' AND qc_status = 'passed') AS FLOAT) / 
                    NULLIF(COUNT(*) FILTER (WHERE status = 'completed'), 0) * 100 as success_rate
            FROM library_preparations
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(json!({
            "total_preps": stats.total_preps.unwrap_or(0),
            "in_progress": stats.in_progress.unwrap_or(0),
            "completed": stats.completed.unwrap_or(0),
            "failed": stats.failed.unwrap_or(0),
            "average_yield_ng": stats.avg_yield.unwrap_or(0.0),
            "success_rate": stats.success_rate.unwrap_or(0.0)
        }))
    }

    pub async fn search_by_batch(&self, batch_search: &str) -> Result<Vec<LibraryPreparation>, sqlx::Error> {
        sqlx::query_as::<_, LibraryPreparation>(
            r#"
            SELECT * FROM library_preparations
            WHERE batch_id ILIKE $1
            ORDER BY created_at DESC
            LIMIT 50
            "#
        )
        .bind(format!("%{}%", batch_search))
        .fetch_all(&self.pool)
        .await
    }
}

/// List library prep protocols
pub async fn list_protocols(
    State(pool): State<PgPool>,
    Query(params): Query<serde_json::Value>,
) -> Result<Json<Vec<LibraryPrepProtocol>>, (StatusCode, String)> {
    let pool = &pool;
    
    let manager = LibraryPrepManager::new(pool.clone());
    
    let is_active = params.get("is_active")
        .and_then(|v| v.as_bool());
    
    match manager.list_protocols(is_active).await {
        Ok(protocols) => Ok(Json(protocols)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Get a protocol by ID
pub async fn get_protocol(
    State(pool): State<PgPool>,
    Path(protocol_id): Path<Uuid>,
) -> Result<Json<LibraryPrepProtocol>, (StatusCode, String)> {
    let pool = &pool;
    
    let manager = LibraryPrepManager::new(pool.clone());
    
    match manager.get_protocol(protocol_id).await {
        Ok(Some(protocol)) => Ok(Json(protocol)),
        Ok(None) => Err((StatusCode::NOT_FOUND, "Protocol not found".to_string())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Create a new library prep protocol
pub async fn create_protocol(
    State(pool): State<PgPool>,
    Json(request): Json<CreateLibraryPrepProtocolRequest>,
) -> Result<Json<LibraryPrepProtocol>, (StatusCode, String)> {
    let pool = &pool;
    
    let manager = LibraryPrepManager::new(pool.clone());
    
    // TODO: Get actual user ID from authentication
    let created_by = Uuid::new_v4();
    
    match manager.create_protocol(request, created_by).await {
        Ok(protocol) => Ok(Json(protocol)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Update a library prep protocol
pub async fn update_protocol(
    State(pool): State<PgPool>,
    Path(protocol_id): Path<Uuid>,
    Json(request): Json<UpdateLibraryPrepProtocolRequest>,
) -> Result<Json<LibraryPrepProtocol>, (StatusCode, String)> {
    let pool = &pool;
    
    let manager = LibraryPrepManager::new(pool.clone());
    
    match manager.update_protocol(protocol_id, request).await {
        Ok(protocol) => Ok(Json(protocol)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// List library preparations with filters
pub async fn list_library_preps(
    State(pool): State<PgPool>,
    Query(query): Query<ListLibraryPreparationsQuery>,
) -> Result<Json<Vec<LibraryPrepWithProtocol>>, (StatusCode, String)> {
    let pool = &pool;
    
    let manager = LibraryPrepManager::new(pool.clone());
    
    match manager.list_library_preps(query).await {
        Ok(preps) => Ok(Json(preps)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Get a library preparation by ID
pub async fn get_library_prep(
    State(pool): State<PgPool>,
    Path(prep_id): Path<Uuid>,
) -> Result<Json<LibraryPrepWithProtocol>, (StatusCode, String)> {
    let pool = &pool;
    
    let manager = LibraryPrepManager::new(pool.clone());
    
    match manager.get_library_prep_with_protocol(prep_id).await {
        Ok(Some(prep)) => Ok(Json(prep)),
        Ok(None) => Err((StatusCode::NOT_FOUND, "Library prep not found".to_string())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Create a new library preparation
pub async fn create_library_prep(
    State(pool): State<PgPool>,
    Json(request): Json<CreateLibraryPreparationRequest>,
) -> Result<Json<LibraryPreparation>, (StatusCode, String)> {
    let pool = &pool;
    
    let manager = LibraryPrepManager::new(pool.clone());
    
    match manager.create_library_prep(request).await {
        Ok(prep) => Ok(Json(prep)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Update a library preparation
pub async fn update_library_prep(
    State(pool): State<PgPool>,
    Path(prep_id): Path<Uuid>,
    Json(request): Json<UpdateLibraryPreparationRequest>,
) -> Result<Json<LibraryPreparation>, (StatusCode, String)> {
    let pool = &pool;
    
    let manager = LibraryPrepManager::new(pool.clone());
    
    match manager.update_library_prep(prep_id, request).await {
        Ok(prep) => Ok(Json(prep)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Complete a library preparation
pub async fn complete_library_prep(
    State(pool): State<PgPool>,
    Path(prep_id): Path<Uuid>,
    Json(metrics): Json<LibraryPrepMetrics>,
) -> Result<Json<LibraryPreparation>, (StatusCode, String)> {
    let pool = &pool;
    
    let manager = LibraryPrepManager::new(pool.clone());
    
    match manager.complete_library_prep(prep_id, metrics).await {
        Ok(prep) => Ok(Json(prep)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Get library prep statistics
pub async fn get_library_prep_stats(
    State(pool): State<PgPool>,
    Query(params): Query<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let pool = &pool;
    
    let manager = LibraryPrepManager::new(pool.clone());
    
    match manager.get_library_prep_stats().await {
        Ok(stats) => Ok(Json(stats)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Search library preps by batch number
pub async fn search_library_preps(
    State(pool): State<PgPool>,
    Query(params): Query<serde_json::Value>,
) -> Result<Json<Vec<LibraryPreparation>>, (StatusCode, String)> {
    let pool = &pool;
    
    let manager = LibraryPrepManager::new(pool.clone());
    
    let batch_search = params.get("batch_search")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    
    if batch_search.is_empty() {
        return Ok(Json(vec![]));
    }
    
    match manager.search_by_batch(batch_search).await {
        Ok(preps) => Ok(Json(preps)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
} 