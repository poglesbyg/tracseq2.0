use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde_json::json;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::{
    error::{Result, SequencingError},
    models::*,
    AppState,
};

/// Create a new sequencing workflow
pub async fn create_workflow(
    State(state): State<AppState>,
    Json(request): Json<CreateWorkflowRequest>,
) -> Result<Json<serde_json::Value>> {
    let workflow_id = Uuid::new_v4();
    
    let workflow = sqlx::query_as::<_, SequencingWorkflow>(
        r#"
        INSERT INTO sequencing_workflows (
            id, name, description, workflow_type, platform,
            steps, parameters, is_active, created_at, created_by
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), $9)
        RETURNING *
        "#
    )
    .bind(workflow_id)
    .bind(&request.name)
    .bind(request.description.as_deref())
    .bind(&request.workflow_type)
    .bind(&request.platform)
    .bind(&request.steps)
    .bind(&request.parameters)
    .bind(request.is_active.unwrap_or(true))
    .bind(request.created_by.as_deref())
    .fetch_one(&state.db_pool.pool)
    .await?;

    Ok(Json(json!({
        "success": true,
        "data": workflow,
        "message": "Workflow created successfully"
    })))
}

/// Get workflow by ID
pub async fn get_workflow(
    State(state): State<AppState>,
    Path(workflow_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let workflow = sqlx::query_as::<_, SequencingWorkflow>(
        "SELECT * FROM sequencing_workflows WHERE id = $1"
    )
    .bind(workflow_id)
    .fetch_optional(&state.db_pool.pool)
    .await?
    .ok_or(SequencingError::WorkflowNotFound(workflow_id.to_string()))?;

    // Get workflow execution history
    let executions = sqlx::query_as::<_, WorkflowExecution>(
        r#"
        SELECT * FROM workflow_executions 
        WHERE workflow_id = $1 
        ORDER BY started_at DESC 
        LIMIT 10
        "#
    )
    .bind(workflow_id)
    .fetch_all(&state.db_pool.pool)
    .await?;

    Ok(Json(json!({
        "success": true,
        "data": {
            "workflow": workflow,
            "recent_executions": executions,
            "execution_count": executions.len()
        }
    })))
}

/// List all workflows with filtering
pub async fn list_workflows(
    State(state): State<AppState>,
    Query(query): Query<WorkflowListQuery>,
) -> Result<Json<serde_json::Value>> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).min(100);
    let offset = (page - 1) * page_size;

    let mut where_conditions = Vec::new();
    let mut param_count = 0;

    if let Some(workflow_type) = &query.workflow_type {
        param_count += 1;
        where_conditions.push(format!("workflow_type = ${}", param_count));
    }

    if let Some(platform) = &query.platform {
        param_count += 1;
        where_conditions.push(format!("platform = ${}", param_count));
    }

    if let Some(is_active) = query.is_active {
        param_count += 1;
        where_conditions.push(format!("is_active = ${}", param_count));
    }

    let where_clause = if where_conditions.is_empty() {
        "".to_string()
    } else {
        format!("WHERE {}", where_conditions.join(" AND "))
    };

    // Get total count
    let total_count: i64 = sqlx::query_scalar(&format!(
        "SELECT COUNT(*) FROM sequencing_workflows {}", where_clause
    ))
    .fetch_one(&state.db_pool.pool)
    .await?;

    // Get workflows
    let workflows = sqlx::query_as::<_, SequencingWorkflow>(&format!(
        "SELECT * FROM sequencing_workflows {} ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
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
            "workflows": workflows,
            "pagination": {
                "page": page,
                "page_size": page_size,
                "total_count": total_count,
                "total_pages": total_pages
            }
        }
    })))
}

