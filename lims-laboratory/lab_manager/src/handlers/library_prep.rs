use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    assembly::AppComponents,
    models::library_prep::*,
};

// Request/Response structs
#[derive(Debug, Deserialize)]
pub struct ListLibraryPreparationsQuery {
    pub batch_id: Option<Uuid>,
    pub protocol_id: Option<Uuid>,
    pub status: Option<String>,
    pub search: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLibraryPrepProtocolRequest {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub kit_name: Option<String>,
    pub kit_catalog_number: Option<String>,
    pub input_type: String,
    pub output_type: String,
    pub min_input_amount: Option<f64>,
    pub max_input_amount: Option<f64>,
    pub typical_yield: Option<f64>,
    pub protocol_steps: Option<serde_json::Value>,
    pub reagents: Option<serde_json::Value>,
    pub equipment_required: Option<Vec<String>>,
    pub estimated_time_hours: Option<f64>,
    pub hazards: Option<Vec<String>>,
    pub storage_conditions: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLibraryPrepProtocolRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub protocol_steps: Option<serde_json::Value>,
    pub is_active: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLibraryPreparationRequest {
    pub batch_id: Uuid,
    pub protocol_id: Uuid,
    pub prepared_by: Uuid,
    pub input_samples: Vec<serde_json::Value>,
    pub kit_lot_number: Option<String>,
    pub reagent_lots: Option<serde_json::Value>,
    pub notes: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLibraryPreparationRequest {
    pub status: Option<String>,
    pub qc_status: Option<String>,
    pub notes: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

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
                name, version, description, kit_name, kit_catalog_number,
                input_type, output_type, min_input_amount, max_input_amount,
                typical_yield, protocol_steps, reagents, equipment_required,
                estimated_time_hours, hazards, storage_conditions, metadata, created_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
            RETURNING *
            "#,
        )
        .bind(&request.name)
        .bind(&request.version)
        .bind(&request.description)
        .bind(&request.kit_name)
        .bind(&request.kit_catalog_number)
        .bind(&request.input_type)
        .bind(&request.output_type)
        .bind(&request.min_input_amount)
        .bind(&request.max_input_amount)
        .bind(&request.typical_yield)
        .bind(&request.protocol_steps)
        .bind(&request.reagents)
        .bind(&request.equipment_required)
        .bind(&request.estimated_time_hours)
        .bind(&request.hazards)
        .bind(&request.storage_conditions)
        .bind(&request.metadata)
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
                description = COALESCE($3, description),
                protocol_steps = COALESCE($4, protocol_steps),
                is_active = COALESCE($5, is_active),
                metadata = COALESCE($6, metadata),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(protocol_id)
        .bind(&request.name)
        .bind(&request.description)
        .bind(&request.protocol_steps)
        .bind(&request.is_active)
        .bind(&request.metadata)
        .fetch_one(&self.pool)
        .await
    }

    // Library preparation operations
    pub async fn create_library_prep(&self, request: CreateLibraryPreparationRequest) -> Result<LibraryPreparation, sqlx::Error> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let prep_number = format!("LP-{}-{:06}", Utc::now().format("%Y%m%d"), rng.gen_range(0..1000000));
        
        sqlx::query_as::<_, LibraryPreparation>(
            r#"
            INSERT INTO library_preparations (
                prep_number, batch_id, protocol_id, prepared_by,
                input_samples, kit_lot_number, reagent_lots,
                status, notes, metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
        )
        .bind(&prep_number)
        .bind(&request.batch_id)
        .bind(&request.protocol_id)
        .bind(&request.prepared_by)
        .bind(serde_json::to_value(&request.input_samples).unwrap())
        .bind(&request.kit_lot_number)
        .bind(&request.reagent_lots)
        .bind("in_progress")
        .bind(&request.notes)
        .bind(&request.metadata)
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

        if let Some(batch_id) = filters.batch_id {
            query.push_str(&format!(" AND batch_id = ${}", param_count));
            bindings.push(format!("{}", batch_id));
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

        if let Some(search) = filters.search {
            query.push_str(&format!(" AND (prep_number ILIKE ${} OR notes ILIKE ${})", param_count, param_count));
            bindings.push(format!("%{}%", search));
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
                qc_status = COALESCE($3, qc_status),
                notes = COALESCE($4, notes),
                metadata = COALESCE($5, metadata),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(prep_id)
        .bind(&request.status)
        .bind(&request.qc_status)
        .bind(&request.notes)
        .bind(&request.metadata)
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
        let stats = sqlx::query!(
            r#"
            SELECT 
                COUNT(*) as total_preps,
                COUNT(*) FILTER (WHERE status = 'in_progress') as in_progress,
                COUNT(*) FILTER (WHERE status = 'completed') as completed,
                COUNT(*) FILTER (WHERE status = 'failed') as failed,
                AVG(total_yield_ng) FILTER (WHERE status = 'completed') as avg_yield,
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
            SELECT lp.* FROM library_preparations lp
            JOIN batches b ON lp.batch_id = b.id
            WHERE b.batch_number ILIKE $1
            ORDER BY lp.created_at DESC
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
    State(state): State<Arc<AppComponents>>,
    Query(params): Query<serde_json::Value>,
) -> Result<Json<Vec<LibraryPrepProtocol>>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
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
    State(state): State<Arc<AppComponents>>,
    Path(protocol_id): Path<Uuid>,
) -> Result<Json<LibraryPrepProtocol>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = LibraryPrepManager::new(pool.clone());
    
    match manager.get_protocol(protocol_id).await {
        Ok(Some(protocol)) => Ok(Json(protocol)),
        Ok(None) => Err((StatusCode::NOT_FOUND, "Protocol not found".to_string())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Create a new library prep protocol
pub async fn create_protocol(
    State(state): State<Arc<AppComponents>>,
    Json(request): Json<CreateLibraryPrepProtocolRequest>,
) -> Result<Json<LibraryPrepProtocol>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
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
    State(state): State<Arc<AppComponents>>,
    Path(protocol_id): Path<Uuid>,
    Json(request): Json<UpdateLibraryPrepProtocolRequest>,
) -> Result<Json<LibraryPrepProtocol>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = LibraryPrepManager::new(pool.clone());
    
    match manager.update_protocol(protocol_id, request).await {
        Ok(protocol) => Ok(Json(protocol)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// List library preparations with filters
pub async fn list_library_preps(
    State(state): State<Arc<AppComponents>>,
    Query(query): Query<ListLibraryPreparationsQuery>,
) -> Result<Json<Vec<LibraryPrepWithProtocol>>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = LibraryPrepManager::new(pool.clone());
    
    match manager.list_library_preps(query).await {
        Ok(preps) => Ok(Json(preps)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Get a library preparation by ID
pub async fn get_library_prep(
    State(state): State<Arc<AppComponents>>,
    Path(prep_id): Path<Uuid>,
) -> Result<Json<LibraryPrepWithProtocol>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = LibraryPrepManager::new(pool.clone());
    
    match manager.get_library_prep_with_protocol(prep_id).await {
        Ok(Some(prep)) => Ok(Json(prep)),
        Ok(None) => Err((StatusCode::NOT_FOUND, "Library prep not found".to_string())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Create a new library preparation
pub async fn create_library_prep(
    State(state): State<Arc<AppComponents>>,
    Json(request): Json<CreateLibraryPreparationRequest>,
) -> Result<Json<LibraryPreparation>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = LibraryPrepManager::new(pool.clone());
    
    match manager.create_library_prep(request).await {
        Ok(prep) => Ok(Json(prep)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Update a library preparation
pub async fn update_library_prep(
    State(state): State<Arc<AppComponents>>,
    Path(prep_id): Path<Uuid>,
    Json(request): Json<UpdateLibraryPreparationRequest>,
) -> Result<Json<LibraryPreparation>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = LibraryPrepManager::new(pool.clone());
    
    match manager.update_library_prep(prep_id, request).await {
        Ok(prep) => Ok(Json(prep)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Complete a library preparation
pub async fn complete_library_prep(
    State(state): State<Arc<AppComponents>>,
    Path(prep_id): Path<Uuid>,
    Json(metrics): Json<LibraryPrepMetrics>,
) -> Result<Json<LibraryPreparation>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = LibraryPrepManager::new(pool.clone());
    
    match manager.complete_library_prep(prep_id, metrics).await {
        Ok(prep) => Ok(Json(prep)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Get library prep statistics
pub async fn get_library_prep_stats(
    State(state): State<Arc<AppComponents>>,
    Query(params): Query<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = LibraryPrepManager::new(pool.clone());
    
    match manager.get_library_prep_stats().await {
        Ok(stats) => Ok(Json(stats)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Search library preps by batch number
pub async fn search_library_preps(
    State(state): State<Arc<AppComponents>>,
    Query(params): Query<serde_json::Value>,
) -> Result<Json<Vec<LibraryPreparation>>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
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