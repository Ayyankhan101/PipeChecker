//! Schema auditor - validates YAML structure against JSON schemas
//!
//! This module performs structural validation of pipeline YAML
//! against expected schemas for each CI/CD provider.
//! Unlike full JSON Schema validation, this checks for:
//! - Required top-level keys
//! - Valid job structures
//! - Known keyword validation
//! - Type checking for common fields

use crate::error::Result;
use crate::models::{Issue, Pipeline, Severity};
use serde_yaml::Value;
use std::collections::HashSet;

/// Audit a pipeline for schema/structural issues
pub fn audit(pipeline: &Pipeline) -> Result<Vec<Issue>> {
    let mut issues = Vec::new();

    // Parse the source YAML
    let yaml: Value = match serde_yaml::from_str(&pipeline.source) {
        Ok(v) => v,
        Err(e) => {
            issues.push(Issue::new(
                Severity::Error,
                &format!("Invalid YAML syntax: {}", e),
                Some("Fix YAML syntax errors first".to_string()),
            ));
            return Ok(issues);
        }
    };

    let mapping = match yaml.as_mapping() {
        Some(m) => m,
        None => {
            issues.push(Issue::new(
                Severity::Error,
                "Pipeline is not a YAML mapping (object)",
                Some("Pipeline must be a YAML object with key-value pairs".to_string()),
            ));
            return Ok(issues);
        }
    };

    // Provider-specific validation
    match pipeline.provider {
        crate::models::Provider::GitHubActions => {
            issues.extend(validate_github_actions(mapping, pipeline)?);
        }
        crate::models::Provider::GitLabCI => {
            issues.extend(validate_gitlab_ci(mapping, pipeline)?);
        }
        crate::models::Provider::CircleCI => {
            issues.extend(validate_circleci(mapping, pipeline)?);
        }
    }

    Ok(issues)
}

/// Validate GitHub Actions structure
fn validate_github_actions(
    mapping: &serde_yaml::Mapping,
    _pipeline: &Pipeline,
) -> Result<Vec<Issue>> {
    let mut issues = Vec::new();

    // Check for required 'on' key
    if !mapping.contains_key("on") && !mapping.contains_key("workflow_on") {
        issues.push(Issue::new(
            Severity::Warning,
            "GitHub Actions workflow missing 'on' trigger",
            Some("Add workflow triggers (e.g., on: push)".to_string()),
        ));
    }

    // Check for required 'jobs' key
    if !mapping.contains_key("jobs") {
        issues.push(Issue::new(
            Severity::Error,
            "GitHub Actions workflow missing 'jobs' key",
            Some("Define at least one job in 'jobs: section".to_string()),
        ));
    }

    // Validate job structure
    if let Some(jobs_val) = mapping.get("jobs") {
        if let Some(jobs_map) = jobs_val.as_mapping() {
            for (job_id, job_val) in jobs_map {
                let job_name = job_id.as_str().unwrap_or("unknown");
                if let Some(job_map) = job_val.as_mapping() {
                    // Check required job fields
                    let has_runs_on = job_map.contains_key("runs-on");
                    let has_container = job_map.contains_key("container");
                    let has_steps = job_map.contains_key("steps");

                    if !has_runs_on && !has_container {
                        issues.push(Issue::for_job(
                            Severity::Error,
                            &format!("Job '{}' missing 'runs-on' or 'container'", job_name),
                            job_name,
                            0,
                            0,
                            Some("Specify where to run the job".to_string()),
                        ));
                    }

                    if !has_steps {
                        issues.push(Issue::for_job(
                            Severity::Warning,
                            &format!("Job '{}' missing 'steps'", job_name),
                            job_name,
                            0,
                            0,
                            Some("Add steps to perform work".to_string()),
                        ));
                    }

                    // Validate 'needs' is array or single job
                    if let Some(needs_val) = job_map.get("needs") {
                        if !needs_val.is_sequence() && !needs_val.is_string() {
                            issues.push(Issue::for_job(
                                Severity::Warning,
                                &format!("Job '{}' 'needs' should be string or array", job_name),
                                job_name,
                                0,
                                0,
                                Some("Use 'job-name' or ['job1', 'job2']".to_string()),
                            ));
                        }
                    }
                } else {
                    issues.push(Issue::for_job(
                        Severity::Error,
                        &format!("Job '{}' is not a mapping", job_name),
                        job_name,
                        0,
                        0,
                        Some("Job must be a YAML object".to_string()),
                    ));
                }
            }
        }
    }

    // Check for unknown top-level keys
    let known_keys: HashSet<&str> = [
        "name",
        "on",
        "workflow_on",
        "jobs",
        "env",
        "defaults",
        "run-name",
        "concurrency",
    ]
    .iter()
    .copied()
    .collect();

    for (key, _) in mapping {
        if let Some(k) = key.as_str() {
            if !known_keys.contains(k) && !k.starts_with('.') {
                issues.push(Issue::new(
                    Severity::Info,
                    &format!("Unknown top-level key in GitHub Actions: '{}'", k),
                    None,
                ));
            }
        }
    }

    Ok(issues)
}

