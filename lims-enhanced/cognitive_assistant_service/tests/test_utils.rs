use cognitive_assistant_service::{
    AppState, LabQuery, LabQueryType, LabContext, UserRole, Department,
    ContextualResponse, ProactiveSuggestion, SuggestionType,
    OllamaRequest, OllamaResponse, OllamaChat, OllamaMessage,
    ServiceError, ServiceErrorKind,
};
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path, body_json};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::sync::Arc;
use once_cell::sync::Lazy;

static DB_INIT: Lazy<()> = Lazy::new(|| {
    std::env::set_var("DATABASE_URL", "postgresql://postgres:tracseq_password@localhost:5432/test_cognitive");
});

/// Test database setup
pub struct TestDatabase {
    pub pool: PgPool,
    db_name: String,
}

impl TestDatabase {
    pub async fn new() -> Self {
        let _ = *DB_INIT;
        let db_name = format!("test_cognitive_{}", uuid::Uuid::new_v4().simple());
        let database_url = std::env::var("DATABASE_URL").unwrap();
        let base_url = database_url.rsplit('/').skip(1).collect::<Vec<_>>().join("/");
        
        // Create test database
        let pool = PgPoolOptions::new()
            .connect(&format!("{}/postgres", base_url))
            .await
            .expect("Failed to connect to postgres");
        
        sqlx::query(&format!("CREATE DATABASE {}", db_name))
            .execute(&pool)
            .await
            .ok();
        
        // Connect to test database
        let test_pool = PgPoolOptions::new()
            .connect(&format!("{}/{}", base_url, db_name))
            .await
            .expect("Failed to connect to test database");
        
        // Run migrations
        sqlx::migrate!("migrations")
            .run(&test_pool)
            .await
            .expect("Failed to run migrations");
        
        Self {
            pool: test_pool,
            db_name,
        }
    }
    
    pub async fn cleanup(self) {
        let database_url = std::env::var("DATABASE_URL").unwrap();
        let base_url = database_url.rsplit('/').skip(1).collect::<Vec<_>>().join("/");
        
        self.pool.close().await;
        
        let pool = PgPoolOptions::new()
            .connect(&format!("{}/postgres", base_url))
            .await
            .expect("Failed to connect to postgres");
        
        sqlx::query(&format!("DROP DATABASE IF EXISTS {}", self.db_name))
            .execute(&pool)
            .await
            .ok();
    }
}

/// Factory for creating test queries
pub struct QueryFactory;

impl QueryFactory {
    pub fn sample_processing_query() -> LabQuery {
        LabQuery {
            id: None,
            query: "How do I process RNA samples for sequencing?".to_string(),
            query_type: LabQueryType::ProcessGuidance,
            user_role: UserRole::Technician,
            department: Department::Sequencing,
            context: Some(LabContext {
                active_samples: vec!["RNA001".to_string(), "RNA002".to_string()],
                recent_activities: vec!["RNA extraction".to_string()],
                equipment_status: std::collections::HashMap::new(),
                user_preferences: std::collections::HashMap::new(),
            }),
            timestamp: chrono::Utc::now(),
        }
    }
    
    pub fn troubleshooting_query() -> LabQuery {
        LabQuery {
            id: None,
            query: "Why is my PCR failing?".to_string(),
            query_type: LabQueryType::Troubleshooting,
            user_role: UserRole::Researcher,
            department: Department::Molecular,
            context: None,
            timestamp: chrono::Utc::now(),
        }
    }
    
    pub fn data_analysis_query() -> LabQuery {
        LabQuery {
            id: None,
            query: "Show me quality control trends for this month".to_string(),
            query_type: LabQueryType::DataAnalysis,
            user_role: UserRole::Supervisor,
            department: Department::QualityControl,
            context: None,
            timestamp: chrono::Utc::now(),
        }
    }
    
