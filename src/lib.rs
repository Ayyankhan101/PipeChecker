//! # pipecheck
//!
//! A Rust-native CI/CD pipeline auditor that catches errors before you push.
//!
//! ## Features
//! - Syntax validation for GitHub Actions, GitLab CI, CircleCI
//! - Dependency graph analysis with cycle detection
//! - Secrets and environment variable auditing
//! - Docker image validation
//!
//! ## Example
//! ```no_run
//! use pipecheck::{audit_file, AuditOptions};
//!
//! let results = audit_file(".github/workflows/ci.yml", AuditOptions::default())?;
//! for issue in results.issues {
//!     println!("{:?}: {}", issue.severity, issue.message);
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod auditors;
pub mod config;
pub mod error;
pub mod models;
pub mod parsers;
pub mod tui;

pub use config::Config;
pub use error::{PipecheckError, Result};
pub use models::{AuditOptions, AuditResult, Issue, Severity};

/// Audit a pipeline configuration file
pub fn audit_file(path: &str, options: AuditOptions) -> Result<AuditResult> {
    let content = std::fs::read_to_string(path)?;
    audit_content(&content, options)
}

/// Audit pipeline configuration content
pub fn audit_content(content: &str, options: AuditOptions) -> Result<AuditResult> {
    let provider = parsers::detect_provider(content)?;
    let pipeline = parsers::parse(content, provider)?;
    
    let mut issues = Vec::new();
    
    // Run all auditors
    issues.extend(auditors::syntax::audit(&pipeline)?);
    issues.extend(auditors::dag::audit(&pipeline)?);
    issues.extend(auditors::secrets::audit(&pipeline)?);
    
    #[cfg(feature = "network")]
    if options.check_docker_images {
        issues.extend(auditors::docker::audit(&pipeline)?);
    }
    
    let summary = generate_summary(&issues);
    
    Ok(AuditResult {
        provider,
        issues,
        summary,
    })
}

fn generate_summary(issues: &[Issue]) -> String {
    let errors = issues.iter().filter(|i| i.severity == Severity::Error).count();
    let warnings = issues.iter().filter(|i| i.severity == Severity::Warning).count();
    format!("{} errors, {} warnings", errors, warnings)
}
