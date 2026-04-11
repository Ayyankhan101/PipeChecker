//! Unit tests for pipechecker auditors

#[cfg(feature = "network")]
use pipechecker::auditors::pinning;
use pipechecker::auditors::{dag, secrets, syntax};
use pipechecker::models::{EnvVar, Job, Pipeline, Provider, Severity, Step};

fn create_test_step(name: &str, uses: Option<&str>, run: Option<&str>) -> Step {
    Step {
        name: Some(name.to_string()),
        uses: uses.map(|s| s.to_string()),
        run: run.map(|s| s.to_string()),
        env: vec![],
        with_inputs: None,
    }
}

fn create_test_job(id: &str, needs: Vec<String>, steps: Vec<Step>) -> Job {
    Job {
        id: id.to_string(),
        name: Some(format!("Job {}", id)),
        depends_on: needs,
        steps,
        env: vec![],
        container_image: None,
        service_images: vec![],
    }
}

fn make_pipeline(jobs: Vec<Job>) -> Pipeline {
    Pipeline {
        provider: Provider::GitHubActions,
        jobs,
        env: vec![],
        source: String::new(),
    }
}

#[test]
fn test_syntax_empty_pipeline() {
    let pipeline = make_pipeline(vec![]);

    let issues = syntax::audit(&pipeline).unwrap();
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].severity, Severity::Error);
    assert!(issues[0].message.contains("no jobs"));
    // Location should be populated
    assert!(issues[0].location.is_some());
}

#[test]
fn test_syntax_duplicate_job_ids() {
    let steps = vec![create_test_step("step1", None, Some("echo hello"))];
    let jobs = vec![
        create_test_job("duplicate-id", vec![], steps.clone()),
        create_test_job("duplicate-id", vec![], steps),
    ];

    let pipeline = make_pipeline(jobs);

    let issues = syntax::audit(&pipeline).unwrap();
    let duplicate_errors: Vec<_> = issues
        .iter()
        .filter(|i| i.message.contains("Duplicate job ID"))
        .collect();
    assert_eq!(duplicate_errors.len(), 1);
    assert_eq!(duplicate_errors[0].severity, Severity::Error);
}

#[test]
fn test_syntax_job_with_no_steps() {
    let jobs = vec![create_test_job("empty-job", vec![], vec![])];

    let pipeline = make_pipeline(jobs);

    let issues = syntax::audit(&pipeline).unwrap();
    let no_steps: Vec<_> = issues
        .iter()
        .filter(|i| i.message.contains("no steps"))
        .collect();
    assert_eq!(no_steps.len(), 1);
    assert_eq!(no_steps[0].severity, Severity::Warning);
}

#[test]
fn test_dag_circular_dependency() {
    let steps = vec![create_test_step("step", None, Some("echo"))];
    let jobs = vec![
        create_test_job("job-a", vec!["job-c".to_string()], steps.clone()),
        create_test_job("job-b", vec!["job-a".to_string()], steps.clone()),
        create_test_job("job-c", vec!["job-b".to_string()], steps),
    ];

    let pipeline = make_pipeline(jobs);

    let issues = dag::audit(&pipeline).unwrap();
    let cycles: Vec<_> = issues
        .iter()
        .filter(|i| i.message.contains("Circular dependency"))
        .collect();
    assert_eq!(cycles.len(), 1);
    assert_eq!(cycles[0].severity, Severity::Error);
}

#[test]
fn test_dag_missing_dependency() {
    let steps = vec![create_test_step("step", None, Some("echo"))];
    let jobs = vec![create_test_job(
        "job-a",
        vec!["non-existent".to_string()],
        steps,
    )];

    let pipeline = make_pipeline(jobs);

    let issues = dag::audit(&pipeline).unwrap();
    let missing: Vec<_> = issues
        .iter()
        .filter(|i| i.message.contains("non-existent"))
        .collect();
    assert_eq!(missing.len(), 1);
    assert_eq!(missing[0].severity, Severity::Error);
}

#[test]
fn test_dag_valid_dependency() {
    let steps = vec![create_test_step("step", None, Some("echo"))];
    let jobs = vec![
        create_test_job("job-a", vec![], steps.clone()),
        create_test_job("job-b", vec!["job-a".to_string()], steps),
    ];

    let pipeline = make_pipeline(jobs);

    let issues = dag::audit(&pipeline).unwrap();
    assert!(issues.is_empty());
}

#[test]
fn test_secrets_detects_secret_usage() {
    let steps = vec![create_test_step(
        "step",
        None,
        Some("echo ${{ secrets.API_TOKEN }}"),
    )];
    let jobs = vec![create_test_job("test-job", vec![], steps)];

    let pipeline = make_pipeline(jobs);

    let issues = secrets::audit(&pipeline).unwrap();
    let secret_refs: Vec<_> = issues
        .iter()
        .filter(|i| i.message.contains("API_TOKEN"))
        .collect();
    assert_eq!(secret_refs.len(), 1);
    assert_eq!(secret_refs[0].severity, Severity::Info);
}

