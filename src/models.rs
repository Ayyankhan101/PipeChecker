use serde::{Deserialize, Serialize};

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
        for (idx, line) in self.source.lines().enumerate() {
            let trimmed = line.trim();
            // Detect job block start: "  job_id:" (exact match or followed by whitespace)
            let prefix = format!("{}:", job_id);
            if trimmed.starts_with(&prefix) {
                // Ensure we aren't matching a longer job name that merely starts with this prefix
                let remainder = &trimmed[prefix.len()..];
                if remainder.is_empty() || remainder.starts_with(char::is_whitespace) {
                    let column = line.len() - line.trim_start().len() + 1;
                    return (idx + 1, column);
                }
            }
            // Inside job block, look for the key
            if trimmed.starts_with(&format!("{}:", key_hint)) {
                let column = line.len() - line.trim_start().len() + 1;
                return (idx + 1, column);
            }
        }
        (0, 0)
    }
}

#[derive(Debug, Clone)]
pub struct AuditOptions {
    pub check_docker_images: bool,
    pub strict_mode: bool,
}

impl Default for AuditOptions {
    fn default() -> Self {
        Self {
            check_docker_images: true,
            strict_mode: false,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AuditResult {
    pub provider: Provider,
    pub issues: Vec<Issue>,
    pub summary: String,
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

#[derive(Debug)]
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
