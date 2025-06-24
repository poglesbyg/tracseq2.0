/// Intelligent Sample Routing AI Module
///
/// This module implements AI-powered algorithms for optimal sample placement
/// and storage location routing in laboratory storage systems.
use super::{AIError, AIInput, AIModel, AIOutput, TrainingData, UpdateData};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

/// Sample routing optimization model
#[derive(Debug)]
pub struct SampleRoutingModel {
    model_version: String,
    routing_algorithms: Vec<RoutingAlgorithm>,
    optimization_weights: OptimizationWeights,
    storage_layout: StorageLayout,
    historical_patterns: Vec<AccessPattern>,
    last_updated: DateTime<Utc>,
}

impl SampleRoutingModel {
    pub fn new() -> Self {
        Self {
            model_version: "1.0.0".to_string(),
            routing_algorithms: vec![
                RoutingAlgorithm::ProximityBased,
                RoutingAlgorithm::TemperatureOptimized,
                RoutingAlgorithm::AccessFrequencyBased,
                RoutingAlgorithm::EnergyEfficient,
                RoutingAlgorithm::LoadBalanced,
            ],
            optimization_weights: OptimizationWeights::default(),
            storage_layout: StorageLayout::new(),
            historical_patterns: Vec::new(),
            last_updated: Utc::now(),
        }
    }

    /// Find optimal storage location for a sample
    pub fn find_optimal_location(
        &self,
        request: &RoutingRequest,
    ) -> Result<RoutingResult, AIError> {
        let mut candidate_locations = self.find_compatible_locations(&request.sample)?;

        if candidate_locations.is_empty() {
            return Err(AIError::InferenceFailed(
                "No compatible storage locations found".to_string(),
            ));
        }

        // Score each candidate location
        for location in &mut candidate_locations {
            location.score = self.calculate_location_score(location, request)?;
        }

        // Sort by score (highest first)
        candidate_locations.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        // Select top recommendations
        let primary_location = candidate_locations[0].clone();
        let alternative_locations = candidate_locations.into_iter().take(5).skip(1).collect();

        // Calculate routing path
        let routing_path =
            self.calculate_routing_path(&request.current_location, &primary_location)?;

        // Generate optimization insights
        let insights = self.generate_optimization_insights(&primary_location, request);

        let estimated_placement_time = self.estimate_placement_time(&routing_path);
        let energy_impact = self.calculate_energy_impact(&routing_path);
        let optimization_score = self.calculate_optimization_score(&primary_location, request);

        Ok(RoutingResult {
            request_id: request.request_id,
            primary_location,
            alternative_locations,
            routing_path,
            estimated_placement_time,
            energy_impact,
            optimization_score,
            insights,
            generated_at: Utc::now(),
        })
    }

    /// Find all storage locations compatible with the sample requirements
    fn find_compatible_locations(
        &self,
        sample: &SampleInfo,
    ) -> Result<Vec<LocationCandidate>, AIError> {
        let mut candidates = Vec::new();

        for zone in &self.storage_layout.zones {
            // Check temperature compatibility
            if !self
                .is_temperature_compatible(&zone.temperature_range, &sample.required_temperature)
            {
                continue;
            }

            for rack in &zone.racks {
                // Check available capacity
                if rack.available_positions == 0 {
                    continue;
                }

                // Check sample type compatibility
                if !self
                    .is_sample_type_compatible(&rack.supported_sample_types, &sample.sample_type)
                {
                    continue;
                }

                // Check access restrictions
                if !self.check_access_permissions(&rack.access_level, &sample.security_level) {
                    continue;
                }

                for position in &rack.positions {
                    if position.available {
                        candidates.push(LocationCandidate {
                            zone_id: zone.id.clone(),
                            rack_id: rack.id.clone(),
                            position_id: position.id.clone(),
                            coordinates: position.coordinates.clone(),
                            temperature: zone.current_temperature,
                            capacity_utilization: rack.capacity_utilization,
                            access_frequency: position.access_frequency,
                            proximity_to_entrance: position.proximity_to_entrance,
                            energy_efficiency: zone.energy_efficiency,
                            score: 0.0, // Will be calculated later
                        });
                    }
                }
            }
        }

        Ok(candidates)
    }

