use thiserror::Error;

#[derive(Error, Debug)]
pub enum DesignTrackError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("RON serialization error: {0}")]
    RonSerialization(#[from] ron::Error),
    
    #[error("RON deserialization error: {0}")]
    RonDeserialization(#[from] ron::error::SpannedError),
    
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),
    
    #[error("UUID parse error: {0}")]
    Uuid(#[from] uuid::Error),
    
    #[error("Item not found: {0}")]
    NotFound(String),
    
    #[error("Invalid reference: {0}")]
    InvalidReference(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Module error: {0}")]
    Module(String),
    
    #[error("User interface error: {0}")]
    Ui(String),
}

impl From<inquire::InquireError> for DesignTrackError {
    fn from(err: inquire::InquireError) -> Self {
        DesignTrackError::Ui(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, DesignTrackError>;