pub mod handlers;
pub mod models;
pub mod services;

pub use models::{
    LabQuery, LabQueryType, LabContext, UserRole, Department,
    ContextualResponse, SuggestionType, ProactiveSuggestion,
    OllamaRequest, OllamaResponse, OllamaChat, OllamaMessage,
    OllamaEmbedding, OllamaEmbeddingRequest, OllamaEmbeddingResponse,
    ServiceError, ServiceErrorKind
};

pub use services::{
    lab_context_service::LabContextService,
    ollama_service::OllamaService,
};

// Re-export main app state for tests
#[derive(Clone)]
pub struct AppState {
    pub ollama_url: String,
    pub database_url: String,
}

// Re-export handler functions for testing
pub use handlers::cognitive_handler::{
    handle_lab_query,
    generate_proactive_suggestions,
    analyze_lab_context,
    get_contextual_help,
};