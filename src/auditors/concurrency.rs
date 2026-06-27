use crate::error::Result;
use crate::models::{rule_codes, Issue, Pipeline, Provider, Severity};
use serde_yaml::Value;

/// Audit a pipeline for concurrency group configurations.
/// Checks that `cancel-in-progress: true` is present when `concurrency` is used.
pub fn audit(pipeline: &Pipeline) -> Result<Vec<Issue>> {
    let mut issues = Vec::new();

    // Concurrency is a GitHub Actions feature
    if pipeline.provider != Provider::GitHubActions {
        return Ok(issues);
    }

    let doc: Value = match serde_yaml::from_str(&pipeline.source) {
        Ok(d) => d,
        Err(_) => return Ok(issues),
    };

    // 1. Check workflow-level concurrency
    if let Some(concurrency) = doc.get("concurrency") {
        check_concurrency_block(concurrency, "workflow", pipeline, "", &mut issues);
    }

    // 2. Check job-level concurrency
    if let Some(jobs) = doc.get("jobs").and_then(|j| j.as_mapping()) {
        for (job_id_val, job_val) in jobs {
            if let Some(job_id) = job_id_val.as_str() {
                if let Some(concurrency) = job_val.get("concurrency") {
                    check_concurrency_block(concurrency, "job", pipeline, job_id, &mut issues);
                }
            }
        }
    }

    Ok(issues)
}

fn check_concurrency_block(
    concurrency: &Value,
    level: &str,
    pipeline: &Pipeline,
    job_id: &str,
    issues: &mut Vec<Issue>,
) {
    // concurrency can be a string (just the group name) or an object
    let has_cancel_in_progress = match concurrency {
        Value::Mapping(map) => {
            if let Some(cancel_val) = map.get("cancel-in-progress") {
                // Check if it's true, or a dynamic expression like ${{ ... }}
                match cancel_val {
                    Value::Bool(b) => *b,
                    Value::String(_) => true, // Assuming expression evaluates to boolean
                    _ => false,
                }
            } else {
                false
            }
        }
        Value::String(_) => false,
        _ => false,
    };

    if !has_cancel_in_progress {
        let (line, col) = if level == "workflow" {
            pipeline.find_line("concurrency:")
        } else {
            pipeline.find_job_line(job_id, "concurrency")
        };

        let msg = if level == "workflow" {
            "Workflow-level concurrency group is missing 'cancel-in-progress: true'".to_string()
        } else {
            format!(
                "Job '{}' concurrency group is missing 'cancel-in-progress: true'",
                job_id
            )
        };

        issues.push(Issue::for_job_with_code(
            Severity::Warning,
            &msg,
            if level == "job" { job_id } else { "" },
            line,
            col,
            Some("Add 'cancel-in-progress: true' to cancel outdated queued runs and save CI minutes.".to_string()),
            rule_codes::CONCURRENCY_CANCEL_MISSING,
        ));
    }
}
