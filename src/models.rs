use crate::config::Rules;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Provider {
    GitHubActions,
    GitLabCI,
    CircleCI,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// Location of an issue within a YAML file
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Location {
    /// 1-based line number
    pub line: usize,
    /// 1-based column number
    pub column: usize,
    /// Job ID this issue belongs to (if applicable)
    pub job: Option<String>,
}

/// A single issue found during auditing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub severity: Severity,
    pub message: String,
    pub location: Option<Location>,
    pub suggestion: Option<String>,
}

impl Issue {
    /// Create an issue with location info for a specific job
    pub fn for_job(
        severity: Severity,
        message: &str,
        job_id: &str,
        line: usize,
        column: usize,
        suggestion: Option<String>,
    ) -> Self {
        Issue {
            severity,
            message: message.to_string(),
            location: Some(Location {
                line,
                column,
                job: Some(job_id.to_string()),
            }),
            suggestion,
        }
    }

    /// Create an issue without location
    pub fn new(severity: Severity, message: &str, suggestion: Option<String>) -> Self {
        Issue {
            severity,
            message: message.to_string(),
            location: None,
            suggestion,
        }
    }
}

impl Pipeline {
    /// Find the 1-based line and column of a key within the source
    pub fn find_line(&self, key: &str) -> (usize, usize) {
        for (idx, line) in self.source.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with(key) {
                let column = line.len() - trimmed.len() + 1;
                return (idx + 1, column);
            }
        }
        (0, 0)
    }

    /// Find the line containing a job ID's key (e.g. job_id + ":" or job_id + "\n")
    pub fn find_job_line(&self, job_id: &str, key_hint: &str) -> (usize, usize) {
        let mut job_line = 0;
        let mut job_indent = 0;

        for (idx, line) in self.source.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Detect job block start: "  job_id:"
            let prefix = format!("{}:", job_id);
            if trimmed.starts_with(&prefix) {
                let remainder = &trimmed[prefix.len()..];
                if remainder.is_empty() || remainder.starts_with(char::is_whitespace) {
                    job_line = idx + 1;
                    job_indent = line.len() - trimmed.len();
                    break;
                }
            }
        }

        if job_line > 0 {
            // Search for key_hint within the job block
            for (idx, line) in self.source.lines().enumerate().skip(job_line) {
                let trimmed = line.trim_start();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }

                let current_indent = line.len() - trimmed.len();
                // If we've reached a line with the same or less indentation as the job ID, we've left the job block
                if current_indent <= job_indent {
                    break;
                }

                if trimmed.starts_with(&format!("{}:", key_hint)) {
                    let column = current_indent + 1;
                    return (idx + 1, column);
                }
            }
            // If hint not found in block, return the job header line
            let column = job_indent + 1;
            return (job_line, column);
        }

        (0, 0)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AuditOptions {
    pub check_docker_images: bool,
    pub strict_mode: bool,
    /// Rule toggles loaded from config file
    pub rules: Option<Rules>,
}

