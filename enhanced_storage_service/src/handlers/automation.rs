use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use tracing::{info, error, warn};

use crate::{
    error::{StorageError, StorageResult},
    models::*,
    AppState,
};

/// Automated sample placement
/// POST /automation/samples/:sample_id/place
pub async fn automated_placement(
    State(state): State<AppState>,
    Path(sample_id): Path<Uuid>,
    Json(request): Json<AutomatedPlacementRequest>,
) -> StorageResult<Json<ApiResponse<AutomationTask>>> {
    info!("Initiating automated sample placement for: {}", sample_id);

    // Validate sample exists and is ready for automation
    validate_sample_for_automation(&state, sample_id).await?;

    // Create automation task
    let task = AutomationTask {
        id: Uuid::new_v4(),
        task_type: "automated_placement".to_string(),
        sample_id: Some(sample_id),
        robot_id: request.robot_id.clone(),
        status: "queued".to_string(),
        estimated_duration_minutes: estimate_placement_duration(&request),
        priority: request.priority.unwrap_or("normal".to_string()),
        location_id: request.target_location_id,
        position: request.target_position,
        instructions: request.instructions,
        created_at: Utc::now(),
        started_at: None,
        completed_at: None,
        error_message: None,
        metadata: Some(json!({
            "automation_type": "placement",
            "requested_by": request.requested_by,
            "temperature_requirements": request.temperature_requirements
        })),
    };

    // Queue task for robot execution
    queue_automation_task(&state, &task).await?;

    Ok(Json(ApiResponse::success(task)))
}

/// Automated sample retrieval
/// POST /automation/samples/:sample_id/retrieve
pub async fn automated_retrieval(
    State(state): State<AppState>,
    Path(sample_id): Path<Uuid>,
    Json(request): Json<AutomatedRetrievalRequest>,
) -> StorageResult<Json<ApiResponse<AutomationTask>>> {
    info!("Initiating automated sample retrieval for: {}", sample_id);

    // Validate sample location and accessibility
    validate_sample_for_retrieval(&state, sample_id).await?;

    let task = AutomationTask {
        id: Uuid::new_v4(),
        task_type: "automated_retrieval".to_string(),
        sample_id: Some(sample_id),
        robot_id: request.robot_id.clone(),
        status: "queued".to_string(),
        estimated_duration_minutes: estimate_retrieval_duration(&request),
        priority: request.priority.unwrap_or("normal".to_string()),
        location_id: None,
        position: None,
        instructions: request.instructions,
        created_at: Utc::now(),
        started_at: None,
        completed_at: None,
        error_message: None,
        metadata: Some(json!({
            "automation_type": "retrieval",
            "requested_by": request.requested_by,
            "destination": request.destination
        })),
    };

    queue_automation_task(&state, &task).await?;

    Ok(Json(ApiResponse::success(task)))
}

/// Get robot status and capabilities
/// GET /automation/robots/:robot_id/status
pub async fn get_robot_status(
    State(state): State<AppState>,
    Path(robot_id): Path<String>,
) -> StorageResult<Json<ApiResponse<RobotStatus>>> {
    info!("Getting robot status for: {}", robot_id);

    let robot_status = get_robot_details(&state, &robot_id).await?;

    Ok(Json(ApiResponse::success(robot_status)))
}

/// List all available robots
/// GET /automation/robots
pub async fn list_robots(
    State(state): State<AppState>,
    Query(query): Query<RobotListQuery>,
) -> StorageResult<Json<ApiResponse<PaginatedResponse<RobotStatus>>>> {
    info!("Listing automation robots");

    let robots = get_available_robots(&state, &query).await?;

    Ok(Json(ApiResponse::success(robots)))
}

