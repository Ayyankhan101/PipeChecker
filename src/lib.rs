//! # pipechecker
//!
//! A Rust-native CI/CD pipeline auditor that catches errors before you push.
//!
//! # Features
//! - Syntax validation for GitHub Actions, GitLab CI, CircleCI
//! - Dependency graph analysis with cycle detection
//! - Secrets and environment variable auditing
//! - Docker image and action pinning validation
//!
//! # Example
//! ```no_run
//! use pipechecker::{audit_file, AuditOptions};
//!
//! let results = audit_file(".github/workflows/ci.yml", AuditOptions::default())?;
//! for issue in results.issues {
//!     println!("{:?}: {}", issue.severity, issue.message);
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::fs;
use std::path::Path;

pub mod auditors;
pub mod config;
pub mod error;
pub mod fix;
pub mod models;
pub mod parsers;
pub mod tui;

pub use config::load as load_config;
pub use config::Config;
pub use error::{PipecheckError, Result};
pub use models::{AuditOptions, AuditResult, Issue, Severity};

/// Audit a pipeline configuration file
#[must_use = "audit results should be handled"]
pub fn audit_file(path: &str, options: AuditOptions) -> Result<AuditResult> {
    let content = std::fs::read_to_string(path)?;
    audit_content(&content, options)
}

/// Audit pipeline configuration content
#[must_use = "audit results should be handled"]
pub fn audit_content(content: &str, options: AuditOptions) -> Result<AuditResult> {
    let provider = parsers::detect_provider(content)?;
    let pipeline = parsers::parse(content, provider)?;

    let mut issues = Vec::new();

    // Run all auditors
    issues.extend(auditors::syntax::audit(&pipeline)?);
    issues.extend(auditors::dag::audit(&pipeline)?);
    issues.extend(auditors::secrets::audit(&pipeline)?);

    if options.check_docker_images {
        #[cfg(feature = "network")]
        issues.extend(auditors::pinning::audit(&pipeline)?);
    }

    let summary = generate_summary(&issues);

    Ok(AuditResult {
        provider,
        issues,
        summary,
    })
}

fn generate_summary(issues: &[Issue]) -> String {
    let errors = issues
        .iter()
        .filter(|i| i.severity == Severity::Error)
        .count();
    let warnings = issues
        .iter()
        .filter(|i| i.severity == Severity::Warning)
        .count();
    format!("{} errors, {} warnings", errors, warnings)
}

/// Discovery configuration for finding CI/CD workflow files
#[derive(Debug, Clone)]
pub struct DiscoveryOptions {
    /// Whether to include GitHub Actions workflows
    pub include_github: bool,
    /// Whether to include GitLab CI configuration
    pub include_gitlab: bool,
    /// Whether to include CircleCI configuration
    pub include_circleci: bool,
}

impl Default for DiscoveryOptions {
    fn default() -> Self {
        Self {
            include_github: true,
            include_gitlab: true,
            include_circleci: true,
        }
    }
}

/// Discover CI/CD workflow files in the given directory
///
/// Searches for:
/// - `.github/workflows/*.yml` and `*.yaml`
/// - `.gitlab-ci.yml`
/// - `.circleci/config.yml`
pub fn discover_workflows(base: &Path, options: &DiscoveryOptions) -> Vec<String> {
    let mut files = Vec::new();

    if options.include_github {
        let wf_dir = base.join(".github/workflows");
        if let Ok(entries) = fs::read_dir(&wf_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "yml" || ext == "yaml" {
                        if let Some(s) = path.to_str() {
                            files.push(s.to_string());
                        }
                    }
                }
            }
        }
    }

    if options.include_gitlab {
        let p = base.join(".gitlab-ci.yml");
        if p.exists() {
            if let Some(s) = p.to_str() {
                files.push(s.to_string());
            }
        }
    }

    if options.include_circleci {
        let p = base.join(".circleci/config.yml");
        if p.exists() {
            if let Some(s) = p.to_str() {
                files.push(s.to_string());
            }
        }
    }

    files
}

/// Find the line number of a key or pattern in raw YAML content
///
/// Returns a 1-based line number, or 0 if not found.
/// Also returns the column (indentation) of the key.
pub fn find_line(content: &str, key: &str) -> (usize, usize) {
    for (idx, line) in content.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with(key) {
            let column = line.len() - trimmed.len() + 1;
            return (idx + 1, column);
        }
    }
    (0, 0)
}

/// Find the line number of a value within a specific key's context
///
/// Searches for a line containing both the key prefix and the search term.
/// Returns 1-based (line, column) or (0, 0) if not found.
pub fn find_line_with_prefix(content: &str, key_prefix: &str, search: &str) -> (usize, usize) {
    for (idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.contains(key_prefix) && trimmed.contains(search) {
            let column = line.len() - line.trim_start().len() + 1;
            return (idx + 1, column);
        }
    }
    (0, 0)
}
