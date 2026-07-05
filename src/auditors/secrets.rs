//! Secrets auditor - detects hardcoded secrets and secret usage patterns
//!
//! This module scans pipeline configurations for:
//! - Hardcoded secrets in environment variables at all levels (pipeline, job, step)
//! - Secret references in `run:` blocks (`${{ secrets.X }}`)
//! - Secret references in `with:` input blocks
//! - Undeclared environment variable references (`${{ env.X }}`)
//! - Hardcoded values in `env:` fields that look like secrets

use crate::error::Result;
use crate::models::{rule_codes, Issue, Pipeline, Severity};
use regex::Regex;
use std::sync::OnceLock;
static SECRET_REGEX: OnceLock<Regex> = OnceLock::new();
static ENV_REGEX: OnceLock<Regex> = OnceLock::new();
use std::collections::HashSet;

/// Patterns that indicate hardcoded secrets
const SECRET_VALUE_PATTERNS: &[&str] = &[
    "api_key",
    "apikey",
    "api-key",
    "api_secret",
    "apisecret",
    "api-secret",
    "secret_key",
    "secretkey",
    "secret-key",
    "access_key",
    "accesskey",
    "access-key",
    "auth_token",
    "authtoken",
    "auth-token",
    "private_key",
    "privatekey",
    "private-key",
    "password",
    "passwd",
    "token",
];

/// Audit a pipeline for secret-related issues
pub fn audit(pipeline: &Pipeline) -> Result<Vec<Issue>> {
    let mut issues = Vec::new();

    let secret_pattern =
        SECRET_REGEX.get_or_init(|| Regex::new(r"\$\{\{\s*secrets\.(\w+)\s*\}\}").unwrap());
    let env_pattern = ENV_REGEX.get_or_init(|| Regex::new(r"\$\{\{\s*env\.(\w+)\s*\}\}").unwrap());

    // Check for pull_request_target security risk (GitHub Actions only)
    if pipeline.provider == crate::models::Provider::GitHubActions {
        check_pull_request_target(pipeline, &mut issues);
    }

    let mut declared_env: HashSet<String> = pipeline.env.iter().map(|e| e.key.clone()).collect();

    // Check pipeline-level env vars for hardcoded secrets
    for env_var in &pipeline.env {
        if is_potential_secret(&env_var.key, &env_var.value) {
            let (line, col) = pipeline.find_line(&format!("  {}", env_var.key));
            issues.push(Issue::for_job_with_code(
                Severity::Warning,
                &format!(
                    "Pipeline env '{}' may contain a hardcoded secret",
                    env_var.key
                ),
                "",
                line,
                col,
                Some(format!(
                    "Use secrets.{} instead of hardcoding",
                    env_var.key.to_lowercase().replace("_", "-")
                )),
                rule_codes::HARDCODED_SECRET,
            ));
        }
    }

    for job in &pipeline.jobs {
        // Add job-level env vars to declared set
        for env_var in &job.env {
            declared_env.insert(env_var.key.clone());
        }

        // Check job-level env vars for hardcoded secrets
        for env_var in &job.env {
            if is_potential_secret(&env_var.key, &env_var.value) {
                let (line, col) = pipeline.find_job_line(&job.id, "env");
                issues.push(Issue::for_job_with_code(
                    Severity::Warning,
                    &format!(
                        "Job '{}' env '{}' may contain a hardcoded secret",
                        job.id, env_var.key
                    ),
                    &job.id,
                    line,
                    col,
                    Some("Use repository secrets or workflow-level env vars instead".to_string()),
                    rule_codes::HARDCODED_SECRET,
                ));
            }
        }

        for step in &job.steps {
            // Check step-level env vars for hardcoded secrets
            for env_var in &step.env {
                if is_potential_secret(&env_var.key, &env_var.value) {
                    let (line, col) = pipeline.find_job_line(&job.id, "env");
                    issues.push(Issue::for_job_with_code(
                        Severity::Warning,
                        &format!(
                            "Job '{}' step env '{}' may contain a hardcoded secret",
                            job.id, env_var.key
                        ),
                        &job.id,
                        line,
                        col,
                        Some("Use repository secrets instead of hardcoded values".to_string()),
                        rule_codes::HARDCODED_SECRET,
                    ));
                }
                declared_env.insert(env_var.key.clone());
            }

            // Check run commands for secret references
            if let Some(run) = &step.run {
                for cap in secret_pattern.captures_iter(run) {
                    let secret_name = &cap[1];
                    let (line, col) = pipeline.find_job_line(&job.id, "run");
                    issues.push(Issue::for_job_with_code(
                        Severity::Info,
                        &format!("Job '{}' uses secret: {}", job.id, secret_name),
                        &job.id,
                        line,
                        col,
                        Some("Ensure this secret is configured in repository settings".to_string()),
                        rule_codes::SECRET_REFERENCE,
                    ));
                }

                for cap in env_pattern.captures_iter(run) {
                    let env_name = &cap[1];
                    if !declared_env.contains(env_name) {
                        let (line, col) = pipeline.find_job_line(&job.id, "run");
                        issues.push(Issue::for_job_with_code(
                            Severity::Warning,
                            &format!(
                                "Job '{}' references undeclared env var: {}",
                                job.id, env_name
                            ),
                            &job.id,
                            line,
                            col,
                            Some(format!("Declare '{}' in env section", env_name)),
                            rule_codes::UNDECLARED_ENV_VAR,
                        ));
                    }
                }
            }

            // Check 'with:' input blocks for secret references
            if let Some(with_val) = &step.with_inputs {
                scan_yaml_for_secrets(
                    with_val,
                    &job.id,
                    secret_pattern,
                    env_pattern,
                    &declared_env,
                    pipeline,
                    &mut issues,
                );
            }
        }
    }

    Ok(issues)
}

