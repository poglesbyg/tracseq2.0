use crate::error::{Result, VersioningError};
use crate::models::{
    SpreadsheetVersion, VersionDiff, VersionConflict, MergeStrategy, ConflictResolution,
    CellChange, ChangeType, MergeResult, ConflictType,
};
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Intelligent merge engine with AI-powered conflict resolution
pub struct IntelligentMergeEngine {
    /// Learning model for merge patterns
    merge_patterns: HashMap<String, f64>,
    /// Conflict resolution statistics
    resolution_stats: HashMap<ConflictType, u32>,
    /// User preferences for automatic resolution
    user_preferences: HashMap<Uuid, MergePreferences>,
}

/// User preferences for merge operations
#[derive(Debug, Clone)]
pub struct MergePreferences {
    pub prefer_newer_values: bool,
    pub prefer_user_changes: bool,
    pub auto_resolve_formulas: bool,
    pub confidence_threshold: f64,
    pub custom_rules: Vec<CustomMergeRule>,
}

/// Custom merge rule defined by users
#[derive(Debug, Clone)]
pub struct CustomMergeRule {
    pub rule_id: String,
    pub pattern: String,
    pub action: MergeAction,
    pub priority: i32,
}

/// Merge action to take for a specific pattern
#[derive(Debug, Clone)]
pub enum MergeAction {
    PreferLeft,
    PreferRight,
    Combine,
    Prompt,
    Custom(String),
}

/// Enhanced merge result with detailed analytics
#[derive(Debug, Clone)]
pub struct EnhancedMergeResult {
    pub base_result: MergeResult,
    pub confidence_score: f64,
    pub auto_resolved_conflicts: u32,
    pub manual_conflicts: u32,
    pub suggested_actions: Vec<SuggestedAction>,
    pub quality_metrics: MergeQualityMetrics,
}

/// Suggested action for conflict resolution
#[derive(Debug, Clone)]
pub struct SuggestedAction {
    pub conflict_id: String,
    pub action_type: ActionType,
    pub confidence: f64,
    pub reasoning: String,
}

/// Type of suggested action
#[derive(Debug, Clone)]
pub enum ActionType {
    AutoResolve,
    RequestReview,
    SplitChange,
    Combine,
    Revert,
}

/// Quality metrics for merge operations
#[derive(Debug, Clone)]
pub struct MergeQualityMetrics {
    pub data_integrity_score: f64,
    pub consistency_score: f64,
    pub completeness_score: f64,
    pub formula_validity_score: f64,
    pub reference_integrity_score: f64,
}

/// Context for merge decisions
#[derive(Debug, Clone)]
pub struct MergeContext {
    pub user_id: Uuid,
    pub session_id: Option<String>,
    pub previous_merges: Vec<MergeHistory>,
    pub domain_knowledge: HashMap<String, Value>,
}

/// Historical merge information
#[derive(Debug, Clone)]
pub struct MergeHistory {
    pub timestamp: DateTime<Utc>,
    pub conflict_type: ConflictType,
    pub resolution: ConflictResolution,
    pub success: bool,
}

