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

        let metric = sqlx::query_as!(
            QualityControlMetric,
            r#"
            INSERT INTO quality_control_metrics (
                id, library_id, metric_type, value, unit,
                threshold_min, threshold_max, status,
                measured_at, equipment_id, operator_id, notes, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING id, library_id, metric_type, value, unit,
                      threshold_min, threshold_max, status as "status: QCStatus",
                      measured_at, equipment_id, operator_id, notes, created_at
            "#,
            id,
            request.library_id,
            request.metric_type,
            request.value,
            request.unit,
            request.threshold_min,
            request.threshold_max,
            status as QCStatus,
            now,
            request.equipment_id,
            request.operator_id,
            request.notes,
            now
        )
        .fetch_one(&self.pool)
        .await?;

        tracing::info!("Created QC metric: {} for library {}", metric.metric_type, metric.library_id);
        Ok(metric)
    }

    pub async fn list_qc_metrics(&self, library_id: Option<Uuid>, metric_type: Option<String>) -> Result<Vec<QualityControlMetric>> {
        let metrics = sqlx::query_as!(
            QualityControlMetric,
            r#"
            SELECT id, library_id, metric_type, value, unit,
                   threshold_min, threshold_max, status as "status: QCStatus",
                   measured_at, equipment_id, operator_id, notes, created_at
            FROM quality_control_metrics
            WHERE ($1::uuid IS NULL OR library_id = $1)
              AND ($2::text IS NULL OR metric_type = $2)
            ORDER BY measured_at DESC
            "#,
            library_id,
            metric_type
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(metrics)
    }

    pub async fn assess_library_quality(&self, library_id: Uuid) -> Result<QualityReport> {
        // Get all QC metrics for the library
        let metrics = self.list_qc_metrics(Some(library_id), None).await?;

        if metrics.is_empty() {
            return Ok(QualityReport {
                library_id,
                overall_status: QCStatus::Pending,
                metrics,
                recommendations: vec!["No quality control metrics available".to_string()],
                generated_at: Utc::now(),
            });
        }

        // Calculate overall status
        let overall_status = self.calculate_overall_status(&metrics);

        // Generate recommendations
        let recommendations = self.generate_recommendations(&metrics);

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

        // Default thresholds for common metrics
        thresholds.insert("concentration".to_string(), (Some(0.5), Some(1000.0))); // ng/μL
        thresholds.insert("260_280_ratio".to_string(), (Some(1.8), Some(2.2)));
        thresholds.insert("260_230_ratio".to_string(), (Some(2.0), Some(2.2)));
        thresholds.insert("rin".to_string(), (Some(7.0), None)); // RNA Integrity Number
        thresholds.insert("dv200".to_string(), (Some(30.0), None)); // % DNA > 200bp
        thresholds.insert("fragment_size".to_string(), (Some(150.0), Some(800.0))); // bp
        thresholds.insert("library_molarity".to_string(), (Some(2.0), Some(20.0))); // nM

        Ok(thresholds)
    }

    pub async fn generate_quality_report(&self, library_ids: Vec<Uuid>) -> Result<Vec<QualityReport>> {
        let mut reports = Vec::new();

        for library_id in library_ids {
            let report = self.assess_library_quality(library_id).await?;
            reports.push(report);
        }

        Ok(reports)
    }

    fn evaluate_qc_status(&self, value: f64, min_threshold: Option<f64>, max_threshold: Option<f64>) -> QCStatus {
        match (min_threshold, max_threshold) {
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
            (None, None) => QCStatus::Pass,
        }
    }

    fn calculate_overall_status(&self, metrics: &[QualityControlMetric]) -> QCStatus {
        let mut has_fail = false;
        let mut has_warning = false;
        let mut has_pass = false;

        for metric in metrics {
            match metric.status {
                QCStatus::Fail => has_fail = true,
                QCStatus::Warning => has_warning = true,
                QCStatus::Pass => has_pass = true,
                QCStatus::Pending => {}
            }
        }

        if has_fail {
            QCStatus::Fail
        } else if has_warning {
            QCStatus::Warning
        } else if has_pass {
            QCStatus::Pass
        } else {
            QCStatus::Pending
        }
    }

    fn generate_recommendations(&self, metrics: &[QualityControlMetric]) -> Vec<String> {
        let mut recommendations = Vec::new();

        for metric in metrics {
            match metric.status {
                QCStatus::Fail => {
                    match metric.metric_type.as_str() {
                        "concentration" => {
                            if let Some(min) = metric.threshold_min {
                                if metric.value < min {
                                    recommendations.push(format!(
                                        "Library concentration ({:.2} ng/μL) is below minimum threshold ({:.2} ng/μL). Consider concentrating the library.",
                                        metric.value, min
                                    ));
                                }
                            }
                            if let Some(max) = metric.threshold_max {
                                if metric.value > max {
                                    recommendations.push(format!(
                                        "Library concentration ({:.2} ng/μL) is above maximum threshold ({:.2} ng/μL). Consider diluting the library.",
                                        metric.value, max
                                    ));
                                }
                            }
                        }
                        "260_280_ratio" => {
                            recommendations.push(format!(
                                "260/280 ratio ({:.2}) indicates protein contamination. Consider protein removal or re-extraction.",
                                metric.value
                            ));
                        }
                        "260_230_ratio" => {
                            recommendations.push(format!(
                                "260/230 ratio ({:.2}) indicates salt or organic contamination. Consider cleanup or re-extraction.",
                                metric.value
                            ));
                        }
                        "rin" => {
                            recommendations.push(format!(
                                "RNA Integrity Number ({:.1}) is too low. Consider using degraded RNA protocol or re-extraction.",
                                metric.value
                            ));
                        }
                        "fragment_size" => {
                            recommendations.push(format!(
                                "Fragment size ({:.0} bp) is outside acceptable range. Review fragmentation protocol.",
                                metric.value
                            ));
                        }
                        _ => {
                            recommendations.push(format!(
                                "{} value ({:.2}) failed quality control. Review measurement and protocol.",
                                metric.metric_type, metric.value
                            ));
                        }
                    }
                }
                QCStatus::Warning => {
                    recommendations.push(format!(
                        "{} value ({:.2}) is borderline. Monitor closely in subsequent steps.",
                        metric.metric_type, metric.value
                    ));
                }
                _ => {}
            }
        }

        if recommendations.is_empty() {
            recommendations.push("All quality control metrics are within acceptable ranges.".to_string());
        }

        recommendations
    }

    pub async fn update_qc_thresholds(&self, library_id: Uuid, _thresholds: HashMap<String, (Option<f64>, Option<f64>)>) -> Result<()> {
        // This would update protocol-specific thresholds
        // For now, we'll just log the operation
        tracing::info!("Updated QC thresholds for library: {}", library_id);
        Ok(())
    }

    pub async fn get_qc_statistics(&self, _library_type: Option<String>) -> Result<HashMap<String, f64>> {
        let mut stats = HashMap::new();

        // Calculate pass rates by metric type
        let metrics = self.list_qc_metrics(None, None).await?;
        
        let mut metric_counts: HashMap<String, (u32, u32)> = HashMap::new(); // (pass_count, total_count)

        for metric in metrics {
            let entry = metric_counts.entry(metric.metric_type.clone()).or_insert((0, 0));
            entry.1 += 1; // increment total
            
            if matches!(metric.status, QCStatus::Pass) {
                entry.0 += 1; // increment pass
            }
        }

        for (metric_type, (pass_count, total_count)) in metric_counts {
            if total_count > 0 {
                let pass_rate = (pass_count as f64 / total_count as f64) * 100.0;
                stats.insert(format!("{}_pass_rate", metric_type), pass_rate);
            }
        }

        Ok(stats)
    }
}