//! Secrets auditor - detects hardcoded secrets and secret usage patterns
//!
//! This module scans pipeline configurations for:
//! - Hardcoded secrets in environment variables at all levels (pipeline, job, step)
//! - Secret references in `run:` blocks (`${{ secrets.X }}`)
//! - Secret references in `with:` input blocks
//! - Undeclared environment variable references (`${{ env.X }}`)
//! - Hardcoded values in `env:` fields that look like secrets

use crate::error::Result;
use crate::models::{Issue, Pipeline, Severity};
use regex::Regex;
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

    let secret_pattern = Regex::new(r"\$\{\{\s*secrets\.(\w+)\s*\}\}").unwrap();
    let env_pattern = Regex::new(r"\$\{\{\s*env\.(\w+)\s*\}\}").unwrap();

    let mut declared_env: HashSet<String> = pipeline.env.iter().map(|e| e.key.clone()).collect();

    // Check pipeline-level env vars for hardcoded secrets
    for env_var in &pipeline.env {
        if is_potential_secret_value(&env_var.value) {
            let (line, col) = pipeline.find_line(&format!("  {}", env_var.key));
            issues.push(Issue::for_job(
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
            if is_potential_secret_value(&env_var.value) {
                let (line, col) = pipeline.find_job_line(&job.id, "env");
                issues.push(Issue::for_job(
                    Severity::Warning,
                    &format!(
                        "Job '{}' env '{}' may contain a hardcoded secret",
                        job.id, env_var.key
                    ),
                    &job.id,
                    line,
                    col,
                    Some("Use repository secrets or workflow-level env vars instead".to_string()),
                ));
            }
        }

        for step in &job.steps {
            // Check step-level env vars for hardcoded secrets
            for env_var in &step.env {
                if is_potential_secret_value(&env_var.value) {
                    let (line, col) = pipeline.find_job_line(&job.id, "env");
                    issues.push(Issue::for_job(
                        Severity::Warning,
                        &format!(
                            "Job '{}' step env '{}' may contain a hardcoded secret",
                            job.id, env_var.key
                        ),
                        &job.id,
                        line,
                        col,
                        Some("Use repository secrets instead of hardcoded values".to_string()),
                    ));
                }
                declared_env.insert(env_var.key.clone());
            }

            // Check run commands for secret references
            if let Some(run) = &step.run {
                for cap in secret_pattern.captures_iter(run) {
                    let secret_name = &cap[1];
                    let (line, col) = pipeline.find_job_line(&job.id, "run");
                    issues.push(Issue::for_job(
                        Severity::Info,
                        &format!("Job '{}' uses secret: {}", job.id, secret_name),
                        &job.id,
                        line,
                        col,
                        Some("Ensure this secret is configured in repository settings".to_string()),
                    ));
                }

                for cap in env_pattern.captures_iter(run) {
                    let env_name = &cap[1];
                    if !declared_env.contains(env_name) {
                        let (line, col) = pipeline.find_job_line(&job.id, "run");
                        issues.push(Issue::for_job(
                            Severity::Warning,
                            &format!(
                                "Job '{}' references undeclared env var: {}",
                                job.id, env_name
                            ),
                            &job.id,
                            line,
                            col,
                            Some(format!("Declare '{}' in env section", env_name)),
                        ));
                    }
                }
            }

            // Check 'with:' input blocks for secret references
            if let Some(with_val) = &step.with_inputs {
                scan_yaml_for_secrets(
                    with_val,
                    &job.id,
                    &secret_pattern,
                    &env_pattern,
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
                issues.push(Issue::for_job(
                    Severity::Info,
                    &format!("Job '{}' uses secret: {}", job_id, secret_name),
                    job_id,
                    line,
                    col,
                    Some("Ensure this secret is configured in repository settings".to_string()),
                ));
            }
            for cap in env_pattern.captures_iter(s) {
                let env_name = &cap[1];
                if !declared_env.contains(env_name) {
                    let (line, col) = pipeline.find_job_line(job_id, "with");
                    issues.push(Issue::for_job(
                        Severity::Warning,
                        &format!(
                            "Job '{}' references undeclared env var: {}",
                            job_id, env_name
                        ),
                        job_id,
                        line,
                        col,
                        Some(format!("Declare '{}' in env section", env_name)),
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

/// Check if a value looks like it might be a secret
fn is_potential_secret_value(value: &str) -> bool {
    let lower = value.to_lowercase();

    // Check if the value contains suspicious patterns
    for pattern in SECRET_VALUE_PATTERNS {
        if lower.contains(pattern) {
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
