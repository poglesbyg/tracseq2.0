use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRequest {
    pub flow_cell_type_id: Uuid,
    pub libraries: Vec<LibraryInfo>,
    pub optimization_goals: Vec<OptimizationGoal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryInfo {
    pub id: Uuid,
    pub concentration: f64,
    pub fragment_size: i32,
    pub index_type: String,
    pub project_id: Option<Uuid>,
    pub priority: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationGoal {
    MaximizeBalance,
    MinimizeIndexCollisions,
    GroupByProject,
    PrioritizeHighValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub lane_assignments: HashMap<i32, Vec<Uuid>>,
    pub score: f64,
    pub suggestions: Vec<String>,
    pub balance_score: f64,
    pub index_diversity_score: f64,
}

pub struct FlowCellOptimizer;

impl FlowCellOptimizer {
    pub fn optimize(
        request: OptimizationRequest,
        lane_count: i32,
    ) -> OptimizationResult {
        let mut optimizer = FlowCellOptimizer;
        
        // Initialize lane assignments
        let mut lane_assignments: HashMap<i32, Vec<Uuid>> = HashMap::new();
        for lane in 1..=lane_count {
            lane_assignments.insert(lane, Vec::new());
        }
        
        // Sort libraries by priority
        let mut libraries = request.libraries.clone();
        libraries.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        // Apply optimization strategies based on goals
        for goal in &request.optimization_goals {
            match goal {
                OptimizationGoal::MaximizeBalance => {
                    optimizer.balance_distribution(&mut lane_assignments, &libraries, lane_count);
                }
                OptimizationGoal::MinimizeIndexCollisions => {
                    optimizer.optimize_index_diversity(&mut lane_assignments, &libraries);
                }
                OptimizationGoal::GroupByProject => {
                    optimizer.group_by_project(&mut lane_assignments, &libraries, lane_count);
                }
                OptimizationGoal::PrioritizeHighValue => {
                    // Already sorted by priority
                }
            }
        }
        
        // Calculate scores
        let balance_score = optimizer.calculate_balance_score(&lane_assignments, lane_count);
        let index_diversity_score = optimizer.calculate_index_diversity_score(&lane_assignments, &libraries);
        let overall_score = (balance_score + index_diversity_score) / 2.0;
        
        // Generate suggestions
        let suggestions = optimizer.generate_suggestions(&lane_assignments, &libraries, lane_count);
        
        OptimizationResult {
            lane_assignments,
            score: overall_score,
            suggestions,
            balance_score,
            index_diversity_score,
        }
    }
    
    fn balance_distribution(
        &mut self,
        assignments: &mut HashMap<i32, Vec<Uuid>>,
        libraries: &[LibraryInfo],
        lane_count: i32,
    ) {
        // Simple round-robin distribution with balancing
        let mut lane_index = 1;
        
        for library in libraries {
            // Find the lane with the fewest libraries
            let min_lane = (1..=lane_count)
                .min_by_key(|&lane| assignments.get(&lane).map(|v| v.len()).unwrap_or(0))
                .unwrap_or(lane_index);
            
            assignments.get_mut(&min_lane).unwrap().push(library.id);
            
            lane_index = (lane_index % lane_count) + 1;
        }
    }
    
    fn optimize_index_diversity(
        &mut self,
        assignments: &mut HashMap<i32, Vec<Uuid>>,
        libraries: &[LibraryInfo],
    ) {
        // Group libraries by index type to minimize collisions
        let mut index_groups: HashMap<String, Vec<&LibraryInfo>> = HashMap::new();
        
        for library in libraries {
            index_groups
                .entry(library.index_type.clone())
                .or_insert_with(Vec::new)
                .push(library);
        }
        
        // Distribute different index types across lanes
        for (_, group_libraries) in index_groups {
            for (i, library) in group_libraries.iter().enumerate() {
                let lane = ((i as i32) % assignments.len() as i32) + 1;
                if let Some(lane_libs) = assignments.get_mut(&lane) {
                    if !lane_libs.contains(&library.id) {
                        lane_libs.push(library.id);
                    }
                }
            }
        }
    }
    
    fn group_by_project(
        &mut self,
        assignments: &mut HashMap<i32, Vec<Uuid>>,
        libraries: &[LibraryInfo],
        lane_count: i32,
    ) {
        // Group libraries by project
        let mut project_groups: HashMap<Option<Uuid>, Vec<&LibraryInfo>> = HashMap::new();
        
        for library in libraries {
            project_groups
                .entry(library.project_id)
                .or_insert_with(Vec::new)
                .push(library);
        }
        
        // Assign project groups to lanes
        let mut lane_index = 1;
        for (_, project_libraries) in project_groups {
            for library in project_libraries {
                if !assignments.values().any(|libs| libs.contains(&library.id)) {
                    assignments.get_mut(&lane_index).unwrap().push(library.id);
                }
            }
            lane_index = (lane_index % lane_count) + 1;
        }
    }
    
    fn calculate_balance_score(&self, assignments: &HashMap<i32, Vec<Uuid>>, lane_count: i32) -> f64 {
        let counts: Vec<usize> = (1..=lane_count)
            .map(|lane| assignments.get(&lane).map(|v| v.len()).unwrap_or(0))
            .collect();
        
        let avg = counts.iter().sum::<usize>() as f64 / counts.len() as f64;
        let variance = counts.iter()
            .map(|&count| (count as f64 - avg).powi(2))
            .sum::<f64>() / counts.len() as f64;
        
        // Convert variance to a score (0-100)
        (100.0 - variance.min(100.0)).max(0.0)
    }
    
    fn calculate_index_diversity_score(
        &self,
        assignments: &HashMap<i32, Vec<Uuid>>,
        libraries: &[LibraryInfo],
    ) -> f64 {
        let mut total_score = 0.0;
        let mut lane_count = 0;
        
        for (_, lane_libraries) in assignments {
            if lane_libraries.is_empty() {
                continue;
            }
            
            let mut index_types = std::collections::HashSet::new();
            for lib_id in lane_libraries {
                if let Some(library) = libraries.iter().find(|l| l.id == *lib_id) {
                    index_types.insert(&library.index_type);
                }
            }
            
            // Score based on index diversity (more unique indexes = better)
            let diversity_ratio = index_types.len() as f64 / lane_libraries.len() as f64;
            total_score += diversity_ratio * 100.0;
            lane_count += 1;
        }
        
        if lane_count > 0 {
            total_score / lane_count as f64
        } else {
            0.0
        }
    }
    
    fn generate_suggestions(
        &self,
        assignments: &HashMap<i32, Vec<Uuid>>,
        libraries: &[LibraryInfo],
        lane_count: i32,
    ) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        // Check for balance
        let counts: Vec<usize> = (1..=lane_count)
            .map(|lane| assignments.get(&lane).map(|v| v.len()).unwrap_or(0))
            .collect();
        
        let max_count = counts.iter().max().unwrap_or(&0);
        let min_count = counts.iter().min().unwrap_or(&0);
        
        if max_count - min_count > 2 {
            suggestions.push(format!(
                "Lane imbalance detected: consider redistributing libraries for better balance"
            ));
        }
        
        // Check for index collisions
        for (lane, lane_libraries) in assignments {
            let mut index_counts: HashMap<&str, usize> = HashMap::new();
            
            for lib_id in lane_libraries {
                if let Some(library) = libraries.iter().find(|l| l.id == *lib_id) {
                    *index_counts.entry(&library.index_type).or_insert(0) += 1;
                }
            }
            
            for (index_type, count) in index_counts {
                if count > 1 {
                    suggestions.push(format!(
                        "Lane {}: Multiple libraries with index type '{}' may cause issues",
                        lane, index_type
                    ));
                }
            }
        }
        
        // Check for concentration differences
        for (lane, lane_libraries) in assignments {
            let concentrations: Vec<f64> = lane_libraries
                .iter()
                .filter_map(|lib_id| libraries.iter().find(|l| l.id == *lib_id))
                .map(|l| l.concentration)
                .collect();
            
            if !concentrations.is_empty() {
                let max_conc = concentrations.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                let min_conc = concentrations.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                
                if max_conc / min_conc > 10.0 {
                    suggestions.push(format!(
                        "Lane {}: Large concentration differences may affect sequencing quality",
                        lane
                    ));
                }
            }
        }
        
        if suggestions.is_empty() {
            suggestions.push("Optimization complete: design looks well-balanced".to_string());
        }
        
        suggestions
    }
} 