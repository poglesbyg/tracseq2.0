#[macro_use]
pub mod test_utils;

#[cfg(test)]
pub mod unit {
    pub mod test_ollama_service;
    pub mod test_lab_context_service;
    pub mod test_cognitive_handlers;
    pub mod test_models;
}

#[cfg(test)]
pub mod integration {
    pub mod test_ai_queries;
    pub mod test_proactive_suggestions;
}

// Re-export test utilities
pub use test_utils::*;