/// Recursively scan a YAML value for secret patterns
fn scan_yaml_for_secrets(
    val: &serde_yaml::Value,
    job_id: &str,
    secret_pattern: &Regex,
    env_pattern: &Regex,
    declared_env: &HashSet<String>,
    pipeline: &Pipeline,
    issues: &mut Vec<Issue>,
) {
    match val {
        serde_yaml::Value::String(s) => {
            for cap in secret_pattern.captures_iter(s) {
                let secret_name = &cap[1];
                let (line, col) = pipeline.find_job_line(job_id, "with");
                issues.push(Issue::for_job_with_code(
                    Severity::Info,
                    &format!("Job '{}' uses secret: {}", job_id, secret_name),
                    job_id,
                    line,
                    col,
                    Some("Ensure this secret is configured in repository settings".to_string()),
                    rule_codes::SECRET_REFERENCE,
                ));
            }
            for cap in env_pattern.captures_iter(s) {
                let env_name = &cap[1];
                if !declared_env.contains(env_name) {
                    let (line, col) = pipeline.find_job_line(job_id, "with");
                    issues.push(Issue::for_job_with_code(
                        Severity::Warning,
                        &format!(
                            "Job '{}' references undeclared env var: {}",
                            job_id, env_name
                        ),
                        job_id,
                        line,
                        col,
                        Some(format!("Declare '{}' in env section", env_name)),
                        rule_codes::UNDECLARED_ENV_VAR,
                    ));
                }
            }
        }
        serde_yaml::Value::Mapping(m) => {
            for (_k, v) in m {
                scan_yaml_for_secrets(
                    v,
                    job_id,
                    secret_pattern,
                    env_pattern,
                    declared_env,
                    pipeline,
                    issues,
                );
            }
        }
        serde_yaml::Value::Sequence(seq) => {
            for item in seq {
                scan_yaml_for_secrets(
                    item,
                    job_id,
                    secret_pattern,
                    env_pattern,
                    declared_env,
                    pipeline,
                    issues,
                );
            }
        }
        _ => {}
    }
}

/// Check if a key-value pair looks like it might be a hardcoded secret
fn is_potential_secret(key: &str, value: &str) -> bool {
    // Skip GitHub Actions secret references — these are the correct way to use secrets
    if value.contains("${{ secrets.") || value.contains("${{secrets.") {
        return false;
    }

    let lower_key = key.to_lowercase();
    let lower_val = value.to_lowercase();

    // Check if the key name suggests it's a secret
    for pattern in SECRET_VALUE_PATTERNS {
        if lower_key.contains(pattern) {
            // If the key is suspicious, any non-empty value that isn't a reference is suspect
            return !value.trim().is_empty();
        }
    }

    // Check if the value contains suspicious patterns
    for pattern in SECRET_VALUE_PATTERNS {
        if lower_val.contains(pattern) {
            return true;
        }
    }

    // Check if value looks like a key/token (long alphanumeric string)
    if value.len() > 20
        && value
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return true;
    }

    // Check for base64-like strings that might be encoded secrets
    if value.len() > 40
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
    {
        return true;
    }

    false
}

