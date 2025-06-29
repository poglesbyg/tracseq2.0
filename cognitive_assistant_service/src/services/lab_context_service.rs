use crate::models::{LabContext, DataPoint, SourceType};
use anyhow::Result;
use chrono::Utc;
use sqlx::PgPool;
use std::collections::HashMap;
use tracing::{info, warn};

pub struct LabContextService {
    database: PgPool,
}

impl LabContextService {
    pub fn new(database: PgPool) -> Self {
        Self { database }
    }

    pub async fn get_enriched_context(&self, base_context: &LabContext) -> Result<LabContext> {
        let mut enriched_context = base_context.clone();

        // Enrich with current sample data
        if let Ok(samples) = self.get_current_samples(&base_context.user_department).await {
            enriched_context.current_samples = samples;
        }

        // Enrich with active workflows
        if let Ok(workflows) = self.get_active_workflows(&base_context.lab_location).await {
            enriched_context.active_workflows = workflows;
        }

        // Enrich with recent activities
        if let Ok(activities) = self.get_recent_activities(&base_context.user_department).await {
            enriched_context.recent_activities = activities;
        }

        Ok(enriched_context)
    }

    pub async fn get_relevant_data_points(&self, query: &str, context: &LabContext) -> Result<Vec<DataPoint>> {
        let mut data_points = Vec::new();

        // Sample-related data
        if query.to_lowercase().contains("sample") {
            data_points.extend(self.get_sample_statistics(context).await?);
        }

        // Storage-related data
        if query.to_lowercase().contains("storage") || query.to_lowercase().contains("capacity") {
            data_points.extend(self.get_storage_metrics(context).await?);
        }

        // Workflow-related data
        if query.to_lowercase().contains("workflow") || query.to_lowercase().contains("process") {
            data_points.extend(self.get_workflow_metrics(context).await?);
        }

        // Quality-related data
        if query.to_lowercase().contains("quality") || query.to_lowercase().contains("qc") {
            data_points.extend(self.get_quality_metrics(context).await?);
        }

        Ok(data_points)
    }

    async fn get_current_samples(&self, department: &str) -> Result<Vec<String>> {
        let query = r#"
            SELECT name, barcode, status
            FROM samples 
            WHERE department = $1 
            AND status IN ('pending', 'in_process', 'validated')
            ORDER BY created_at DESC 
            LIMIT 10
        "#;

        let rows = sqlx::query(query)
            .bind(department)
            .fetch_all(&self.database)
            .await?;

        let samples: Vec<String> = rows
            .iter()
            .map(|row| {
                let name: String = row.get("name");
                let barcode: String = row.get("barcode");
                let status: String = row.get("status");
                format!("{} ({}) - {}", name, barcode, status)
            })
            .collect();

        Ok(samples)
    }

    async fn get_active_workflows(&self, lab_location: &str) -> Result<Vec<String>> {
        // Simulate workflow data - in production this would query actual workflow tables
        let workflows = vec![
            "Sample Processing Workflow #12345".to_string(),
            "QC Validation Workflow #12346".to_string(),
            "Sequencing Preparation #12347".to_string(),
        ];

        Ok(workflows)
    }

    async fn get_recent_activities(&self, department: &str) -> Result<Vec<String>> {
        // Simulate recent activity data
        let activities = vec![
            "Sample SMPL-001234 created 2 hours ago".to_string(),
            "Storage assignment completed for 5 samples".to_string(),
            "QC validation passed for batch #456".to_string(),
        ];

        Ok(activities)
    }

    async fn get_sample_statistics(&self, context: &LabContext) -> Result<Vec<DataPoint>> {
        let query = r#"
            SELECT 
                COUNT(*) as total_samples,
                COUNT(CASE WHEN status = 'pending' THEN 1 END) as pending_samples,
                COUNT(CASE WHEN status = 'validated' THEN 1 END) as validated_samples,
                COUNT(CASE WHEN status = 'in_storage' THEN 1 END) as stored_samples
            FROM samples 
            WHERE department = $1
        "#;

        let row = sqlx::query(query)
            .bind(&context.user_department)
            .fetch_one(&self.database)
            .await?;

        let total_samples: i64 = row.get("total_samples");
        let pending_samples: i64 = row.get("pending_samples");
        let validated_samples: i64 = row.get("validated_samples");
        let stored_samples: i64 = row.get("stored_samples");

        Ok(vec![
            DataPoint {
                data_type: "total_samples".to_string(),
                value: serde_json::json!(total_samples),
                source: "database".to_string(),
                timestamp: Utc::now(),
                relevance_score: 0.9,
            },
            DataPoint {
                data_type: "pending_samples".to_string(),
                value: serde_json::json!(pending_samples),
                source: "database".to_string(),
                timestamp: Utc::now(),
                relevance_score: 0.85,
            },
            DataPoint {
                data_type: "validated_samples".to_string(),
                value: serde_json::json!(validated_samples),
                source: "database".to_string(),
                timestamp: Utc::now(),
                relevance_score: 0.8,
            },
            DataPoint {
                data_type: "stored_samples".to_string(),
                value: serde_json::json!(stored_samples),
                source: "database".to_string(),
                timestamp: Utc::now(),
                relevance_score: 0.8,
            },
        ])
    }