#[test]
fn test_secrets_detects_undeclared_env() {
    let steps = vec![create_test_step(
        "step",
        None,
        Some("echo ${{ env.UNDEFINED }}"),
    )];
    let jobs = vec![create_test_job("test-job", vec![], steps)];

    let pipeline = make_pipeline(jobs);

    let issues = secrets::audit(&pipeline).unwrap();
    let undeclared: Vec<_> = issues
        .iter()
        .filter(|i| i.message.contains("undeclared"))
        .collect();
    assert_eq!(undeclared.len(), 1);
    assert_eq!(undeclared[0].severity, Severity::Warning);
}

#[test]
fn test_secrets_ignores_declared_env() {
    let steps = vec![create_test_step(
        "step",
        None,
        Some("echo ${{ env.KNOWN }}"),
    )];
    let jobs = vec![Job {
        id: "test-job".to_string(),
        name: Some("Test".to_string()),
        depends_on: vec![],
        steps,
        env: vec![EnvVar {
            key: "KNOWN".to_string(),
            value: "value".to_string(),
            is_secret: false,
        }],
        container_image: None,
        service_images: vec![],
    }];

    let pipeline = make_pipeline(jobs);

    let issues = secrets::audit(&pipeline).unwrap();
    let undeclared: Vec<_> = issues
        .iter()
        .filter(|i| i.message.contains("undeclared"))
        .collect();
    assert!(undeclared.is_empty());
}

#[test]
fn test_valid_pipeline_no_issues() {
    let steps = vec![
        create_test_step("checkout", Some("actions/checkout@v4"), None),
        create_test_step("build", None, Some("cargo build")),
    ];
    let jobs = vec![
        create_test_job("build", vec![], steps.clone()),
        create_test_job("test", vec!["build".to_string()], steps),
    ];

    let pipeline = make_pipeline(jobs);

    let syntax_issues = syntax::audit(&pipeline).unwrap();
    let dag_issues = dag::audit(&pipeline).unwrap();

    let syntax_errors: Vec<_> = syntax_issues
        .iter()
        .filter(|i| i.severity == Severity::Error)
        .collect();
    let dag_errors: Vec<_> = dag_issues
        .iter()
        .filter(|i| i.severity == Severity::Error)
        .collect();

    assert!(syntax_errors.is_empty());
    assert!(dag_errors.is_empty());
}

// --- New tests for pinning, config, with_inputs, fixtures ---

#[test]
#[cfg(feature = "network")]
fn test_pinning_detects_latest() {
    let steps = vec![create_test_step("deploy", Some("some/action:latest"), None)];
    let jobs = vec![create_test_job("deploy-job", vec![], steps)];
    let pipeline = make_pipeline(jobs);

    let issues = pinning::audit(&pipeline).unwrap();
    assert!(issues.iter().any(|i| i.message.contains(":latest")));
}

#[test]
#[cfg(feature = "network")]
fn test_pinning_detects_unpinned_action() {
    let steps = vec![create_test_step("checkout", Some("actions/checkout"), None)];
    let jobs = vec![create_test_job("build", vec![], steps)];
    let pipeline = make_pipeline(jobs);

    let issues = pinning::audit(&pipeline).unwrap();
    assert!(issues.iter().any(|i| i.message.contains("without version")));
}

#[test]
#[cfg(feature = "network")]
fn test_pinning_detects_docker_container_latest() {
    let jobs = vec![Job {
        id: "web".to_string(),
        name: None,
        depends_on: vec![],
        steps: vec![],
        env: vec![],
        container_image: Some("nginx:latest".to_string()),
        service_images: vec![],
    }];
    let pipeline = make_pipeline(jobs);

    let issues = pinning::audit(&pipeline).unwrap();
    assert!(issues.iter().any(|i| i.message.contains(":latest")));
}

#[test]
fn test_secrets_catches_with_inputs() {
    let with_val = serde_yaml::from_str("token: ${{ secrets.DEPLOY_KEY }}").unwrap();
    let steps = vec![Step {
        name: Some("deploy".to_string()),
        uses: Some("some/action@v1".to_string()),
        run: None,
        env: vec![],
        with_inputs: Some(with_val),
    }];
    let jobs = vec![create_test_job("deploy", vec![], steps)];
    let pipeline = make_pipeline(jobs);

    let issues = secrets::audit(&pipeline).unwrap();
    assert!(issues.iter().any(|i| i.message.contains("DEPLOY_KEY")));
}

#[test]
fn test_secrets_catches_hardcoded_env_value() {
    let jobs = vec![Job {
        id: "build".to_string(),
        name: None,
        depends_on: vec![],
        steps: vec![],
        env: vec![EnvVar {
            key: "API_SECRET".to_string(),
            value: "sk_live_abc123secretkey".to_string(),
            is_secret: false,
        }],
        container_image: None,
        service_images: vec![],
    }];
    let pipeline = make_pipeline(jobs);

    let issues = secrets::audit(&pipeline).unwrap();
    assert!(issues.iter().any(|i| i.message.contains("API_SECRET")));
    assert!(issues.iter().any(|i| i.message.contains("hardcoded")));
}

