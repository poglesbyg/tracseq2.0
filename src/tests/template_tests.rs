#[cfg(test)]
use crate::models::template::{CreateTemplate, Template};
use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_template() {
        let template = Template {
            id: Uuid::new_v4(),
            name: "Test Template".to_string(),
            description: Some("Test Description".to_string()),
            file_path: "/path/to/template.xlsx".to_string(),
            file_type: "xlsx".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: json!({
                "version": "1.0",
                "columns": ["sample_id", "concentration", "volume"]
            }),
        };

        assert_eq!(template.name, "Test Template");
        assert_eq!(template.file_type, "xlsx");
        assert!(template.metadata.get("version").is_some());
    }

    #[test]
    fn test_create_template_request() {
        let create_template = CreateTemplate {
            name: "New Template".to_string(),
            description: Some("New Description".to_string()),
            file_path: "/path/to/new_template.xlsx".to_string(),
            file_type: "xlsx".to_string(),
            metadata: Some(json!({
                "version": "1.0",
                "columns": ["sample_id", "concentration", "volume"]
            })),
        };

        assert_eq!(create_template.name, "New Template");
        assert!(create_template.description.is_some());
        assert!(create_template.metadata.is_some());
    }

    #[test]
    fn test_template_validation() {
        let template = Template {
            id: Uuid::new_v4(),
            name: "Valid Template Name".to_string(),
            description: None,
            file_path: "/path/to/template.xlsx".to_string(),
            file_type: "xlsx".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: json!({}),
        };

        // Basic validation checks
        assert!(
            !template.name.is_empty(),
            "Template name should not be empty"
        );
        assert!(
            !template.file_path.is_empty(),
            "File path should not be empty"
        );
        assert!(
            !template.file_type.is_empty(),
            "File type should not be empty"
        );
    }
}
