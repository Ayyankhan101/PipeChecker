//! Permissions auditor - checks for missing or overly-broad permissions in GitHub Actions
//!
//! GitHub Actions jobs that do not declare explicit `permissions:` inherit the
//! repository-level `GITHUB_TOKEN` permission, which defaults to `read-write`
//! for most scopes. This is a security risk.
//!
//! This auditor warns when:
//! - A GitHub Actions workflow has no top-level `permissions:` block AND
//!   at least one job has no job-level `permissions:` block.

use crate::error::Result;
use crate::models::{rule_codes, Issue, Pipeline, Provider, Severity};
use serde_yaml::Value;

/// Audit a pipeline for missing or overly-broad permissions declarations.
///
/// Only runs for GitHub Actions — GitLab CI and CircleCI handle permissions differently.
pub fn audit(pipeline: &Pipeline) -> Result<Vec<Issue>> {
    // Permissions concept is GitHub Actions-specific
    if pipeline.provider != Provider::GitHubActions {
        return Ok(vec![]);
    }

    let yaml: Value = match serde_yaml::from_str(&pipeline.source) {
        Ok(v) => v,
        Err(_) => return Ok(vec![]),
    };

    let mapping = match yaml.as_mapping() {
        Some(m) => m,
        None => return Ok(vec![]),
    };

    // Check for a top-level `permissions:` block
    let has_top_level_permissions = mapping.contains_key("permissions");

    let jobs_val = match mapping.get("jobs").and_then(|v| v.as_mapping()) {
        Some(m) => m,
        None => return Ok(vec![]),
    };

    let mut issues = Vec::new();

    for (job_id_val, job_val) in jobs_val {
        let job_name = job_id_val.as_str().unwrap_or("unknown");
        let job_map = match job_val.as_mapping() {
            Some(m) => m,
            None => continue,
        };

        let has_job_permissions = job_map.contains_key("permissions");

        if !has_top_level_permissions && !has_job_permissions {
            let (line, col) = pipeline.find_job_line(job_name, "runs-on");
            issues.push(Issue::for_job_with_code(
                Severity::Warning,
                &format!(
                    "Job '{}' has no 'permissions:' block — inherits repo-level token permissions",
                    job_name
                ),
                job_name,
                line,
                col,
                Some(
                    "Add 'permissions: read-all' (or specific scopes) to the job \
                     to follow the principle of least privilege. \
                     See: https://docs.github.com/en/actions/security-guides/automatic-token-authentication"
                        .to_string(),
                ),
                rule_codes::MISSING_PERMISSIONS,
            ));
        }
    }

    Ok(issues)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsers::github;

    fn make_pipeline(yaml: &str) -> Pipeline {
        github::parse(yaml).expect("yaml parse failed")
    }

    #[test]
    fn test_no_permissions_warns() {
        let yaml = r#"on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo hi
"#;
        let pipeline = make_pipeline(yaml);
        let issues = audit(&pipeline).unwrap();
        assert!(issues.iter().any(|i| i.message.contains("permissions")));
        assert!(issues.iter().all(|i| i.rule_code == Some("PC014")));
    }

    #[test]
    fn test_top_level_permissions_no_warn() {
        let yaml = r#"on: push
permissions: read-all
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo hi
"#;
        let pipeline = make_pipeline(yaml);
        let issues = audit(&pipeline).unwrap();
        assert!(issues.is_empty());
    }

    #[test]
    fn test_job_level_permissions_no_warn() {
        let yaml = r#"on: push
jobs:
  build:
    runs-on: ubuntu-latest
    permissions:
      contents: read
    steps:
      - run: echo hi
"#;
        let pipeline = make_pipeline(yaml);
        let issues = audit(&pipeline).unwrap();
        assert!(issues.is_empty());
    }

    #[test]
    fn test_gitlab_not_checked() {
        let pipeline = Pipeline {
            provider: Provider::GitLabCI,
            jobs: vec![],
            env: vec![],
            source: String::new(),
            is_reusable: false,
            workflow_call_inputs: Vec::new(),
            workflow_call_secrets: Vec::new(),
            workflow_rules: Vec::new(),
        };
        let issues = audit(&pipeline).unwrap();
        assert!(issues.is_empty());
    }
}
