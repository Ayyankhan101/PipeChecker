//! Deprecated features auditor — detects usage of deprecated GitHub Actions features
//!
//! This module checks for:
//! - `set-output` command in run blocks (deprecated, should use $GITHUB_OUTPUT)
//! - `save-state` command in run blocks (deprecated, should use $GITHUB_STATE)
//! - `node:12` or `node:16` container images (EOL)
//! - Very old action versions (`@v1`, `@v2`) for common actions
//!
//! All findings use rule code PC019 with Warning severity.

use crate::error::Result;
use crate::models::{rule_codes, Issue, Pipeline, Provider, Severity};
use regex::Regex;
use std::sync::OnceLock;

static SET_OUTPUT_REGEX: OnceLock<Regex> = OnceLock::new();
static SAVE_STATE_REGEX: OnceLock<Regex> = OnceLock::new();
static OLD_VERSION_REGEX: OnceLock<Regex> = OnceLock::new();

/// EOL node container image tags
const EOL_NODE_TAGS: &[&str] = &["node:12", "node:16", "node:12-", "node:16-"];

/// Old action versions to flag
const OLD_VERSIONS: &[&str] = &["@v1", "@v2"];

/// Common actions that should be on modern versions
const COMMON_ACTIONS: &[&str] = &[
    "actions/checkout",
    "actions/setup-node",
    "actions/setup-python",
    "actions/setup-java",
    "actions/setup-go",
    "actions/cache",
    "actions/upload-artifact",
    "actions/download-artifact",
    "actions/setup-dotnet",
    "actions/setup-php",
    "actions/setup-ruby",
    "actions/setup-java-github",
];

