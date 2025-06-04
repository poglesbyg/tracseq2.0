use super::{ErrorSeverity, ValidationError, ValidationResult, Validator, ValidatorConfig};
use std::collections::HashMap;

/// Sample validator for validating sample data
pub struct SampleValidator {
    config: ValidatorConfig,
}

impl SampleValidator {
    pub fn new() -> Self {
        Self {
            config: ValidatorConfig {
                name: "SampleValidator".to_string(),
                version: "1.0.0".to_string(),
                strict_mode: false,
                custom_rules: HashMap::new(),
            },
        }
    }

    pub fn with_strict_mode(mut self) -> Self {
        self.config.strict_mode = true;
        self
    }
}

impl<T> Validator<T> for SampleValidator
where
    T: SampleValidatable,
{
    fn validate(&self, item: &T) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut metadata = HashMap::new();

        // Validate sample name
        if item.get_name().is_empty() {
            errors.push(
                ValidationError::new(
                    "EMPTY_NAME".to_string(),
                    "Sample name cannot be empty".to_string(),
                )
                .with_field("name".to_string())
                .with_severity(ErrorSeverity::High),
            );
        } else if item.get_name().len() < 3 {
            errors.push(
                ValidationError::new(
                    "NAME_TOO_SHORT".to_string(),
                    "Sample name must be at least 3 characters".to_string(),
                )
                .with_field("name".to_string())
                .with_severity(ErrorSeverity::Medium),
            );
        }

        // Validate barcode
        if item.get_barcode().is_empty() {
            errors.push(
                ValidationError::new(
                    "EMPTY_BARCODE".to_string(),
                    "Barcode cannot be empty".to_string(),
                )
                .with_field("barcode".to_string())
                .with_severity(ErrorSeverity::High),
            );
        } else if item.get_barcode().len() < 6 {
            errors.push(
                ValidationError::new(
                    "BARCODE_TOO_SHORT".to_string(),
                    "Barcode must be at least 6 characters".to_string(),
                )
                .with_field("barcode".to_string())
                .with_severity(ErrorSeverity::Medium),
            );
        }

        // Validate location
        if item.get_location().is_empty() && self.config.strict_mode {
            errors.push(
                ValidationError::new(
                    "EMPTY_LOCATION".to_string(),
                    "Location cannot be empty in strict mode".to_string(),
                )
                .with_field("location".to_string())
                .with_severity(ErrorSeverity::Medium),
            );
        }

        metadata.insert("validator".to_string(), "SampleValidator".to_string());
        metadata.insert(
            "strict_mode".to_string(),
            self.config.strict_mode.to_string(),
        );

        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            metadata,
        }
    }

    fn config(&self) -> ValidatorConfig {
        self.config.clone()
    }
}

impl Default for SampleValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for types that can be validated as samples
pub trait SampleValidatable {
    fn get_name(&self) -> &str;
    fn get_barcode(&self) -> &str;
    fn get_location(&self) -> &str;
}

/// Template validator for validating template data
pub struct TemplateValidator {
    config: ValidatorConfig,
    allowed_types: Vec<String>,
}

impl TemplateValidator {
    pub fn new(allowed_types: Vec<String>) -> Self {
        Self {
            config: ValidatorConfig {
                name: "TemplateValidator".to_string(),
                version: "1.0.0".to_string(),
                strict_mode: false,
                custom_rules: HashMap::new(),
            },
            allowed_types,
        }
    }
}

impl<T> Validator<T> for TemplateValidator
where
    T: TemplateValidatable,
{
    fn validate(&self, item: &T) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut metadata = HashMap::new();

        // Validate template name
        if item.get_name().is_empty() {
            errors.push(
                ValidationError::new(
                    "EMPTY_TEMPLATE_NAME".to_string(),
                    "Template name cannot be empty".to_string(),
                )
                .with_field("name".to_string())
                .with_severity(ErrorSeverity::High),
            );
        }

        // Validate file type
        if !self.allowed_types.is_empty() && !self.allowed_types.contains(&item.get_file_type()) {
            errors.push(
                ValidationError::new(
                    "INVALID_FILE_TYPE".to_string(),
                    format!("File type '{}' is not allowed", item.get_file_type()),
                )
                .with_field("file_type".to_string())
                .with_severity(ErrorSeverity::High),
            );
        }

        metadata.insert("validator".to_string(), "TemplateValidator".to_string());
        metadata.insert("allowed_types".to_string(), self.allowed_types.join(","));

        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            metadata,
        }
    }

    fn config(&self) -> ValidatorConfig {
        self.config.clone()
    }
}

/// Trait for types that can be validated as templates
pub trait TemplateValidatable {
    fn get_name(&self) -> &str;
    fn get_file_type(&self) -> String;
    fn get_description(&self) -> Option<&str>;
}

/// String validator for basic string validation
pub struct StringValidator {
    config: ValidatorConfig,
    min_length: Option<usize>,
    max_length: Option<usize>,
}

impl StringValidator {
    pub fn new() -> Self {
        Self {
            config: ValidatorConfig {
                name: "StringValidator".to_string(),
                version: "1.0.0".to_string(),
                strict_mode: false,
                custom_rules: HashMap::new(),
            },
            min_length: None,
            max_length: None,
        }
    }

    pub fn with_length_range(mut self, min: usize, max: usize) -> Self {
        self.min_length = Some(min);
        self.max_length = Some(max);
        self
    }
}

impl Validator<String> for StringValidator {
    fn validate(&self, item: &String) -> ValidationResult {
        let mut errors = Vec::new();
        let mut metadata = HashMap::new();

        if let Some(min) = self.min_length {
            if item.len() < min {
                errors.push(
                    ValidationError::new(
                        "STRING_TOO_SHORT".to_string(),
                        format!("String must be at least {} characters", min),
                    )
                    .with_severity(ErrorSeverity::Medium),
                );
            }
        }

        if let Some(max) = self.max_length {
            if item.len() > max {
                errors.push(
                    ValidationError::new(
                        "STRING_TOO_LONG".to_string(),
                        format!("String must be at most {} characters", max),
                    )
                    .with_severity(ErrorSeverity::Medium),
                );
            }
        }

        metadata.insert("validator".to_string(), "StringValidator".to_string());
        metadata.insert("length".to_string(), item.len().to_string());

        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings: Vec::new(),
            metadata,
        }
    }

    fn config(&self) -> ValidatorConfig {
        self.config.clone()
    }
}

impl Default for StringValidator {
    fn default() -> Self {
        Self::new()
    }
}
