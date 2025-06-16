use super::{ErrorSeverity, RuleMetadata, RuleResult, ValidationError, ValidationRule};

/// Required field validation rule
pub struct RequiredFieldRule {
    field_name: String,
}

impl RequiredFieldRule {
    pub fn new(field_name: String) -> Self {
        Self { field_name }
    }
}

impl<T> ValidationRule<T> for RequiredFieldRule {
    fn applies_to(&self, _item: &T) -> bool {
        true // Always applies - specific field checking would be in validate()
    }

    fn validate(&self, _item: &T) -> RuleResult {
        // This is a generic implementation - specific types would implement their own logic
        RuleResult {
            passed: true,
            error: None,
            warning: None,
        }
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            name: format!("required_{}", self.field_name),
            description: format!("Validates that {} field is not empty", self.field_name),
            category: "required".to_string(),
            severity: ErrorSeverity::High,
        }
    }
}

/// String length validation rule
pub struct StringLengthRule {
    min_length: usize,
    max_length: Option<usize>,
    field_name: String,
}

impl StringLengthRule {
    pub fn new(field_name: String, min_length: usize, max_length: Option<usize>) -> Self {
        Self {
            field_name,
            min_length,
            max_length,
        }
    }
}

impl ValidationRule<String> for StringLengthRule {
    fn applies_to(&self, _item: &String) -> bool {
        true
    }

    fn validate(&self, item: &String) -> RuleResult {
        let len = item.len();

        if len < self.min_length {
            return RuleResult {
                passed: false,
                error: Some(
                    ValidationError::new(
                        "STRING_TOO_SHORT".to_string(),
                        format!(
                            "{} must be at least {} characters",
                            self.field_name, self.min_length
                        ),
                    )
                    .with_field(self.field_name.clone())
                    .with_severity(ErrorSeverity::Medium),
                ),
                warning: None,
            };
        }

        if let Some(max_len) = self.max_length {
            if len > max_len {
                return RuleResult {
                    passed: false,
                    error: Some(
                        ValidationError::new(
                            "STRING_TOO_LONG".to_string(),
                            format!("{} must be at most {} characters", self.field_name, max_len),
                        )
                        .with_field(self.field_name.clone())
                        .with_severity(ErrorSeverity::Medium),
                    ),
                    warning: None,
                };
            }
        }

        RuleResult {
            passed: true,
            error: None,
            warning: None,
        }
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            name: format!("length_{}", self.field_name),
            description: format!("Validates {} string length", self.field_name),
            category: "format".to_string(),
            severity: ErrorSeverity::Medium,
        }
    }
}

/// Email format validation rule
pub struct EmailFormatRule;

impl ValidationRule<String> for EmailFormatRule {
    fn applies_to(&self, _item: &String) -> bool {
        true
    }

    fn validate(&self, item: &String) -> RuleResult {
        // Simple email validation without regex dependency
        let has_at = item.contains('@');
        let has_dot = item.contains('.');
        let parts: Vec<&str> = item.split('@').collect();
        let is_valid =
            has_at && has_dot && parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty();

        if is_valid {
            RuleResult {
                passed: true,
                error: None,
                warning: None,
            }
        } else {
            RuleResult {
                passed: false,
                error: Some(
                    ValidationError::new(
                        "INVALID_EMAIL".to_string(),
                        "Invalid email format".to_string(),
                    )
                    .with_severity(ErrorSeverity::Medium),
                ),
                warning: None,
            }
        }
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            name: "email_format".to_string(),
            description: "Validates email format".to_string(),
            category: "format".to_string(),
            severity: ErrorSeverity::Medium,
        }
    }
}
