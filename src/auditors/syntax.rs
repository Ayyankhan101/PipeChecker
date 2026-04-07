use crate::error::Result;
use crate::models::{Issue, Pipeline, Severity};

pub fn audit(pipeline: &Pipeline) -> Result<Vec<Issue>> {
    let mut issues = Vec::new();

    // Check for empty jobs
    if pipeline.jobs.is_empty() {
        issues.push(Issue {
            severity: Severity::Error,
            message: "Pipeline has no jobs defined".to_string(),
            location: None,
            suggestion: Some("Add at least one job to your pipeline".to_string()),
        });
    }

    // Check for duplicate job IDs
    let mut seen_ids = std::collections::HashSet::new();
    for job in &pipeline.jobs {
        if !seen_ids.insert(&job.id) {
            issues.push(Issue {
                severity: Severity::Error,
                message: format!("Duplicate job ID: {}", job.id),
                location: None,
                suggestion: Some("Each job must have a unique ID".to_string()),
            });
        }

        // Check for empty steps
        if job.steps.is_empty() {
            issues.push(Issue {
                severity: Severity::Warning,
                message: format!("Job '{}' has no steps", job.id),
                location: None,
                suggestion: Some("Add steps to perform work in this job".to_string()),
            });
        }
    }

    Ok(issues)
}
