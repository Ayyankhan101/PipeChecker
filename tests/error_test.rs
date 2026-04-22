//! Error handling and conversion tests

use pipechecker::error::{PipecheckError, Result};
use pipechecker::{audit_content, AuditOptions};
use std::io;

#[test]
fn test_error_unknown_provider() {
    let yaml = "not: a pipeline";
    let result = audit_content(yaml, AuditOptions::default());

    match result {
        Err(PipecheckError::UnknownProvider(_)) => (),
        _ => panic!("Expected UnknownProvider error, got {:?}", result),
    }
}

#[test]
fn test_error_invalid_pipeline() {
    // UnknownProvider usually happens before InvalidPipeline if detection fails
    // But we can manually create and test the variant
    let err = PipecheckError::InvalidPipeline("empty".to_string());
    assert_eq!(format!("{}", err), "Invalid pipeline structure: empty");
}

#[test]
fn test_error_io_conversion() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "file missing");
    let pipe_err: PipecheckError = io_err.into();

    match pipe_err {
        PipecheckError::IoError(_) => (),
        _ => panic!("Expected IoError, got {:?}", pipe_err),
    }
}

#[test]
fn test_error_yaml_conversion() {
    let yaml_err = serde_yaml::from_str::<serde_yaml::Value>(": invalid").unwrap_err();
    let pipe_err: PipecheckError = yaml_err.into();

    match pipe_err {
        PipecheckError::YamlError(_) => (),
        _ => panic!("Expected YamlError, got {:?}", pipe_err),
    }
}

#[test]
fn test_error_circular_dependency_message() {
    let err = PipecheckError::CircularDependency("a -> b -> a".to_string());
    assert!(format!("{}", err).contains("Circular dependency detected: a -> b -> a"));
}

#[test]
fn test_error_not_implemented_message() {
    let err = PipecheckError::NotImplemented("Azure Pipelines".to_string());
    assert!(format!("{}", err).contains("Feature not yet implemented: Azure Pipelines"));
}

#[test]
fn test_result_type_alias() {
    fn produces_error() -> Result<()> {
        Err(PipecheckError::InvalidPipeline("test".to_string()))
    }

    assert!(produces_error().is_err());
}