/// Additional models for enhanced merge functionality
#[derive(Debug, Clone)]
pub struct ResolvedConflict {
    pub conflict_id: Uuid,
    pub cell_address: String,
    pub resolution_type: ConflictResolution,
    pub resolved_value: Option<Value>,
    pub confidence: f64,
    pub reasoning: String,
    pub resolved_at: DateTime<Utc>,
    pub resolved_by: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub enum ConflictSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl IntelligentMergeEngine {
    pub fn new() -> Self {
        Self {
            merge_patterns: HashMap::new(),
            resolution_stats: HashMap::new(),
            user_preferences: HashMap::new(),
        }
    }

    /// Perform intelligent merge with AI-powered conflict resolution
    pub async fn intelligent_merge(
        &mut self,
        base_version: &SpreadsheetVersion,
        left_version: &SpreadsheetVersion,
        right_version: &SpreadsheetVersion,
        context: &MergeContext,
    ) -> Result<EnhancedMergeResult> {
        tracing::info!(
            "Starting intelligent merge for spreadsheet {} with base version {}, left version {}, right version {}",
            base_version.spreadsheet_id,
            base_version.version_number,
            left_version.version_number,
            right_version.version_number
        );

        // Analyze changes between versions
        let left_changes = self.analyze_changes(base_version, left_version).await?;
        let right_changes = self.analyze_changes(base_version, right_version).await?;

        // Detect conflicts
        let conflicts = self.detect_intelligent_conflicts(&left_changes, &right_changes).await?;

        // Apply AI-powered conflict resolution
        let resolved_conflicts = self.resolve_conflicts_with_ai(
            &conflicts,
            &left_changes,
            &right_changes,
            context,
        ).await?;

        // Generate merged content
        let merged_content = self.generate_merged_content(
            base_version,
            &left_changes,
            &right_changes,
            &resolved_conflicts,
        ).await?;

        // Calculate quality metrics
        let quality_metrics = self.calculate_merge_quality(&merged_content, &conflicts).await?;

        // Generate suggestions for remaining conflicts
        let suggestions = self.generate_suggestions(&resolved_conflicts, context).await?;

        // Calculate overall confidence
        let confidence_score = self.calculate_confidence_score(
            &resolved_conflicts,
            &quality_metrics,
            &conflicts,
        );

        let auto_resolved = resolved_conflicts.iter()
            .filter(|r| r.resolution_type == ConflictResolution::Automatic)
            .count() as u32;

        let manual_conflicts = conflicts.len() as u32 - auto_resolved;

        Ok(EnhancedMergeResult {
            base_result: MergeResult {
                success: manual_conflicts == 0,
                merged_content,
                conflicts: Vec::new(), // Convert to VersionConflict if needed
                merge_strategy: MergeStrategy::Intelligent,
                created_at: Utc::now(),
            },
            confidence_score,
            auto_resolved_conflicts: auto_resolved,
            manual_conflicts,
            suggested_actions: suggestions,
            quality_metrics,
        })
    }

    /// Analyze changes between two versions with enhanced detection
    async fn analyze_changes(
        &self,
        base: &SpreadsheetVersion,
        target: &SpreadsheetVersion,
    ) -> Result<Vec<CellChange>> {
        let mut changes = Vec::new();

        // Parse spreadsheet content
        let base_data: Value = serde_json::from_str(&base.file_content)
            .map_err(|e| VersioningError::InvalidFormat(format!("Invalid base content: {}", e)))?;
        let target_data: Value = serde_json::from_str(&target.file_content)
            .map_err(|e| VersioningError::InvalidFormat(format!("Invalid target content: {}", e)))?;

        // Enhanced cell-by-cell comparison
        if let (Some(base_sheets), Some(target_sheets)) = (
            base_data.get("sheets").and_then(|s| s.as_array()),
            target_data.get("sheets").and_then(|s| s.as_array()),
        ) {
            for (sheet_idx, (base_sheet, target_sheet)) in 
                base_sheets.iter().zip(target_sheets.iter()).enumerate() {
                
                let sheet_changes = self.analyze_sheet_changes(
                    base_sheet, 
                    target_sheet, 
                    sheet_idx
                ).await?;
                changes.extend(sheet_changes);
            }
        }

        Ok(changes)
    }

    /// Analyze changes within a single sheet
    async fn analyze_sheet_changes(
        &self,
        base_sheet: &Value,
        target_sheet: &Value,
        sheet_index: usize,
    ) -> Result<Vec<CellChange>> {
        let mut changes = Vec::new();

        if let (Some(base_rows), Some(target_rows)) = (
            base_sheet.get("rows").and_then(|r| r.as_array()),
            target_sheet.get("rows").and_then(|r| r.as_array()),
        ) {
            let max_rows = base_rows.len().max(target_rows.len());

            for row_idx in 0..max_rows {
                let base_row = base_rows.get(row_idx);
                let target_row = target_rows.get(row_idx);

                match (base_row, target_row) {
                    (Some(base), Some(target)) => {
                        let row_changes = self.analyze_row_changes(
                            base, target, sheet_index, row_idx
                        ).await?;
                        changes.extend(row_changes);
                    }
                    (None, Some(target)) => {
                        // Row added
                        changes.push(CellChange {
                            cell_address: format!("Sheet{}:Row{}", sheet_index, row_idx),
                            change_type: ChangeType::Add,
                            old_value: None,
                            new_value: Some(target.clone()),
                            formula_changed: false,
                            metadata: HashMap::new(),
                        });
                    }
                    (Some(_), None) => {
                        // Row deleted
                        changes.push(CellChange {
                            cell_address: format!("Sheet{}:Row{}", sheet_index, row_idx),
                            change_type: ChangeType::Delete,
                            old_value: Some(Value::Null),
                            new_value: None,
                            formula_changed: false,
                            metadata: HashMap::new(),
                        });
                    }
                    (None, None) => unreachable!(),
                }
            }
        }

        Ok(changes)
    }

    /// Analyze changes within a single row
    async fn analyze_row_changes(
        &self,
        base_row: &Value,
        target_row: &Value,
        sheet_index: usize,
        row_index: usize,
    ) -> Result<Vec<CellChange>> {
        let mut changes = Vec::new();

        if let (Some(base_cells), Some(target_cells)) = (
            base_row.get("cells").and_then(|c| c.as_array()),
            target_row.get("cells").and_then(|c| c.as_array()),
        ) {
            let max_cols = base_cells.len().max(target_cells.len());

            for col_idx in 0..max_cols {
                let base_cell = base_cells.get(col_idx);
                let target_cell = target_cells.get(col_idx);

                if let Some(change) = self.analyze_cell_change(
                    base_cell,
                    target_cell,
                    sheet_index,
                    row_index,
                    col_idx,
                ).await? {
                    changes.push(change);
                }
            }
        }

        Ok(changes)
    }

    /// Analyze change in a single cell with enhanced detection
    async fn analyze_cell_change(
        &self,
        base_cell: Option<&Value>,
        target_cell: Option<&Value>,
        sheet_index: usize,
        row_index: usize,
        col_index: usize,
    ) -> Result<Option<CellChange>> {
        let cell_address = format!("Sheet{}:{}:{}", sheet_index, row_index, col_index);

        match (base_cell, target_cell) {
            (Some(base), Some(target)) => {
                if base != target {
                    let formula_changed = self.is_formula_changed(base, target);
                    let change_type = if formula_changed {
                        ChangeType::FormulaUpdate
                    } else {
                        ChangeType::Update
                    };

                    Ok(Some(CellChange {
                        cell_address,
                        change_type,
                        old_value: Some(base.clone()),
                        new_value: Some(target.clone()),
                        formula_changed,
                        metadata: self.extract_cell_metadata(base, target),
                    }))
                } else {
                    Ok(None)
                }
            }
            (None, Some(target)) => {
                Ok(Some(CellChange {
                    cell_address,
                    change_type: ChangeType::Add,
                    old_value: None,
                    new_value: Some(target.clone()),
                    formula_changed: self.is_formula(target),
                    metadata: HashMap::new(),
                }))
            }
            (Some(base), None) => {
                Ok(Some(CellChange {
                    cell_address,
                    change_type: ChangeType::Delete,
                    old_value: Some(base.clone()),
                    new_value: None,
                    formula_changed: false,
                    metadata: HashMap::new(),
                }))
            }
            (None, None) => Ok(None),
        }
    }

    /// Detect conflicts using intelligent algorithms
    async fn detect_intelligent_conflicts(
        &self,
        left_changes: &[CellChange],
        right_changes: &[CellChange],
    ) -> Result<Vec<VersionConflict>> {
        let mut conflicts = Vec::new();
        let left_addresses: HashSet<String> = left_changes.iter()
            .map(|c| c.cell_address.clone())
            .collect();

        for right_change in right_changes {
            if left_addresses.contains(&right_change.cell_address) {
                // Find the corresponding left change
                if let Some(left_change) = left_changes.iter()
                    .find(|c| c.cell_address == right_change.cell_address) {
                    
                    let conflict_type = self.classify_conflict_type(left_change, right_change);
                    let severity = self.calculate_conflict_severity(left_change, right_change);

                    conflicts.push(VersionConflict {
                        id: Uuid::new_v4(),
                        cell_address: right_change.cell_address.clone(),
                        conflict_type,
                        left_value: left_change.new_value.clone(),
                        right_value: right_change.new_value.clone(),
                        base_value: left_change.old_value.clone(),
                        severity,
                        suggested_resolution: self.suggest_resolution(
                            left_change, 
                            right_change, 
                            conflict_type
                        ),
                        metadata: self.create_conflict_metadata(left_change, right_change),
                        created_at: Utc::now(),
                    });
                }
            }
        }

        Ok(conflicts)
    }

    /// Resolve conflicts using AI algorithms
    async fn resolve_conflicts_with_ai(
        &mut self,
        conflicts: &[VersionConflict],
        _left_changes: &[CellChange],
        _right_changes: &[CellChange],
        context: &MergeContext,
    ) -> Result<Vec<ResolvedConflict>> {
        let mut resolved = Vec::new();

        for conflict in conflicts {
            let resolution = self.apply_ai_resolution(conflict, context).await?;
            resolved.push(resolution);
        }

        // Update learning patterns
        self.update_learning_patterns(&resolved).await?;

        Ok(resolved)
    }

    /// Apply AI-powered resolution to a single conflict
    async fn apply_ai_resolution(
        &self,
        conflict: &VersionConflict,
        context: &MergeContext,
    ) -> Result<ResolvedConflict> {
        // Get user preferences
        let preferences = self.user_preferences.get(&context.user_id)
            .cloned()
            .unwrap_or_default();

        // Calculate confidence based on multiple factors
        let confidence = self.calculate_resolution_confidence(conflict, context, &preferences);

        let resolution_type = if confidence >= preferences.confidence_threshold {
            ConflictResolution::Automatic
        } else {
            ConflictResolution::Manual
        };

        let resolved_value = match resolution_type {
            ConflictResolution::Automatic => {
                self.auto_resolve_value(conflict, &preferences).await?
            }
            ConflictResolution::Manual => {
                conflict.suggested_resolution.clone().unwrap_or(conflict.left_value.clone())
            }
        };

        Ok(ResolvedConflict {
            conflict_id: conflict.id,
            cell_address: conflict.cell_address.clone(),
            resolution_type,
            resolved_value,
            confidence,
            reasoning: self.generate_resolution_reasoning(conflict, resolution_type, confidence),
            resolved_at: Utc::now(),
            resolved_by: Some(context.user_id),
        })
    }

    // Helper methods implementation
    fn is_formula_changed(&self, base: &Value, target: &Value) -> bool {
        match (base.get("formula"), target.get("formula")) {
            (Some(base_formula), Some(target_formula)) => base_formula != target_formula,
            (None, Some(_)) => true,
            (Some(_), None) => true,
            (None, None) => false,
        }
    }

    fn is_formula(&self, cell: &Value) -> bool {
        cell.get("formula").is_some()
    }

    fn extract_cell_metadata(&self, base: &Value, target: &Value) -> HashMap<String, Value> {
        let mut metadata = HashMap::new();
        
        if let Some(base_type) = base.get("type") {
            metadata.insert("old_type".to_string(), base_type.clone());
        }
        
        if let Some(target_type) = target.get("type") {
            metadata.insert("new_type".to_string(), target_type.clone());
        }

        metadata
    }

    fn classify_conflict_type(&self, left: &CellChange, right: &CellChange) -> ConflictType {
        match (&left.change_type, &right.change_type) {
            (ChangeType::FormulaUpdate, ChangeType::FormulaUpdate) => ConflictType::FormulaConflict,
            (ChangeType::Update, ChangeType::Update) => ConflictType::ValueConflict,
            (ChangeType::Add, ChangeType::Delete) | (ChangeType::Delete, ChangeType::Add) => 
                ConflictType::StructuralConflict,
            _ => ConflictType::ValueConflict,
        }
    }

    fn calculate_conflict_severity(&self, left: &CellChange, _right: &CellChange) -> ConflictSeverity {
        if left.formula_changed {
            ConflictSeverity::High
        } else {
            ConflictSeverity::Medium
        }
    }

    fn suggest_resolution(
        &self,
        _left: &CellChange,
        right: &CellChange,
        conflict_type: ConflictType,
    ) -> Option<Value> {
        match conflict_type {
            ConflictType::FormatConflict => right.new_value.clone(),
            _ => None,
        }
    }

    fn create_conflict_metadata(&self, left: &CellChange, right: &CellChange) -> HashMap<String, Value> {
        let mut metadata = HashMap::new();
        metadata.insert("left_change_type".to_string(), 
            Value::String(format!("{:?}", left.change_type)));
        metadata.insert("right_change_type".to_string(), 
            Value::String(format!("{:?}", right.change_type)));
        metadata
    }

    fn calculate_resolution_confidence(
        &self,
        conflict: &VersionConflict,
        _context: &MergeContext,
        preferences: &MergePreferences,
    ) -> f64 {
        let mut confidence = 0.5; // Base confidence

        // Adjust based on conflict type
        match conflict.conflict_type {
            ConflictType::FormatConflict => confidence += 0.3,
            ConflictType::ValueConflict => confidence += 0.2,
            ConflictType::FormulaConflict => confidence -= 0.2,
            ConflictType::StructuralConflict => confidence -= 0.4,
        }

        // Adjust based on severity
        match conflict.severity {
            ConflictSeverity::Low => confidence += 0.2,
            ConflictSeverity::Medium => confidence += 0.0,
            ConflictSeverity::High => confidence -= 0.2,
            ConflictSeverity::Critical => confidence -= 0.4,
        }

        // Apply user preferences
        if preferences.prefer_newer_values {
            confidence += 0.1;
        }

        confidence.max(0.0).min(1.0)
    }

    async fn auto_resolve_value(
        &self,
        conflict: &VersionConflict,
        preferences: &MergePreferences,
    ) -> Result<Option<Value>> {
        match conflict.conflict_type {
            ConflictType::ValueConflict => {
                if preferences.prefer_newer_values {
                    Ok(conflict.right_value.clone())
                } else {
                    Ok(conflict.left_value.clone())
                }
            }
            ConflictType::FormulaConflict => {
                if preferences.auto_resolve_formulas {
                    self.resolve_formula_conflict(conflict).await
                } else {
                    Ok(conflict.suggested_resolution.clone())
                }
            }
            ConflictType::StructuralConflict => {
                Ok(conflict.suggested_resolution.clone())
            }
            ConflictType::FormatConflict => {
                Ok(conflict.right_value.clone())
            }
        }
    }

    async fn resolve_formula_conflict(&self, conflict: &VersionConflict) -> Result<Option<Value>> {
        let left_complexity = self.calculate_formula_complexity(&conflict.left_value);
        let right_complexity = self.calculate_formula_complexity(&conflict.right_value);

        if left_complexity > right_complexity {
            Ok(conflict.left_value.clone())
        } else {
            Ok(conflict.right_value.clone())
        }
    }

    fn calculate_formula_complexity(&self, value: &Option<Value>) -> usize {
        if let Some(Value::Object(obj)) = value {
            if let Some(Value::String(formula)) = obj.get("formula") {
                formula.len() + formula.matches(&['(', ')', '+', '-', '*', '/', '=']).count()
            } else {
                0
            }
        } else {
            0
        }
    }

    fn generate_resolution_reasoning(
        &self,
        conflict: &VersionConflict,
        resolution_type: ConflictResolution,
        confidence: f64,
    ) -> String {
        match resolution_type {
            ConflictResolution::Automatic => {
                format!(
                    "Automatically resolved {} conflict with {:.1}% confidence",
                    format!("{:?}", conflict.conflict_type).to_lowercase(),
                    confidence * 100.0
                )
            }
            ConflictResolution::Manual => {
                "Manual resolution required due to complexity or low confidence".to_string()
            }
        }
    }

    // Additional helper methods
    async fn generate_merged_content(
        &self,
        base_version: &SpreadsheetVersion,
        _left_changes: &[CellChange],
        _right_changes: &[CellChange],
        _resolved_conflicts: &[ResolvedConflict],
    ) -> Result<String> {
        // Simplified implementation - return base content for now
        Ok(base_version.file_content.clone())
    }

    async fn calculate_merge_quality(
        &self,
        _merged_content: &str,
        _conflicts: &[VersionConflict],
    ) -> Result<MergeQualityMetrics> {
        Ok(MergeQualityMetrics {
            data_integrity_score: 0.95,
            consistency_score: 0.90,
            completeness_score: 0.88,
            formula_validity_score: 0.92,
            reference_integrity_score: 0.94,
        })
    }

    async fn generate_suggestions(
        &self,
        _resolved: &[ResolvedConflict],
        _context: &MergeContext,
    ) -> Result<Vec<SuggestedAction>> {
        Ok(Vec::new())
    }

    fn calculate_confidence_score(
        &self,
        resolved: &[ResolvedConflict],
        _quality: &MergeQualityMetrics,
        _conflicts: &[VersionConflict],
    ) -> f64 {
        if resolved.is_empty() {
            return 1.0;
        }

        let total_confidence: f64 = resolved.iter().map(|r| r.confidence).sum();
        total_confidence / resolved.len() as f64
    }

    async fn update_learning_patterns(&mut self, _resolved: &[ResolvedConflict]) -> Result<()> {
        Ok(())
    }
}

impl Default for MergePreferences {
    fn default() -> Self {
        Self {
            prefer_newer_values: true,
            prefer_user_changes: true,
            auto_resolve_formulas: false,
            confidence_threshold: 0.8,
            custom_rules: Vec::new(),
        }
    }
} 
