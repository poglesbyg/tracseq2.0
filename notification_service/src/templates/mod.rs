//! Notification templates for different notification types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Template for notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationTemplate {
    pub id: String,
    pub name: String,
    pub category: TemplateCategory,
    pub subject_template: String,
    pub body_template: String,
    pub variables: Vec<TemplateVariable>,
    pub metadata: serde_json::Value,
}

/// Template categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemplateCategory {
    SampleProcessing,
    QualityControl,
    SequencingUpdate,
    SystemAlert,
    UserManagement,
    StorageAlert,
    General,
}

/// Template variable definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub default_value: Option<String>,
}

/// Template engine for rendering notifications
pub struct TemplateEngine {
    templates: HashMap<String, NotificationTemplate>,
}

impl TemplateEngine {
    /// Create a new template engine
    pub fn new() -> Self {
        let mut engine = Self {
            templates: HashMap::new(),
        };
        
        // Load default templates
        engine.load_default_templates();
        engine
    }
    
    /// Load default templates
    fn load_default_templates(&mut self) {
        // Sample processing template
        self.templates.insert(
            "sample_received".to_string(),
            NotificationTemplate {
                id: "sample_received".to_string(),
                name: "Sample Received".to_string(),
                category: TemplateCategory::SampleProcessing,
                subject_template: "Sample {{sample_id}} Received".to_string(),
                body_template: "Your sample {{sample_id}} has been received and is being processed.".to_string(),
                variables: vec![
                    TemplateVariable {
                        name: "sample_id".to_string(),
                        description: "Sample identifier".to_string(),
                        required: true,
                        default_value: None,
                    }
                ],
                metadata: serde_json::json!({}),
            }
        );
        
        // QC result template
        self.templates.insert(
            "qc_complete".to_string(),
            NotificationTemplate {
                id: "qc_complete".to_string(),
                name: "QC Complete".to_string(),
                category: TemplateCategory::QualityControl,
                subject_template: "QC Results for Sample {{sample_id}}".to_string(),
                body_template: "Quality control for sample {{sample_id}} is complete. Status: {{qc_status}}".to_string(),
                variables: vec![
                    TemplateVariable {
                        name: "sample_id".to_string(),
                        description: "Sample identifier".to_string(),
                        required: true,
                        default_value: None,
                    },
                    TemplateVariable {
                        name: "qc_status".to_string(),
                        description: "QC pass/fail status".to_string(),
                        required: true,
                        default_value: None,
                    }
                ],
                metadata: serde_json::json!({}),
            }
        );
    }
    
    /// Render a template with variables
    pub fn render(&self, template_id: &str, variables: &HashMap<String, String>) -> Result<(String, String), String> {
        let template = self.templates.get(template_id)
            .ok_or_else(|| format!("Template not found: {}", template_id))?;
        
        // Simple variable substitution
        let mut subject = template.subject_template.clone();
        let mut body = template.body_template.clone();
        
        for (key, value) in variables {
            let placeholder = format!("{{{{{}}}}}", key);
            subject = subject.replace(&placeholder, value);
            body = body.replace(&placeholder, value);
        }
        
        Ok((subject, body))
    }
    
    /// Get all templates
    pub fn get_templates(&self) -> Vec<&NotificationTemplate> {
        self.templates.values().collect()
    }
    
    /// Get template by ID
    pub fn get_template(&self, id: &str) -> Option<&NotificationTemplate> {
        self.templates.get(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_template_rendering() {
        let engine = TemplateEngine::new();
        
        let mut variables = HashMap::new();
        variables.insert("sample_id".to_string(), "SMP-12345".to_string());
        
        let result = engine.render("sample_received", &variables);
        assert!(result.is_ok());
        
        let (subject, body) = result.unwrap();
        assert_eq!(subject, "Sample SMP-12345 Received");
        assert!(body.contains("SMP-12345"));
    }
}