impl Default for AuditOptions {
    fn default() -> Self {
        Self {
            check_docker_images: true,
            strict_mode: false,
            rules: None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AuditResult {
    pub provider: Provider,
    pub issues: Vec<Issue>,
    pub summary: String,
    /// Time taken to perform the audit
    #[serde(skip)]
    pub elapsed: Duration,
}

/// Common pipeline representation
#[derive(Debug)]
pub struct Pipeline {
    pub provider: Provider,
    pub jobs: Vec<Job>,
    pub env: Vec<EnvVar>,
    /// Raw YAML content, for computing line locations
    pub source: String,
}

#[derive(Debug, Default)]
pub struct Job {
    pub id: String,
    pub name: Option<String>,
    pub depends_on: Vec<String>,
    pub steps: Vec<Step>,
    pub env: Vec<EnvVar>,
    /// Docker image used in job-level `container:` (GitHub Actions)
    pub container_image: Option<String>,
    /// Docker images used in job-level `services:` (GitHub Actions)
    pub service_images: Vec<String>,
    /// Timeout in minutes for the job (GitHub Actions `timeout-minutes`)
    pub timeout_minutes: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct Step {
    pub name: Option<String>,
    pub uses: Option<String>,
    pub run: Option<String>,
    pub env: Vec<EnvVar>,
    /// Raw `with:` block as YAML, for secret scanning
    pub with_inputs: Option<serde_yaml::Value>,
}

#[derive(Debug, Clone)]
pub struct EnvVar {
    pub key: String,
    pub value: String,
    pub is_secret: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Issue constructor tests ---

    #[test]
    fn test_issue_new_without_location() {
        let issue = Issue::new(
            Severity::Error,
            "something broke",
            Some("fix it".to_string()),
        );

        assert_eq!(issue.severity, Severity::Error);
        assert_eq!(issue.message, "something broke");
        assert!(issue.location.is_none());
        assert_eq!(issue.suggestion, Some("fix it".to_string()));
    }

    #[test]
    fn test_issue_new_no_suggestion() {
        let issue = Issue::new(Severity::Warning, "watch out", None);

        assert_eq!(issue.severity, Severity::Warning);
        assert!(issue.suggestion.is_none());
        assert!(issue.location.is_none());
    }

    #[test]
    fn test_issue_for_job() {
        let issue = Issue::for_job(
            Severity::Error,
            "cycle detected",
            "build",
            10,
            3,
            Some("break the cycle".to_string()),
        );

        assert_eq!(issue.severity, Severity::Error);
        assert_eq!(issue.message, "cycle detected");
        assert!(issue.location.is_some());
        let loc = issue.location.as_ref().unwrap();
        assert_eq!(loc.line, 10);
        assert_eq!(loc.column, 3);
        assert_eq!(loc.job, Some("build".to_string()));
        assert_eq!(issue.suggestion, Some("break the cycle".to_string()));
    }

    #[test]
    fn test_issue_for_job_no_suggestion() {
        let issue = Issue::for_job(Severity::Warning, "missing step", "test", 5, 1, None);

        assert_eq!(issue.message, "missing step");
        assert!(issue.suggestion.is_none());
        assert!(issue.location.is_some());
    }

    // --- Location tests ---

    #[test]
    fn test_location_default() {
        let loc = Location::default();
        assert_eq!(loc.line, 0);
        assert_eq!(loc.column, 0);
        assert!(loc.job.is_none());
    }

    // --- Issue with full location ---

    #[test]
    fn test_issue_with_explicit_location() {
        let issue = Issue {
            severity: Severity::Info,
            message: "info message".to_string(),
            location: Some(Location {
                line: 42,
                column: 7,
                job: Some("deploy".to_string()),
            }),
            suggestion: None,
        };

        assert_eq!(issue.location.as_ref().unwrap().line, 42);
        assert_eq!(issue.location.as_ref().unwrap().column, 7);
    }

    // --- Pipeline::find_line tests ---

    #[test]
    fn test_pipeline_find_line_basic() {
        let source = "name: CI\njobs:\n  build:\n    runs-on: ubuntu";
        let pipeline = Pipeline {
            provider: Provider::GitHubActions,
            jobs: vec![],
            env: vec![],
            source: source.to_string(),
        };

        let (line, col) = pipeline.find_line("jobs:");
        assert_eq!(line, 2);
        assert_eq!(col, 1);
    }

    #[test]
    fn test_pipeline_find_line_indented() {
        let source = "name: CI\njobs:\n  build:\n    runs-on: ubuntu";
        let pipeline = Pipeline {
            provider: Provider::GitHubActions,
            jobs: vec![],
            env: vec![],
            source: source.to_string(),
        };

        let (line, col) = pipeline.find_line("runs-on:");
        assert_eq!(line, 4);
        assert_eq!(col, 5);
    }

    #[test]
    fn test_pipeline_find_line_not_found() {
        let source = "name: CI\non: push";
        let pipeline = Pipeline {
            provider: Provider::GitHubActions,
            jobs: vec![],
            env: vec![],
            source: source.to_string(),
        };

        let (line, col) = pipeline.find_line("missing:");
        assert_eq!(line, 0);
        assert_eq!(col, 0);
    }

    // --- Pipeline::find_job_line tests ---

    #[test]
    fn test_pipeline_find_job_line_returns_hint_line() {
        // find_job_line now finds the specific line of the key_hint within the job block
        let source = "jobs:\n  build:\n    runs-on: ubuntu\n    steps:";
        let pipeline = Pipeline {
            provider: Provider::GitHubActions,
            jobs: vec![],
            env: vec![],
            source: source.to_string(),
        };

        let (line, col) = pipeline.find_job_line("build", "runs-on");
        // Returns the hint line (runs-on:), not the job header
        assert_eq!(line, 3);
        assert_eq!(col, 5);
    }

    #[test]
    fn test_pipeline_find_job_line_scoped_to_correct_job() {
        // "build-extra:" has a "runs-on" but we want the one under "build:"
        let source = "jobs:\n  build-extra:\n    runs-on: ubuntu\n  build:\n    runs-on: ubuntu";
        let pipeline = Pipeline {
            provider: Provider::GitHubActions,
            jobs: vec![],
            env: vec![],
            source: source.to_string(),
        };

        let (line, _col) = pipeline.find_job_line("build", "runs-on");
        // Should find the "runs-on" on line 5 (under "build:"), not line 3
        assert_eq!(line, 5);
    }

    #[test]
    fn test_pipeline_find_job_line_no_job_match_returns_zero() {
        // When job_id isn't found, it should not leak hints from other jobs
        let source = "jobs:\n  build:\n    runs-on: ubuntu";
        let pipeline = Pipeline {
            provider: Provider::GitHubActions,
            jobs: vec![],
            env: vec![],
            source: source.to_string(),
        };

        let (line, col) = pipeline.find_job_line("deploy", "runs-on");
        assert_eq!(line, 0);
        assert_eq!(col, 0);
    }

    #[test]
    fn test_pipeline_find_job_line_not_found_at_all() {
        let source = "jobs:\n  build:\n    runs-on: ubuntu";
        let pipeline = Pipeline {
            provider: Provider::GitHubActions,
            jobs: vec![],
            env: vec![],
            source: source.to_string(),
        };

        let (line, col) = pipeline.find_job_line("deploy", "steps");
        assert_eq!(line, 0);
        assert_eq!(col, 0);
    }

    // --- Severity equality tests ---

    #[test]
    fn test_severity_equality() {
        assert_eq!(Severity::Error, Severity::Error);
        assert_ne!(Severity::Error, Severity::Warning);
        assert_ne!(Severity::Warning, Severity::Info);
    }

    #[test]
    fn test_severity_clone() {
        let s = Severity::Error;
        assert_eq!(s.clone(), Severity::Error);
    }

    // --- EnvVar tests ---

    #[test]
    fn test_env_var_creation() {
        let env = EnvVar {
            key: "API_KEY".to_string(),
            value: "secret123".to_string(),
            is_secret: true,
        };

        assert_eq!(env.key, "API_KEY");
        assert_eq!(env.value, "secret123");
        assert!(env.is_secret);
    }

    // --- Step tests ---

    #[test]
    fn test_step_with_all_fields() {
        let step = Step {
            name: Some("Build".to_string()),
            uses: Some("actions/checkout@v4".to_string()),
            run: None,
            env: vec![],
            with_inputs: None,
        };

        assert_eq!(step.name, Some("Build".to_string()));
        assert_eq!(step.uses, Some("actions/checkout@v4".to_string()));
        assert!(step.run.is_none());
    }

    #[test]
    fn test_step_with_run_command() {
        let step = Step {
            name: Some("Test".to_string()),
            uses: None,
            run: Some("cargo test".to_string()),
            env: vec![EnvVar {
                key: "RUST_BACKTRACE".to_string(),
                value: "1".to_string(),
                is_secret: false,
            }],
            with_inputs: None,
        };

        assert_eq!(step.run, Some("cargo test".to_string()));
        assert_eq!(step.env.len(), 1);
    }

    // --- Job tests ---

    #[test]
    fn test_job_with_dependencies() {
        let job = Job {
            id: "deploy".to_string(),
            name: Some("Deploy".to_string()),
            depends_on: vec!["build".to_string(), "test".to_string()],
            steps: vec![],
            env: vec![],
            container_image: None,
            service_images: vec![],
            timeout_minutes: None,
        };

        assert_eq!(job.depends_on.len(), 2);
        assert!(job.depends_on.contains(&"build".to_string()));
        assert!(job.depends_on.contains(&"test".to_string()));
    }

    #[test]
    fn test_job_with_container_and_services() {
        let job = Job {
            id: "ci".to_string(),
            name: None,
            depends_on: vec![],
            steps: vec![],
            env: vec![],
            container_image: Some("node:18".to_string()),
            service_images: vec!["postgres:15".to_string(), "redis:7".to_string()],
            timeout_minutes: None,
        };

        assert_eq!(job.container_image, Some("node:18".to_string()));
        assert_eq!(job.service_images.len(), 2);
    }

    // --- Provider tests ---

    #[test]
    fn test_provider_clone() {
        let p = Provider::GitHubActions;
        assert_eq!(p.clone(), Provider::GitHubActions);
    }

    #[test]
    fn test_provider_equality() {
        assert_eq!(Provider::GitLabCI, Provider::GitLabCI);
        assert_ne!(Provider::GitHubActions, Provider::CircleCI);
    }
}