/// Validate GitLab CI structure
fn validate_gitlab_ci(mapping: &serde_yaml::Mapping, _pipeline: &Pipeline) -> Result<Vec<Issue>> {
    let mut issues = Vec::new();

    // Check for stages (recommended but not required)
    if !mapping.contains_key("stages") {
        issues.push(Issue::new(
            Severity::Info,
            "GitLab CI missing 'stages' definition",
            Some("Define stages for better pipeline control".to_string()),
        ));
    }

    // Check that we have jobs (top-level keys that aren't reserved)
    let reserved = [
        "stages",
        "variables",
        "image",
        "before_script",
        "after_script",
        "cache",
        "services",
        "include",
        "default",
    ];
    let job_count = mapping
        .keys()
        .filter(|k| {
            if let Some(s) = k.as_str() {
                !reserved.contains(&s) && !s.starts_with('.')
            } else {
                false
            }
        })
        .count();

    if job_count == 0 {
        issues.push(Issue::new(
            Severity::Error,
            "GitLab CI has no jobs defined",
            Some("Define at least one job".to_string()),
        ));
    }

    // Check for unknown top-level keys
    let known_keys: HashSet<&str> = [
        "stages",
        "variables",
        "image",
        "before_script",
        "after_script",
        "cache",
        "services",
        "include",
        "default",
        "workflow",
    ]
    .iter()
    .copied()
    .collect();

    for (key, _) in mapping {
        if let Some(k) = key.as_str() {
            if !known_keys.contains(k) && !k.starts_with('.') && !k.ends_with(':') {
                issues.push(Issue::new(
                    Severity::Info,
                    &format!("Unknown top-level key in GitLab CI: '{}'", k),
                    None,
                ));
            }
        }
    }

    Ok(issues)
}

/// Validate CircleCI structure
fn validate_circleci(mapping: &serde_yaml::Mapping, _pipeline: &Pipeline) -> Result<Vec<Issue>> {
    let mut issues = Vec::new();

    // Check for required 'version'
    if !mapping.contains_key("version") {
        issues.push(Issue::new(
            Severity::Error,
            "CircleCI missing required 'version' key",
            Some("Add 'version: 2.1' or higher".to_string()),
        ));
    }

    // Check for jobs or workflows
    let has_jobs = mapping.contains_key("jobs");
    let has_workflows = mapping.contains_key("workflows");

    if !has_jobs && !has_workflows {
        issues.push(Issue::new(
            Severity::Warning,
            "CircleCI missing 'jobs' or 'workflows'",
            Some("Define jobs or workflows".to_string()),
        ));
    }

    // Validate job structure if present
    if let Some(jobs_val) = mapping.get("jobs") {
        if let Some(jobs_map) = jobs_val.as_mapping() {
            for (job_id, job_val) in jobs_map {
                let job_name = job_id.as_str().unwrap_or("unknown");
                if let Some(job_map) = job_val.as_mapping() {
                    // CircleCI requires docker or machine executor
                    let has_docker = job_map.contains_key("docker");
                    let has_machine = job_map.contains_key("machine");
                    let has_resource = job_map.contains_key("resource_class");

                    if !has_docker && !has_machine && !has_resource {
                        issues.push(Issue::for_job(
                            Severity::Info,
                            &format!("Job '{}' has no executor specified", job_name),
                            job_name,
                            0,
                            0,
                            Some("Consider adding docker, machine, or resource_class".to_string()),
                        ));
                    }
                }
            }
        }
    }

    // Check for unknown top-level keys
    let known_keys: HashSet<&str> = [
        "version",
        "jobs",
        "workflows",
        "commands",
        "executors",
        "orbs",
    ]
    .iter()
    .copied()
    .collect();

    for (key, _) in mapping {
        if let Some(k) = key.as_str() {
            if !known_keys.contains(k) && !k.starts_with('.') {
                issues.push(Issue::new(
                    Severity::Info,
                    &format!("Unknown top-level key in CircleCI: '{}'", k),
                    None,
                ));
            }
        }
    }

    Ok(issues)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_yaml_rejected() {
        let pipeline = Pipeline {
            provider: crate::models::Provider::GitHubActions,
            jobs: vec![],
            env: vec![],
            source: "invalid: [}\n".to_string(),
        };

        let issues = audit(&pipeline).unwrap();
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.message.contains("Invalid YAML")));
    }

    #[test]
    fn test_github_missing_jobs() {
        let pipeline = Pipeline {
            provider: crate::models::Provider::GitHubActions,
            jobs: vec![],
            env: vec![],
            source: "on: push\n".to_string(),
        };

        let issues = audit(&pipeline).unwrap();
        assert!(issues.iter().any(|i| i.message.contains("missing 'jobs'")));
    }

    #[test]
    fn test_circleci_missing_version() {
        let pipeline = Pipeline {
            provider: crate::models::Provider::CircleCI,
            jobs: vec![],
            env: vec![],
            source: "jobs:\n  build:\n    docker:\n      - image: node:18\n".to_string(),
        };

        let issues = audit(&pipeline).unwrap();
        assert!(issues.iter().any(|i| i.message.contains("version")));
    }
}
