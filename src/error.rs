//! Error types for pipechecker
//!
//! Defines the `PipecheckError` enum and the `Result` type alias
//! used throughout the crate.

use thiserror::Error;

/// Result type alias using `PipecheckError`
pub type Result<T> = std::result::Result<T, PipecheckError>;

/// All error variants that can occur during pipeline auditing
#[derive(Error, Debug)]
pub enum PipecheckError {
    /// Failed to read a file from disk
    #[error("Failed to read file: {0}")]
    IoError(#[from] std::io::Error),

    /// Failed to parse YAML content
    #[error("YAML parsing error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    /// Could not determine the CI/CD provider
    #[error("Unknown provider: {0}")]
    UnknownProvider(String),

    /// Pipeline structure is invalid or malformed
    #[error("Invalid pipeline structure: {0}")]
    InvalidPipeline(String),

    /// Circular dependency detected in job graph
    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),

    /// Feature or parser not yet implemented
    #[error("Feature not yet implemented: {0}")]
    NotImplemented(String),

    /// Network request failed (only available with `network` feature)
    #[cfg(feature = "network")]
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
}