    async fn get_storage_metrics(&self, context: &LabContext) -> Result<Vec<DataPoint>> {
        // Simulate storage metrics - in production this would query storage tables
        Ok(vec![
            DataPoint {
                data_type: "storage_capacity_utilization".to_string(),
                value: serde_json::json!(75.5),
                source: "storage_monitoring".to_string(),
                timestamp: Utc::now(),
                relevance_score: 0.9,
            },
            DataPoint {
                data_type: "freezer_minus80_capacity".to_string(),
                value: serde_json::json!({"used": 450, "total": 600}),
                source: "storage_monitoring".to_string(),
                timestamp: Utc::now(),
                relevance_score: 0.85,
            },
            DataPoint {
                data_type: "temperature_alerts_24h".to_string(),
                value: serde_json::json!(0),
                source: "iot_sensors".to_string(),
                timestamp: Utc::now(),
                relevance_score: 0.8,
            },
        ])
    }

    async fn get_workflow_metrics(&self, context: &LabContext) -> Result<Vec<DataPoint>> {
        // Simulate workflow performance metrics
        Ok(vec![
            DataPoint {
                data_type: "average_processing_time".to_string(),
                value: serde_json::json!({"value": 4.2, "unit": "hours"}),
                source: "workflow_analytics".to_string(),
                timestamp: Utc::now(),
                relevance_score: 0.85,
            },
            DataPoint {
                data_type: "workflow_efficiency".to_string(),
                value: serde_json::json!(92.5),
                source: "workflow_analytics".to_string(),
                timestamp: Utc::now(),
                relevance_score: 0.8,
            },
            DataPoint {
                data_type: "bottleneck_stages".to_string(),
                value: serde_json::json!(["QC_validation", "storage_assignment"]),
                source: "workflow_analytics".to_string(),
                timestamp: Utc::now(),
                relevance_score: 0.9,
            },
        ])
    }

    async fn get_quality_metrics(&self, context: &LabContext) -> Result<Vec<DataPoint>> {
        // Simulate quality control metrics
        Ok(vec![
            DataPoint {
                data_type: "qc_pass_rate".to_string(),
                value: serde_json::json!(97.8),
                source: "qc_system".to_string(),
                timestamp: Utc::now(),
                relevance_score: 0.95,
            },
            DataPoint {
                data_type: "qc_processing_time".to_string(),
                value: serde_json::json!({"average": 45, "unit": "minutes"}),
                source: "qc_system".to_string(),
                timestamp: Utc::now(),
                relevance_score: 0.8,
            },
            DataPoint {
                data_type: "quality_alerts_today".to_string(),
                value: serde_json::json!(2),
                source: "qc_system".to_string(),
                timestamp: Utc::now(),
                relevance_score: 0.9,
            },
        ])
    }

    pub async fn get_lab_knowledge_base(&self) -> Result<HashMap<String, String>> {
        let mut knowledge = HashMap::new();

        // Laboratory best practices
        knowledge.insert(
            "sample_storage_guidelines".to_string(),
            "DNA/RNA samples: -80°C for long-term, -20°C for short-term. Proteins: -80°C or -20°C depending on stability. Avoid freeze-thaw cycles.".to_string()
        );

        knowledge.insert(
            "temperature_zones".to_string(),
            "-80°C: Long-term storage, -20°C: Medium-term storage, 4°C: Short-term storage, RT: Ambient storage, 37°C: Incubation".to_string()
        );

        knowledge.insert(
            "quality_standards".to_string(),
            "DNA quality: A260/A280 ratio 1.8-2.0, A260/A230 ratio >2.0. RNA quality: RIN score >7 for most applications.".to_string()
        );

        knowledge.insert(
            "compliance_requirements".to_string(),
            "HIPAA: Protect patient data, FDA 21 CFR Part 11: Electronic records, GLP: Good Laboratory Practice standards".to_string()
        );

        knowledge.insert(
            "workflow_optimization".to_string(),
            "Batch processing improves efficiency. Automated QC reduces errors. Predictive maintenance prevents downtime.".to_string()
        );

        Ok(knowledge)
    }

    pub async fn store_query_history(&self, user_id: uuid::Uuid, query: &str, response: &str, confidence: f64, response_time_ms: u64) -> Result<()> {
        let query_sql = r#"
            INSERT INTO query_history (id, user_id, query, response, confidence, response_time_ms, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#;

        sqlx::query(query_sql)
            .bind(uuid::Uuid::new_v4())
            .bind(user_id)
            .bind(query)
            .bind(response)
            .bind(confidence)
            .bind(response_time_ms as i64)
            .bind(Utc::now())
            .execute(&self.database)
            .await?;

        info!("✅ Stored query history for user {}", user_id);
        Ok(())
    }

    pub async fn get_user_context_from_history(&self, user_id: uuid::Uuid, limit: i64) -> Result<Vec<String>> {
        let query = r#"
            SELECT query, response 
            FROM query_history 
            WHERE user_id = $1 
            ORDER BY created_at DESC 
            LIMIT $2
        "#;

        let rows = sqlx::query(query)
            .bind(user_id)
            .bind(limit)
            .fetch_all(&self.database)
            .await?;

        let context: Vec<String> = rows
            .iter()
            .map(|row| {
                let query: String = row.get("query");
                let response: String = row.get("response");
                format!("Q: {} | A: {}", query, response.chars().take(100).collect::<String>())
            })
            .collect();

        Ok(context)
    }
} 