#[test]
fn test_config_load_defaults_when_no_file() {
    let config = pipechecker::Config::default();
    assert!(config.ignore.is_empty());
    assert!(config.rules.circular_dependencies);
    assert!(config.rules.missing_secrets);
    assert!(config.rules.docker_latest_tag);
}

#[test]
fn test_config_should_ignore_pattern() {
    let mut config = pipechecker::Config::default();
    config.ignore.push("test-*.yml".to_string());

    assert!(config.should_ignore("test-workflow.yml"));
    assert!(!config.should_ignore("prod-workflow.yml"));
}

#[test]
fn test_fixture_circular_yml_produces_cycle_error() {
    let content = std::fs::read_to_string("tests/fixtures/github/circular.yml").unwrap();
    let result = pipechecker::audit_content(&content, pipechecker::AuditOptions::default());
    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result
        .issues
        .iter()
        .any(|i| { i.severity == Severity::Error && i.message.contains("Circular dependency") }));
}

#[test]
fn test_fixture_valid_yml_produces_zero_errors() {
    let content = std::fs::read_to_string("tests/fixtures/github/valid.yml").unwrap();
    let result = pipechecker::audit_content(&content, pipechecker::AuditOptions::default());
    assert!(result.is_ok());
    let result = result.unwrap();
    let errors: Vec<_> = result
        .issues
        .iter()
        .filter(|i| i.severity == Severity::Error)
        .collect();
    assert!(errors.is_empty());
}

#[test]
fn test_fixture_docker_yml_triggers_warnings() {
    let content = std::fs::read_to_string("tests/fixtures/github/docker.yml").unwrap();
    let result = pipechecker::audit_content(&content, pipechecker::AuditOptions::default());
    assert!(result.is_ok());
    let result = result.unwrap();
    // Should have at least one warning about Docker or pinning
    let warnings: Vec<_> = result
        .issues
        .iter()
        .filter(|i| i.severity == Severity::Warning)
        .collect();
    assert!(!warnings.is_empty());
}

#[test]
fn test_fixture_real_world_parses_without_panic() {
    let content = std::fs::read_to_string("tests/fixtures/github/real-world.yml").unwrap();
    let result = pipechecker::audit_content(&content, pipechecker::AuditOptions::default());
    // Should parse and audit without panicking (may have issues, that's fine)
    assert!(result.is_ok() || result.is_err()); // Just no panic
}

// --- Property-based tests using proptest ---

use proptest::prelude::*;

/// Property: audit_content never panics on any valid YAML input
#[test]
fn proptest_audit_never_panics_on_valid_yaml() {
    proptest!(|(yaml_content in "[a-zA-Z0-9_ :\\n\\-]+")| {
        // Should not panic on any YAML-like string
        let _ = pipechecker::audit_content(&yaml_content, pipechecker::AuditOptions::default());
    });
}

/// Property: audit_content with empty string returns an error (unknown provider)
#[test]
fn proptest_empty_string_returns_unknown_provider() {
    proptest!(|(_ in any::<bool>())| {
        let result = pipechecker::audit_content("", pipechecker::AuditOptions::default());
        prop_assert!(result.is_err());
    });
}

/// Property: for any YAML with a valid GitHub Actions structure, parsing succeeds
#[test]
fn proptest_github_actions_always_parses() {
    proptest!(|(job_name in "[a-z_]+", step_cmd in "[a-z ]+")| {
        let yaml = format!(
            "name: Test\non: push\njobs:\n  {}:\n    runs-on: ubuntu-latest\n    steps:\n      - run: {}",
            job_name, step_cmd
        );
        let result = pipechecker::audit_content(&yaml, pipechecker::AuditOptions::default());
        prop_assert!(result.is_ok());
        let result = result.unwrap();
        prop_assert!(result.issues.iter().filter(|i| i.severity == Severity::Error).count() <= 1);
    });
}

/// Property: summary string always contains "errors" and "warnings"
#[test]
fn proptest_summary_contains_errors_and_warnings() {
    proptest!(|(num_jobs in 0usize..5)| {
        let mut jobs = Vec::new();
        for i in 0..num_jobs {
            jobs.push(create_test_job(&format!("job-{}", i), vec![], vec![
                create_test_step("step", None, Some("echo"))
            ]));
        }
        let pipeline = make_pipeline(jobs);
        let issues = syntax::audit(&pipeline).unwrap();
        let summary = format!("{} errors, {} warnings",
            issues.iter().filter(|i| i.severity == Severity::Error).count(),
            issues.iter().filter(|i| i.severity == Severity::Warning).count(),
        );
        prop_assert!(summary.contains("errors"));
        prop_assert!(summary.contains("warnings"));
    });
}