    /// Calculate a score for a storage location based on multiple optimization criteria
    fn calculate_location_score(
        &self,
        location: &LocationCandidate,
        request: &RoutingRequest,
    ) -> Result<f64, AIError> {
        let mut score = 0.0;

        // Proximity score (closer to entrance/current location is better)
        let proximity_score =
            (1.0 - location.proximity_to_entrance / 100.0) * self.optimization_weights.proximity;
        score += proximity_score;

        // Energy efficiency score
        let energy_score = location.energy_efficiency * self.optimization_weights.energy_efficiency;
        score += energy_score;

        // Capacity utilization score (balanced utilization is preferred)
        let optimal_utilization = 0.75; // 75% utilization is ideal
        let utilization_diff = (location.capacity_utilization - optimal_utilization).abs();
        let capacity_score = (1.0 - utilization_diff) * self.optimization_weights.load_balancing;
        score += capacity_score;

        // Access frequency score (for frequently accessed samples, choose easily accessible locations)
        let access_score = if request.sample.access_frequency > 0.5 {
            (1.0 - location.access_frequency / 10.0) * self.optimization_weights.accessibility
        } else {
            // For rarely accessed samples, access frequency is less important
            0.5 * self.optimization_weights.accessibility
        };
        score += access_score;

        // Temperature stability score
        let temp_diff = (location.temperature - request.sample.required_temperature).abs();
        let temp_score =
            (1.0 - temp_diff / 10.0).max(0.0) * self.optimization_weights.temperature_stability;
        score += temp_score;

        // Sample clustering score (samples from the same project/type should be close)
        let clustering_score = self.calculate_clustering_score(location, &request.sample)
            * self.optimization_weights.sample_clustering;
        score += clustering_score;

        Ok(score.max(0.0).min(1.0))
    }

    fn calculate_clustering_score(&self, location: &LocationCandidate, sample: &SampleInfo) -> f64 {
        // Check if there are related samples nearby
        let nearby_samples = self.get_nearby_samples(&location.coordinates, 5.0); // Within 5 units

        let related_count = nearby_samples
            .iter()
            .filter(|s| s.project_id == sample.project_id || s.sample_type == sample.sample_type)
            .count();

        // Score based on number of related samples nearby
        (related_count as f64 / nearby_samples.len().max(1) as f64).min(1.0)
    }

    fn get_nearby_samples(&self, coordinates: &Coordinates, radius: f64) -> Vec<SampleInfo> {
        // In a real implementation, this would query the database for samples
        // within the specified radius of the given coordinates
        Vec::new()
    }

    /// Calculate the optimal routing path from current location to target location
    fn calculate_routing_path(
        &self,
        from: &Coordinates,
        to: &LocationCandidate,
    ) -> Result<RoutingPath, AIError> {
        // Simple path calculation - in a real implementation, this would use
        // more sophisticated pathfinding algorithms considering obstacles, traffic, etc.

        let distance = self.calculate_distance(from, &to.coordinates);
        let waypoints = self.generate_waypoints(from, &to.coordinates);

        Ok(RoutingPath {
            waypoints,
            total_distance: distance,
            estimated_travel_time: Duration::seconds((distance * 2.0) as i64), // 2 seconds per unit
            path_type: "direct".to_string(),
            obstacles: Vec::new(),
            traffic_level: "low".to_string(),
        })
    }

    fn calculate_distance(&self, from: &Coordinates, to: &Coordinates) -> f64 {
        ((to.x - from.x).powi(2) + (to.y - from.y).powi(2) + (to.z - from.z).powi(2)).sqrt()
    }

    fn generate_waypoints(&self, from: &Coordinates, to: &Coordinates) -> Vec<Coordinates> {
        vec![from.clone(), to.clone()] // Simplified - direct path
    }

    fn estimate_placement_time(&self, path: &RoutingPath) -> Duration {
        // Base time for sample handling
        let base_time = Duration::seconds(30);

        // Add travel time
        let travel_time = path.estimated_travel_time;

        // Add complexity factors
        let complexity_factor = match path.traffic_level.as_str() {
            "high" => 1.5,
            "medium" => 1.2,
            "low" => 1.0,
            _ => 1.0,
        };

        Duration::seconds((base_time.num_seconds() as f64 * complexity_factor) as i64) + travel_time
    }

