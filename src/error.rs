#![allow(dead_code)]
use std::path::PathBuf;

use chrono::Datelike;
use chrono::Timelike;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppError {
    FileSystem { operation: String, path: PathBuf, source: String },
    ProfileService { operation: ProfileOperation, details: String },
    ModService { operation: ModOperation, mod_name: String, details: String },
    UiService { operation: UiOperation, details: String },
    Configuration { setting: String, value: String, reason: String },
    System { component: String, details: String },
    UserInput { field: String, value: String, expected: String },
    External { service: String, details: String },
    Validation(ValidationError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProfileOperation {
    Switch,
    Create,
    Delete,
    Update,
    Load,
    Save,
}

impl ProfileOperation {
    pub fn failure_message(&self) -> String {
        match self {
            ProfileOperation::Switch => "Failed to switch profile".to_string(),
            ProfileOperation::Create => "Failed to create new profile".to_string(),
            ProfileOperation::Delete => "Failed to delete profile".to_string(),
            ProfileOperation::Update => "Failed to update profile".to_string(),
            ProfileOperation::Load => "Failed to load profile".to_string(),
            ProfileOperation::Save => "Failed to save profile".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModOperation {
    Install,
    Uninstall,
    Toggle,
    Load,
    Validate,
}

impl ModOperation {
    pub fn failure_message(&self) -> String {
        match self {
            ModOperation::Install => "Failed to install mod".to_string(),
            ModOperation::Uninstall => "Failed to uninstall mod".to_string(),
            ModOperation::Toggle => "Failed to toggle mod".to_string(),
            ModOperation::Load => "Failed to load mod".to_string(),
            ModOperation::Validate => "Mod validation failed".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UiOperation {
    DirectoryLoad,
    FileToggle,
    ThemeSwitch,
    WindowManagement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationError {
    EmptyField(String),
    InvalidPath(PathBuf),
    InvalidFormat { field: String, expected: String },
    OutOfRange { field: String, min: i32, max: i32, actual: i32 },
    Conflict { field1: String, field2: String, reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub error: AppError,
    pub timestamp: ErrorTime,
    pub user_message: String,
    pub technical_details: Option<String>,
    pub suggested_action: Option<String>,
    pub recoverable: bool,
}

impl std::default::Default for ErrorContext {
    fn default() -> Self {
        Self {
            error: AppError::Validation(ValidationError::EmptyField("".to_string())),
            timestamp: chrono::Utc::now().into(),
            user_message: "".to_string(),
            technical_details: None,
            suggested_action: None,
            recoverable: true,
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::FileSystem { operation, path, source } => {
                write!(
                    f,
                    "File system error during '{}' on '{}': {}",
                    operation,
                    path.display(),
                    source
                )
            }
            AppError::ProfileService { operation, details } => {
                write!(f, "Profile service error during {operation:?}: {details}")
            }
            AppError::ModService { operation, mod_name, details } => {
                write!(f, "Mod service error during {operation:?} for '{mod_name}': {details}",)
            }
            AppError::UiService { operation, details } => {
                write!(f, "UI service error during {operation:?}: {details}")
            }
            AppError::Configuration { setting, value, reason } => {
                write!(f, "Configuration error for '{setting}' with value '{value}': {reason}",)
            }
            AppError::Validation(err) => write!(f, "Validation error: {err:?}"),
            AppError::System { component, details } => {
                write!(f, "System error in '{component}': {details}")
            }
            AppError::UserInput { field, value, expected } => {
                write!(f, "Invalid input for '{field}': got '{value}', expected {expected}")
            }
            AppError::External { service, details } => {
                write!(f, "External service error from '{service}': {details}")
            }
        }
    }
}

impl ErrorContext {
    pub fn new(error: AppError) -> Self {
        let (user_message, technical_details, suggested_action, recoverable) =
            Self::generate_context(&error);

        Self {
            error,
            timestamp: chrono::Utc::now().into(),
            user_message,
            technical_details,
            suggested_action,
            recoverable,
        }
    }

    fn generate_context(error: &AppError) -> (String, Option<String>, Option<String>, bool) {
        match error {
            AppError::FileSystem { operation, path, .. } => (
                format!("Failed to {operation} file at {}", path.display()),
                Some("Check if the file exists and you have proper permissions".to_owned()),
                Some("Try selecting a different file or check file permissions".to_string()),
                true,
            ),

            AppError::ProfileService { operation, .. } => (
                operation.failure_message(),
                None,
                Some("Check your profile configuration and try again".to_string()),
                true,
            ),

            AppError::ModService { operation, .. } => (
                operation.failure_message(),
                None,
                Some("Check if the mod file is valid and try again".to_string()),
                true,
            ),

            AppError::Validation(ValidationError::EmptyField(field)) => (
                format!("The {field} field cannot be empty"),
                None,
                Some(format!("Please enter a value for {field}")),
                true,
            ),

            AppError::Validation(ValidationError::InvalidPath(path)) => (
                format!("Invalid path: {}", path.display()),
                None,
                Some("Please select a valid file or directory path".to_string()),
                true,
            ),

            AppError::System { component, .. } => {
                (format!("System error in {component}"), None, None, false)
            }
            _ => (
                "An unexpected error occurred".to_string(),
                None,
                Some("Please try the operation again".to_string()),
                true,
            ),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorTime {
    pub seconds: u64,
    pub minutes: u64,
    pub hour: u64,
    pub day: u64,
    pub month: u64,
    pub year: u64,
}

impl From<chrono::DateTime<chrono::Utc>> for ErrorTime {
    fn from(value: chrono::DateTime<chrono::Utc>) -> Self {
        Self {
            seconds: value.timestamp() as u64,
            minutes: value.minute() as u64,
            hour: value.hour() as u64,
            day: value.day() as u64,
            month: value.month() as u64,
            year: value.year() as u64,
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::System { component: "IO".to_string(), details: err.to_string() }
    }
}

impl From<ValidationError> for AppError {
    fn from(err: ValidationError) -> Self {
        AppError::Validation(err)
    }
}