/// Send command to robot
/// POST /automation/robots/:robot_id/commands
pub async fn send_robot_command(
    State(state): State<AppState>,
    Path(robot_id): Path<String>,
    Json(request): Json<RobotCommandRequest>,
) -> StorageResult<Json<ApiResponse<RobotCommandResponse>>> {
    info!("Sending command to robot {}: {}", robot_id, request.command);

    // Validate robot exists and is available
    let robot = get_robot_details(&state, &robot_id).await?;
    
    if robot.status != "online" && robot.status != "idle" {
        return Err(StorageError::Validation(
            format!("Robot {} is not available for commands (status: {})", robot_id, robot.status)
        ));
    }

    // Execute command based on type
    let response = match request.command.as_str() {
        "calibrate" => execute_calibration_command(&state, &robot_id, &request).await?,
        "home" => execute_home_command(&state, &robot_id, &request).await?,
        "stop" => execute_stop_command(&state, &robot_id, &request).await?,
        "resume" => execute_resume_command(&state, &robot_id, &request).await?,
        "maintenance" => execute_maintenance_command(&state, &robot_id, &request).await?,
        _ => return Err(StorageError::Validation(format!("Unknown command: {}", request.command))),
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Schedule automated task
/// POST /automation/tasks/schedule
pub async fn schedule_task(
    State(state): State<AppState>,
    Json(request): Json<ScheduleTaskRequest>,
) -> StorageResult<Json<ApiResponse<ScheduledTask>>> {
    info!("Scheduling automation task: {}", request.task_type);

    // Validate scheduling parameters
    validate_scheduling_request(&request)?;

    let scheduled_task = ScheduledTask {
        id: Uuid::new_v4(),
        task_type: request.task_type.clone(),
        schedule_type: request.schedule_type.clone(),
        scheduled_time: request.scheduled_time,
        next_execution: calculate_next_execution(&request),
        recurrence_pattern: request.recurrence_pattern,
        robot_id: request.robot_id,
        parameters: request.parameters,
        status: "scheduled".to_string(),
        created_at: Utc::now(),
        last_execution: None,
        execution_count: 0,
        failure_count: 0,
        enabled: true,
    };

    // Store scheduled task
    store_scheduled_task(&state, &scheduled_task).await?;

    Ok(Json(ApiResponse::success(scheduled_task)))
}

/// List automation jobs
/// GET /automation/jobs
pub async fn list_jobs(
    State(state): State<AppState>,
    Query(query): Query<JobListQuery>,
) -> StorageResult<Json<ApiResponse<PaginatedResponse<AutomationJob>>>> {
    info!("Listing automation jobs");

    let jobs = get_automation_jobs(&state, &query).await?;

    Ok(Json(ApiResponse::success(jobs)))
}

/// Get specific job status
/// GET /automation/jobs/:job_id
pub async fn get_job_status(
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
) -> StorageResult<Json<ApiResponse<AutomationJob>>> {
    info!("Getting job status for: {}", job_id);

    let job = get_job_details(&state, job_id).await?;

    Ok(Json(ApiResponse::success(job)))
}

/// Cancel automation job
/// POST /automation/jobs/:job_id/cancel
pub async fn cancel_job(
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
) -> StorageResult<Json<ApiResponse<AutomationJob>>> {
    info!("Cancelling automation job: {}", job_id);

    let job = cancel_automation_job(&state, job_id).await?;

    Ok(Json(ApiResponse::success(job)))
}

/// Get automation workflows
/// GET /automation/workflows
pub async fn list_workflows(
    State(state): State<AppState>,
    Query(query): Query<WorkflowListQuery>,
) -> StorageResult<Json<ApiResponse<PaginatedResponse<AutomationWorkflow>>>> {
    info!("Listing automation workflows");

    let workflows = get_automation_workflows(&state, &query).await?;

    Ok(Json(ApiResponse::success(workflows)))
}

/// Create automation workflow
/// POST /automation/workflows
pub async fn create_workflow(
    State(state): State<AppState>,
    Json(request): Json<CreateWorkflowRequest>,
) -> StorageResult<Json<ApiResponse<AutomationWorkflow>>> {
    info!("Creating automation workflow: {}", request.name);

    // Validate workflow definition
    validate_workflow_definition(&request)?;

    let workflow = AutomationWorkflow {
        id: Uuid::new_v4(),
        name: request.name.clone(),
        description: request.description,
        workflow_type: request.workflow_type,
        steps: request.steps,
        triggers: request.triggers,
        conditions: request.conditions,
        status: "active".to_string(),
        version: 1,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        created_by: request.created_by,
        execution_count: 0,
        success_rate: 1.0,
        average_duration_minutes: 0.0,
    };

    // Store workflow
    store_automation_workflow(&state, &workflow).await?;

    Ok(Json(ApiResponse::success(workflow)))
}

/// Execute workflow
/// POST /automation/workflows/:workflow_id/execute
pub async fn execute_workflow(
    State(state): State<AppState>,
    Path(workflow_id): Path<Uuid>,
    Json(request): Json<ExecuteWorkflowRequest>,
) -> StorageResult<Json<ApiResponse<WorkflowExecution>>> {
    info!("Executing automation workflow: {}", workflow_id);

    let execution = start_workflow_execution(&state, workflow_id, &request).await?;

    Ok(Json(ApiResponse::success(execution)))
}

/// Get automation analytics
/// GET /automation/analytics
pub async fn get_automation_analytics(
    State(state): State<AppState>,
    Query(query): Query<AnalyticsQuery>,
) -> StorageResult<Json<ApiResponse<AutomationAnalytics>>> {
    info!("Getting automation analytics");

    let analytics = calculate_automation_analytics(&state, &query).await?;

    Ok(Json(ApiResponse::success(analytics)))
}

/// Get maintenance schedule
/// GET /automation/maintenance
pub async fn get_maintenance_schedule(
    State(state): State<AppState>,
    Query(query): Query<MaintenanceQuery>,
) -> StorageResult<Json<ApiResponse<PaginatedResponse<MaintenanceTask>>>> {
    info!("Getting automation maintenance schedule");

    let maintenance_tasks = get_maintenance_tasks(&state, &query).await?;

    Ok(Json(ApiResponse::success(maintenance_tasks)))
}

/// Schedule maintenance
/// POST /automation/maintenance
pub async fn schedule_maintenance(
    State(state): State<AppState>,
    Json(request): Json<ScheduleMaintenanceRequest>,
) -> StorageResult<Json<ApiResponse<MaintenanceTask>>> {
    info!("Scheduling maintenance for robot: {:?}", request.robot_id);

    let maintenance_task = create_maintenance_task(&state, &request).await?;

    Ok(Json(ApiResponse::success(maintenance_task)))
}

// Helper functions
async fn validate_sample_for_automation(state: &AppState, sample_id: Uuid) -> StorageResult<()> {
    // Mock validation - in production would check sample status, location accessibility, etc.
    info!("Validating sample {} for automation", sample_id);
    Ok(())
}

async fn validate_sample_for_retrieval(state: &AppState, sample_id: Uuid) -> StorageResult<()> {
    // Mock validation - in production would verify sample location and robot accessibility
    info!("Validating sample {} for retrieval", sample_id);
    Ok(())
}

async fn queue_automation_task(state: &AppState, task: &AutomationTask) -> StorageResult<()> {
    // Mock implementation - in production would add to robot task queue
    info!("Queuing automation task: {}", task.id);
    Ok(())
}

async fn get_robot_details(state: &AppState, robot_id: &str) -> StorageResult<RobotStatus> {
    // Mock implementation - in production would query robot system
    Ok(RobotStatus {
        robot_id: robot_id.to_string(),
        name: format!("Storage Robot {}", robot_id),
        robot_type: "articulated_arm".to_string(),
        status: "idle".to_string(),
        location: "storage_area_1".to_string(),
        battery_level: 85,
        current_task: None,
        capabilities: vec![
            "sample_placement".to_string(),
            "sample_retrieval".to_string(),
            "temperature_handling".to_string(),
        ],
        last_maintenance: Some(Utc::now() - Duration::days(7)),
        operational_hours: 1248.5,
        error_count_24h: 0,
        position: Some(json!({"x": 1.5, "y": 2.3, "z": 0.8})),
        load_capacity: 50.0,
        precision_mm: 0.1,
        speed_ms: 0.5,
    })
}

async fn get_available_robots(state: &AppState, query: &RobotListQuery) -> StorageResult<PaginatedResponse<RobotStatus>> {
    // Mock implementation
    let robots = vec![
        RobotStatus {
            robot_id: "ROBOT001".to_string(),
            name: "Primary Storage Robot".to_string(),
            robot_type: "articulated_arm".to_string(),
            status: "idle".to_string(),
            location: "storage_area_1".to_string(),
            battery_level: 85,
            current_task: None,
            capabilities: vec!["sample_placement".to_string(), "sample_retrieval".to_string()],
            last_maintenance: Some(Utc::now() - Duration::days(7)),
            operational_hours: 1248.5,
            error_count_24h: 0,
            position: Some(json!({"x": 1.5, "y": 2.3, "z": 0.8})),
            load_capacity: 50.0,
            precision_mm: 0.1,
            speed_ms: 0.5,
        }
    ];

    Ok(PaginatedResponse {
        data: robots,
        pagination: PaginationInfo {
            page: query.page.unwrap_or(1),
            per_page: query.per_page.unwrap_or(50),
            total_pages: 1,
            total_items: 1,
            has_next: false,
            has_prev: false,
        },
    })
}

async fn execute_calibration_command(state: &AppState, robot_id: &str, request: &RobotCommandRequest) -> StorageResult<RobotCommandResponse> {
    Ok(RobotCommandResponse {
        command_id: Uuid::new_v4(),
        robot_id: robot_id.to_string(),
        command: request.command.clone(),
        status: "completed".to_string(),
        result: "Calibration completed successfully".to_string(),
        execution_time_ms: 5000,
        executed_at: Utc::now(),
        error_message: None,
    })
}

async fn execute_home_command(state: &AppState, robot_id: &str, request: &RobotCommandRequest) -> StorageResult<RobotCommandResponse> {
    Ok(RobotCommandResponse {
        command_id: Uuid::new_v4(),
        robot_id: robot_id.to_string(),
        command: request.command.clone(),
        status: "completed".to_string(),
        result: "Robot moved to home position".to_string(),
        execution_time_ms: 2000,
        executed_at: Utc::now(),
        error_message: None,
    })
}

async fn execute_stop_command(state: &AppState, robot_id: &str, request: &RobotCommandRequest) -> StorageResult<RobotCommandResponse> {
    Ok(RobotCommandResponse {
        command_id: Uuid::new_v4(),
        robot_id: robot_id.to_string(),
        command: request.command.clone(),
        status: "completed".to_string(),
        result: "Robot stopped successfully".to_string(),
        execution_time_ms: 100,
        executed_at: Utc::now(),
        error_message: None,
    })
}

async fn execute_resume_command(state: &AppState, robot_id: &str, request: &RobotCommandRequest) -> StorageResult<RobotCommandResponse> {
    Ok(RobotCommandResponse {
        command_id: Uuid::new_v4(),
        robot_id: robot_id.to_string(),
        command: request.command.clone(),
        status: "completed".to_string(),
        result: "Robot resumed operations".to_string(),
        execution_time_ms: 500,
        executed_at: Utc::now(),
        error_message: None,
    })
}

async fn execute_maintenance_command(state: &AppState, robot_id: &str, request: &RobotCommandRequest) -> StorageResult<RobotCommandResponse> {
    Ok(RobotCommandResponse {
        command_id: Uuid::new_v4(),
        robot_id: robot_id.to_string(),
        command: request.command.clone(),
        status: "initiated".to_string(),
        result: "Maintenance mode activated".to_string(),
        execution_time_ms: 1000,
        executed_at: Utc::now(),
        error_message: None,
    })
}

fn validate_scheduling_request(request: &ScheduleTaskRequest) -> StorageResult<()> {
    if request.schedule_type == "one_time" && request.scheduled_time.is_none() {
        return Err(StorageError::Validation("One-time tasks require a scheduled_time".to_string()));
    }

    if request.schedule_type == "recurring" && request.recurrence_pattern.is_none() {
        return Err(StorageError::Validation("Recurring tasks require a recurrence_pattern".to_string()));
    }

    Ok(())
}

fn calculate_next_execution(request: &ScheduleTaskRequest) -> Option<DateTime<Utc>> {
    match request.schedule_type.as_str() {
        "one_time" => request.scheduled_time,
        "recurring" => {
            // Mock calculation - in production would parse recurrence pattern
            Some(Utc::now() + Duration::hours(24))
        }
        _ => None,
    }
}

async fn store_scheduled_task(state: &AppState, task: &ScheduledTask) -> StorageResult<()> {
    // Mock implementation - in production would store in database
    info!("Storing scheduled task: {}", task.id);
    Ok(())
}

async fn get_automation_jobs(state: &AppState, query: &JobListQuery) -> StorageResult<PaginatedResponse<AutomationJob>> {
    // Mock implementation
    let jobs = vec![
        AutomationJob {
            id: Uuid::new_v4(),
            job_type: "batch_placement".to_string(),
            status: "running".to_string(),
            progress: 0.65,
            total_tasks: 100,
            completed_tasks: 65,
            failed_tasks: 2,
            started_at: Utc::now() - Duration::hours(1),
            estimated_completion: Some(Utc::now() + Duration::minutes(30)),
            robot_id: Some("ROBOT001".to_string()),
            workflow_id: None,
            metadata: Some(json!({"batch_id": "BATCH001", "priority": "high"})),
        }
    ];

    Ok(PaginatedResponse {
        data: jobs,
        pagination: PaginationInfo {
            page: query.page.unwrap_or(1),
            per_page: query.per_page.unwrap_or(50),
            total_pages: 1,
            total_items: 1,
            has_next: false,
            has_prev: false,
        },
    })
}

async fn get_job_details(state: &AppState, job_id: Uuid) -> StorageResult<AutomationJob> {
    // Mock implementation
    Ok(AutomationJob {
        id: job_id,
        job_type: "sample_placement".to_string(),
        status: "completed".to_string(),
        progress: 1.0,
        total_tasks: 1,
        completed_tasks: 1,
        failed_tasks: 0,
        started_at: Utc::now() - Duration::minutes(5),
        estimated_completion: None,
        robot_id: Some("ROBOT001".to_string()),
        workflow_id: None,
        metadata: Some(json!({"sample_id": "SAM001"})),
    })
}

async fn cancel_automation_job(state: &AppState, job_id: Uuid) -> StorageResult<AutomationJob> {
    // Mock implementation - in production would actually cancel the job
    let mut job = get_job_details(state, job_id).await?;
    job.status = "cancelled".to_string();
    Ok(job)
}

fn estimate_placement_duration(request: &AutomatedPlacementRequest) -> i32 {
    // Mock estimation based on request parameters
    let base_duration = 3; // 3 minutes base
    let complexity_factor = if request.temperature_requirements.is_some() { 2 } else { 1 };
    base_duration * complexity_factor
}

fn estimate_retrieval_duration(request: &AutomatedRetrievalRequest) -> i32 {
    // Mock estimation
    2 // 2 minutes for retrieval
}

async fn get_automation_workflows(state: &AppState, query: &WorkflowListQuery) -> StorageResult<PaginatedResponse<AutomationWorkflow>> {
    // Mock implementation
    let workflows = vec![
        AutomationWorkflow {
            id: Uuid::new_v4(),
            name: "Sample Storage Workflow".to_string(),
            description: Some("Automated sample storage with validation".to_string()),
            workflow_type: "storage".to_string(),
            steps: json!([
                {"step": "validate_sample", "timeout": 60},
                {"step": "find_optimal_location", "timeout": 120},
                {"step": "place_sample", "timeout": 180},
                {"step": "verify_placement", "timeout": 60}
            ]),
            triggers: json!(["sample_ready", "storage_requested"]),
            conditions: json!({"sample_validated": true, "robot_available": true}),
            status: "active".to_string(),
            version: 1,
            created_at: Utc::now() - Duration::days(30),
            updated_at: Utc::now() - Duration::days(1),
            created_by: "system".to_string(),
            execution_count: 150,
            success_rate: 0.98,
            average_duration_minutes: 7.5,
        }
    ];

    Ok(PaginatedResponse {
        data: workflows,
        pagination: PaginationInfo {
            page: query.page.unwrap_or(1),
            per_page: query.per_page.unwrap_or(50),
            total_pages: 1,
            total_items: 1,
            has_next: false,
            has_prev: false,
        },
    })
}

fn validate_workflow_definition(request: &CreateWorkflowRequest) -> StorageResult<()> {
    if request.name.trim().is_empty() {
        return Err(StorageError::Validation("Workflow name cannot be empty".to_string()));
    }

    if request.steps.as_array().map_or(true, |arr| arr.is_empty()) {
        return Err(StorageError::Validation("Workflow must have at least one step".to_string()));
    }

    Ok(())
}

async fn store_automation_workflow(state: &AppState, workflow: &AutomationWorkflow) -> StorageResult<()> {
    // Mock implementation
    info!("Storing automation workflow: {}", workflow.id);
    Ok(())
}

async fn start_workflow_execution(state: &AppState, workflow_id: Uuid, request: &ExecuteWorkflowRequest) -> StorageResult<WorkflowExecution> {
    Ok(WorkflowExecution {
        id: Uuid::new_v4(),
        workflow_id,
        status: "running".to_string(),
        current_step: 1,
        total_steps: 4,
        started_at: Utc::now(),
        completed_at: None,
        input_parameters: request.parameters.clone(),
        execution_log: vec![
            "Workflow execution started".to_string(),
            "Step 1: Validating input parameters".to_string(),
        ],
        error_message: None,
    })
}

async fn calculate_automation_analytics(state: &AppState, query: &AnalyticsQuery) -> StorageResult<AutomationAnalytics> {
    Ok(AutomationAnalytics {
        time_period_days: query.days.unwrap_or(30),
        total_tasks_executed: 1250,
        successful_tasks: 1220,
        failed_tasks: 30,
        success_rate: 0.976,
        average_task_duration_minutes: 4.2,
        robot_utilization: 0.72,
        peak_throughput_tasks_per_hour: 45,
        energy_consumption_kwh: 125.5,
        cost_savings_usd: 15600.0,
        top_failure_reasons: vec![
            ("sensor_timeout".to_string(), 12),
            ("mechanical_error".to_string(), 8),
            ("location_occupied".to_string(), 6),
        ],
        efficiency_trend: "improving".to_string(),
    })
}

async fn get_maintenance_tasks(state: &AppState, query: &MaintenanceQuery) -> StorageResult<PaginatedResponse<MaintenanceTask>> {
    let tasks = vec![
        MaintenanceTask {
            id: Uuid::new_v4(),
            robot_id: "ROBOT001".to_string(),
            task_type: "calibration".to_string(),
            priority: "medium".to_string(),
            scheduled_date: Utc::now() + Duration::days(3),
            estimated_duration_hours: 2.0,
            description: "Monthly precision calibration".to_string(),
            status: "scheduled".to_string(),
            assigned_technician: Some("tech_001".to_string()),
            created_at: Utc::now(),
            last_updated: Utc::now(),
        }
    ];

    Ok(PaginatedResponse {
        data: tasks,
        pagination: PaginationInfo {
            page: query.page.unwrap_or(1),
            per_page: query.per_page.unwrap_or(50),
            total_pages: 1,
            total_items: 1,
            has_next: false,
            has_prev: false,
        },
    })
}

async fn create_maintenance_task(state: &AppState, request: &ScheduleMaintenanceRequest) -> StorageResult<MaintenanceTask> {
    Ok(MaintenanceTask {
        id: Uuid::new_v4(),
        robot_id: request.robot_id.clone().unwrap_or("ROBOT001".to_string()),
        task_type: request.task_type.clone(),
        priority: request.priority.clone().unwrap_or("medium".to_string()),
        scheduled_date: request.scheduled_date,
        estimated_duration_hours: request.estimated_duration_hours.unwrap_or(1.0),
        description: request.description.clone(),
        status: "scheduled".to_string(),
        assigned_technician: request.assigned_technician.clone(),
        created_at: Utc::now(),
        last_updated: Utc::now(),
    })
}

// Request/Response structures
#[derive(Debug, Deserialize)]
pub struct AutomatedPlacementRequest {
    pub robot_id: Option<String>,
    pub target_location_id: Option<Uuid>,
    pub target_position: Option<String>,
    pub priority: Option<String>,
    pub instructions: Option<String>,
    pub requested_by: String,
    pub temperature_requirements: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct AutomatedRetrievalRequest {
    pub robot_id: Option<String>,
    pub destination: String,
    pub priority: Option<String>,
    pub instructions: Option<String>,
    pub requested_by: String,
}

#[derive(Debug, Deserialize)]
pub struct RobotListQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub status: Option<String>,
    pub robot_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RobotCommandRequest {
    pub command: String,
    pub parameters: Option<serde_json::Value>,
    pub timeout_seconds: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct ScheduleTaskRequest {
    pub task_type: String,
    pub schedule_type: String, // "one_time", "recurring"
    pub scheduled_time: Option<DateTime<Utc>>,
    pub recurrence_pattern: Option<String>, // "daily", "weekly", "monthly"
    pub robot_id: Option<String>,
    pub parameters: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct JobListQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub status: Option<String>,
    pub robot_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WorkflowListQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub workflow_type: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateWorkflowRequest {
    pub name: String,
    pub description: Option<String>,
    pub workflow_type: String,
    pub steps: serde_json::Value,
    pub triggers: serde_json::Value,
    pub conditions: serde_json::Value,
    pub created_by: String,
}

#[derive(Debug, Deserialize)]
pub struct ExecuteWorkflowRequest {
    pub parameters: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct AnalyticsQuery {
    pub days: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct MaintenanceQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub robot_id: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ScheduleMaintenanceRequest {
    pub robot_id: Option<String>,
    pub task_type: String,
    pub priority: Option<String>,
    pub scheduled_date: DateTime<Utc>,
    pub estimated_duration_hours: Option<f64>,
    pub description: String,
    pub assigned_technician: Option<String>,
}

// Data structures
#[derive(Debug, Serialize)]
pub struct AutomationTask {
    pub id: Uuid,
    pub task_type: String,
    pub sample_id: Option<Uuid>,
    pub robot_id: Option<String>,
    pub status: String,
    pub priority: String,
    pub location_id: Option<Uuid>,
    pub position: Option<String>,
    pub instructions: Option<String>,
    pub estimated_duration_minutes: i32,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct RobotStatus {
    pub robot_id: String,
    pub name: String,
    pub robot_type: String,
    pub status: String,
    pub location: String,
    pub battery_level: i32,
    pub current_task: Option<String>,
    pub capabilities: Vec<String>,
    pub last_maintenance: Option<DateTime<Utc>>,
    pub operational_hours: f64,
    pub error_count_24h: i32,
    pub position: Option<serde_json::Value>,
    pub load_capacity: f64,
    pub precision_mm: f64,
    pub speed_ms: f64,
}

#[derive(Debug, Serialize)]
pub struct RobotCommandResponse {
    pub command_id: Uuid,
    pub robot_id: String,
    pub command: String,
    pub status: String,
    pub result: String,
    pub execution_time_ms: i32,
    pub executed_at: DateTime<Utc>,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ScheduledTask {
    pub id: Uuid,
    pub task_type: String,
    pub schedule_type: String,
    pub scheduled_time: Option<DateTime<Utc>>,
    pub recurrence_pattern: Option<String>,
    pub robot_id: Option<String>,
    pub parameters: Option<serde_json::Value>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub last_execution: Option<DateTime<Utc>>,
    pub next_execution: Option<DateTime<Utc>>,
    pub execution_count: i32,
    pub failure_count: i32,
    pub enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct AutomationJob {
    pub id: Uuid,
    pub job_type: String,
    pub status: String,
    pub progress: f64,
    pub total_tasks: i32,
    pub completed_tasks: i32,
    pub failed_tasks: i32,
    pub started_at: DateTime<Utc>,
    pub estimated_completion: Option<DateTime<Utc>>,
    pub robot_id: Option<String>,
    pub workflow_id: Option<Uuid>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct AutomationWorkflow {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub workflow_type: String,
    pub steps: serde_json::Value,
    pub triggers: serde_json::Value,
    pub conditions: serde_json::Value,
    pub status: String,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: String,
    pub execution_count: i32,
    pub success_rate: f64,
    pub average_duration_minutes: f64,
}

#[derive(Debug, Serialize)]
pub struct WorkflowExecution {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub status: String,
    pub current_step: i32,
    pub total_steps: i32,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub input_parameters: Option<serde_json::Value>,
    pub execution_log: Vec<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AutomationAnalytics {
    pub time_period_days: i32,
    pub total_tasks_executed: i32,
    pub successful_tasks: i32,
    pub failed_tasks: i32,
    pub success_rate: f64,
    pub average_task_duration_minutes: f64,
    pub robot_utilization: f64,
    pub peak_throughput_tasks_per_hour: i32,
    pub energy_consumption_kwh: f64,
    pub cost_savings_usd: f64,
    pub top_failure_reasons: Vec<(String, i32)>,
    pub efficiency_trend: String,
}

#[derive(Debug, Serialize)]
pub struct MaintenanceTask {
    pub id: Uuid,
    pub robot_id: String,
    pub task_type: String,
    pub priority: String,
    pub scheduled_date: DateTime<Utc>,
    pub estimated_duration_hours: f64,
    pub description: String,
    pub status: String,
    pub assigned_technician: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}