/// Update workflow
pub async fn update_workflow(
    State(state): State<AppState>,
    Path(workflow_id): Path<Uuid>,
    Json(request): Json<UpdateWorkflowRequest>,
) -> Result<Json<serde_json::Value>> {
    // Check if workflow exists
    let existing_workflow = sqlx::query("SELECT id FROM sequencing_workflows WHERE id = $1")
        .bind(workflow_id)
        .fetch_optional(&state.db_pool.pool)
        .await?
        .ok_or(SequencingError::WorkflowNotFound(workflow_id.to_string()))?;

    let workflow = sqlx::query_as::<_, SequencingWorkflow>(
        r#"
        UPDATE sequencing_workflows 
        SET 
            name = COALESCE($2, name),
            description = COALESCE($3, description),
            steps = COALESCE($4, steps),
            parameters = COALESCE($5, parameters),
            is_active = COALESCE($6, is_active),
            updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#
    )
    .bind(workflow_id)
    .bind(request.name.as_deref())
    .bind(request.description.as_deref())
    .bind(request.steps.as_ref())
    .bind(request.parameters.as_ref())
    .bind(request.is_active)
    .fetch_one(&state.db_pool.pool)
    .await?;

    Ok(Json(json!({
        "success": true,
        "data": workflow,
        "message": "Workflow updated successfully"
    })))
}

/// Delete workflow
pub async fn delete_workflow(
    State(state): State<AppState>,
    Path(workflow_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    // Check if workflow has active executions
    let active_executions: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM workflow_executions 
        WHERE workflow_id = $1 AND status IN ('running', 'queued')
        "#
    )
    .bind(workflow_id)
    .fetch_one(&state.db_pool.pool)
    .await?;

    if active_executions > 0 {
        return Err(SequencingError::WorkflowInUse { 
            workflow_id,
            active_executions: active_executions as u32,
        });
    }

    let deleted_workflow = sqlx::query_as::<_, SequencingWorkflow>(
        "DELETE FROM sequencing_workflows WHERE id = $1 RETURNING *"
    )
    .bind(workflow_id)
    .fetch_optional(&state.db_pool.pool)
    .await?
    .ok_or(SequencingError::WorkflowNotFound(workflow_id.to_string()))?;

    Ok(Json(json!({
        "success": true,
        "data": deleted_workflow,
        "message": "Workflow deleted successfully"
    })))
}

/// Execute workflow for a job
pub async fn execute_workflow(
    State(state): State<AppState>,
    Path(workflow_id): Path<Uuid>,
    Json(request): Json<ExecuteWorkflowRequest>,
) -> Result<Json<serde_json::Value>> {
    // Verify workflow exists and is active
    let workflow = sqlx::query_as::<_, SequencingWorkflow>(
        "SELECT * FROM sequencing_workflows WHERE id = $1 AND is_active = true"
    )
    .bind(workflow_id)
    .fetch_optional(&state.db_pool.pool)
    .await?
    .ok_or(SequencingError::WorkflowNotFound(workflow_id.to_string()))?;

    // Verify job exists if provided
    if let Some(job_id) = request.job_id {
        let job_exists = sqlx::query("SELECT id FROM sequencing_jobs WHERE id = $1")
            .bind(job_id)
            .fetch_optional(&state.db_pool.pool)
            .await?
            .is_some();

        if !job_exists {
            return Err(SequencingError::JobNotFound(job_id.to_string()));
        }
    }

    // Create workflow execution
    let execution_id = Uuid::new_v4();
    let execution = sqlx::query_as::<_, WorkflowExecution>(
        r#"
        INSERT INTO workflow_executions (
            id, workflow_id, job_id, status, parameters,
            started_at, created_by
        ) VALUES ($1, $2, $3, $4, $5, NOW(), $6)
        RETURNING *
        "#
    )
    .bind(execution_id)
    .bind(workflow_id)
    .bind(request.job_id)
    .bind(ExecutionStatus::Running)
    .bind(&request.parameters)
    .bind(request.executed_by.as_deref())
    .fetch_one(&state.db_pool.pool)
    .await?;

    // Initialize workflow steps
    let steps: Vec<WorkflowStep> = serde_json::from_value(workflow.steps.clone())
        .map_err(|_| SequencingError::WorkflowValidation {
            message: "Invalid workflow steps format".to_string(),
        })?;

    for (index, step) in steps.iter().enumerate() {
        sqlx::query(
            r#"
            INSERT INTO workflow_step_executions (
                id, execution_id, step_name, step_order, status,
                parameters, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, NOW())
            "#
        )
        .bind(Uuid::new_v4())
        .bind(execution_id)
        .bind(&step.name)
        .bind(index as i32)
        .bind(if index == 0 { StepStatus::Running } else { StepStatus::Pending })
        .bind(&step.parameters)
        .execute(&state.db_pool.pool)
        .await?;
    }

    Ok(Json(json!({
        "success": true,
        "data": {
            "execution": execution,
            "workflow": workflow,
            "steps_initialized": steps.len()
        },
        "message": "Workflow execution started successfully"
    })))
}

