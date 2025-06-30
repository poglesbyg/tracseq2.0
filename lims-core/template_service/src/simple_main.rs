use axum::{
    extract::{Path, State, Multipart},
    http::StatusCode,
    routing::{get, post, put, delete},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTemplate {
    pub name: String,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTemplate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

// Simple in-memory storage for demonstration
type TemplateStore = Arc<Mutex<HashMap<Uuid, Template>>>;

#[derive(Clone)]
struct AppState {
    templates: TemplateStore,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("🚀 Starting Template Service (Simplified)");
    
    // Initialize in-memory storage with some sample data
    let templates = Arc::new(Mutex::new(HashMap::new()));
    
    // Add sample templates
    {
        let mut store = templates.lock().unwrap();
        
        let sample_template_1 = Template {
            id: Uuid::new_v4(),
            name: "Sample Collection Form".to_string(),
            description: Some("Standard template for sample collection metadata".to_string()),
            created_at: chrono::Utc::now(),
            metadata: Some(json!({
                "fields": ["sample_id", "collection_date", "location", "collector"],
                "category": "collection"
            })),
        };
        
        let sample_template_2 = Template {
            id: Uuid::new_v4(),
            name: "Sequencing Request".to_string(),
            description: Some("Template for submitting sequencing requests".to_string()),
            created_at: chrono::Utc::now(),
            metadata: Some(json!({
                "fields": ["sample_id", "sequence_type", "priority", "requester"],
                "category": "sequencing"
            })),
        };
        
        store.insert(sample_template_1.id, sample_template_1);
        store.insert(sample_template_2.id, sample_template_2);
    }
    
    let app_state = AppState { templates };
    
    // Build the router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/templates", get(list_templates))
        .route("/api/templates", post(create_template))
        .route("/api/templates/upload", post(upload_template))
        .route("/api/templates/:id", get(get_template))
        .route("/api/templates/:id/data", get(get_template_data))
        .route("/api/templates/:id", put(update_template))
        .route("/api/templates/:id", delete(delete_template))
        .layer(CorsLayer::permissive())
        .with_state(app_state);
    
    // Start the server
    let listener = TcpListener::bind("0.0.0.0:8083").await.unwrap();
    info!("✅ Template Service listening on port 8083");
    
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "Template Service",
        "version": "1.0.0",
        "timestamp": chrono::Utc::now()
    }))
}

async fn list_templates(
    State(state): State<AppState>,
) -> Result<Json<Vec<Template>>, StatusCode> {
    let templates = state.templates.lock().unwrap();
    let template_list: Vec<Template> = templates.values().cloned().collect();
    Ok(Json(template_list))
}

async fn get_template(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Template>, StatusCode> {
    let templates = state.templates.lock().unwrap();
    templates
        .get(&id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn get_template_data(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let templates = state.templates.lock().unwrap();
    
    if let Some(template) = templates.get(&id) {
        // Extract content from metadata if it exists
        let content = if let Some(metadata) = &template.metadata {
            metadata.get("content")
                .and_then(|c| c.as_str())
                .unwrap_or("No content available")
                .to_string()
        } else {
            "No content available".to_string()
        };
        
        Ok(Json(json!({
            "id": template.id,
            "name": template.name,
            "description": template.description,
            "content": content,
            "created_at": template.created_at,
            "metadata": template.metadata
        })))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn create_template(
    State(state): State<AppState>,
    Json(payload): Json<CreateTemplate>,
) -> Result<Json<Template>, StatusCode> {
    let new_template = Template {
        id: Uuid::new_v4(),
        name: payload.name,
        description: payload.description,
        created_at: chrono::Utc::now(),
        metadata: payload.metadata,
    };
    
    let mut templates = state.templates.lock().unwrap();
    templates.insert(new_template.id, new_template.clone());
    
    info!("✅ Created new template: {}", new_template.name);
    Ok(Json(new_template))
}

async fn update_template(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateTemplate>,
) -> Result<Json<Template>, StatusCode> {
    let mut templates = state.templates.lock().unwrap();
    
    let template = templates.get_mut(&id).ok_or(StatusCode::NOT_FOUND)?;
    
    if let Some(name) = payload.name {
        template.name = name;
    }
    if let Some(description) = payload.description {
        template.description = Some(description);
    }
    if let Some(metadata) = payload.metadata {
        template.metadata = Some(metadata);
    }
    
    info!("✅ Updated template: {}", template.name);
    Ok(Json(template.clone()))
}

async fn delete_template(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let mut templates = state.templates.lock().unwrap();
    
    if templates.remove(&id).is_some() {
        info!("✅ Deleted template: {}", id);
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn upload_template(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut template_name = String::new();
    let mut template_content = String::new();
    
    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        let name = field.name().unwrap_or("").to_string();
        let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
        
        match name.as_str() {
            "name" => {
                template_name = String::from_utf8(data.to_vec()).map_err(|_| StatusCode::BAD_REQUEST)?;
            }
            "file" => {
                template_content = String::from_utf8(data.to_vec()).map_err(|_| StatusCode::BAD_REQUEST)?;
            }
            _ => {
                // Ignore unknown fields
            }
        }
    }
    
    if template_name.is_empty() {
        template_name = format!("Uploaded Template {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"));
    }
    
    // Create new template from uploaded content
    let new_template = Template {
        id: Uuid::new_v4(),
        name: template_name.clone(),
        description: Some("Template uploaded via file upload".to_string()),
        created_at: chrono::Utc::now(),
        metadata: Some(json!({
            "content": template_content,
            "upload_method": "file_upload",
            "upload_timestamp": chrono::Utc::now()
        })),
    };
    
    let mut templates = state.templates.lock().unwrap();
    templates.insert(new_template.id, new_template.clone());
    
    info!("✅ Uploaded new template: {}", template_name);
    
    Ok(Json(json!({
        "status": "success",
        "message": "Template uploaded successfully",
        "template": new_template
    })))
} 
