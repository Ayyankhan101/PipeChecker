//! Timeout auditor — warns when jobs lack `timeout-minutes` (or equivalent)
//!
//! Jobs without timeouts risk running forever if something hangs,
//! wasting CI minutes and money.
//!
//! Platform keywords:
//! - GitHub Actions: `timeout-minutes`
//! - GitLab CI:      `timeout`
//! - CircleCI:       `max_time`

use crate::error::Result;
use crate::models::{rule_codes, Issue, Pipeline, Severity};

/// Audit a pipeline for missing job timeouts
pub fn audit(pipeline: &Pipeline) -> Result<Vec<Issue>> {
    let mut issues = Vec::new();

    for job in &pipeline.jobs {
        if job.timeout_minutes.is_none() {
            let (line, col) = pipeline.find_job_line(&job.id, "steps");
            let keyword = match pipeline.provider {
                crate::models::Provider::GitHubActions => "timeout-minutes",
                crate::models::Provider::GitLabCI => "timeout",
                crate::models::Provider::CircleCI => "max_time",
            };
            issues.push(Issue::for_job_with_code(
                Severity::Warning,
                &format!(
                    "Job '{}' has no '{}' — may run indefinitely if something hangs",
                    job.id, keyword
                ),
                &job.id,
                line,
                col,
                Some(format!(
                    "Add '{}' to prevent runaway jobs (e.g. 30)",
                    keyword
                )),
                rule_codes::MISSING_TIMEOUT,
            ));
        }
    }

    Ok(issues)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Provider;

    #[test]
    fn test_no_timeout_warns() {
        let pipeline = Pipeline {
            provider: Provider::GitHubActions,
            jobs: vec![crate::models::Job {
                id: "build".to_string(),
                name: None,
                depends_on: vec![],
                steps: vec![],
                env: vec![],
                container_image: None,
                service_images: vec![],
                timeout_minutes: None,
                rules: vec![],
            }],
            env: vec![],
            source: "jobs:\n  build:\n    steps:\n      - run: echo hi\n".to_string(),
            is_reusable: false,
            workflow_call_inputs: vec![],
            workflow_call_secrets: vec![],
            workflow_rules: vec![],
        };

        let issues = audit(&pipeline).unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity, Severity::Warning);
        assert!(issues[0].message.contains("build"));
    }

    #[test]
    fn test_with_timeout_ok() {
        let pipeline = Pipeline {
            provider: Provider::GitHubActions,
            jobs: vec![crate::models::Job {
                id: "build".to_string(),
                name: None,
                depends_on: vec![],
                steps: vec![],
                env: vec![],
                container_image: None,
                service_images: vec![],
                timeout_minutes: Some(30),
                rules: vec![],
            }],
            env: vec![],
            source: "jobs:\n  build:\n    timeout-minutes: 30\n    steps: []\n".to_string(),
            is_reusable: false,
            workflow_call_inputs: vec![],
            workflow_call_secrets: vec![],
            workflow_rules: vec![],
        };

        let issues = audit(&pipeline).unwrap();
        assert!(issues.is_empty());
    }

    #[test]
    fn test_multiple_jobs_mixed() {
        let pipeline = Pipeline {
            provider: Provider::GitHubActions,
            jobs: vec![
                crate::models::Job {
                    id: "build".to_string(),
                    name: None,
                    depends_on: vec![],
                    steps: vec![],
                    env: vec![],
                    container_image: None,
                    service_images: vec![],
                    timeout_minutes: Some(15),
                    rules: vec![],
                },
                crate::models::Job {
                    id: "deploy".to_string(),
                    name: None,
                    depends_on: vec!["build".to_string()],
                    steps: vec![],
                    env: vec![],
                    container_image: None,
                    service_images: vec![],
                    timeout_minutes: None,
                    rules: vec![],
                },
            ],
            env: vec![],
            source: "jobs:\n  build:\n    timeout-minutes: 15\n  deploy:\n    needs: [build]\n"
                .to_string(),
            is_reusable: false,
            workflow_call_inputs: vec![],
            workflow_call_secrets: vec![],
            workflow_rules: vec![],
        };

        let issues = audit(&pipeline).unwrap();
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("deploy"));
        assert!(!issues[0].message.contains("build"));
    }
}
