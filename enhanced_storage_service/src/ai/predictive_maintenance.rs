/// Predictive Maintenance AI Module
///
/// This module implements machine learning models for predicting equipment failures
/// and maintenance needs in laboratory storage systems.
use super::{AIError, AIInput, AIModel, AIOutput};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

/// Equipment failure prediction model
#[derive(Debug)]
pub struct EquipmentFailureModel {
    model_version: String,
    features: Vec<String>,
    thresholds: HashMap<String, f64>,
    historical_patterns: Vec<FailurePattern>,
    last_updated: DateTime<Utc>,
}

impl EquipmentFailureModel {
    pub fn new() -> Self {
        Self {
            model_version: "1.0.0".to_string(),
            features: vec![
                "temperature_variance".to_string(),
                "power_consumption_trend".to_string(),
                "vibration_levels".to_string(),
                "runtime_hours".to_string(),
                "maintenance_history".to_string(),
                "environmental_conditions".to_string(),
                "usage_patterns".to_string(),
                "component_age".to_string(),
            ],
            thresholds: Self::default_thresholds(),
            historical_patterns: Vec::new(),
            last_updated: Utc::now(),
        }
    }

    fn default_thresholds() -> HashMap<String, f64> {
        let mut thresholds = HashMap::new();
        thresholds.insert("temperature_variance_critical".to_string(), 2.0);
        thresholds.insert("power_consumption_increase".to_string(), 1.2);
        thresholds.insert("vibration_warning".to_string(), 0.8);
        thresholds.insert("maintenance_overdue_days".to_string(), 30.0);
        thresholds.insert("component_age_years".to_string(), 5.0);
        thresholds
    }

    /// Predict equipment failure probability
    pub fn predict_failure(
        &self,
        equipment_data: &EquipmentData,
    ) -> Result<FailurePrediction, AIError> {
        let mut risk_factors = Vec::new();
        let mut total_risk_score = 0.0;

        // Analyze temperature variance
        let temp_risk = self.analyze_temperature_variance(&equipment_data.temperature_history)?;
        risk_factors.push(temp_risk.clone());
        total_risk_score += temp_risk.weight * temp_risk.severity;

        // Analyze power consumption trends
        let power_risk =
            self.analyze_power_consumption(&equipment_data.power_consumption_history)?;
        risk_factors.push(power_risk.clone());
        total_risk_score += power_risk.weight * power_risk.severity;

        // Analyze maintenance history
        let maintenance_risk =
            self.analyze_maintenance_history(&equipment_data.maintenance_records)?;
        risk_factors.push(maintenance_risk.clone());
        total_risk_score += maintenance_risk.weight * maintenance_risk.severity;

        // Analyze component age
        let age_risk = self.analyze_component_age(&equipment_data.components)?;
        risk_factors.push(age_risk.clone());
        total_risk_score += age_risk.weight * age_risk.severity;

        // Calculate failure probability (0.0 to 1.0)
        let failure_probability = (total_risk_score / risk_factors.len() as f64)
            .min(1.0)
            .max(0.0);

        // Determine time to failure estimate
        let estimated_time_to_failure =
            self.estimate_time_to_failure(failure_probability, &equipment_data);

        // Generate maintenance recommendations
        let recommendations =
            self.generate_maintenance_recommendations(&risk_factors, failure_probability);

        Ok(FailurePrediction {
            equipment_id: equipment_data.equipment_id,
            failure_probability,
            confidence: self.calculate_confidence(&risk_factors),
            estimated_time_to_failure,
            risk_factors,
            recommendations,
            severity_level: Self::classify_severity(failure_probability),
            prediction_date: Utc::now(),
            model_version: self.model_version.clone(),
        })
    }