    fn calculate_energy_impact(&self, path: &RoutingPath) -> EnergyImpact {
        let base_energy = 0.5; // kWh base energy for robotic movement
        let distance_energy = path.total_distance * 0.1; // 0.1 kWh per unit distance

        EnergyImpact {
            movement_energy_kwh: base_energy + distance_energy,
            cooling_adjustment_kwh: 0.05, // Minor cooling adjustment for door opening
            total_energy_kwh: base_energy + distance_energy + 0.05,
            carbon_footprint_kg: (base_energy + distance_energy + 0.05) * 0.5, // 0.5 kg CO2 per kWh
        }
    }

    fn calculate_optimization_score(
        &self,
        location: &LocationCandidate,
        request: &RoutingRequest,
    ) -> f64 {
        location.score
    }

    fn generate_optimization_insights(
        &self,
        location: &LocationCandidate,
        request: &RoutingRequest,
    ) -> Vec<String> {
        let mut insights = Vec::new();

        if location.energy_efficiency > 0.9 {
            insights.push("Selected location has excellent energy efficiency".to_string());
        }

        if location.capacity_utilization < 0.5 {
            insights.push("Zone has low utilization - good for future expansion".to_string());
        } else if location.capacity_utilization > 0.9 {
            insights.push("Zone approaching capacity - consider load balancing".to_string());
        }

        if location.proximity_to_entrance < 10.0 {
            insights.push("Location provides easy access for frequent retrieval".to_string());
        }

        insights
    }

    // Helper methods
    fn is_temperature_compatible(&self, zone_range: &(f64, f64), required_temp: &f64) -> bool {
        *required_temp >= zone_range.0 && *required_temp <= zone_range.1
    }

    fn is_sample_type_compatible(&self, supported_types: &[String], sample_type: &str) -> bool {
        supported_types.contains(&sample_type.to_string())
            || supported_types.contains(&"all".to_string())
    }

    fn check_access_permissions(&self, rack_level: &str, sample_security: &str) -> bool {
        match (rack_level, sample_security) {
            ("public", _) => true,
            ("restricted", "low") => true,
            ("restricted", "medium") => true,
            ("secure", "medium") => true,
            ("secure", "high") => true,
            ("maximum_security", "high") => true,
            _ => false,
        }
    }
}

impl AIModel for SampleRoutingModel {
    fn model_type(&self) -> &str {
        "sample_routing_optimization"
    }

    fn version(&self) -> &str {
        &self.model_version
    }

    fn predict(&self, input: &AIInput) -> Result<AIOutput, AIError> {
        let routing_request: RoutingRequest = serde_json::from_value(input.data.clone())
            .map_err(|e| AIError::InvalidInput(format!("Invalid routing request: {}", e)))?;

        let start_time = std::time::Instant::now();
        let result = self.find_optimal_location(&routing_request)?;
        let inference_time = start_time.elapsed().as_millis() as u64;

        Ok(AIOutput {
            prediction: serde_json::to_value(result)?,
            confidence: 0.9, // High confidence for deterministic algorithm
            model_version: self.model_version.clone(),
            inference_time_ms: inference_time,
            metadata: std::collections::HashMap::new(),
            generated_at: Utc::now(),
        })
    }

    fn train(&mut self, _data: &TrainingData) -> Result<(), AIError> {
        // In a real implementation, this would train the model with new data
        Ok(())
    }

    fn update(&mut self, _data: &UpdateData) -> Result<(), AIError> {
        // In a real implementation, this would update the model with new data
        Ok(())
    }

    fn save(&self, _path: &str) -> Result<(), AIError> {
        // In a real implementation, this would serialize the model to disk
        Ok(())
    }

    fn load(_path: &str) -> Result<Self, AIError> where Self: Sized {
        // In a real implementation, this would load the model from disk
        Ok(Self::new())
    }
}

