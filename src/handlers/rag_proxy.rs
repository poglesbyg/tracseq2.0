use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    Json,
};
use reqwest::Client;
use serde_json::Value;

use crate::AppComponents;

/// Get RAG submissions by proxying to the RAG API Bridge
pub async fn get_rag_submissions(
    State(_state): State<AppComponents>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let client = Client::new();
    let url = "http://localhost:3002/api/rag/submissions";

    match client.get(url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<Value>().await {
                    Ok(data) => Ok(Json(data)),
                    Err(e) => Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to parse RAG response: {}", e),
                    )),
                }
            } else {
                Err((
                    StatusCode::from_u16(response.status().as_u16())
                        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                    format!("RAG API Bridge error: {}", response.status()),
                ))
            }
        }
        Err(e) => Err((
            StatusCode::SERVICE_UNAVAILABLE,
            format!("Failed to connect to RAG API Bridge: {}", e),
        )),
    }
}

/// Process document by proxying to the RAG API Bridge
pub async fn process_rag_document(
    State(_state): State<AppComponents>,
    multipart: Multipart,
) -> Result<Json<Value>, (StatusCode, String)> {
    let client = Client::new();
    let url = "http://localhost:3002/api/rag/process";

    // Convert axum multipart to reqwest multipart
    let mut form = reqwest::multipart::Form::new();
    let mut multipart_stream = multipart;

    while let Some(field) = multipart_stream.next_field().await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("Failed to read multipart field: {}", e),
        )
    })? {
        let name = field.name().unwrap_or("file").to_string();
        let filename = field.file_name().map(|s| s.to_string());
        let content_type = field.content_type().map(|s| s.to_string());
        let data = field.bytes().await.map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                format!("Failed to read field data: {}", e),
            )
        })?;

        let mut part = reqwest::multipart::Part::bytes(data.to_vec());
        if let Some(filename) = filename {
            part = part.file_name(filename);
        }

        form = form.part(name, part);
    }

    match client.post(url).multipart(form).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<Value>().await {
                    Ok(data) => Ok(Json(data)),
                    Err(e) => Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to parse RAG response: {}", e),
                    )),
                }
            } else {
                let status_code = response.status();
                let error_text = response.text().await.unwrap_or_default();
                Err((
                    StatusCode::from_u16(status_code.as_u16())
                        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                    format!("RAG API Bridge error: {}", error_text),
                ))
            }
        }
        Err(e) => Err((
            StatusCode::SERVICE_UNAVAILABLE,
            format!("Failed to connect to RAG API Bridge: {}", e),
        )),
    }
}

/// Get RAG statistics by proxying to the RAG API Bridge
pub async fn get_rag_stats(
    State(_state): State<AppComponents>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let client = Client::new();
    let url = "http://localhost:3002/api/rag/stats";

    match client.get(url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<Value>().await {
                    Ok(data) => Ok(Json(data)),
                    Err(e) => Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to parse RAG response: {}", e),
                    )),
                }
            } else {
                Err((
                    StatusCode::from_u16(response.status().as_u16())
                        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                    format!("RAG API Bridge error: {}", response.status()),
                ))
            }
        }
        Err(e) => Err((
            StatusCode::SERVICE_UNAVAILABLE,
            format!("Failed to connect to RAG API Bridge: {}", e),
        )),
    }
}

/// Get RAG system health by proxying to the RAG API Bridge
pub async fn get_rag_health(
    State(_state): State<AppComponents>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let client = Client::new();
    let url = "http://localhost:3002/health";

    match client.get(url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<Value>().await {
                    Ok(data) => Ok(Json(data)),
                    Err(e) => Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to parse RAG response: {}", e),
                    )),
                }
            } else {
                Err((
                    StatusCode::from_u16(response.status().as_u16())
                        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                    format!("RAG API Bridge error: {}", response.status()),
                ))
            }
        }
        Err(e) => Err((
            StatusCode::SERVICE_UNAVAILABLE,
            format!("Failed to connect to RAG API Bridge: {}", e),
        )),
    }
}