/// Get workflow execution status
pub async fn get_execution_status(
    State(state): State<AppState>,
    Path(execution_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let execution = sqlx::query_as::<_, WorkflowExecution>(
        "SELECT * FROM workflow_executions WHERE id = $1"
    )
    .bind(execution_id)
    .fetch_optional(&state.db_pool.pool)
    .await?
    .ok_or(SequencingError::ExecutionNotFound { execution_id })?;

    // Get step execution details
    let step_executions = sqlx::query_as::<_, WorkflowStepExecution>(
        r#"
        SELECT * FROM workflow_step_executions 
        WHERE execution_id = $1 
        ORDER BY step_order ASC
        "#
    )
    .bind(execution_id)
    .fetch_all(&state.db_pool.pool)
    .await?;

    // Calculate overall progress
    let total_steps = step_executions.len();
    let completed_steps = step_executions.iter()
        .filter(|s| s.status == StepStatus::Completed)
        .count();
    let failed_steps = step_executions.iter()
        .filter(|s| s.status == StepStatus::Failed)
        .count();

    let progress_percentage = if total_steps > 0 {
        (completed_steps as f32 / total_steps as f32 * 100.0).round()
    } else {
        0.0
    };

    Ok(Json(json!({
        "success": true,
        "data": {
            "execution": execution,
            "steps": step_executions,
            "progress": {
                "total_steps": total_steps,
                "completed_steps": completed_steps,
                "failed_steps": failed_steps,
                "progress_percentage": progress_percentage
            }
        }
    })))
}

/// Update workflow step status
pub async fn update_step_status(
    State(state): State<AppState>,
    Path((execution_id, step_name)): Path<(Uuid, String)>,
    Json(request): Json<UpdateStepStatusRequest>,
) -> Result<Json<serde_json::Value>> {
    let step_execution = sqlx::query_as::<_, WorkflowStepExecution>(
        r#"
        UPDATE workflow_step_executions 
        SET 
            status = $3,
            started_at = COALESCE(started_at, CASE WHEN $3 = 'running' THEN NOW() ELSE started_at END),
            completed_at = CASE WHEN $3 IN ('completed', 'failed') THEN NOW() ELSE completed_at END,
            error_message = $4,
            result_data = $5,
            updated_at = NOW()
        WHERE execution_id = $1 AND step_name = $2
        RETURNING *
        "#
    )
    .bind(execution_id)
    .bind(&step_name)
    .bind(&request.status)
    .bind(request.error_message.as_deref())
    .bind(request.result_data.as_ref())
    .fetch_optional(&state.db_pool.pool)
    .await?
    .ok_or(SequencingError::StepNotFound { 
        execution_id, 
        step_name: step_name.clone() 
    })?;

    // Check if workflow should progress to next step or complete
    if request.status == StepStatus::Completed {
        // Find next pending step and start it
        let next_step = sqlx::query_as::<_, WorkflowStepExecution>(
            r#"
            SELECT * FROM workflow_step_executions 
            WHERE execution_id = $1 AND status = 'pending'
            ORDER BY step_order ASC
            LIMIT 1
            "#
        )
        .bind(execution_id)
        .fetch_optional(&state.db_pool.pool)
        .await?;

        if let Some(next_step) = next_step {
            // Start next step
            sqlx::query(
                r#"
                UPDATE workflow_step_executions 
                SET status = 'running', started_at = NOW(), updated_at = NOW()
                WHERE id = $1
                "#
            )
            .bind(next_step.id)
            .execute(&state.db_pool.pool)
            .await?;
        } else {
            // Check if all steps are completed
            let remaining_steps: i64 = sqlx::query_scalar(
                r#"
                SELECT COUNT(*) FROM workflow_step_executions 
                WHERE execution_id = $1 AND status NOT IN ('completed', 'failed')
                "#
            )
            .bind(execution_id)
            .fetch_one(&state.db_pool.pool)
            .await?;

            if remaining_steps == 0 {
                // Complete the workflow execution
                sqlx::query(
                    r#"
                    UPDATE workflow_executions 
                    SET status = 'completed', completed_at = NOW()
                    WHERE id = $1
                    "#
                )
                .bind(execution_id)
                .execute(&state.db_pool.pool)
                .await?;
            }
        }
    } else if request.status == StepStatus::Failed {
        // Fail the entire workflow execution
        sqlx::query(
            r#"
            UPDATE workflow_executions 
            SET status = 'failed', completed_at = NOW()
            WHERE id = $1
            "#
        )
        .bind(execution_id)
        .execute(&state.db_pool.pool)
        .await?;
    }

    Ok(Json(json!({
        "success": true,
        "data": step_execution,
        "message": "Step status updated successfully"
    })))
}

/// Cancel workflow execution
pub async fn cancel_execution(
    State(state): State<AppState>,
    Path(execution_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let execution = sqlx::query_as::<_, WorkflowExecution>(
        r#"
        UPDATE workflow_executions 
        SET status = 'cancelled', completed_at = NOW()
        WHERE id = $1 AND status IN ('running', 'queued')
        RETURNING *
        "#
    )
    .bind(execution_id)
    .fetch_optional(&state.db_pool.pool)
    .await?
    .ok_or(SequencingError::ExecutionNotFound { execution_id })?;

    // Cancel all running steps
    sqlx::query(
        r#"
        UPDATE workflow_step_executions 
        SET status = 'cancelled', completed_at = NOW()
        WHERE execution_id = $1 AND status IN ('running', 'pending')
        "#
    )
    .bind(execution_id)
    .execute(&state.db_pool.pool)
    .await?;

    Ok(Json(json!({
        "success": true,
        "data": execution,
        "message": "Workflow execution cancelled successfully"
    })))
}

/// Get workflow templates
pub async fn get_workflow_templates(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    let templates = sqlx::query_as::<_, WorkflowTemplate>(
        "SELECT * FROM workflow_templates WHERE is_active = true ORDER BY category, name"
    )
    .fetch_all(&state.db_pool.pool)
    .await?;

    // Group by category
    let mut grouped_templates: std::collections::HashMap<String, Vec<WorkflowTemplate>> = 
        std::collections::HashMap::new();
    
    for template in templates {
        grouped_templates
            .entry(template.category.clone())
            .or_insert_with(Vec::new)
            .push(template);
    }

    Ok(Json(json!({
        "success": true,
        "data": {
            "templates": grouped_templates,
            "categories": grouped_templates.keys().collect::<Vec<_>>()
        }
    })))
}

/// Request and response structures
#[derive(serde::Deserialize)]
pub struct CreateWorkflowRequest {
    pub name: String,
    pub description: Option<String>,
    pub workflow_type: String,
    pub platform: String,
    pub steps: serde_json::Value,
    pub parameters: serde_json::Value,
    pub is_active: Option<bool>,
    pub created_by: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct UpdateWorkflowRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub steps: Option<serde_json::Value>,
    pub parameters: Option<serde_json::Value>,
    pub is_active: Option<bool>,
}

#[derive(serde::Deserialize)]
pub struct ExecuteWorkflowRequest {
    pub job_id: Option<Uuid>,
    pub parameters: serde_json::Value,
    pub executed_by: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct UpdateStepStatusRequest {
    pub status: StepStatus,
    pub error_message: Option<String>,
    pub result_data: Option<serde_json::Value>,
}

#[derive(serde::Deserialize)]
pub struct WorkflowListQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub workflow_type: Option<String>,
    pub platform: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(serde::Deserialize)]
pub struct WorkflowStep {
    pub name: String,
    pub description: Option<String>,
    pub step_type: String,
    pub parameters: serde_json::Value,
    pub timeout_seconds: Option<i32>,
    pub retry_count: Option<i32>,
}
