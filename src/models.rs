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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub severity: Severity,
    pub message: String,
    pub location: Option<Location>,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub line: usize,
    pub column: Option<usize>,
    pub job: Option<String>,
}

#[derive(Debug, Default)]
pub struct AuditOptions {
    pub check_docker_images: bool,
    pub strict_mode: bool,
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
}

#[derive(Debug)]
pub struct Job {
    pub id: String,
    pub name: Option<String>,
    pub depends_on: Vec<String>,
    pub steps: Vec<Step>,
    pub env: Vec<EnvVar>,
}

#[derive(Debug)]
pub struct Step {
    pub name: Option<String>,
    pub uses: Option<String>,
    pub run: Option<String>,
    pub env: Vec<EnvVar>,
}

#[derive(Debug)]
pub struct EnvVar {
    pub key: String,
    pub value: String,
    pub is_secret: bool,
}
