//! Syntax auditor - validates pipeline structure and basic requirements
//!
//! This module checks for structural issues:
//! - Empty pipelines (no jobs defined)
//! - Duplicate job IDs
//! - Jobs with no steps
//! - Missing `needs` targets

use crate::error::Result;
use crate::models::{Issue, Pipeline, Severity};
use std::collections::HashSet;

/// Audit a pipeline for syntax and structural issues
///
/// Checks for:
/// - Empty job list
/// - Duplicate job IDs
/// - Jobs with no steps
/// - Missing dependency targets
pub fn audit(pipeline: &Pipeline) -> Result<Vec<Issue>> {
    let mut issues = Vec::new();
    let job_ids: HashSet<&str> = pipeline.jobs.iter().map(|j| j.id.as_str()).collect();

    // Check for empty jobs
    if pipeline.jobs.is_empty() {
        let (line, col) = pipeline.find_line("jobs:");
        issues.push(Issue::for_job(
            Severity::Error,
            "Pipeline has no jobs defined",
            "",
            line,
            col,
            Some("Add at least one job to your pipeline".to_string()),
        ));
    }

    // Check for duplicate job IDs
    let mut seen_ids = HashSet::new();
    for job in &pipeline.jobs {
        if !seen_ids.insert(&job.id) {
            let (line, col) = pipeline.find_job_line(&job.id, "runs-on");
            issues.push(Issue::for_job(
                Severity::Error,
                &format!("Duplicate job ID: {}", job.id),
                &job.id,
                line,
                col,
                Some("Each job must have a unique ID".to_string()),
            ));
        }

        // Check for empty steps
        if job.steps.is_empty() {
            let (line, col) = pipeline.find_job_line(&job.id, "runs-on");
            issues.push(Issue::for_job(
                Severity::Warning,
                &format!("Job '{}' has no steps", job.id),
                &job.id,
                line,
                col,
                Some("Add steps to perform work in this job".to_string()),
            ));
        }

        // Check for missing needs targets
        for dep in &job.depends_on {
            if !job_ids.contains(dep.as_str()) {
                let (line, col) = pipeline.find_job_line(&job.id, "needs");
                issues.push(Issue::for_job(
                    Severity::Error,
                    &format!("Job '{}' depends on non-existent job '{}'", job.id, dep),
                    &job.id,
                    line,
                    col,
                    Some(format!(
                        "Add a job with id '{}' or remove the dependency",
                        dep
                    )),
                ));
            }
        }
    }

    Ok(issues)
}
