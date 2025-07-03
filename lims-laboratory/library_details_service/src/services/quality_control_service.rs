use crate::error::{Result, ServiceError};
use crate::models::*;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use validator::Validate;
use std::collections::HashMap;

#[derive(Clone)]
pub struct QualityControlService {
    pool: PgPool,
}

impl QualityControlService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_qc_metric(&self, request: CreateQCMetricRequest) -> Result<QualityControlMetric> {
        request.validate().map_err(|e| ServiceError::Validation {
            message: e.to_string(),
        })?;

        let id = Uuid::new_v4();
        let now = Utc::now();

        // Determine QC status based on thresholds
        let status = self.evaluate_qc_status(
            request.value,
            request.threshold_min,
            request.threshold_max,
        );

        let metric = sqlx::query_as::<_, QualityControlMetric>(
            r#"
            INSERT INTO quality_control_metrics (
                id, library_id, metric_type, value, unit,
                threshold_min, threshold_max, status, measured_at,
                equipment_id, operator_id, notes, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING id, library_id, metric_type, value, unit,
                      threshold_min, threshold_max, status, measured_at,
                      equipment_id, operator_id, notes, created_at
            "#,
        )
        .bind(id)
        .bind(request.library_id)
        .bind(&request.metric_type)
        .bind(request.value)
        .bind(&request.unit)
        .bind(request.threshold_min)
        .bind(request.threshold_max)
        .bind(format!("{:?}", status).to_lowercase())
        .bind(now)
        .bind(&request.equipment_id)
        .bind(&request.operator_id)
        .bind(&request.notes)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        tracing::info!("Created QC metric: {} for library: {}", metric.id, metric.library_id);
        Ok(metric)
    }

    pub async fn list_qc_metrics(&self, library_id: Option<Uuid>, metric_type: Option<String>) -> Result<Vec<QualityControlMetric>> {
        let metrics = sqlx::query_as::<_, QualityControlMetric>(
            r#"
            SELECT id, library_id, metric_type, value, unit,
                   threshold_min, threshold_max, status, measured_at,
                   equipment_id, operator_id, notes, created_at
            FROM quality_control_metrics
            WHERE ($1::uuid IS NULL OR library_id = $1)
              AND ($2::text IS NULL OR metric_type = $2)
            ORDER BY created_at DESC
            "#,
        )
        .bind(library_id)
        .bind(&metric_type)
        .fetch_all(&self.pool)
        .await?;

        Ok(metrics)
    }

    pub async fn assess_library_quality(&self, library_id: Uuid) -> Result<QualityReport> {
        let metrics = self.list_qc_metrics(Some(library_id), None).await?;
        
        if metrics.is_empty() {
            return Err(ServiceError::QualityControlFailed {
                reason: "No quality control metrics found for library".to_string(),
            });
        }

        let mut overall_status = QCStatus::Pass;
        let mut recommendations = Vec::new();

        // Evaluate overall status based on individual metrics
        for metric in &metrics {
            match metric.status {
                QCStatus::Fail => {
                    overall_status = QCStatus::Fail;
                    recommendations.push(format!("Failed metric: {} with value {}", metric.metric_type, metric.value));
                }
                QCStatus::Warning => {
                    if overall_status == QCStatus::Pass {
                        overall_status = QCStatus::Warning;
                    }
                    recommendations.push(format!("Warning for metric: {} with value {}", metric.metric_type, metric.value));
                }
                QCStatus::Pending => {
                    if matches!(overall_status, QCStatus::Pass) {
                        overall_status = QCStatus::Pending;
                    }
                }
                QCStatus::Pass => {
                    // No action needed for pass
                }
            }
        }

        // Add general recommendations based on overall status
        match overall_status {
            QCStatus::Pass => {
                recommendations.push("Library meets all quality requirements".to_string());
            }
            QCStatus::Warning => {
                recommendations.push("Library has some quality concerns but may proceed with caution".to_string());
            }
            QCStatus::Fail => {
                recommendations.push("Library fails quality requirements and should not proceed".to_string());
            }
            QCStatus::Pending => {
                recommendations.push("Library quality assessment is incomplete".to_string());
            }
        }

        Ok(QualityReport {
            library_id,
            overall_status,
            metrics,
            recommendations,
            generated_at: Utc::now(),
        })
    }

    pub async fn get_quality_thresholds(&self) -> Result<HashMap<String, (Option<f64>, Option<f64>)>> {
        let mut thresholds = HashMap::new();

        // Default quality thresholds for common metrics
        thresholds.insert("concentration".to_string(), (Some(0.1), Some(1000.0))); // ng/μL
        thresholds.insert("volume".to_string(), (Some(1.0), Some(200.0))); // μL
        thresholds.insert("fragment_size".to_string(), (Some(100.0), Some(1000.0))); // bp
        thresholds.insert("quality_score".to_string(), (Some(0.7), Some(1.0))); // 0-1 scale
        thresholds.insert("rqn".to_string(), (Some(7.0), Some(10.0))); // RNA Quality Number
        thresholds.insert("dv200".to_string(), (Some(30.0), Some(100.0))); // DNA Integrity Number
        thresholds.insert("molarity".to_string(), (Some(1.0), Some(100.0))); // nM

        Ok(thresholds)
    }

    pub async fn generate_quality_report(&self, library_id: Uuid) -> Result<QualityReport> {
        self.assess_library_quality(library_id).await
    }

    fn evaluate_qc_status(&self, value: f64, threshold_min: Option<f64>, threshold_max: Option<f64>) -> QCStatus {
        match (threshold_min, threshold_max) {
            (Some(min), Some(max)) => {
                if value < min || value > max {
                    QCStatus::Fail
                } else if value < min * 1.1 || value > max * 0.9 {
                    QCStatus::Warning
                } else {
                    QCStatus::Pass
                }
            }
            (Some(min), None) => {
                if value < min {
                    QCStatus::Fail
                } else if value < min * 1.1 {
                    QCStatus::Warning
                } else {
                    QCStatus::Pass
                }
            }
            (None, Some(max)) => {
                if value > max {
                    QCStatus::Fail
                } else if value > max * 0.9 {
                    QCStatus::Warning
                } else {
                    QCStatus::Pass
                }
            }
            (None, None) => QCStatus::Pass, // No thresholds defined
        }
    }

    pub async fn get_library_qc_summary(&self, library_id: Uuid) -> Result<HashMap<String, QCStatus>> {
        let metrics = self.list_qc_metrics(Some(library_id), None).await?;
        
        let mut summary = HashMap::new();
        
        for metric in metrics {
            summary.insert(metric.metric_type, metric.status);
        }

        Ok(summary)
    }

    pub async fn validate_library_for_sequencing(&self, library_id: Uuid) -> Result<bool> {
        let report = self.assess_library_quality(library_id).await?;
        
        match report.overall_status {
            QCStatus::Pass => Ok(true),
            QCStatus::Warning => {
                // Allow warning status libraries with manual approval
                tracing::warn!("Library {} has quality warnings but may proceed", library_id);
                Ok(true)
            }
            QCStatus::Fail => {
                tracing::error!("Library {} failed quality control", library_id);
                Ok(false)
            }
            QCStatus::Pending => {
                tracing::info!("Library {} quality assessment is pending", library_id);
                Ok(false)
            }
        }
    }
}