/// Audit a pipeline for deprecated feature usage
pub fn audit(pipeline: &Pipeline) -> Result<Vec<Issue>> {
    let mut issues = Vec::new();

    if pipeline.provider != Provider::GitHubActions {
        return Ok(issues);
    }

    let set_output_re =
        SET_OUTPUT_REGEX.get_or_init(|| Regex::new(r"::set-output\s+name=\w+::").unwrap());
    let save_state_re =
        SAVE_STATE_REGEX.get_or_init(|| Regex::new(r"::save-state\s+name=\w+::").unwrap());
    let _old_version_re = OLD_VERSION_REGEX.get_or_init(|| Regex::new(r"@\w+").unwrap());

    for job in &pipeline.jobs {
        // Check for EOL node container images
        if let Some(image) = &job.container_image {
            let lower = image.to_lowercase();
            for eol_tag in EOL_NODE_TAGS {
                if lower.starts_with(eol_tag) || lower == *eol_tag {
                    let (line, col) = pipeline.find_job_line(&job.id, "container");
                    issues.push(Issue::for_job_with_code(
                        Severity::Warning,
                        &format!(
                            "Job '{}' uses end-of-life container image: {}",
                            job.id, image
                        ),
                        &job.id,
                        line,
                        col,
                        Some("Upgrade to node:18 or node:20".to_string()),
                        rule_codes::DEPRECATED_ACTION,
                    ));
                    break;
                }
            }
        }

        for step in &job.steps {
            // Check run commands for deprecated set-output
            if let Some(run) = &step.run {
                if set_output_re.is_match(run) {
                    let (line, col) = pipeline.find_job_line(&job.id, "run");
                    issues.push(Issue::for_job_with_code(
                        Severity::Warning,
                        &format!("Job '{}' uses deprecated 'set-output' command", job.id),
                        &job.id,
                        line,
                        col,
                        Some(
                            "Use $GITHUB_OUTPUT instead: echo 'name=value' >> $GITHUB_OUTPUT"
                                .to_string(),
                        ),
                        rule_codes::DEPRECATED_ACTION,
                    ));
                }

                if save_state_re.is_match(run) {
                    let (line, col) = pipeline.find_job_line(&job.id, "run");
                    issues.push(Issue::for_job_with_code(
                        Severity::Warning,
                        &format!("Job '{}' uses deprecated 'save-state' command", job.id),
                        &job.id,
                        line,
                        col,
                        Some(
                            "Use $GITHUB_STATE instead: echo 'name=value' >> $GITHUB_STATE"
                                .to_string(),
                        ),
                        rule_codes::DEPRECATED_ACTION,
                    ));
                }
            }

            // Check for old action versions on common actions
            if let Some(uses) = &step.uses {
                for common in COMMON_ACTIONS {
                    if uses.starts_with(common) {
                        for old_ver in OLD_VERSIONS {
                            if uses.ends_with(old_ver) {
                                let (line, col) = pipeline.find_job_line(&job.id, "uses");
                                issues.push(Issue::for_job_with_code(
                                    Severity::Warning,
                                    &format!(
                                        "Job '{}' uses old version of {}: {}",
                                        job.id, common, uses
                                    ),
                                    &job.id,
                                    line,
                                    col,
                                    Some(format!(
                                        "Upgrade to the latest version (e.g. {}@v4)",
                                        common
                                    )),
                                    rule_codes::DEPRECATED_ACTION,
                                ));
                                break;
                            }
                        }
                    }
                }
            }
        }
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

    fn make_job(
        id: &str,
        run: Option<String>,
        uses: Option<String>,
        container: Option<String>,
    ) -> crate::models::Job {
        crate::models::Job {
            id: id.to_string(),
            name: None,
            depends_on: vec![],
            steps: vec![crate::models::Step {
                name: None,
                uses,
                run,
                env: vec![],
                with_inputs: None,
            }],
            env: vec![],
            container_image: container,
            service_images: vec![],
            timeout_minutes: None,
            rules: Vec::new(),
        }
    }

    #[test]
    fn test_set_output_detected() {
        let source = "jobs:\n  build:\n    runs-on: ubuntu\n    steps:\n      - run: echo \"::set-output name=result::done\"\n";
        let pipeline = make_pipeline(
            source,
            vec![make_job(
                "build",
                Some("echo \"::set-output name=result::done\"".into()),
                None,
                None,
            )],
        );
        let issues = audit(&pipeline).unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity, Severity::Warning);
        assert!(issues[0].message.contains("set-output"));
        assert_eq!(issues[0].rule_code, Some(rule_codes::DEPRECATED_ACTION));
    }

    #[test]
    fn test_save_state_detected() {
        let source = "jobs:\n  build:\n    runs-on: ubuntu\n    steps:\n      - run: echo \"::save-state name=result::done\"\n";
        let pipeline = make_pipeline(
            source,
            vec![make_job(
                "build",
                Some("echo \"::save-state name=result::done\"".into()),
                None,
                None,
            )],
        );
        let issues = audit(&pipeline).unwrap();
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("save-state"));
    }

    #[test]
    fn test_eol_node_image_detected() {
        let source = "jobs:\n  build:\n    runs-on: ubuntu\n    container: node:12\n    steps:\n      - run: echo hi\n";
        let pipeline = make_pipeline(
            source,
            vec![make_job(
                "build",
                Some("echo hi".into()),
                None,
                Some("node:12".into()),
            )],
        );
        let issues = audit(&pipeline).unwrap();
        assert!(issues.iter().any(|i| i.message.contains("node:12")));
    }

    #[test]
    fn test_old_action_version_detected() {
        let source =
            "jobs:\n  build:\n    runs-on: ubuntu\n    steps:\n      - uses: actions/checkout@v1\n";
        let pipeline = make_pipeline(
            source,
            vec![make_job(
                "build",
                None,
                Some("actions/checkout@v1".into()),
                None,
            )],
        );
        let issues = audit(&pipeline).unwrap();
        assert!(issues
            .iter()
            .any(|i| i.message.contains("actions/checkout@v1")));
    }

    #[test]
    fn test_no_issues_for_modern_workflow() {
        let source = "jobs:\n  build:\n    runs-on: ubuntu\n    container: node:20\n    steps:\n      - uses: actions/checkout@v4\n      - run: echo \"result=$value\" >> $GITHUB_OUTPUT\n";
        let pipeline = make_pipeline(
            source,
            vec![make_job(
                "build",
                Some("echo \"result=$value\" >> $GITHUB_OUTPUT".into()),
                Some("actions/checkout@v4".into()),
                Some("node:20".into()),
            )],
        );
        let issues = audit(&pipeline).unwrap();
        assert!(issues.is_empty());
    }

    #[test]
    fn test_gitlab_provider_skipped() {
        let source = "stages:\n  - build\nbuild:\n  script: echo hi\n";
        let pipeline = Pipeline {
            provider: Provider::GitLabCI,
            jobs: vec![],
            env: vec![],
            source: source.to_string(),
            is_reusable: false,
            workflow_call_inputs: Vec::new(),
            workflow_call_secrets: Vec::new(),
            workflow_rules: Vec::new(),
        };
        let issues = audit(&pipeline).unwrap();
        assert!(issues.is_empty());
    }
}