    fn analyze_temperature_variance(
        &self,
        history: &[TemperatureReading],
    ) -> Result<RiskFactor, AIError> {
        if history.is_empty() {
            return Ok(RiskFactor::new(
                "temperature_variance",
                0.0,
                0.2,
                "No temperature data available",
            ));
        }

        let mut variances = Vec::new();
        for window in history.windows(24) {
            // 24-hour windows
            let mean = window.iter().map(|r| r.temperature).sum::<f64>() / window.len() as f64;
            let variance = window
                .iter()
                .map(|r| (r.temperature - mean).powi(2))
                .sum::<f64>()
                / window.len() as f64;
            variances.push(variance);
        }

        let avg_variance = variances.iter().sum::<f64>() / variances.len() as f64;
        let threshold = self
            .thresholds
            .get("temperature_variance_critical")
            .unwrap_or(&2.0);

        let severity = (avg_variance / threshold).min(1.0);
        let description = if severity > 0.8 {
            "Critical temperature instability detected"
        } else if severity > 0.5 {
            "Moderate temperature variance observed"
        } else {
            "Temperature stable within normal range"
        };

        Ok(RiskFactor::new(
            "temperature_variance",
            severity,
            0.3,
            description,
        ))
    }

    fn analyze_power_consumption(&self, history: &[PowerReading]) -> Result<RiskFactor, AIError> {
        if history.len() < 2 {
            return Ok(RiskFactor::new(
                "power_consumption",
                0.0,
                0.2,
                "Insufficient power data",
            ));
        }

        // Calculate trend over the last 30 readings
        let recent_data = if history.len() > 30 {
            &history[history.len() - 30..]
        } else {
            history
        };

        let baseline = recent_data.iter().take(10).map(|r| r.power_kw).sum::<f64>() / 10.0;
        let current = recent_data
            .iter()
            .skip(recent_data.len().saturating_sub(10))
            .map(|r| r.power_kw)
            .sum::<f64>()
            / 10.0;

        let increase_ratio = current / baseline;
        let threshold = self
            .thresholds
            .get("power_consumption_increase")
            .unwrap_or(&1.2);

        let severity = ((increase_ratio - 1.0) / (threshold - 1.0))
            .min(1.0)
            .max(0.0);
        let description = if severity > 0.8 {
            format!(
                "Significant power increase: {:.1}% above baseline",
                (increase_ratio - 1.0) * 100.0
            )
        } else if severity > 0.5 {
            format!(
                "Moderate power increase: {:.1}% above baseline",
                (increase_ratio - 1.0) * 100.0
            )
        } else {
            "Power consumption within normal range"
        };

        Ok(RiskFactor::new(
            "power_consumption",
            severity,
            0.25,
            &description,
        ))
    }

    fn analyze_maintenance_history(
        &self,
        records: &[MaintenanceRecord],
    ) -> Result<RiskFactor, AIError> {
        let now = Utc::now();
        let overdue_threshold = Duration::days(
            self.thresholds
                .get("maintenance_overdue_days")
                .unwrap_or(&30.0) as i64,
        );

        let last_maintenance = records
            .iter()
            .filter(|r| r.maintenance_type == "preventive" || r.maintenance_type == "scheduled")
            .max_by_key(|r| r.completed_at);

        let (severity, description) = match last_maintenance {
            Some(record) => {
                let time_since = now.signed_duration_since(record.completed_at);
                if time_since > overdue_threshold * 2 {
                    (0.9, "Maintenance significantly overdue")
                } else if time_since > overdue_threshold {
                    (0.6, "Maintenance overdue")
                } else {
                    (0.1, "Maintenance up to date")
                }
            }
            None => (0.8, "No maintenance history available"),
        };

        Ok(RiskFactor::new(
            "maintenance_history",
            severity,
            0.35,
            description,
        ))
    }

    fn analyze_component_age(&self, components: &[ComponentInfo]) -> Result<RiskFactor, AIError> {
        if components.is_empty() {
            return Ok(RiskFactor::new(
                "component_age",
                0.3,
                0.15,
                "No component information available",
            ));
        }

        let age_threshold_years = self.thresholds.get("component_age_years").unwrap_or(&5.0);
        let now = Utc::now();

        let aged_components: Vec<_> = components
            .iter()
            .filter(|c| {
                let age_years =
                    now.signed_duration_since(c.installed_date).num_days() as f64 / 365.25;
                age_years > *age_threshold_years
            })
            .collect();

        let severity = aged_components.len() as f64 / components.len() as f64;
        let description = if severity > 0.5 {
            format!(
                "{} of {} components exceed age threshold",
                aged_components.len(),
                components.len()
            )
        } else {
            "Component ages within acceptable range"
        };

        Ok(RiskFactor::new(
            "component_age",
            severity,
            0.2,
            &description,
        ))
    }