/// Check for `on: pull_request_target` combined with `actions/checkout` (security risk)
fn check_pull_request_target(pipeline: &Pipeline, issues: &mut Vec<Issue>) {
    let source = &pipeline.source;

    // Detect pull_request_target trigger
    let has_pr_target = source.contains("pull_request_target");
    if !has_pr_target {
        return;
    }

    // Check if any job uses actions/checkout without ref: restriction
    let mut has_unrestricted_checkout = false;
    for job in &pipeline.jobs {
        for step in &job.steps {
            if let Some(uses) = &step.uses {
                if uses.starts_with("actions/checkout") {
                    // Check if there's a 'ref:' restriction in with_inputs
                    let has_ref = step
                        .with_inputs
                        .as_ref()
                        .and_then(|v| v.get("ref"))
                        .is_some();
                    if !has_ref {
                        has_unrestricted_checkout = true;
                        let (line, col) = pipeline.find_job_line(&job.id, "uses");
                        issues.push(Issue::for_job_with_code(
                            Severity::Warning,
                            &format!(
                                "Job '{}' uses actions/checkout in a pull_request_target workflow without ref: restriction",
                                job.id
                            ),
                            &job.id,
                            line,
                            col,
                            Some(
                                "pull_request_target runs in the context of the base branch. \
                                 Using actions/checkout without ref: may expose secrets to untrusted code. \
                                 Add 'ref: ${{ github.event.pull_request.head.sha }}' or restrict the checkout."
                                    .to_string(),
                            ),
                            rule_codes::PR_TARGET_INSECURE,
                        ));
                    }
                }
            }
        }
    }

    // If no checkout found at all but pull_request_target is used, still warn
    if !has_unrestricted_checkout && !has_pr_target {
        return;
    }

    // General warning about pull_request_target if no specific checkout issue was flagged
    if has_pr_target && !has_unrestricted_checkout {
        // Check if there's any checkout at all
        let has_any_checkout = pipeline.jobs.iter().any(|job| {
            job.steps.iter().any(|s| {
                s.uses
                    .as_deref()
                    .is_some_and(|u| u.starts_with("actions/checkout"))
            })
        });
        if !has_any_checkout {
            let (line, col) = pipeline.find_line("pull_request_target");
            issues.push(Issue::with_code(
                Severity::Warning,
                "Workflow uses 'pull_request_target' trigger — ensure secrets are not exposed to untrusted code",
                Some(
                    "pull_request_target runs with the base branch's secrets. \
                     Review all steps to ensure they cannot be manipulated by the PR author."
                        .to_string(),
                ),
                rule_codes::PR_TARGET_INSECURE,
            ));
            let _ = (line, col);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Provider;

    #[test]
    fn test_secret_reference_is_not_secret() {
        // Secret references are the correct way, not hardcoded
        assert!(!is_potential_secret("API_KEY", "${{ secrets.API_KEY }}"));
        assert!(!is_potential_secret("token", "${{secrets.TOKEN}}"));
    }

    #[test]
    fn test_keyword_patterns_detected() {
        assert!(is_potential_secret("API_KEY", "abc123"));
        assert!(is_potential_secret("some_var", "password=hunter2"));
    }

    #[test]
    fn test_keyword_case_insensitive() {
        assert!(is_potential_secret("api_key", "abc"));
        assert!(is_potential_secret("Password", "123"));
    }

    #[test]
    fn test_long_alphanumeric_detected_as_secret() {
        assert!(is_potential_secret("var", "aB3dEf6hIjKlMnOpQrStUvWx"));
    }

    #[test]
    fn test_short_alphanumeric_not_secret() {
        assert!(!is_potential_secret("var", "hello"));
    }

    #[test]
    fn test_base64_like_detected() {
        assert!(is_potential_secret(
            "var",
            "abcDEF123+/xyzABC456===GHJklmno789pqrSTUV"
        ));
    }

    #[test]
    fn test_normal_values_not_flagged() {
        assert!(!is_potential_secret("image", "node:18-alpine"));
        assert!(!is_potential_secret("run", "echo hello world"));
    }

    #[test]
    fn test_values_with_special_chars_not_long_enough() {
        // Contains spaces, so not all alphanumeric
        assert!(!is_potential_secret(
            "run",
            "hello world this is a test string"
        ));
    }

    #[test]
    fn test_empty_string_not_secret() {
        assert!(!is_potential_secret("var", ""));
    }

    // --- pull_request_target tests ---

    #[test]
    fn test_pr_target_with_unrestricted_checkout_warns() {
        let source = "name: CI\non:\n  pull_request_target:\njobs:\n  build:\n    runs-on: ubuntu\n    steps:\n      - uses: actions/checkout@v4\n";
        let pipeline = Pipeline {
            provider: Provider::GitHubActions,
            jobs: vec![crate::models::Job {
                id: "build".to_string(),
                name: None,
                depends_on: vec![],
                steps: vec![crate::models::Step {
                    name: None,
                    uses: Some("actions/checkout@v4".to_string()),
                    run: None,
                    env: vec![],
                    with_inputs: None,
                }],
                env: vec![],
                container_image: None,
                service_images: vec![],
                timeout_minutes: None,
                rules: vec![],
            }],
            env: vec![],
            source: source.to_string(),
            is_reusable: false,
            workflow_call_inputs: vec![],
            workflow_call_secrets: vec![],
            workflow_rules: vec![],
        };
        let issues = audit(&pipeline).unwrap();
        assert!(issues
            .iter()
            .any(|i| i.rule_code == Some(rule_codes::PR_TARGET_INSECURE)));
    }

    #[test]
    fn test_pr_target_with_ref_checkout_ok() {
        let source = "name: CI\non:\n  pull_request_target:\njobs:\n  build:\n    runs-on: ubuntu\n    steps:\n      - uses: actions/checkout@v4\n        with:\n          ref: ${{ github.event.pull_request.head.sha }}\n";
        let pipeline = Pipeline {
            provider: Provider::GitHubActions,
            jobs: vec![crate::models::Job {
                id: "build".to_string(),
                name: None,
                depends_on: vec![],
                steps: vec![crate::models::Step {
                    name: None,
                    uses: Some("actions/checkout@v4".to_string()),
                    run: None,
                    env: vec![],
                    with_inputs: Some(
                        serde_yaml::from_str("ref: ${{ github.event.pull_request.head.sha }}")
                            .unwrap(),
                    ),
                }],
                env: vec![],
                container_image: None,
                service_images: vec![],
                timeout_minutes: None,
                rules: vec![],
            }],
            env: vec![],
            source: source.to_string(),
            is_reusable: false,
            workflow_call_inputs: vec![],
            workflow_call_secrets: vec![],
            workflow_rules: vec![],
        };
        let issues = audit(&pipeline).unwrap();
        assert!(!issues
            .iter()
            .any(|i| i.rule_code == Some(rule_codes::PR_TARGET_INSECURE)));
    }

    #[test]
    fn test_pr_target_without_checkout_warns() {
        let source = "name: CI\non:\n  pull_request_target:\njobs:\n  build:\n    runs-on: ubuntu\n    steps:\n      - run: echo hi\n";
        let pipeline = Pipeline {
            provider: Provider::GitHubActions,
            jobs: vec![crate::models::Job {
                id: "build".to_string(),
                name: None,
                depends_on: vec![],
                steps: vec![crate::models::Step {
                    name: None,
                    uses: None,
                    run: Some("echo hi".to_string()),
                    env: vec![],
                    with_inputs: None,
                }],
                env: vec![],
                container_image: None,
                service_images: vec![],
                timeout_minutes: None,
                rules: vec![],
            }],
            env: vec![],
            source: source.to_string(),
            is_reusable: false,
            workflow_call_inputs: vec![],
            workflow_call_secrets: vec![],
            workflow_rules: vec![],
        };
        let issues = audit(&pipeline).unwrap();
        assert!(issues
            .iter()
            .any(|i| i.rule_code == Some(rule_codes::PR_TARGET_INSECURE)));
    }

    #[test]
    fn test_no_pr_target_no_issue() {
        let source = "name: CI\non: push:\njobs:\n  build:\n    runs-on: ubuntu\n    steps:\n      - uses: actions/checkout@v4\n";
        let pipeline = Pipeline {
            provider: Provider::GitHubActions,
            jobs: vec![crate::models::Job {
                id: "build".to_string(),
                name: None,
                depends_on: vec![],
                steps: vec![crate::models::Step {
                    name: None,
                    uses: Some("actions/checkout@v4".to_string()),
                    run: None,
                    env: vec![],
                    with_inputs: None,
                }],
                env: vec![],
                container_image: None,
                service_images: vec![],
                timeout_minutes: None,
                rules: vec![],
            }],
            env: vec![],
            source: source.to_string(),
            is_reusable: false,
            workflow_call_inputs: vec![],
            workflow_call_secrets: vec![],
            workflow_rules: vec![],
        };
        let issues = audit(&pipeline).unwrap();
        assert!(!issues
            .iter()
            .any(|i| i.rule_code == Some(rule_codes::PR_TARGET_INSECURE)));
    }
}