// Data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRequest {
    pub request_id: Uuid,
    pub sample: SampleInfo,
    pub current_location: Coordinates,
    pub constraints: RoutingConstraints,
    pub optimization_preferences: OptimizationPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleInfo {
    pub sample_id: Uuid,
    pub sample_type: String,
    pub project_id: String,
    pub required_temperature: f64,
    pub volume_ml: f64,
    pub security_level: String,
    pub access_frequency: f64, // 0.0 to 1.0
    pub priority: String,
    pub expiry_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coordinates {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingConstraints {
    pub max_distance: Option<f64>,
    pub required_access_level: Option<String>,
    pub avoid_zones: Vec<String>,
    pub prefer_zones: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationPreferences {
    pub prioritize_energy: bool,
    pub prioritize_speed: bool,
    pub prioritize_clustering: bool,
    pub prioritize_accessibility: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct RoutingResult {
    pub request_id: Uuid,
    pub primary_location: LocationCandidate,
    pub alternative_locations: Vec<LocationCandidate>,
    pub routing_path: RoutingPath,
    pub estimated_placement_time: Duration,
    pub energy_impact: EnergyImpact,
    pub optimization_score: f64,
    pub insights: Vec<String>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LocationCandidate {
    pub zone_id: String,
    pub rack_id: String,
    pub position_id: String,
    pub coordinates: Coordinates,
    pub temperature: f64,
    pub capacity_utilization: f64,
    pub access_frequency: f64,
    pub proximity_to_entrance: f64,
    pub energy_efficiency: f64,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RoutingPath {
    pub waypoints: Vec<Coordinates>,
    pub total_distance: f64,
    pub estimated_travel_time: Duration,
    pub path_type: String,
    pub obstacles: Vec<String>,
    pub traffic_level: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnergyImpact {
    pub movement_energy_kwh: f64,
    pub cooling_adjustment_kwh: f64,
    pub total_energy_kwh: f64,
    pub carbon_footprint_kg: f64,
}

#[derive(Debug, Clone)]
pub struct OptimizationWeights {
    pub proximity: f64,
    pub energy_efficiency: f64,
    pub load_balancing: f64,
    pub accessibility: f64,
    pub temperature_stability: f64,
    pub sample_clustering: f64,
}

impl Default for OptimizationWeights {
    fn default() -> Self {
        Self {
            proximity: 0.2,
            energy_efficiency: 0.15,
            load_balancing: 0.15,
            accessibility: 0.2,
            temperature_stability: 0.2,
            sample_clustering: 0.1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StorageLayout {
    pub zones: Vec<StorageZone>,
    pub total_capacity: usize,
    pub layout_version: String,
}

impl StorageLayout {
    fn new() -> Self {
        Self {
            zones: Self::create_default_zones(),
            total_capacity: 1000,
            layout_version: "1.0.0".to_string(),
        }
    }

    fn create_default_zones() -> Vec<StorageZone> {
        vec![
            StorageZone {
                id: "zone_a".to_string(),
                name: "Ultra-Low Freezer Zone".to_string(),
                temperature_range: (-86.0, -70.0),
                current_temperature: -80.0,
                energy_efficiency: 0.85,
                racks: vec![],
            },
            StorageZone {
                id: "zone_b".to_string(),
                name: "Standard Freezer Zone".to_string(),
                temperature_range: (-25.0, -15.0),
                current_temperature: -20.0,
                energy_efficiency: 0.90,
                racks: vec![],
            },
        ]
    }
}

#[derive(Debug, Clone)]
pub struct StorageZone {
    pub id: String,
    pub name: String,
    pub temperature_range: (f64, f64),
    pub current_temperature: f64,
    pub energy_efficiency: f64,
    pub racks: Vec<StorageRack>,
}

#[derive(Debug, Clone)]
pub struct StorageRack {
    pub id: String,
    pub capacity: usize,
    pub available_positions: usize,
    pub capacity_utilization: f64,
    pub supported_sample_types: Vec<String>,
    pub access_level: String,
    pub positions: Vec<StoragePosition>,
}

#[derive(Debug, Clone)]
pub struct StoragePosition {
    pub id: String,
    pub coordinates: Coordinates,
    pub available: bool,
    pub access_frequency: f64,
    pub proximity_to_entrance: f64,
}

#[derive(Debug, Clone)]
pub struct AccessPattern {
    pub sample_type: String,
    pub access_frequency: f64,
    pub typical_retrieval_time: Duration,
    pub seasonal_variation: f64,
}

#[derive(Debug, Clone)]
pub enum RoutingAlgorithm {
    ProximityBased,
    TemperatureOptimized,
    AccessFrequencyBased,
    EnergyEfficient,
    LoadBalanced,
}

impl From<serde_json::Error> for AIError {
    fn from(error: serde_json::Error) -> Self {
        AIError::InvalidInput(error.to_string())
    }
}
