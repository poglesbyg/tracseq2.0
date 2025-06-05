pub mod rules;
pub mod validators;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Core validation trait
pub trait Validator<T>: Send + Sync {
    /// Validate an item and return validation result
    fn validate(&self, item: &T) -> ValidationResult;

    /// Get validator configuration
    fn config(&self) -> ValidatorConfig;
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub metadata: HashMap<String, String>,
}

/// Validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub code: String,
    pub message: String,
    pub field: Option<String>,
    pub severity: ErrorSeverity,
}

/// Validation warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub code: String,
    pub message: String,
    pub field: Option<String>,
    pub suggestion: Option<String>,
}

/// Validation error severity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Validator configuration
#[derive(Debug, Clone)]
pub struct ValidatorConfig {
    pub name: String,
    pub version: String,
    pub strict_mode: bool,
    pub custom_rules: HashMap<String, String>,
}

/// Validation rule trait
pub trait ValidationRule<T>: Send + Sync {
    /// Check if the rule applies to the item
    fn applies_to(&self, item: &T) -> bool;

    /// Execute the validation rule
    fn validate(&self, item: &T) -> RuleResult;

    /// Get rule metadata
    fn metadata(&self) -> RuleMetadata;
}

/// Rule validation result
#[derive(Debug, Clone)]
pub struct RuleResult {
    pub passed: bool,
    pub error: Option<ValidationError>,
    pub warning: Option<ValidationWarning>,
}

/// Rule metadata
#[derive(Debug, Clone)]
pub struct RuleMetadata {
    pub name: String,
    pub description: String,
    pub category: String,
    pub severity: ErrorSeverity,
}

/// Composite validator that combines multiple validators
pub struct CompositeValidator<T> {
    validators: Vec<Box<dyn Validator<T>>>,
    config: ValidatorConfig,
}

impl<T> CompositeValidator<T> {
    pub fn new(name: String) -> Self {
        Self {
            validators: Vec::new(),
            config: ValidatorConfig {
                name,
                version: "1.0.0".to_string(),
                strict_mode: false,
                custom_rules: HashMap::new(),
            },
        }
    }

    pub fn add_validator(mut self, validator: Box<dyn Validator<T>>) -> Self {
        self.validators.push(validator);
        self
    }

    pub fn with_strict_mode(mut self, strict: bool) -> Self {
        self.config.strict_mode = strict;
        self
    }
}

impl<T> Validator<T> for CompositeValidator<T> {
    fn validate(&self, item: &T) -> ValidationResult {
        let mut all_errors = Vec::new();
        let mut all_warnings = Vec::new();
        let mut metadata = HashMap::new();

        for validator in &self.validators {
            let result = validator.validate(item);
            all_errors.extend(result.errors);
            all_warnings.extend(result.warnings);
            metadata.extend(result.metadata);
        }

        let is_valid = if self.config.strict_mode {
            all_errors.is_empty() && all_warnings.is_empty()
        } else {
            all_errors.is_empty()
        };

        ValidationResult {
            is_valid,
            errors: all_errors,
            warnings: all_warnings,
            metadata,
        }
    }

    fn config(&self) -> ValidatorConfig {
        self.config.clone()
    }
}

/// Validation context for maintaining state across validations
#[derive(Debug, Clone)]
pub struct ValidationContext {
    pub id: Uuid,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub environment: String,
    pub custom_data: HashMap<String, String>,
}

impl ValidationContext {
    pub fn new(environment: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id: None,
            session_id: None,
            environment,
            custom_data: HashMap::new(),
        }
    }

    pub fn with_user(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_session(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub fn add_data(mut self, key: String, value: String) -> Self {
        self.custom_data.insert(key, value);
        self
    }
}

/// Context-aware validator trait
pub trait ContextValidator<T>: Send + Sync {
    /// Validate with context
    fn validate_with_context(&self, item: &T, context: &ValidationContext) -> ValidationResult;
}

/// Builder for creating validation chains
pub struct ValidationChain<T> {
    rules: Vec<Box<dyn ValidationRule<T>>>,
    context: Option<ValidationContext>,
}

impl<T> ValidationChain<T> {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            context: None,
        }
    }

    pub fn add_rule(mut self, rule: Box<dyn ValidationRule<T>>) -> Self {
        self.rules.push(rule);
        self
    }

    pub fn with_context(mut self, context: ValidationContext) -> Self {
        self.context = Some(context);
        self
    }

    pub fn validate(&self, item: &T) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut metadata = HashMap::new();

        for rule in &self.rules {
            if rule.applies_to(item) {
                let result = rule.validate(item);

                if let Some(error) = result.error {
                    errors.push(error);
                }

                if let Some(warning) = result.warning {
                    warnings.push(warning);
                }

                let rule_meta = rule.metadata();
                metadata.insert(
                    format!("rule_{}", rule_meta.name),
                    if result.passed {
                        "passed".to_string()
                    } else {
                        "failed".to_string()
                    },
                );
            }
        }

        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            metadata,
        }
    }
}

impl<T> Default for ValidationChain<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationResult {
    pub fn success() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn failure(error: ValidationError) -> Self {
        Self {
            is_valid: false,
            errors: vec![error],
            warnings: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_warning(mut self, warning: ValidationWarning) -> Self {
        self.warnings.push(warning);
        self
    }

    pub fn add_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn merge(mut self, other: ValidationResult) -> Self {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
        self.metadata.extend(other.metadata);
        self.is_valid = self.is_valid && other.is_valid;
        self
    }
}

impl ValidationError {
    pub fn new(code: String, message: String) -> Self {
        Self {
            code,
            message,
            field: None,
            severity: ErrorSeverity::Medium,
        }
    }

    pub fn with_field(mut self, field: String) -> Self {
        self.field = Some(field);
        self
    }

    pub const fn with_severity(mut self, severity: ErrorSeverity) -> Self {
        self.severity = severity;
        self
    }
}

impl ValidationWarning {
    pub const fn new(code: String, message: String) -> Self {
        Self {
            code,
            message,
            field: None,
            suggestion: None,
        }
    }

    pub fn with_field(mut self, field: String) -> Self {
        self.field = Some(field);
        self
    }

    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestion = Some(suggestion);
        self
    }
}
