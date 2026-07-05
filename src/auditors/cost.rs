//! Cost and efficiency auditor — detects configuration patterns that waste CI resources
//!
//! This module checks for:
//! - Jobs with `timeout-minutes` > 60 (Warning, PC020)
//! - Missing workflow-level `concurrency` group (Info, PC021)
//!
//! These are informational/warning level checks, not errors.

use crate::error::Result;
use crate::models::{rule_codes, Issue, Pipeline, Provider, Severity};
use serde_yaml::Value;

/// Audit a pipeline for cost and efficiency concerns
pub fn audit(pipeline: &Pipeline) -> Result<Vec<Issue>> {
    let mut issues = Vec::new();

    if pipeline.provider != Provider::GitHubActions {
        return Ok(issues);
    }

    let doc: Value = match serde_yaml::from_str(&pipeline.source) {
        Ok(d) => d,
        Err(_) => return Ok(issues),
    };

    // Check for excessive timeout on jobs
    for job in &pipeline.jobs {
        if let Some(timeout) = job.timeout_minutes {
            if timeout > 60 {
                let (line, col) = pipeline.find_job_line(&job.id, "timeout-minutes");
                issues.push(Issue::for_job_with_code(
                    Severity::Warning,
                    &format!(
                        "Job '{}' has an excessive timeout of {} minutes",
                        job.id, timeout
                    ),
                    &job.id,
                    line,
                    col,
                    Some(
                        "Consider reducing timeout to 60 minutes or less to prevent runaway jobs from consuming excessive CI minutes"
                            .to_string(),
                    ),
                    rule_codes::EXCESSIVE_TIMEOUT,
                ));
            }
        }
    }

    // Check for missing concurrency group at workflow level
    if doc.get("concurrency").is_none() {
        let (line, col) = pipeline.find_line("on:");
        issues.push(Issue::with_code(
            Severity::Info,
            "Workflow is missing a 'concurrency' group — multiple runs may execute simultaneously",
            Some(
                "Add a concurrency group to cancel outdated runs and save CI minutes:\nconcurrency:\n  group: ${{ github.workflow }}-${{ github.ref }}\n  cancel-in-progress: true"
                    .to_string(),
            ),
            rule_codes::MISSING_CONCURRENCY,
        ));
        // Use the 'on:' line as reference since concurrency is typically at top level
        let _ = (line, col);
    }

    Ok(issues)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Provider;

    fn make_pipeline(source: &str, jobs: Vec<crate::models::Job>) -> Pipeline {
        Pipeline {
            provider: Provider::GitHubActions,
            jobs,
            env: vec![],
            source: source.to_string(),
            is_reusable: false,
            workflow_call_inputs: Vec::new(),
            workflow_call_secrets: Vec::new(),
            workflow_rules: Vec::new(),
        }
    }

    #[test]
    fn test_excessive_timeout_warns() {
        let source = "concurrency:\n  group: ci\njobs:\n  build:\n    runs-on: ubuntu\n    timeout-minutes: 120\n    steps: []\n";
        let pipeline = make_pipeline(
            source,
            vec![crate::models::Job {
                id: "build".to_string(),
                name: None,
                depends_on: vec![],
                steps: vec![],
                env: vec![],
                container_image: None,
                service_images: vec![],
                timeout_minutes: Some(120),
                rules: Vec::new(),
            }],
        );
        let issues = audit(&pipeline).unwrap();
        assert!(issues
            .iter()
            .any(|i| i.rule_code == Some(rule_codes::EXCESSIVE_TIMEOUT)));
    }

    #[test]
    fn test_normal_timeout_ok() {
        let source = "concurrency:\n  group: ci\njobs:\n  build:\n    runs-on: ubuntu\n    timeout-minutes: 30\n    steps: []\n";
        let pipeline = make_pipeline(
            source,
            vec![crate::models::Job {
                id: "build".to_string(),
                name: None,
                depends_on: vec![],
                steps: vec![],
                env: vec![],
                container_image: None,
                service_images: vec![],
                timeout_minutes: Some(30),
                rules: Vec::new(),
            }],
        );
        let issues = audit(&pipeline).unwrap();
        assert!(!issues
            .iter()
            .any(|i| i.rule_code == Some(rule_codes::EXCESSIVE_TIMEOUT)));
    }

    #[test]
    fn test_missing_concurrency_group() {
        let source = "name: CI\non: push\njobs:\n  build:\n    runs-on: ubuntu\n    steps: []\n";
        let pipeline = make_pipeline(source, vec![]);
        let issues = audit(&pipeline).unwrap();
        assert!(issues
            .iter()
            .any(|i| i.rule_code == Some(rule_codes::MISSING_CONCURRENCY)));
    }

    #[test]
    fn test_has_concurrency_group_ok() {
        let source = "name: CI\non: push\nconcurrency:\n  group: ci\n  cancel-in-progress: true\njobs:\n  build:\n    runs-on: ubuntu\n    steps: []\n";
        let pipeline = make_pipeline(source, vec![]);
        let issues = audit(&pipeline).unwrap();
        assert!(!issues
            .iter()
            .any(|i| i.rule_code == Some(rule_codes::MISSING_CONCURRENCY)));
    }

    #[test]
    fn test_gitlab_provider_skipped() {
        let pipeline = Pipeline {
            provider: Provider::GitLabCI,
            jobs: vec![],
            env: vec![],
            source: "stages:\n  - build\n".to_string(),
            is_reusable: false,
            workflow_call_inputs: Vec::new(),
            workflow_call_secrets: Vec::new(),
            workflow_rules: Vec::new(),
        };
        let issues = audit(&pipeline).unwrap();
        assert!(issues.is_empty());
    }
}