    fn estimate_time_to_failure(
        &self,
        failure_probability: f64,
        _equipment_data: &EquipmentData,
    ) -> Option<DateTime<Utc>> {
        if failure_probability < 0.3 {
            None // Low risk, no immediate failure expected
        } else {
            // Estimate based on failure probability
            let days_to_failure = (365.0 * (1.0 - failure_probability)).max(1.0) as i64;
            Some(Utc::now() + Duration::days(days_to_failure))
        }
    }

    fn generate_maintenance_recommendations(
        &self,
        risk_factors: &[RiskFactor],
        failure_probability: f64,
    ) -> Vec<MaintenanceRecommendation> {
        let mut recommendations = Vec::new();

        for factor in risk_factors {
            if factor.severity > 0.6 {
                let recommendation = match factor.factor_type.as_str() {
                    "temperature_variance" => MaintenanceRecommendation {
                        id: Uuid::new_v4(),
                        priority: Priority::High,
                        action: "Inspect and calibrate temperature control system".to_string(),
                        estimated_duration_hours: 4.0,
                        required_parts: vec![
                            "Temperature sensor".to_string(),
                            "Control valve".to_string(),
                        ],
                        estimated_cost_usd: 850.0,
                        reason: factor.description.clone(),
                    },
                    "power_consumption" => MaintenanceRecommendation {
                        id: Uuid::new_v4(),
                        priority: Priority::Medium,
                        action: "Check compressor efficiency and electrical connections"
                            .to_string(),
                        estimated_duration_hours: 3.0,
                        required_parts: vec!["Electrical contactors".to_string()],
                        estimated_cost_usd: 450.0,
                        reason: factor.description.clone(),
                    },
                    "maintenance_history" => MaintenanceRecommendation {
                        id: Uuid::new_v4(),
                        priority: Priority::High,
                        action: "Schedule immediate preventive maintenance".to_string(),
                        estimated_duration_hours: 6.0,
                        required_parts: vec![
                            "Filter replacement".to_string(),
                            "Lubrication".to_string(),
                        ],
                        estimated_cost_usd: 350.0,
                        reason: factor.description.clone(),
                    },
                    "component_age" => MaintenanceRecommendation {
                        id: Uuid::new_v4(),
                        priority: Priority::Medium,
                        action: "Evaluate components for replacement".to_string(),
                        estimated_duration_hours: 8.0,
                        required_parts: vec!["Aged components".to_string()],
                        estimated_cost_usd: 1200.0,
                        reason: factor.description.clone(),
                    },
                    _ => continue,
                };
                recommendations.push(recommendation);
            }
        }

        // Add general recommendation for high failure probability
        if failure_probability > 0.7 {
            recommendations.push(MaintenanceRecommendation {
                id: Uuid::new_v4(),
                priority: Priority::Critical,
                action: "Immediate comprehensive inspection and service".to_string(),
                estimated_duration_hours: 12.0,
                required_parts: vec!["Full service kit".to_string()],
                estimated_cost_usd: 2500.0,
                reason: "High failure probability detected".to_string(),
            });
        }

        recommendations
    }

    fn calculate_confidence(&self, risk_factors: &[RiskFactor]) -> f64 {
        let data_quality = risk_factors
            .iter()
            .map(|f| {
                if f.description.contains("No") || f.description.contains("Insufficient") {
                    0.5
                } else {
                    1.0
                }
            })
            .sum::<f64>()
            / risk_factors.len() as f64;

        let factor_agreement = if risk_factors.len() > 1 {
            let avg_severity =
                risk_factors.iter().map(|f| f.severity).sum::<f64>() / risk_factors.len() as f64;
            let variance = risk_factors
                .iter()
                .map(|f| (f.severity - avg_severity).powi(2))
                .sum::<f64>()
                / risk_factors.len() as f64;
            (1.0 - variance).max(0.5)
        } else {
            0.7
        };

        (data_quality * 0.6 + factor_agreement * 0.4)
            .min(0.95)
            .max(0.3)
    }

