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
    models::flow_cell::*,
};

// Request/Response structs
#[derive(Debug, Deserialize)]
pub struct ListFlowCellDesignsQuery {
    pub status: Option<String>,
    pub flow_cell_type_id: Option<Uuid>,
    pub created_by: Option<Uuid>,
    pub project_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct CreateFlowCellDesignRequest {
    pub name: String,
    pub flow_cell_type_id: Uuid,
    pub run_parameters: Option<serde_json::Value>,
    pub notes: Option<String>,
    pub lanes: Vec<LaneAssignmentRequest>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFlowCellDesignRequest {
    pub name: Option<String>,
    pub run_parameters: Option<serde_json::Value>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ApproveFlowCellDesignRequest {
    pub comments: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LaneAssignmentRequest {
    pub lane_number: i32,
    pub library_prep_ids: Vec<Uuid>,
    pub target_reads: Option<i64>,
    pub index_type: Option<String>,
    pub index_sequences: Option<Vec<String>>,
    pub loading_concentration_pm: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct OptimizationParameters {
    pub balance_weight: Option<f64>,
    pub index_diversity_weight: Option<f64>,
    pub max_libraries_per_lane: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct OptimizeFlowCellRequest {
    pub flow_cell_type_id: Uuid,
    pub library_preparations: Vec<LibraryPrepInfo>,
    pub optimization_parameters: Option<OptimizationParameters>,
}

#[derive(Debug, Deserialize)]
pub struct LibraryPrepInfo {
    pub library_prep_id: Uuid,
    pub target_reads: i64,
    pub concentration_nm: f64,
    pub index_sequence: String,
}

#[derive(Debug, Serialize)]
pub struct OptimizeFlowCellResponse {
    pub optimization_score: f64,
    pub suggested_assignments: Vec<LaneAssignmentRequest>,
    pub expected_metrics: FlowCellMetrics,
    pub warnings: Vec<String>,
    pub alternative_designs: Option<Vec<AlternativeDesign>>,
}

#[derive(Debug, Serialize)]
pub struct AlternativeDesign {
    pub name: String,
    pub score: f64,
    pub assignments: Vec<LaneAssignmentRequest>,
}

#[derive(Debug, Serialize)]
pub struct FlowCellMetrics {
    pub total_reads: i64,
    pub reads_per_sample: Vec<serde_json::Value>,
    pub lane_balance_score: f64,
    pub index_balance_score: f64,
    pub estimated_cost: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct FlowCellTypeStats {
    pub total_runs: i64,
    pub success_rate: f64,
    pub average_reads_generated: f64,
    pub average_q30_percent: f64,
}

#[derive(Debug, Serialize)]
pub struct FlowCellDesignWithDetails {
    #[serde(flatten)]
    pub design: FlowCellDesign,
    pub flow_cell_type: Option<FlowCellType>,
    pub lanes: Vec<FlowCellLane>,
    pub lane_count: i32,
}

// Manager struct for flow cell database operations
#[derive(Clone)]
pub struct FlowCellManager {
    pool: PgPool,
}

impl FlowCellManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // Flow cell type operations
    pub async fn list_flow_cell_types(&self, is_active: Option<bool>) -> Result<Vec<FlowCellType>, sqlx::Error> {
        let query = if is_active.is_some() {
            "SELECT * FROM flow_cell_types WHERE is_active = $1 ORDER BY manufacturer, model"
        } else {
            "SELECT * FROM flow_cell_types ORDER BY manufacturer, model"
        };

        if let Some(active) = is_active {
            sqlx::query_as::<_, FlowCellType>(query)
                .bind(active)
                .fetch_all(&self.pool)
                .await
        } else {
            sqlx::query_as::<_, FlowCellType>(query)
                .fetch_all(&self.pool)
                .await
        }
    }

    pub async fn get_flow_cell_type_stats(&self, type_id: Uuid) -> Result<FlowCellTypeStats, sqlx::Error> {
        let stats = sqlx::query!(
            r#"
            SELECT 
                COUNT(DISTINCT fc.id) as total_runs,
                CAST(COUNT(DISTINCT fc.id) FILTER (WHERE fc.qc_status = 'passed') AS FLOAT) / 
                    NULLIF(COUNT(DISTINCT fc.id), 0) * 100 as success_rate,
                AVG(fc.total_reads) as avg_reads,
                AVG(fc.percent_q30) as avg_q30
            FROM flow_cell_designs fd
            JOIN flow_cells fc ON fc.design_id = fd.id
            WHERE fd.flow_cell_type_id = $1
            "#,
            type_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(FlowCellTypeStats {
            total_runs: stats.total_runs.unwrap_or(0),
            success_rate: stats.success_rate.unwrap_or(0.0),
            average_reads_generated: stats.avg_reads.unwrap_or(0.0),
            average_q30_percent: stats.avg_q30.unwrap_or(0.0),
        })
    }

    // Flow cell design operations
    pub async fn create_flow_cell_design(&self, request: CreateFlowCellDesignRequest, created_by: Uuid) -> Result<FlowCellDesign, sqlx::Error> {
        // Start transaction
        let mut tx = self.pool.begin().await?;

        // Create the design
        let design = sqlx::query_as::<_, FlowCellDesign>(
            r#"
            INSERT INTO flow_cell_designs (
                name, flow_cell_type_id, status, run_parameters,
                notes, created_by
            )
            VALUES ($1, $2, 'draft', $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(&request.name)
        .bind(&request.flow_cell_type_id)
        .bind(&request.run_parameters)
        .bind(&request.notes)
        .bind(created_by)
        .fetch_one(&mut *tx)
        .await?;

        // Create lanes
        for lane in request.lanes {
            self.create_lane_tx(&mut tx, design.id, lane).await?;
        }

        tx.commit().await?;
        Ok(design)
    }

    async fn create_lane_tx(&self, tx: &mut sqlx::Transaction<'_, sqlx::Postgres>, design_id: Uuid, lane: LaneAssignmentRequest) -> Result<(), sqlx::Error> {
        // Create the lane
        let lane_id = sqlx::query_scalar!(
            r#"
            INSERT INTO flow_cell_lanes (
                design_id, lane_number, target_reads, index_type,
                loading_concentration_pm, status
            )
            VALUES ($1, $2, $3, $4, $5, 'configured')
            RETURNING id
            "#,
            design_id,
            lane.lane_number,
            lane.target_reads,
            lane.index_type,
            lane.loading_concentration_pm
        )
        .fetch_one(&mut **tx)
        .await?;

        // Add library assignments
        for (position, lib_id) in lane.library_prep_ids.iter().enumerate() {
            sqlx::query!(
                r#"
                INSERT INTO flow_cell_lane_libraries (
                    lane_id, library_prep_id, position, demux_index
                )
                VALUES ($1, $2, $3, $4)
                "#,
                lane_id,
                lib_id,
                position as i32,
                lane.index_sequences.as_ref().and_then(|seqs| seqs.get(position).cloned())
            )
            .execute(&mut **tx)
            .await?;
        }

        Ok(())
    }

    pub async fn get_flow_cell_design(&self, design_id: Uuid) -> Result<Option<FlowCellDesign>, sqlx::Error> {
        sqlx::query_as::<_, FlowCellDesign>(
            "SELECT * FROM flow_cell_designs WHERE id = $1"
        )
        .bind(design_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn get_flow_cell_design_with_details(&self, design_id: Uuid) -> Result<Option<FlowCellDesignWithDetails>, sqlx::Error> {
        let design = self.get_flow_cell_design(design_id).await?;
        
        if let Some(design) = design {
            let flow_cell_type = if let Some(type_id) = design.flow_cell_type_id {
                sqlx::query_as::<_, FlowCellType>(
                    "SELECT * FROM flow_cell_types WHERE id = $1"
                )
                .bind(type_id)
                .fetch_optional(&self.pool)
                .await?
            } else {
                None
            };

            let lanes = self.get_design_lanes(design_id).await?;
            let lane_count = lanes.len() as i32;

            Ok(Some(FlowCellDesignWithDetails {
                design,
                flow_cell_type,
                lanes,
                lane_count,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn list_flow_cell_designs(&self, filters: ListFlowCellDesignsQuery) -> Result<Vec<FlowCellDesignWithDetails>, sqlx::Error> {
        let mut query = String::from("SELECT * FROM flow_cell_designs WHERE 1=1");
        let mut bindings = vec![];
        let mut param_count = 1;

        if let Some(status) = filters.status {
            query.push_str(&format!(" AND status = ${}", param_count));
            bindings.push(status);
            param_count += 1;
        }

        if let Some(type_id) = filters.flow_cell_type_id {
            query.push_str(&format!(" AND flow_cell_type_id = ${}", param_count));
            bindings.push(format!("{}", type_id));
            param_count += 1;
        }

        if let Some(created_by) = filters.created_by {
            query.push_str(&format!(" AND created_by = ${}", param_count));
            bindings.push(format!("{}", created_by));
            param_count += 1;
        }

        query.push_str(" ORDER BY created_at DESC LIMIT 100");

        let mut query_builder = sqlx::query_as::<_, FlowCellDesign>(&query);
        for binding in &bindings {
            query_builder = query_builder.bind(binding);
        }

        let designs = query_builder.fetch_all(&self.pool).await?;

        // Fetch details for each design
        let mut results = Vec::new();
        for design in designs {
            if let Some(detailed) = self.get_flow_cell_design_with_details(design.id).await? {
                results.push(detailed);
            }
        }

        Ok(results)
    }

    pub async fn update_flow_cell_design(&self, design_id: Uuid, request: UpdateFlowCellDesignRequest) -> Result<FlowCellDesign, sqlx::Error> {
        sqlx::query_as::<_, FlowCellDesign>(
            r#"
            UPDATE flow_cell_designs
            SET name = COALESCE($2, name),
                run_parameters = COALESCE($3, run_parameters),
                notes = COALESCE($4, notes),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(design_id)
        .bind(&request.name)
        .bind(&request.run_parameters)
        .bind(&request.notes)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn approve_flow_cell_design(&self, design_id: Uuid, approved_by: Uuid, comments: Option<String>) -> Result<FlowCellDesign, sqlx::Error> {
        sqlx::query_as::<_, FlowCellDesign>(
            r#"
            UPDATE flow_cell_designs
            SET status = 'approved',
                approved_by = $2,
                approved_at = NOW(),
                notes = CASE 
                    WHEN $3 IS NOT NULL THEN 
                        COALESCE(notes, '') || E'\n\nApproval Comments: ' || $3
                    ELSE notes
                END,
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(design_id)
        .bind(approved_by)
        .bind(comments)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn delete_flow_cell_design(&self, design_id: Uuid) -> Result<(), sqlx::Error> {
        // Only allow deletion of draft designs
        let result = sqlx::query!(
            "DELETE FROM flow_cell_designs WHERE id = $1 AND status = 'draft'",
            design_id
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }

        Ok(())
    }

    // Lane operations
    pub async fn get_design_lanes(&self, design_id: Uuid) -> Result<Vec<FlowCellLane>, sqlx::Error> {
        sqlx::query_as::<_, FlowCellLane>(
            "SELECT * FROM flow_cell_lanes WHERE design_id = $1 ORDER BY lane_number"
        )
        .bind(design_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn update_lane(&self, design_id: Uuid, lane_number: i32, request: LaneAssignmentRequest) -> Result<FlowCellLane, sqlx::Error> {
        // Start transaction
        let mut tx = self.pool.begin().await?;

        // Update lane
        let lane = sqlx::query_as::<_, FlowCellLane>(
            r#"
            UPDATE flow_cell_lanes
            SET target_reads = COALESCE($3, target_reads),
                index_type = COALESCE($4, index_type),
                loading_concentration_pm = COALESCE($5, loading_concentration_pm),
                updated_at = NOW()
            WHERE design_id = $1 AND lane_number = $2
            RETURNING *
            "#,
        )
        .bind(design_id)
        .bind(lane_number)
        .bind(request.target_reads)
        .bind(&request.index_type)
        .bind(request.loading_concentration_pm)
        .fetch_one(&mut *tx)
        .await?;

        // Remove existing library assignments
        sqlx::query!(
            "DELETE FROM flow_cell_lane_libraries WHERE lane_id = $1",
            lane.id
        )
        .execute(&mut *tx)
        .await?;

        // Add new library assignments
        for (position, lib_id) in request.library_prep_ids.iter().enumerate() {
            sqlx::query!(
                r#"
                INSERT INTO flow_cell_lane_libraries (
                    lane_id, library_prep_id, position, demux_index
                )
                VALUES ($1, $2, $3, $4)
                "#,
                lane.id,
                lib_id,
                position as i32,
                request.index_sequences.as_ref().and_then(|seqs| seqs.get(position).cloned())
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(lane)
    }

    // AI Optimization
    pub async fn optimize_flow_cell_design(&self, request: OptimizeFlowCellRequest) -> Result<OptimizeFlowCellResponse, sqlx::Error> {
        // Get flow cell type info
        let fc_type = sqlx::query!(
            "SELECT lane_count, reads_per_lane FROM flow_cell_types WHERE id = $1",
            request.flow_cell_type_id
        )
        .fetch_one(&self.pool)
        .await?;

        let lane_count = fc_type.lane_count as usize;
        let reads_per_lane = fc_type.reads_per_lane.unwrap_or(2_500_000_000) as i64;

        // Simple optimization algorithm
        let mut lane_assignments = Vec::new();
        let mut warnings = Vec::new();

        // Group libraries by index compatibility
        let mut index_groups: std::collections::HashMap<String, Vec<&LibraryPrepInfo>> = std::collections::HashMap::new();
        for lib in &request.library_preparations {
            let index_prefix = lib.index_sequence.chars().take(4).collect::<String>();
            index_groups.entry(index_prefix).or_insert_with(Vec::new).push(lib);
        }

        // Check for index conflicts
        for (prefix, libs) in &index_groups {
            if libs.len() > lane_count {
                warnings.push(format!("Libraries with index prefix {} exceed lane count", prefix));
            }
        }

        // Distribute libraries across lanes
        let mut current_lane = 1;
        let mut current_lane_reads = 0;
        let mut current_lane_libs = Vec::new();

        for lib in &request.library_preparations {
            if current_lane_reads + lib.target_reads > reads_per_lane && !current_lane_libs.is_empty() {
                // Move to next lane
                lane_assignments.push(LaneAssignmentRequest {
                    lane_number: current_lane,
                    library_prep_ids: current_lane_libs.clone(),
                    target_reads: Some(current_lane_reads),
                    index_type: Some("dual".to_string()),
                    index_sequences: None,
                    loading_concentration_pm: Some(200.0),
                });
                
                current_lane += 1;
                current_lane_reads = 0;
                current_lane_libs.clear();
            }

            current_lane_libs.push(lib.library_prep_id);
            current_lane_reads += lib.target_reads;
        }

        // Add last lane
        if !current_lane_libs.is_empty() {
            lane_assignments.push(LaneAssignmentRequest {
                lane_number: current_lane,
                library_prep_ids: current_lane_libs,
                target_reads: Some(current_lane_reads),
                index_type: Some("dual".to_string()),
                index_sequences: None,
                loading_concentration_pm: Some(200.0),
            });
        }

        // Calculate metrics
        let total_reads = lane_assignments.iter()
            .map(|l| l.target_reads.unwrap_or(0))
            .sum();

        let lane_balance_score = if lane_assignments.len() > 1 {
            let avg_reads = total_reads as f64 / lane_assignments.len() as f64;
            let variance = lane_assignments.iter()
                .map(|l| {
                    let diff = l.target_reads.unwrap_or(0) as f64 - avg_reads;
                    diff * diff
                })
                .sum::<f64>() / lane_assignments.len() as f64;
            1.0 - (variance.sqrt() / avg_reads).min(1.0)
        } else {
            1.0
        };

        Ok(OptimizeFlowCellResponse {
            optimization_score: lane_balance_score,
            suggested_assignments: lane_assignments,
            expected_metrics: FlowCellMetrics {
                total_reads,
                reads_per_sample: vec![],
                lane_balance_score,
                index_balance_score: 0.9, // Simplified
                estimated_cost: Some((total_reads as f64 / 1_000_000_000.0) * 0.25), // $0.25 per Gb
            },
            warnings,
            alternative_designs: None,
        })
    }
}

/// List available flow cell types
pub async fn list_flow_cell_types(
    State(state): State<Arc<AppComponents>>,
) -> Result<Json<Vec<FlowCellType>>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = FlowCellManager::new(pool.clone());
    
    match manager.list_flow_cell_types(Some(true)).await {
        Ok(types) => Ok(Json(types)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Get flow cell type statistics
pub async fn get_flow_cell_type_stats(
    State(state): State<Arc<AppComponents>>,
    Path(type_id): Path<Uuid>,
) -> Result<Json<FlowCellTypeStats>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = FlowCellManager::new(pool.clone());
    
    match manager.get_flow_cell_type_stats(type_id).await {
        Ok(stats) => Ok(Json(stats)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// List flow cell designs
pub async fn list_flow_cell_designs(
    State(state): State<Arc<AppComponents>>,
    Query(query): Query<ListFlowCellDesignsQuery>,
) -> Result<Json<Vec<FlowCellDesignWithDetails>>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = FlowCellManager::new(pool.clone());
    
    match manager.list_flow_cell_designs(query).await {
        Ok(designs) => Ok(Json(designs)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Get a flow cell design by ID
pub async fn get_flow_cell_design(
    State(state): State<Arc<AppComponents>>,
    Path(design_id): Path<Uuid>,
) -> Result<Json<FlowCellDesignWithDetails>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = FlowCellManager::new(pool.clone());
    
    match manager.get_flow_cell_design_with_details(design_id).await {
        Ok(Some(design)) => Ok(Json(design)),
        Ok(None) => Err((StatusCode::NOT_FOUND, "Flow cell design not found".to_string())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Create a new flow cell design
pub async fn create_flow_cell_design(
    State(state): State<Arc<AppComponents>>,
    Json(request): Json<CreateFlowCellDesignRequest>,
) -> Result<Json<FlowCellDesign>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = FlowCellManager::new(pool.clone());
    
    // TODO: Get actual user ID from authentication
    let created_by = Uuid::new_v4();
    
    match manager.create_flow_cell_design(request, created_by).await {
        Ok(design) => Ok(Json(design)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Update a flow cell design
pub async fn update_flow_cell_design(
    State(state): State<Arc<AppComponents>>,
    Path(design_id): Path<Uuid>,
    Json(request): Json<UpdateFlowCellDesignRequest>,
) -> Result<Json<FlowCellDesign>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = FlowCellManager::new(pool.clone());
    
    match manager.update_flow_cell_design(design_id, request).await {
        Ok(design) => Ok(Json(design)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Approve a flow cell design
pub async fn approve_flow_cell_design(
    State(state): State<Arc<AppComponents>>,
    Path(design_id): Path<Uuid>,
    Json(request): Json<ApproveFlowCellDesignRequest>,
) -> Result<Json<FlowCellDesign>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = FlowCellManager::new(pool.clone());
    
    // TODO: Get actual user ID from authentication
    let approved_by = Uuid::new_v4();
    
    match manager.approve_flow_cell_design(design_id, approved_by, request.comments).await {
        Ok(design) => Ok(Json(design)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Delete a flow cell design
pub async fn delete_flow_cell_design(
    State(state): State<Arc<AppComponents>>,
    Path(design_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = FlowCellManager::new(pool.clone());
    
    match manager.delete_flow_cell_design(design_id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(sqlx::Error::RowNotFound) => Err((StatusCode::BAD_REQUEST, "Can only delete draft designs".to_string())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Optimize flow cell design using AI
pub async fn optimize_flow_cell_design(
    State(state): State<Arc<AppComponents>>,
    Json(request): Json<OptimizeFlowCellRequest>,
) -> Result<Json<OptimizeFlowCellResponse>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = FlowCellManager::new(pool.clone());
    
    match manager.optimize_flow_cell_design(request).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Get lane details for a flow cell design
pub async fn get_flow_cell_lanes(
    State(state): State<Arc<AppComponents>>,
    Path(design_id): Path<Uuid>,
) -> Result<Json<Vec<FlowCellLane>>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = FlowCellManager::new(pool.clone());
    
    match manager.get_design_lanes(design_id).await {
        Ok(lanes) => Ok(Json(lanes)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Update a specific lane in a flow cell design
pub async fn update_flow_cell_lane(
    State(state): State<Arc<AppComponents>>,
    Path((design_id, lane_number)): Path<(Uuid, i32)>,
    Json(request): Json<LaneAssignmentRequest>,
) -> Result<Json<FlowCellLane>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = FlowCellManager::new(pool.clone());
    
    match manager.update_lane(design_id, lane_number, request).await {
        Ok(lane) => Ok(Json(lane)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
} 