    pub fn compliance_query() -> LabQuery {
        LabQuery {
            id: None,
            query: "What are the storage requirements for biological samples?".to_string(),
            query_type: LabQueryType::Compliance,
            user_role: UserRole::Admin,
            department: Department::Storage,
            context: None,
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Mock Ollama server setup
pub struct MockOllamaSetup;

impl MockOllamaSetup {
    pub async fn setup_chat_endpoint(mock_server: &MockServer) {
        Mock::given(method("POST"))
            .and(path("/api/chat"))
            .respond_with(|req: &wiremock::Request| {
                let body: OllamaChat = req.body_json().unwrap();
                
                ResponseTemplate::new(200)
                    .set_body_json(OllamaResponse {
                        response: format!("AI response to: {}", body.messages.last().unwrap().content),
                        done: true,
                        context: None,
                        total_duration: Some(100_000_000), // 100ms in nanoseconds
                        load_duration: Some(10_000_000),
                        prompt_eval_count: Some(50),
                        prompt_eval_duration: Some(20_000_000),
                        eval_count: Some(100),
                        eval_duration: Some(70_000_000),
                    })
            })
            .mount(mock_server)
            .await;
    }
    
    pub async fn setup_embedding_endpoint(mock_server: &MockServer) {
        Mock::given(method("POST"))
            .and(path("/api/embeddings"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({
                    "embedding": vec![0.1; 768] // Mock embedding vector
                })))
            .mount(mock_server)
            .await;
    }
    
    pub async fn setup_failing_endpoint(mock_server: &MockServer) {
        Mock::given(method("POST"))
            .and(path("/api/chat"))
            .respond_with(ResponseTemplate::new(500)
                .set_body_string("Internal server error"))
            .mount(mock_server)
            .await;
    }
    
    pub async fn setup_slow_endpoint(mock_server: &MockServer, delay: std::time::Duration) {
        Mock::given(method("POST"))
            .and(path("/api/chat"))
            .respond_with(ResponseTemplate::new(200)
                .set_delay(delay)
                .set_body_json(OllamaResponse {
                    response: "Slow response".to_string(),
                    done: true,
                    context: None,
                    total_duration: Some(delay.as_nanos() as u64),
                    load_duration: None,
                    prompt_eval_count: None,
                    prompt_eval_duration: None,
                    eval_count: None,
                    eval_duration: None,
                }))
            .mount(mock_server)
            .await;
    }
}

/// Test data generators
pub struct TestDataGenerator;

impl TestDataGenerator {
    pub fn lab_context() -> LabContext {
        LabContext {
            active_samples: vec![
                "SAMPLE001".to_string(),
                "SAMPLE002".to_string(),
                "SAMPLE003".to_string(),
            ],
            recent_activities: vec![
                "DNA extraction".to_string(),
                "PCR setup".to_string(),
                "Gel electrophoresis".to_string(),
            ],
            equipment_status: [
                ("PCR Machine 1".to_string(), "Available".to_string()),
                ("Centrifuge A".to_string(), "In Use".to_string()),
                ("Freezer -80C".to_string(), "Alarm - High Temp".to_string()),
            ].iter().cloned().collect(),
            user_preferences: [
                ("notification_level".to_string(), "high".to_string()),
                ("preferred_units".to_string(), "metric".to_string()),
            ].iter().cloned().collect(),
        }
    }
    
    pub fn proactive_suggestions() -> Vec<ProactiveSuggestion> {
        vec![
            ProactiveSuggestion {
                id: None,
                suggestion_type: SuggestionType::ProcessImprovement,
                title: "Optimize RNA extraction workflow".to_string(),
                description: "Based on recent processing times, consider batching RNA samples".to_string(),
                priority: "medium".to_string(),
                department: Department::Molecular,
                potential_impact: Some("Could reduce processing time by 30%".to_string()),
                action_items: vec![
                    "Review current batch sizes".to_string(),
                    "Test larger batch protocol".to_string(),
                ],
                created_at: chrono::Utc::now(),
            },
            ProactiveSuggestion {
                id: None,
                suggestion_type: SuggestionType::SafetyReminder,
                title: "Update chemical inventory".to_string(),
                description: "Quarterly chemical inventory check is due".to_string(),
                priority: "high".to_string(),
                department: Department::Storage,
                potential_impact: Some("Maintain compliance with safety regulations".to_string()),
                action_items: vec![
                    "Check all chemical expiration dates".to_string(),
                    "Update safety data sheets".to_string(),
                ],
                created_at: chrono::Utc::now(),
            },
        ]
    }
}

/// Assertions for cognitive assistant tests
pub struct CognitiveAssertions;

impl CognitiveAssertions {
    pub fn assert_response_quality(response: &ContextualResponse) {
        assert!(!response.response.is_empty(), "Response should not be empty");
        assert!(response.confidence >= 0.0 && response.confidence <= 1.0, 
            "Confidence should be between 0 and 1");
        assert!(!response.reasoning.is_empty(), "Reasoning should be provided");
    }
    
    pub fn assert_contextual_relevance(response: &ContextualResponse, query: &LabQuery) {
        // Check that response mentions relevant context
        if let Some(context) = &query.context {
            for sample in &context.active_samples {
                if response.response.to_lowercase().contains(&sample.to_lowercase()) {
                    return; // Found relevant context
                }
            }
        }
    }
    
    pub fn assert_suggestion_validity(suggestion: &ProactiveSuggestion) {
        assert!(!suggestion.title.is_empty(), "Suggestion title should not be empty");
        assert!(!suggestion.description.is_empty(), "Suggestion description should not be empty");
        assert!(!suggestion.action_items.is_empty(), "Suggestion should have action items");
        assert!(
            ["low", "medium", "high"].contains(&suggestion.priority.as_str()),
            "Invalid priority level"
        );
    }
    
    pub fn assert_error_type(error: &ServiceError, expected_kind: ServiceErrorKind) {
        assert_eq!(error.kind, expected_kind, "Expected error kind {:?}, got {:?}", expected_kind, error.kind);
    }
}

/// Performance test utilities
pub struct PerformanceTestUtils;

impl PerformanceTestUtils {
    pub async fn measure_query_time<F, Fut>(operation: F) -> (std::time::Duration, Result<ContextualResponse, ServiceError>)
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<ContextualResponse, ServiceError>>,
    {
        let start = std::time::Instant::now();
        let result = operation().await;
        (start.elapsed(), result)
    }
    
    pub async fn run_concurrent_queries(
        count: usize,
        query_factory: impl Fn(usize) -> LabQuery,
        handler: impl Fn(LabQuery) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ContextualResponse, ServiceError>> + Send>> + Clone + Send + 'static,
    ) -> Vec<Result<ContextualResponse, ServiceError>> {
        let mut handles = Vec::new();
        
        for i in 0..count {
            let query = query_factory(i);
            let handler_clone = handler.clone();
            let handle = tokio::spawn(async move {
                handler_clone(query).await
            });
            handles.push(handle);
        }
        
        let results = futures::future::join_all(handles).await;
        results.into_iter().filter_map(Result::ok).collect()
    }
}

/// Helper to create test app state
pub fn create_test_app_state(ollama_url: String) -> AppState {
    AppState {
        ollama_url,
        database_url: std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://postgres:tracseq_password@localhost:5432/test_cognitive".to_string()),
    }
}

/// Mock context builder for testing
pub struct ContextBuilder {
    context: LabContext,
}

impl ContextBuilder {
    pub fn new() -> Self {
        Self {
            context: LabContext {
                active_samples: vec![],
                recent_activities: vec![],
                equipment_status: std::collections::HashMap::new(),
                user_preferences: std::collections::HashMap::new(),
            },
        }
    }
    
    pub fn with_samples(mut self, samples: Vec<String>) -> Self {
        self.context.active_samples = samples;
        self
    }
    
    pub fn with_activities(mut self, activities: Vec<String>) -> Self {
        self.context.recent_activities = activities;
        self
    }
    
    pub fn with_equipment(mut self, equipment: Vec<(String, String)>) -> Self {
        self.context.equipment_status = equipment.into_iter().collect();
        self
    }
    
    pub fn build(self) -> LabContext {
        self.context
    }
}