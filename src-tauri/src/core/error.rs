use thiserror::Error;

/// Main error type for EDT application
#[derive(Error, Debug)]
pub enum EdtError {
    #[error("Entity not found: {0}")]
    EntityNotFound(String),

    #[error("Invalid entity data: {0}")]
    ValidationError(String),

    #[error("Entity has {0} dependencies and cannot be deleted")]
    HasDependencies(usize),

    #[error("Link type not allowed between {0} and {1}")]
    InvalidLink(String, String),

    #[error("Calculation failed: {0}")]
    CalculationError(String),

    #[error("File system error: {0}")]
    FileSystemError(#[from] std::io::Error),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("RON serialization error: {0}")]
    RonSerError(#[from] ron::Error),

    #[error("RON parsing error: {0}")]
    RonDeError(#[from] ron::error::SpannedError),

    #[error("TOML parsing error: {0}")]
    TomlError(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    TomlSerError(#[from] toml::ser::Error),

    #[error("UUID parsing error: {0}")]
    UuidError(#[from] uuid::Error),

    #[error("Project not initialized")]
    ProjectNotInitialized,

    #[error("Project already exists at this location")]
    ProjectAlreadyExists,

    #[error("Schema version mismatch: expected {expected}, found {found}")]
    SchemaVersionMismatch { expected: String, found: String },
}

/// Result type alias for EDT operations
pub type EdtResult<T> = Result<T, EdtError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_not_found_error() {
        let error = EdtError::EntityNotFound("test-id".to_string());
        assert_eq!(
            error.to_string(),
            "Entity not found: test-id"
        );
    }

    #[test]
    fn test_validation_error() {
        let error = EdtError::ValidationError("Missing required field: name".to_string());
        assert_eq!(
            error.to_string(),
            "Invalid entity data: Missing required field: name"
        );
    }

    #[test]
    fn test_has_dependencies_error() {
        let error = EdtError::HasDependencies(5);
        assert_eq!(
            error.to_string(),
            "Entity has 5 dependencies and cannot be deleted"
        );
    }

    #[test]
    fn test_invalid_link_error() {
        let error = EdtError::InvalidLink("Task".to_string(), "Hazard".to_string());
        assert_eq!(
            error.to_string(),
            "Link type not allowed between Task and Hazard"
        );
    }

    #[test]
    fn test_calculation_error() {
        let error = EdtError::CalculationError("Division by zero".to_string());
        assert_eq!(
            error.to_string(),
            "Calculation failed: Division by zero"
        );
    }

    #[test]
    fn test_project_not_initialized_error() {
        let error = EdtError::ProjectNotInitialized;
        assert_eq!(
            error.to_string(),
            "Project not initialized"
        );
    }

    #[test]
    fn test_schema_version_mismatch_error() {
        let error = EdtError::SchemaVersionMismatch {
            expected: "1.0.0".to_string(),
            found: "2.0.0".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "Schema version mismatch: expected 1.0.0, found 2.0.0"
        );
    }

    #[test]
    fn test_io_error_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let edt_error: EdtError = io_error.into();
        assert!(matches!(edt_error, EdtError::FileSystemError(_)));
    }

    #[test]
    fn test_uuid_error_conversion() {
        use uuid::Uuid;
        use std::str::FromStr;

        let result = Uuid::from_str("invalid-uuid");
        assert!(result.is_err());

        if let Err(uuid_err) = result {
            let edt_error: EdtError = uuid_err.into();
            assert!(matches!(edt_error, EdtError::UuidError(_)));
        }
    }
}
