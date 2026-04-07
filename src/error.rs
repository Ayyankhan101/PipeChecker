use thiserror::Error;

pub type Result<T> = std::result::Result<T, PipecheckError>;

#[derive(Error, Debug)]
pub enum PipecheckError {
    #[error("Failed to read file: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("YAML parsing error: {0}")]
    YamlError(#[from] serde_yaml::Error),
    
    #[error("Unknown provider: {0}")]
    UnknownProvider(String),
    
    #[error("Invalid pipeline structure: {0}")]
    InvalidPipeline(String),
    
    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),
    
    #[cfg(feature = "network")]
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
}