    fn classify_severity(failure_probability: f64) -> SeverityLevel {
        if failure_probability > 0.8 {
            SeverityLevel::Critical
        } else if failure_probability > 0.6 {
            SeverityLevel::High
        } else if failure_probability > 0.4 {
            SeverityLevel::Medium
        } else {
            SeverityLevel::Low
        }
    }
}

impl AIModel for EquipmentFailureModel {
    fn model_type(&self) -> &str {
        "equipment_failure_prediction"
    }

    fn version(&self) -> &str {
        &self.model_version
    }

    fn predict(&self, input: &AIInput) -> Result<AIOutput, AIError> {
        let equipment_data: EquipmentData = serde_json::from_value(input.data.clone())
            .map_err(|e| AIError::InvalidInput(format!("Invalid equipment data: {}", e)))?;

        let start_time = std::time::Instant::now();
        let prediction = self.predict_failure(&equipment_data)?;
        let inference_time = start_time.elapsed().as_millis() as u64;

        Ok(AIOutput {
            prediction: serde_json::to_value(prediction)?,
            confidence: self.calculate_confidence(&Vec::new()), // Simplified for this example
            model_version: self.model_version.clone(),
            inference_time_ms: inference_time,
            generated_at: Utc::now(),
        })
    }

    fn save(&self, _path: &str) -> Result<(), AIError> {
        // In a real implementation, this would serialize the model to disk
        Ok(())
    }
}

// Data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipmentData {
    pub equipment_id: Uuid,
    pub equipment_type: String,
    pub temperature_history: Vec<TemperatureReading>,
    pub power_consumption_history: Vec<PowerReading>,
    pub maintenance_records: Vec<MaintenanceRecord>,
    pub components: Vec<ComponentInfo>,
    pub installation_date: DateTime<Utc>,
    pub current_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemperatureReading {
    pub timestamp: DateTime<Utc>,
    pub temperature: f64,
    pub setpoint: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerReading {
    pub timestamp: DateTime<Utc>,
    pub power_kw: f64,
    pub voltage: f64,
    pub current: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceRecord {
    pub id: Uuid,
    pub maintenance_type: String,
    pub description: String,
    pub completed_at: DateTime<Utc>,
    pub technician: String,
    pub cost_usd: f64,
    pub parts_replaced: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentInfo {
    pub component_id: String,
    pub component_type: String,
    pub manufacturer: String,
    pub model: String,
    pub installed_date: DateTime<Utc>,
    pub warranty_expiry: Option<DateTime<Utc>>,
    pub expected_lifetime_years: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct FailurePrediction {
    pub equipment_id: Uuid,
    pub failure_probability: f64,
    pub confidence: f64,
    pub estimated_time_to_failure: Option<DateTime<Utc>>,
    pub risk_factors: Vec<RiskFactor>,
    pub recommendations: Vec<MaintenanceRecommendation>,
    pub severity_level: SeverityLevel,
    pub prediction_date: DateTime<Utc>,
    pub model_version: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RiskFactor {
    pub factor_type: String,
    pub severity: f64, // 0.0 to 1.0
    pub weight: f64,   // Importance of this factor
    pub description: String,
}

impl RiskFactor {
    fn new(factor_type: &str, severity: f64, weight: f64, description: &str) -> Self {
        Self {
            factor_type: factor_type.to_string(),
            severity,
            weight,
            description: description.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MaintenanceRecommendation {
    pub id: Uuid,
    pub priority: Priority,
    pub action: String,
    pub estimated_duration_hours: f64,
    pub required_parts: Vec<String>,
    pub estimated_cost_usd: f64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize)]
pub enum SeverityLevel {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize)]
pub struct FailurePattern {
    pub pattern_type: String,
    pub indicators: Vec<String>,
    pub historical_accuracy: f64,
    pub time_to_failure_days: Vec<i32>,
}

impl From<serde_json::Error> for AIError {
    fn from(error: serde_json::Error) -> Self {
        AIError::InvalidInput(error.to_string())
    }
}
