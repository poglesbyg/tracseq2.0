use reqwest::{Client, Error as ReqwestError};
use serde_json::json;
use tracing::{warn, error};

use crate::{
    error::{Result, SequencingError},
    models::SequencingJob,
};

#[derive(Debug, Clone)]
pub struct NotificationClient {
    client: Client,
    base_url: String,
}

impl NotificationClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }

    pub async fn health_check(&self) -> Result<()> {
        let url = format!("{}/health", self.base_url);
        
        match self.client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    Ok(())
                } else {
                    Err(SequencingError::ExternalService { 
                        service: "notification".to_string(),
                        message: format!("Health check failed: {}", response.status())
                    })
                }
            }
            Err(e) => {
                Err(SequencingError::ExternalService {
                    service: "notification".to_string(),
                    message: format!("Health check request failed: {}", e)
                })
            }
        }
    }

    pub async fn send_job_created_notification(&self, job: &SequencingJob) -> Result<()> {
        let notification = json!({
            "event_type": "job_created",
            "entity_type": "sequencing_job",
            "entity_id": job.id,
            "title": format!("Sequencing Job Created: {}", job.name),
            "message": format!("A new sequencing job '{}' has been created with {} priority", job.name, serde_json::to_string(&job.priority).unwrap_or_default()),
            "data": {
                "job_id": job.id,
                "job_name": job.name,
                "platform_id": job.platform_id,
                "priority": job.priority,
                "status": job.status
            },
            "channels": ["email", "in_app"],
            "recipients": [job.created_by]
        });

        self.send_notification(notification).await
    }

    pub async fn send_job_status_notification(&self, job: &SequencingJob) -> Result<()> {
        let notification = json!({
            "event_type": "job_status_changed",
            "entity_type": "sequencing_job",
            "entity_id": job.id,
            "title": format!("Job Status Update: {}", job.name),
            "message": format!("Sequencing job '{}' status changed to {}", job.name, serde_json::to_string(&job.status).unwrap_or_default()),
            "data": {
                "job_id": job.id,
                "job_name": job.name,
                "platform_id": job.platform_id,
                "previous_status": null, // Would need to be passed in
                "current_status": job.status
            },
            "channels": ["email", "in_app"],
            "recipients": [job.created_by]
        });

        // Add assigned user if exists
        if let Some(assigned_to) = job.assigned_to {
            // Would modify recipients to include assigned_to
        }

        self.send_notification(notification).await
    }

    pub async fn send_job_cancelled_notification(&self, job: &SequencingJob) -> Result<()> {
        let notification = json!({
            "event_type": "job_cancelled",
            "entity_type": "sequencing_job",
            "entity_id": job.id,
            "title": format!("Job Cancelled: {}", job.name),
            "message": format!("Sequencing job '{}' has been cancelled", job.name),
            "data": {
                "job_id": job.id,
                "job_name": job.name,
                "platform_id": job.platform_id,
                "cancelled_at": job.actual_completion
            },
            "channels": ["email", "in_app"],
            "recipients": [job.created_by]
        });

        self.send_notification(notification).await
    }

    pub async fn send_job_completed_notification(&self, job: &SequencingJob) -> Result<()> {
        let notification = json!({
            "event_type": "job_completed",
            "entity_type": "sequencing_job",
            "entity_id": job.id,
            "title": format!("Job Completed: {}", job.name),
            "message": format!("Sequencing job '{}' has completed successfully", job.name),
            "data": {
                "job_id": job.id,
                "job_name": job.name,
                "platform_id": job.platform_id,
                "completed_at": job.actual_completion,
                "duration": job.actual_start.and_then(|start| 
                    job.actual_completion.map(|end| 
                        (end - start).num_minutes()
                    )
                )
            },
            "channels": ["email", "in_app"],
            "recipients": [job.created_by]
        });

        self.send_notification(notification).await
    }

    pub async fn send_job_failed_notification(&self, job: &SequencingJob, error_message: Option<&str>) -> Result<()> {
        let notification = json!({
            "event_type": "job_failed",
            "entity_type": "sequencing_job",
            "entity_id": job.id,
            "title": format!("Job Failed: {}", job.name),
            "message": format!("Sequencing job '{}' has failed{}", 
                job.name, 
                error_message.map(|msg| format!(": {}", msg)).unwrap_or_default()
            ),
            "data": {
                "job_id": job.id,
                "job_name": job.name,
                "platform_id": job.platform_id,
                "failed_at": job.actual_completion,
                "error_message": error_message
            },
            "channels": ["email", "in_app", "slack"], // High priority - include Slack
            "recipients": [job.created_by]
        });

        self.send_notification(notification).await
    }

    async fn send_notification(&self, notification: serde_json::Value) -> Result<()> {
        let url = format!("{}/notifications", self.base_url);
        
        match self.client
            .post(&url)
            .json(&notification)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    Ok(())
                } else {
                    warn!("Failed to send notification: {}", response.status());
                    // Don't fail the main operation if notification fails
                    Ok(())
                }
            }
            Err(e) => {
                error!("Notification request failed: {}", e);
                // Don't fail the main operation if notification fails
                Ok(())
            }
        }
    }
}
