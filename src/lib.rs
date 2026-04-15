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
use std::time::Instant;

pub mod auditors;
pub mod config;
pub mod error;
pub mod fix;
pub mod models;
pub mod parsers;
pub mod tui;

pub use config::load as load_config;
pub use config::{Config, Rules};
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
    let start = Instant::now();

    let provider = parsers::detect_provider(content)?;
    let pipeline = parsers::parse(content, provider)?;

    let mut issues = Vec::new();

    // Run all auditors
    issues.extend(auditors::syntax::audit(&pipeline)?);

    // DAG / cycle detection — respect config toggle
    if options
        .rules
        .as_ref()
        .map(|r| r.circular_dependencies)
        .unwrap_or(true)
    {
        issues.extend(auditors::dag::audit(&pipeline)?);
    }

    // Secrets — respect config toggle
    if options
        .rules
        .as_ref()
        .map(|r| r.missing_secrets)
        .unwrap_or(true)
    {
        issues.extend(auditors::secrets::audit(&pipeline)?);
    }

    // Timeout auditor — respect config toggle
    if options
        .rules
        .as_ref()
        .map(|r| r.timeout_validation)
        .unwrap_or(true)
    {
        issues.extend(auditors::timeout::audit(&pipeline)?);
    }

    if options.check_docker_images {
        // Pinning auditor — respect docker_latest_tag toggle
        if options
            .rules
            .as_ref()
            .map(|r| r.docker_latest_tag)
            .unwrap_or(true)
        {
            #[cfg(feature = "network")]
            {
                issues.extend(auditors::pinning::audit(&pipeline)?);
            }
            #[cfg(not(feature = "network"))]
            {
                issues.push(Issue::new(
                    Severity::Info,
                    "Docker image pinning checks are disabled because the 'network' feature is not enabled.",
                    Some("Enable the 'network' feature to run image checks".to_string()),
                ));
            }
        }
    }

    let summary = generate_summary(&issues);
    let elapsed = start.elapsed();

    Ok(AuditResult {
        provider,
        issues,
        summary,
        elapsed,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;

    // --- discover_workflows tests ---

    #[test]
    fn test_discover_workflows_github_only() {
        let dir = std::env::temp_dir().join("pipechecker_test_github_only");
        let _ = fs::remove_dir_all(&dir);
        let wf_dir = dir.join(".github/workflows");
        fs::create_dir_all(&wf_dir).unwrap();
        File::create(wf_dir.join("ci.yml")).unwrap();
        File::create(wf_dir.join("deploy.yaml")).unwrap();

        let opts = DiscoveryOptions {
            include_github: true,
            include_gitlab: false,
            include_circleci: false,
        };
        let files = discover_workflows(&dir, &opts);
        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|f| f.contains("ci.yml")));
        assert!(files.iter().any(|f| f.contains("deploy.yaml")));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_discover_workflows_nonexistent_directory() {
        let dir = std::env::temp_dir().join("pipechecker_nonexistent_dir_xyz");
        // Don't create the directory
        let files = discover_workflows(&dir, &DiscoveryOptions::default());
        assert!(files.is_empty());
    }

    #[test]
    fn test_discover_workflows_filters_extensions() {
        let dir = std::env::temp_dir().join("pipechecker_test_ext_filter");
        let _ = fs::remove_dir_all(&dir);
        let wf_dir = dir.join(".github/workflows");
        fs::create_dir_all(&wf_dir).unwrap();
        File::create(wf_dir.join("good.yml")).unwrap();
        File::create(wf_dir.join("bad.txt")).unwrap();
        File::create(wf_dir.join("skip.toml")).unwrap();

        let opts = DiscoveryOptions {
            include_github: true,
            include_gitlab: false,
            include_circleci: false,
        };
        let files = discover_workflows(&dir, &opts);
        assert_eq!(files.len(), 1);
        assert!(files[0].contains("good.yml"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_discover_workflows_gitlab_file() {
        let dir = std::env::temp_dir().join("pipechecker_test_gitlab");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let mut f = File::create(dir.join(".gitlab-ci.yml")).unwrap();
        f.write_all(b"stages:\n  - build\n").unwrap();

        let opts = DiscoveryOptions {
            include_github: false,
            include_gitlab: true,
            include_circleci: false,
        };
        let files = discover_workflows(&dir, &opts);
        assert_eq!(files.len(), 1);
        assert!(files[0].contains(".gitlab-ci.yml"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_discover_workflows_circleci_file() {
        let dir = std::env::temp_dir().join("pipechecker_test_circleci");
        let _ = fs::remove_dir_all(&dir);
        let cc_dir = dir.join(".circleci");
        fs::create_dir_all(&cc_dir).unwrap();
        File::create(cc_dir.join("config.yml")).unwrap();

        let opts = DiscoveryOptions {
            include_github: false,
            include_gitlab: false,
            include_circleci: true,
        };
        let files = discover_workflows(&dir, &opts);
        assert_eq!(files.len(), 1);
        assert!(files[0].contains("config.yml"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_discover_workflows_all_disabled() {
        let dir = std::env::temp_dir().join("pipechecker_test_disabled");
        let _ = fs::remove_dir_all(&dir);
        let wf_dir = dir.join(".github/workflows");
        fs::create_dir_all(&wf_dir).unwrap();
        File::create(wf_dir.join("ci.yml")).unwrap();

        let opts = DiscoveryOptions {
            include_github: false,
            include_gitlab: false,
            include_circleci: false,
        };
        let files = discover_workflows(&dir, &opts);
        assert!(files.is_empty());

        let _ = fs::remove_dir_all(&dir);
    }

    // --- find_line tests ---

    #[test]
    fn test_find_line_basic() {
        let content = "name: CI\non: push\njobs:\n  build:";
        let (line, col) = find_line(content, "jobs:");
        assert_eq!(line, 3);
        assert_eq!(col, 1);
    }

    #[test]
    fn test_find_line_indented() {
        let content = "name: CI\njobs:\n  build:\n    runs-on: ubuntu";
        let (line, col) = find_line(content, "runs-on:");
        assert_eq!(line, 4);
        assert_eq!(col, 5);
    }

    #[test]
    fn test_find_line_not_found() {
        let content = "name: CI\non: push";
        let (line, col) = find_line(content, "missing:");
        assert_eq!(line, 0);
        assert_eq!(col, 0);
    }

    #[test]
    fn test_find_line_empty_content() {
        let (line, col) = find_line("", "anything:");
        assert_eq!(line, 0);
        assert_eq!(col, 0);
    }

    #[test]
    fn test_find_line_first_match() {
        let content = "name: CI\nname: Duplicate\n";
        let (line, col) = find_line(content, "name:");
        assert_eq!(line, 1); // returns first match
        assert_eq!(col, 1);
    }

    // --- find_line_with_prefix tests ---

    #[test]
    fn test_find_line_with_prefix_both_match() {
        let content = "jobs:\n  build:\n    name: Build Step";
        let (line, _col) = find_line_with_prefix(content, "name:", "Build");
        assert_eq!(line, 3);
    }

    #[test]
    fn test_find_line_with_prefix_no_match() {
        let content = "jobs:\n  build:\n    name: Build";
        let (line, col) = find_line_with_prefix(content, "name:", "Deploy");
        assert_eq!(line, 0);
        assert_eq!(col, 0);
    }

    #[test]
    fn test_find_line_with_prefix_indented() {
        let content = "jobs:\n  test:\n    container:\n      image: node:18";
        let (line, _col) = find_line_with_prefix(content, "image:", "node");
        assert_eq!(line, 4);
    }

    #[test]
    fn test_find_line_with_prefix_partial_match() {
        let content = "services:\n  db:\n    image: postgres:15";
        let (line, _col) = find_line_with_prefix(content, "image:", "postgres");
        assert_eq!(line, 3);
    }

    // --- generate_summary tests ---

    #[test]
    fn test_generate_summary_zero_issues() {
        let issues = vec![];
        let summary = generate_summary(&issues);
        assert_eq!(summary, "0 errors, 0 warnings");
    }

    #[test]
    fn test_generate_summary_errors_only() {
        let issues = vec![
            Issue::new(Severity::Error, "error 1", None),
            Issue::new(Severity::Error, "error 2", None),
            Issue::new(Severity::Error, "error 3", None),
        ];
        let summary = generate_summary(&issues);
        assert_eq!(summary, "3 errors, 0 warnings");
    }

    #[test]
    fn test_generate_summary_warnings_only() {
        let issues = vec![
            Issue::new(Severity::Warning, "warn 1", None),
            Issue::new(Severity::Warning, "warn 2", None),
        ];
        let summary = generate_summary(&issues);
        assert_eq!(summary, "0 errors, 2 warnings");
    }

    #[test]
    fn test_generate_summary_mixed() {
        let issues = vec![
            Issue::new(Severity::Error, "error", None),
            Issue::new(Severity::Warning, "warn 1", None),
            Issue::new(Severity::Warning, "warn 2", None),
            Issue::new(Severity::Info, "info", None),
        ];
        let summary = generate_summary(&issues);
        assert_eq!(summary, "1 errors, 2 warnings");
    }

    // --- AuditOptions tests ---

    #[test]
    fn test_audit_options_defaults() {
        let opts = AuditOptions::default();
        assert!(opts.check_docker_images);
        assert!(!opts.strict_mode);
    }
}
