use pipechecker::config::Rules;
use pipechecker::models::{EnvVar, Job, Pipeline, Provider, Step};
use pipechecker::{audit_content, AuditOptions};

// =============================================================================
// A1: Matrix auditor tests
// =============================================================================

#[test]
fn test_matrix_non_github_provider() {
    let pipeline = Pipeline {
        provider: Provider::GitLabCI,
        source: "stages: [build]\nbuild:\n  script: echo\n".to_string(),
        ..Default::default()
    };
    let issues = pipechecker::auditors::matrix::audit(&pipeline).unwrap();
    assert!(issues.is_empty());
}

#[test]
fn test_matrix_parse_error() {
    let pipeline = Pipeline {
        provider: Provider::GitHubActions,
        source: "not: [valid: yaml: {".to_string(),
        ..Default::default()
    };
    let issues = pipechecker::auditors::matrix::audit(&pipeline).unwrap();
    assert!(issues.is_empty());
}

#[test]
fn test_matrix_large_no_failfast() {
    let source = r#"name: CI
on: push
jobs:
  build:
    runs-on: ubuntu
    strategy:
      matrix:
        os: [a, b, c, d]
        node: [1, 2, 3, 4]
"#;
    let pipeline = Pipeline {
        provider: Provider::GitHubActions,
        source: source.to_string(),
        ..Default::default()
    };
    let issues = pipechecker::auditors::matrix::audit(&pipeline).unwrap();
    assert_eq!(issues.len(), 1);
    assert!(issues[0].message.contains("16 combinations"));
}

#[test]
fn test_matrix_large_with_failfast() {
    let source = r#"name: CI
on: push
jobs:
  build:
    runs-on: ubuntu
    strategy:
      fail-fast: false
      matrix:
        os: [a, b, c, d]
        node: [1, 2, 3, 4]
"#;
    let pipeline = Pipeline {
        provider: Provider::GitHubActions,
        source: source.to_string(),
        ..Default::default()
    };
    let issues = pipechecker::auditors::matrix::audit(&pipeline).unwrap();
    assert!(issues.is_empty());
}

#[test]
fn test_matrix_empty_include() {
    let source = r#"name: CI
on: push
jobs:
  build:
    runs-on: ubuntu
    strategy:
      matrix:
        include: []
"#;
    let pipeline = Pipeline {
        provider: Provider::GitHubActions,
        source: source.to_string(),
        ..Default::default()
    };
    let issues = pipechecker::auditors::matrix::audit(&pipeline).unwrap();
    assert_eq!(issues.len(), 1);
    assert!(issues[0].message.contains("empty"));
}

// =============================================================================
// A2: Concurrency auditor tests
// =============================================================================

#[test]
fn test_concurrency_non_github() {
    let pipeline = Pipeline {
        provider: Provider::GitLabCI,
        source: "stages: [build]\n".to_string(),
        ..Default::default()
    };
    let issues = pipechecker::auditors::concurrency::audit(&pipeline).unwrap();
    assert!(issues.is_empty());
}

#[test]
fn test_concurrency_parse_error() {
    let pipeline = Pipeline {
        provider: Provider::GitHubActions,
        source: "not: [valid: yaml: {".to_string(),
        ..Default::default()
    };
    let issues = pipechecker::auditors::concurrency::audit(&pipeline).unwrap();
    assert!(issues.is_empty());
}

#[test]
fn test_concurrency_workflow_level_no_cancel() {
    let source = r#"name: CI
on: push
concurrency:
  group: deploy
jobs:
  build:
    runs-on: ubuntu
"#;
    let pipeline = Pipeline {
        provider: Provider::GitHubActions,
        source: source.to_string(),
        ..Default::default()
    };
    let issues = pipechecker::auditors::concurrency::audit(&pipeline).unwrap();
    assert_eq!(issues.len(), 1);
    assert!(issues[0].message.contains("Workflow-level"));
}

#[test]
fn test_concurrency_job_level_no_cancel() {
    let source = r#"name: CI
on: push
jobs:
  build:
    runs-on: ubuntu
    concurrency:
      group: build-group
"#;
    let pipeline = Pipeline {
        provider: Provider::GitHubActions,
        source: source.to_string(),
        ..Default::default()
    };
    let issues = pipechecker::auditors::concurrency::audit(&pipeline).unwrap();
    assert_eq!(issues.len(), 1);
    assert!(issues[0].message.contains("Job 'build'"));
}

#[test]
fn test_concurrency_string_form() {
    let source = r#"name: CI
on: push
concurrency: my-group
jobs:
  build:
    runs-on: ubuntu
"#;
    let pipeline = Pipeline {
        provider: Provider::GitHubActions,
        source: source.to_string(),
        ..Default::default()
    };
    let issues = pipechecker::auditors::concurrency::audit(&pipeline).unwrap();
    assert_eq!(issues.len(), 1);
}

#[test]
fn test_concurrency_with_cancel_in_progress_true() {
    let source = r#"name: CI
on: push
concurrency:
  group: deploy
  cancel-in-progress: true
jobs:
  build:
    runs-on: ubuntu
"#;
    let pipeline = Pipeline {
        provider: Provider::GitHubActions,
        source: source.to_string(),
        ..Default::default()
    };
    let issues = pipechecker::auditors::concurrency::audit(&pipeline).unwrap();
    assert!(issues.is_empty());
}

#[test]
fn test_concurrency_cancel_in_progress_expression() {
    let source = r#"name: CI
on: push
concurrency:
  group: deploy
  cancel-in-progress: ${{ github.ref }}
jobs:
  build:
    runs-on: ubuntu
"#;
    let pipeline = Pipeline {
        provider: Provider::GitHubActions,
        source: source.to_string(),
        ..Default::default()
    };
    let issues = pipechecker::auditors::concurrency::audit(&pipeline).unwrap();
    assert!(issues.is_empty());
}

// =============================================================================
// A3: Include auditor tests
// =============================================================================

#[test]
fn test_include_local_not_found() {
    let source = r#"stages: [build]
include:
  - local: "./nonexistent-file.yml"
build:
  script: echo hi
"#;
    let pipeline = Pipeline {
        provider: Provider::GitLabCI,
        source: source.to_string(),
        ..Default::default()
    };
    let issues = pipechecker::auditors::include::audit(&pipeline).unwrap();
    assert!(issues.iter().any(|i| i.message.contains("not found")));
}

#[test]
fn test_include_remote_warning() {
    let source = r#"stages: [build]
include:
  - remote: "https://example.com/ci.yml"
build:
  script: echo hi
"#;
    let pipeline = Pipeline {
        provider: Provider::GitLabCI,
        source: source.to_string(),
        ..Default::default()
    };
    let issues = pipechecker::auditors::include::audit(&pipeline).unwrap();
    assert!(issues.iter().any(|i| i.message.contains("Remote include")));
}

#[test]
fn test_include_project_warning() {
    let source = r#"stages: [build]
include:
  - project: "group/repo"
    file: "ci.yml"
build:
  script: echo hi
"#;
    let pipeline = Pipeline {
        provider: Provider::GitLabCI,
        source: source.to_string(),
        ..Default::default()
    };
    let issues = pipechecker::auditors::include::audit(&pipeline).unwrap();
    assert!(issues.iter().any(|i| i.message.contains("Project include")));
}

// =============================================================================
// A4: Artifacts auditor tests
// =============================================================================

#[test]
fn test_artifacts_non_github() {
    let pipeline = Pipeline {
        provider: Provider::GitLabCI,
        source: "stages: [build]\n".to_string(),
        ..Default::default()
    };
    let issues = pipechecker::auditors::artifacts::audit(&pipeline).unwrap();
    assert!(issues.is_empty());
}

#[test]
fn test_artifacts_static_cache_key() {
    let source = r#"name: CI
on: push
jobs:
  build:
    runs-on: ubuntu
    steps:
      - uses: actions/cache@v4
        with:
          key: my-static-cache
"#;
    let pipeline = Pipeline {
        provider: Provider::GitHubActions,
        jobs: vec![Job {
            id: "build".to_string(),
            steps: vec![Step {
                name: None,
                uses: Some("actions/cache@v4".to_string()),
                run: None,
                env: vec![],
                with_inputs: Some(serde_yaml::from_str("key: my-static-cache").unwrap()),
            }],
            ..Default::default()
        }],
        source: source.to_string(),
        ..Default::default()
    };
    let issues = pipechecker::auditors::artifacts::audit(&pipeline).unwrap();
    assert!(issues.iter().any(|i| i.message.contains("static key")));
}

#[test]
fn test_artifacts_missing_retention() {
    let source = r#"name: CI
on: push
jobs:
  build:
    runs-on: ubuntu
    steps:
      - uses: actions/upload-artifact@v4
        with:
          name: my-artifact
"#;
    let pipeline = Pipeline {
        provider: Provider::GitHubActions,
        jobs: vec![Job {
            id: "build".to_string(),
            steps: vec![Step {
                name: None,
                uses: Some("actions/upload-artifact@v4".to_string()),
                run: None,
                env: vec![],
                with_inputs: Some(serde_yaml::from_str("name: my-artifact").unwrap()),
            }],
            ..Default::default()
        }],
        source: source.to_string(),
        ..Default::default()
    };
    let issues = pipechecker::auditors::artifacts::audit(&pipeline).unwrap();
    assert!(issues.iter().any(|i| i.message.contains("retention-days")));
}

#[test]
fn test_artifacts_with_retention() {
    let source = r#"name: CI
on: push
jobs:
  build:
    runs-on: ubuntu
    steps:
      - uses: actions/upload-artifact@v4
        with:
          name: my-artifact
          retention-days: 7
"#;
    let pipeline = Pipeline {
        provider: Provider::GitHubActions,
        jobs: vec![Job {
            id: "build".to_string(),
            steps: vec![Step {
                name: None,
                uses: Some("actions/upload-artifact@v4".to_string()),
                run: None,
                env: vec![],
                with_inputs: Some(
                    serde_yaml::from_str("name: my-artifact\nretention-days: 7").unwrap(),
                ),
            }],
            ..Default::default()
        }],
        source: source.to_string(),
        ..Default::default()
    };
    let issues = pipechecker::auditors::artifacts::audit(&pipeline).unwrap();
    assert!(issues.is_empty());
}

// =============================================================================
// A5: Secrets auditor — step env and run ref tests
// =============================================================================

#[test]
fn test_secrets_step_env_hardcoded() {
    let pipeline = Pipeline {
        provider: Provider::GitHubActions,
        jobs: vec![Job {
            id: "build".to_string(),
            steps: vec![Step {
                name: None,
                uses: None,
                run: Some("echo hi".to_string()),
                env: vec![EnvVar {
                    key: "API_KEY".to_string(),
                    value: "sk_live_abc123def456".to_string(),
                    is_secret: false,
                }],
                with_inputs: None,
            }],
            ..Default::default()
        }],
        source: "name: CI\non: push\njobs:\n  build:\n    runs-on: ubuntu\n    steps:\n      - run: echo hi\n".to_string(),
        ..Default::default()
    };
    let issues = pipechecker::auditors::secrets::audit(&pipeline).unwrap();
    assert!(issues
        .iter()
        .any(|i| i.message.contains("step env 'API_KEY'")));
}

#[test]
fn test_secrets_run_secret_ref() {
    let pipeline = Pipeline {
        provider: Provider::GitHubActions,
        jobs: vec![Job {
            id: "build".to_string(),
            steps: vec![Step {
                name: None,
                uses: None,
                run: Some("echo ${{ secrets.MY_TOKEN }}".to_string()),
                env: vec![],
                with_inputs: None,
            }],
            ..Default::default()
        }],
        source: "name: CI\non: push\njobs:\n  build:\n    runs-on: ubuntu\n    steps:\n      - run: echo\n".to_string(),
        ..Default::default()
    };
    let issues = pipechecker::auditors::secrets::audit(&pipeline).unwrap();
    assert!(issues
        .iter()
        .any(|i| i.message.contains("uses secret: MY_TOKEN")));
}

#[test]
fn test_secrets_run_undeclared_env_ref() {
    let pipeline = Pipeline {
        provider: Provider::GitHubActions,
        jobs: vec![Job {
            id: "build".to_string(),
            steps: vec![Step {
                name: None,
                uses: None,
                run: Some("echo ${{ env.UNDECLARED_VAR }}".to_string()),
                env: vec![],
                with_inputs: None,
            }],
            ..Default::default()
        }],
        source: "name: CI\non: push\njobs:\n  build:\n    runs-on: ubuntu\n".to_string(),
        ..Default::default()
    };
    let issues = pipechecker::auditors::secrets::audit(&pipeline).unwrap();
    assert!(issues
        .iter()
        .any(|i| i.message.contains("undeclared env var: UNDECLARED_VAR")));
}

#[test]
fn test_secrets_with_inputs_secret_ref() {
    let pipeline = Pipeline {
        provider: Provider::GitHubActions,
        jobs: vec![Job {
            id: "build".to_string(),
            steps: vec![Step {
                name: None,
                uses: Some("actions/checkout@v4".to_string()),
                run: None,
                env: vec![],
                with_inputs: Some(
                    serde_yaml::from_str("token: ${{ secrets.GITHUB_TOKEN }}").unwrap(),
                ),
            }],
            ..Default::default()
        }],
        source: "name: CI\non: push\njobs:\n  build:\n    runs-on: ubuntu\n".to_string(),
        ..Default::default()
    };
    let issues = pipechecker::auditors::secrets::audit(&pipeline).unwrap();
    assert!(issues
        .iter()
        .any(|i| i.message.contains("uses secret: GITHUB_TOKEN")));
}

#[test]
fn test_secrets_with_inputs_undeclared_env() {
    let pipeline = Pipeline {
        provider: Provider::GitHubActions,
        jobs: vec![Job {
            id: "build".to_string(),
            steps: vec![Step {
                name: None,
                uses: Some("actions/checkout@v4".to_string()),
                run: None,
                env: vec![],
                with_inputs: Some(serde_yaml::from_str("ref: ${{ env.MY_REF }}").unwrap()),
            }],
            ..Default::default()
        }],
        source: "name: CI\non: push\njobs:\n  build:\n    runs-on: ubuntu\n".to_string(),
        ..Default::default()
    };
    let issues = pipechecker::auditors::secrets::audit(&pipeline).unwrap();
    assert!(issues
        .iter()
        .any(|i| i.message.contains("undeclared env var: MY_REF")));
}

#[test]
fn test_secrets_long_alphanumeric_in_env() {
    let pipeline = Pipeline {
        provider: Provider::GitHubActions,
        env: vec![EnvVar {
            key: "MY_TOKEN".to_string(),
            value: "aB3dEf6hIjKlMnOpQrStUvWx".to_string(),
            is_secret: false,
        }],
        jobs: vec![],
        source: "name: CI\non: push\n".to_string(),
        ..Default::default()
    };
    let issues = pipechecker::auditors::secrets::audit(&pipeline).unwrap();
    assert!(issues
        .iter()
        .any(|i| i.message.contains("hardcoded secret")));
}

#[test]
fn test_secrets_base64_in_env() {
    let pipeline = Pipeline {
        provider: Provider::GitHubActions,
        env: vec![EnvVar {
            key: "ENCODED".to_string(),
            value: "abcDEF123+/xyzABC456===GHJklmno789pqrSTUV".to_string(),
            is_secret: false,
        }],
        jobs: vec![],
        source: "name: CI\non: push\n".to_string(),
        ..Default::default()
    };
    let issues = pipechecker::auditors::secrets::audit(&pipeline).unwrap();
    assert!(issues
        .iter()
        .any(|i| i.message.contains("hardcoded secret")));
}

#[test]
fn test_secrets_with_inputs_sequence() {
    let pipeline = Pipeline {
        provider: Provider::GitHubActions,
        jobs: vec![Job {
            id: "build".to_string(),
            steps: vec![Step {
                name: None,
                uses: Some("actions/checkout@v4".to_string()),
                run: None,
                env: vec![],
                with_inputs: Some(
                    serde_yaml::from_str("- ${{ secrets.TOKEN }}\n- ${{ env.UNDECLARED }}")
                        .unwrap(),
                ),
            }],
            ..Default::default()
        }],
        source: "name: CI\non: push\njobs:\n  build:\n    runs-on: ubuntu\n".to_string(),
        ..Default::default()
    };
    let issues = pipechecker::auditors::secrets::audit(&pipeline).unwrap();
    assert!(issues
        .iter()
        .any(|i| i.message.contains("uses secret: TOKEN")));
    assert!(issues
        .iter()
        .any(|i| i.message.contains("undeclared env var: UNDECLARED")));
}

// =============================================================================
// A6: Schema auditor tests
// =============================================================================

#[test]
fn test_schema_non_mapping_yaml() {
    let pipeline = Pipeline {
        provider: Provider::GitHubActions,
        source: "- just a string\n".to_string(),
        ..Default::default()
    };
    let issues = pipechecker::auditors::schema::audit(&pipeline).unwrap();
    assert!(issues
        .iter()
        .any(|i| i.message.contains("not a YAML mapping")));
}

#[test]
fn test_schema_github_missing_on() {
    let pipeline = Pipeline {
        provider: Provider::GitHubActions,
        source: "jobs:\n  build:\n    runs-on: ubuntu\n".to_string(),
        ..Default::default()
    };
    let issues = pipechecker::auditors::schema::audit(&pipeline).unwrap();
    assert!(issues
        .iter()
        .any(|i| i.message.contains("missing 'on' trigger")));
}

#[test]
fn test_schema_github_job_no_runs_on() {
    let pipeline = Pipeline {
        provider: Provider::GitHubActions,
        source: "on: push\njobs:\n  build:\n    steps:\n      - run: echo\n".to_string(),
        ..Default::default()
    };
    let issues = pipechecker::auditors::schema::audit(&pipeline).unwrap();
    assert!(issues
        .iter()
        .any(|i| i.message.contains("missing 'runs-on' or 'container'")));
}

#[test]
fn test_schema_github_job_no_steps() {
    let pipeline = Pipeline {
        provider: Provider::GitHubActions,
        source: "on: push\njobs:\n  build:\n    runs-on: ubuntu\n".to_string(),
        ..Default::default()
    };
    let issues = pipechecker::auditors::schema::audit(&pipeline).unwrap();
    assert!(issues.iter().any(|i| i.message.contains("missing 'steps'")));
}

#[test]
fn test_schema_github_unknown_key() {
    let pipeline = Pipeline {
        provider: Provider::GitHubActions,
        source: "on: push\nsome_unknown_key: true\njobs:\n  build:\n    runs-on: ubuntu\n    steps:\n      - run: echo\n".to_string(),
        ..Default::default()
    };
    let issues = pipechecker::auditors::schema::audit(&pipeline).unwrap();
    assert!(issues
        .iter()
        .any(|i| i.message.contains("Unknown top-level key")));
}

#[test]
fn test_schema_github_needs_non_array_string() {
    let pipeline = Pipeline {
        provider: Provider::GitHubActions,
        source: "on: push\njobs:\n  build:\n    runs-on: ubuntu\n    needs: 42\n    steps:\n      - run: echo\n".to_string(),
        ..Default::default()
    };
    let issues = pipechecker::auditors::schema::audit(&pipeline).unwrap();
    assert!(issues
        .iter()
        .any(|i| i.message.contains("'needs' should be string or array")));
}

#[test]
fn test_schema_github_job_not_mapping() {
    let pipeline = Pipeline {
        provider: Provider::GitHubActions,
        source: "on: push\njobs:\n  build: not-a-mapping\n".to_string(),
        ..Default::default()
    };
    let issues = pipechecker::auditors::schema::audit(&pipeline).unwrap();
    assert!(issues
        .iter()
        .any(|i| i.message.contains("is not a mapping")));
}

#[test]
fn test_schema_gitlab_missing_stages() {
    let pipeline = Pipeline {
        provider: Provider::GitLabCI,
        source: "build:\n  script: echo\n".to_string(),
        ..Default::default()
    };
    let issues = pipechecker::auditors::schema::audit(&pipeline).unwrap();
    assert!(issues
        .iter()
        .any(|i| i.message.contains("missing 'stages'")));
}

#[test]
fn test_schema_gitlab_no_jobs() {
    let pipeline = Pipeline {
        provider: Provider::GitLabCI,
        source: "stages: [build]\nvariables:\n  FOO: bar\n".to_string(),
        ..Default::default()
    };
    let issues = pipechecker::auditors::schema::audit(&pipeline).unwrap();
    assert!(issues.iter().any(|i| i.message.contains("no jobs defined")));
}

#[test]
fn test_schema_gitlab_unknown_key() {
    let pipeline = Pipeline {
        provider: Provider::GitLabCI,
        source: "stages: [build]\nfoo: bar\nbuild:\n  script: echo\n".to_string(),
        ..Default::default()
    };
    let issues = pipechecker::auditors::schema::audit(&pipeline).unwrap();
    assert!(issues
        .iter()
        .any(|i| i.message.contains("Unknown top-level key in GitLab CI")));
}

#[test]
fn test_schema_circleci_no_jobs_workflows() {
    let pipeline = Pipeline {
        provider: Provider::CircleCI,
        source: "version: 2.1\n".to_string(),
        ..Default::default()
    };
    let issues = pipechecker::auditors::schema::audit(&pipeline).unwrap();
    assert!(issues
        .iter()
        .any(|i| i.message.contains("missing 'jobs' or 'workflows'")));
}

#[test]
fn test_schema_circleci_unknown_key() {
    let pipeline = Pipeline {
        provider: Provider::CircleCI,
        source: "version: 2.1\nsome_unknown_key: true\njobs:\n  build:\n    docker:\n      - image: node:18\n".to_string(),
        ..Default::default()
    };
    let issues = pipechecker::auditors::schema::audit(&pipeline).unwrap();
    assert!(issues
        .iter()
        .any(|i| i.message.contains("Unknown top-level key in CircleCI")));
}

#[test]
fn test_schema_circleci_job_no_executor() {
    let pipeline = Pipeline {
        provider: Provider::CircleCI,
        source: "version: 2.1\njobs:\n  build:\n    steps:\n      - run: echo\n".to_string(),
        ..Default::default()
    };
    let issues = pipechecker::auditors::schema::audit(&pipeline).unwrap();
    assert!(issues
        .iter()
        .any(|i| i.message.contains("no executor specified")));
}

// =============================================================================
// A7: GitLab parser tests
// =============================================================================

#[test]
fn test_gitlab_extends_single_parent() {
    let source = r#".base:
  image: node:18
  script: echo base
build:
  extends: .base
  script: echo build
"#;
    let pipeline = pipechecker::parsers::gitlab::parse(source).unwrap();
    let build_job = pipeline.jobs.iter().find(|j| j.id == "build").unwrap();
    assert_eq!(build_job.container_image, Some("node:18".to_string()));
    // child overrides parent's script
    assert!(build_job
        .steps
        .iter()
        .any(|s| { s.run.as_deref().is_some_and(|r| r.contains("echo build")) }));
}

#[test]
fn test_gitlab_extends_multiple_parents() {
    let source = r#".base:
  image: node:18
.deploy:
  script: echo deploy
build:
  extends:
    - .base
    - .deploy
"#;
    let pipeline = pipechecker::parsers::gitlab::parse(source).unwrap();
    let build_job = pipeline.jobs.iter().find(|j| j.id == "build").unwrap();
    assert_eq!(build_job.container_image, Some("node:18".to_string()));
    assert!(build_job
        .steps
        .iter()
        .any(|s| { s.run.as_deref().is_some_and(|r| r.contains("echo deploy")) }));
}

#[test]
fn test_gitlab_extends_not_found() {
    let source = r#"build:
  extends: .nonexistent
  script: echo build
"#;
    let pipeline = pipechecker::parsers::gitlab::parse(source).unwrap();
    let build_job = pipeline.jobs.iter().find(|j| j.id == "build").unwrap();
    // should still parse, just no parent fields merged
    assert!(build_job
        .steps
        .iter()
        .any(|s| { s.run.as_deref().is_some_and(|r| r.contains("echo build")) }));
}

#[test]
fn test_gitlab_extends_non_string() {
    let source = r#".base:
  image: node:18
build:
  extends: 123
  script: echo build
"#;
    let pipeline = pipechecker::parsers::gitlab::parse(source).unwrap();
    let build_job = pipeline.jobs.iter().find(|j| j.id == "build").unwrap();
    assert!(build_job
        .steps
        .iter()
        .any(|s| { s.run.as_deref().is_some_and(|r| r.contains("echo build")) }));
}

#[test]
fn test_gitlab_workflow_rules() {
    let source = r#"workflow:
  rules:
    - when: always
    - if: $CI_PIPELINE_SOURCE == "push"
      when: on_success
stages: [build]
build:
  script: echo hi
"#;
    let pipeline = pipechecker::parsers::gitlab::parse(source).unwrap();
    assert_eq!(pipeline.workflow_rules.len(), 2);
    assert_eq!(pipeline.workflow_rules[0].when.as_deref(), Some("always"));
    assert_eq!(
        pipeline.workflow_rules[1].if_condition.as_deref(),
        Some("$CI_PIPELINE_SOURCE == \"push\"")
    );
}

#[test]
fn test_gitlab_job_rules() {
    let source = r#"stages: [build]
build:
  script: echo hi
  rules:
    - when: manual
    - if: $CI_COMMIT_BRANCH == "main"
      exists:
        - Dockerfile
      allow_failure: true
"#;
    let pipeline = pipechecker::parsers::gitlab::parse(source).unwrap();
    let build_job = pipeline.jobs.iter().find(|j| j.id == "build").unwrap();
    assert_eq!(build_job.rules.len(), 2);
    assert_eq!(build_job.rules[0].when.as_deref(), Some("manual"));
    assert_eq!(build_job.rules[1].allow_failure, Some(true));
    assert_eq!(
        build_job.rules[1].exists.as_ref().unwrap(),
        &vec!["Dockerfile".to_string()]
    );
}

#[test]
fn test_gitlab_image_mapping() {
    let source = r#"stages: [build]
build:
  image:
    name: node:18-alpine
  script: echo hi
"#;
    let pipeline = pipechecker::parsers::gitlab::parse(source).unwrap();
    let build_job = pipeline.jobs.iter().find(|j| j.id == "build").unwrap();
    assert_eq!(
        build_job.container_image,
        Some("node:18-alpine".to_string())
    );
}

#[test]
fn test_gitlab_job_inherits_global_image() {
    let source = r#"image: node:20
stages: [build]
build:
  script: echo hi
"#;
    let pipeline = pipechecker::parsers::gitlab::parse(source).unwrap();
    let build_job = pipeline.jobs.iter().find(|j| j.id == "build").unwrap();
    assert_eq!(build_job.container_image, Some("node:20".to_string()));
}

#[test]
fn test_gitlab_services_string() {
    let source = r#"stages: [build]
build:
  script: echo hi
  services:
    - postgres:15
    - redis:7
"#;
    let pipeline = pipechecker::parsers::gitlab::parse(source).unwrap();
    let build_job = pipeline.jobs.iter().find(|j| j.id == "build").unwrap();
    assert_eq!(build_job.service_images.len(), 2);
    assert!(build_job
        .service_images
        .contains(&"postgres:15".to_string()));
    assert!(build_job.service_images.contains(&"redis:7".to_string()));
}

#[test]
fn test_gitlab_services_mapping() {
    let source = r#"stages: [build]
build:
  script: echo hi
  services:
    - name: postgres:15
    - name: redis:7
"#;
    let pipeline = pipechecker::parsers::gitlab::parse(source).unwrap();
    let build_job = pipeline.jobs.iter().find(|j| j.id == "build").unwrap();
    assert_eq!(build_job.service_images.len(), 2);
}

#[test]
fn test_gitlab_trigger_string() {
    let source = r#"stages: [build]
trigger_downstream:
  trigger: downstream
"#;
    let pipeline = pipechecker::parsers::gitlab::parse(source).unwrap();
    let job = pipeline
        .jobs
        .iter()
        .find(|j| j.id == "trigger_downstream")
        .unwrap();
    assert!(job.steps.iter().any(|s| {
        s.run
            .as_deref()
            .is_some_and(|r| r.contains("trigger: downstream"))
    }));
}

#[test]
fn test_gitlab_trigger_mapping() {
    let source = r#"stages: [build]
trigger_downstream:
  trigger:
    project: my/group/repo
"#;
    let pipeline = pipechecker::parsers::gitlab::parse(source).unwrap();
    let job = pipeline
        .jobs
        .iter()
        .find(|j| j.id == "trigger_downstream")
        .unwrap();
    assert!(job.steps.iter().any(|s| {
        s.run
            .as_deref()
            .is_some_and(|r| r.contains("project: my/group/repo"))
    }));
}

#[test]
fn test_gitlab_needs_as_mapping() {
    let source = r#"stages: [build, test]
build:
  script: echo build
test:
  script: echo test
  needs:
    - job: build
"#;
    let pipeline = pipechecker::parsers::gitlab::parse(source).unwrap();
    let test_job = pipeline.jobs.iter().find(|j| j.id == "test").unwrap();
    assert!(test_job.depends_on.contains(&"build".to_string()));
}

#[test]
fn test_gitlab_dependencies_fallback() {
    let source = r#"stages: [build, test]
build:
  script: echo build
test:
  script: echo test
  dependencies:
    - build
"#;
    let pipeline = pipechecker::parsers::gitlab::parse(source).unwrap();
    let test_job = pipeline.jobs.iter().find(|j| j.id == "test").unwrap();
    assert!(test_job.depends_on.contains(&"build".to_string()));
}

#[test]
fn test_gitlab_script_as_string() {
    let source = r#"stages: [build]
build:
  script: echo single line
"#;
    let pipeline = pipechecker::parsers::gitlab::parse(source).unwrap();
    let build_job = pipeline.jobs.iter().find(|j| j.id == "build").unwrap();
    assert!(build_job.steps.iter().any(|s| {
        s.run
            .as_deref()
            .is_some_and(|r| r.contains("echo single line"))
    }));
}

#[test]
fn test_gitlab_hidden_jobs_filtered() {
    let source = r#"stages: [build]
.base:
  image: node:18
build:
  extends: .base
  script: echo build
"#;
    let pipeline = pipechecker::parsers::gitlab::parse(source).unwrap();
    assert_eq!(pipeline.jobs.len(), 1);
    assert_eq!(pipeline.jobs[0].id, "build");
}

#[test]
fn test_gitlab_include_string() {
    let source = r#"include: "./local.yml"
stages: [build]
build:
  script: echo hi
"#;
    let info = pipechecker::parsers::gitlab::parse_includes(source).unwrap();
    assert_eq!(info.local.len(), 1);
    assert_eq!(info.local[0], "./local.yml");
}

#[test]
fn test_gitlab_include_http() {
    let source = r#"include:
  - https://example.com/ci.yml
stages: [build]
build:
  script: echo hi
"#;
    let info = pipechecker::parsers::gitlab::parse_includes(source).unwrap();
    assert_eq!(info.remote.len(), 1);
}

#[test]
fn test_gitlab_include_project() {
    let source = r#"include:
  - project: "group/repo"
    file: "ci.yml"
stages: [build]
build:
  script: echo hi
"#;
    let info = pipechecker::parsers::gitlab::parse_includes(source).unwrap();
    assert_eq!(info.project.len(), 1);
}

#[test]
fn test_gitlab_include_project_string() {
    let source = r#"include: "group/project::path/to/ci.yml"
stages: [build]
build:
  script: echo hi
"#;
    let info = pipechecker::parsers::gitlab::parse_includes(source).unwrap();
    assert_eq!(info.project.len(), 1);
    assert!(info.project[0].contains("::"));
}

#[test]
fn test_gitlab_rule_condition_empty_exists() {
    let source = r#"stages: [build]
build:
  script: echo hi
  rules:
    - exists: []
      when: always
"#;
    let pipeline = pipechecker::parsers::gitlab::parse(source).unwrap();
    let build_job = pipeline.jobs.iter().find(|j| j.id == "build").unwrap();
    // empty exists returns None, so only when is present
    assert_eq!(build_job.rules.len(), 1);
    assert!(build_job.rules[0].exists.is_none());
}

#[test]
fn test_gitlab_rule_no_recognized_fields() {
    let source = r#"stages: [build]
build:
  script: echo hi
  rules:
    - some_unknown_field: value
"#;
    let pipeline = pipechecker::parsers::gitlab::parse(source).unwrap();
    let build_job = pipeline.jobs.iter().find(|j| j.id == "build").unwrap();
    // no recognized fields → parse_rule_condition returns None → empty rules
    assert!(build_job.rules.is_empty());
}

#[test]
fn test_gitlab_global_image_string() {
    let source = r#"image: node:18
stages: [build]
build:
  script: echo hi
"#;
    let pipeline = pipechecker::parsers::gitlab::parse(source).unwrap();
    assert_eq!(
        pipeline.jobs[0].container_image,
        Some("node:18".to_string())
    );
}

#[test]
fn test_gitlab_before_after_script() {
    let source = r#"stages: [build]
build:
  before_script: echo before
  script: echo main
  after_script: echo after
"#;
    let pipeline = pipechecker::parsers::gitlab::parse(source).unwrap();
    let build_job = pipeline.jobs.iter().find(|j| j.id == "build").unwrap();
    assert_eq!(build_job.steps.len(), 3);
    assert!(build_job
        .steps
        .iter()
        .any(|s| { s.name.as_deref() == Some("before_script") }));
    assert!(build_job
        .steps
        .iter()
        .any(|s| { s.name.as_deref() == Some("after_script") }));
}

#[test]
fn test_gitlab_job_timeout() {
    let source = r#"stages: [build]
build:
  script: echo hi
  timeout: 30
"#;
    let pipeline = pipechecker::parsers::gitlab::parse(source).unwrap();
    let build_job = pipeline.jobs.iter().find(|j| j.id == "build").unwrap();
    assert_eq!(build_job.timeout_minutes, Some(30));
}

#[test]
fn test_gitlab_job_variables() {
    let source = r#"stages: [build]
build:
  script: echo hi
  variables:
    FOO: bar
    BAZ: "123"
"#;
    let pipeline = pipechecker::parsers::gitlab::parse(source).unwrap();
    let build_job = pipeline.jobs.iter().find(|j| j.id == "build").unwrap();
    assert_eq!(build_job.env.len(), 2);
    assert!(build_job
        .env
        .iter()
        .any(|e| e.key == "FOO" && e.value == "bar"));
}

#[test]
fn test_gitlab_global_variables() {
    let source = r#"variables:
  GLOBAL_VAR: hello
stages: [build]
build:
  script: echo hi
"#;
    let pipeline = pipechecker::parsers::gitlab::parse(source).unwrap();
    assert_eq!(pipeline.env.len(), 1);
    assert_eq!(pipeline.env[0].key, "GLOBAL_VAR");
}

// =============================================================================
// A8: CircleCI parser tests
// =============================================================================

#[test]
fn test_circleci_global_env() {
    let source = r#"version: 2.1
environment:
  FOO: bar
  BAZ: "123"
jobs:
  build:
    docker:
      - image: node:18
    steps:
      - run: echo hi
"#;
    let pipeline = pipechecker::parsers::circleci::parse(source).unwrap();
    assert_eq!(pipeline.env.len(), 2);
    assert!(pipeline.env.iter().any(|e| e.key == "FOO"));
}

#[test]
fn test_circleci_workflow_string_job() {
    let source = r#"version: 2.1
jobs:
  build:
    docker:
      - image: node:18
    steps:
      - run: echo build
  test:
    docker:
      - image: node:18
    steps:
      - run: echo test
workflows:
  version: 2
  build_and_test:
    jobs:
      - build
      - test
"#;
    let pipeline = pipechecker::parsers::circleci::parse(source).unwrap();
    assert!(pipeline.jobs.iter().any(|j| j.id == "build"));
    assert!(pipeline.jobs.iter().any(|j| j.id == "test"));
}

#[test]
fn test_circleci_workflow_mapping_job_with_requires() {
    let source = r#"version: 2.1
jobs:
  build:
    docker:
      - image: node:18
    steps:
      - run: echo build
  test:
    docker:
      - image: node:18
    steps:
      - run: echo test
workflows:
  version: 2
  build_and_test:
    jobs:
      - build
      - test:
          requires:
            - build
"#;
    let pipeline = pipechecker::parsers::circleci::parse(source).unwrap();
    let test_job = pipeline.jobs.iter().find(|j| j.id == "test").unwrap();
    assert!(test_job.depends_on.contains(&"build".to_string()));
}

#[test]
fn test_circleci_named_executor() {
    let source = r#"version: 2.1
executors:
  my-exec:
    docker:
      - image: node:18
jobs:
  build:
    executor: my-exec
    steps:
      - run: echo hi
"#;
    let pipeline = pipechecker::parsers::circleci::parse(source).unwrap();
    let build_job = pipeline.jobs.iter().find(|j| j.id == "build").unwrap();
    assert_eq!(
        build_job.container_image,
        Some("executor:my-exec".to_string())
    );
}

#[test]
fn test_circleci_executor_mapping() {
    let source = r#"version: 2.1
jobs:
  build:
    executor:
      name: my-exec
    steps:
      - run: echo hi
"#;
    let pipeline = pipechecker::parsers::circleci::parse(source).unwrap();
    let build_job = pipeline.jobs.iter().find(|j| j.id == "build").unwrap();
    assert_eq!(build_job.container_image, Some("my-exec".to_string()));
}

#[test]
fn test_circleci_inline_docker_multiple() {
    let source = r#"version: 2.1
jobs:
  build:
    docker:
      - image: node:18
      - image: postgres:15
    steps:
      - run: echo hi
"#;
    let pipeline = pipechecker::parsers::circleci::parse(source).unwrap();
    let build_job = pipeline.jobs.iter().find(|j| j.id == "build").unwrap();
    assert_eq!(build_job.container_image, Some("node:18".to_string()));
    assert_eq!(build_job.service_images, vec!["postgres:15".to_string()]);
}

#[test]
fn test_circleci_step_run_string() {
    let source = r#"version: 2.1
jobs:
  build:
    docker:
      - image: node:18
    steps:
      - run: echo single string
"#;
    let pipeline = pipechecker::parsers::circleci::parse(source).unwrap();
    let build_job = pipeline.jobs.iter().find(|j| j.id == "build").unwrap();
    assert!(build_job.steps.iter().any(|s| {
        s.run
            .as_deref()
            .is_some_and(|r| r.contains("echo single string"))
    }));
}

#[test]
fn test_circleci_step_run_mapping() {
    let source = r#"version: 2.1
jobs:
  build:
    docker:
      - image: node:18
    steps:
      - run:
          command: echo mapped command
"#;
    let pipeline = pipechecker::parsers::circleci::parse(source).unwrap();
    let build_job = pipeline.jobs.iter().find(|j| j.id == "build").unwrap();
    assert!(build_job.steps.iter().any(|s| {
        s.run
            .as_deref()
            .is_some_and(|r| r.contains("echo mapped command"))
    }));
}

#[test]
fn test_circleci_step_checkout() {
    let source = r#"version: 2.1
jobs:
  build:
    docker:
      - image: node:18
    steps:
      - checkout: {}
"#;
    let pipeline = pipechecker::parsers::circleci::parse(source).unwrap();
    let build_job = pipeline.jobs.iter().find(|j| j.id == "build").unwrap();
    assert!(build_job
        .steps
        .iter()
        .any(|s| { s.uses.as_deref() == Some("circleci/checkout") }));
}

#[test]
fn test_circleci_step_save_cache() {
    let source = r#"version: 2.1
jobs:
  build:
    docker:
      - image: node:18
    steps:
      - save_cache:
          key: v1-deps
          paths:
            - node_modules
"#;
    let pipeline = pipechecker::parsers::circleci::parse(source).unwrap();
    let build_job = pipeline.jobs.iter().find(|j| j.id == "build").unwrap();
    assert!(build_job
        .steps
        .iter()
        .any(|s| { s.uses.as_deref() == Some("circleci/save_cache") }));
}

#[test]
fn test_circleci_step_custom_orb() {
    let source = r#"version: 2.1
jobs:
  build:
    docker:
      - image: node:18
    steps:
      - my-orb/my-command:
          param: value
"#;
    let pipeline = pipechecker::parsers::circleci::parse(source).unwrap();
    let build_job = pipeline.jobs.iter().find(|j| j.id == "build").unwrap();
    assert!(build_job
        .steps
        .iter()
        .any(|s| s.uses.as_deref() == Some("my-orb/my-command")));
}

#[test]
fn test_circleci_max_time() {
    let source = r#"version: 2.1
jobs:
  build:
    docker:
      - image: node:18
    max_time: 30
    steps:
      - run: echo hi
"#;
    let pipeline = pipechecker::parsers::circleci::parse(source).unwrap();
    let build_job = pipeline.jobs.iter().find(|j| j.id == "build").unwrap();
    assert_eq!(build_job.timeout_minutes, Some(30));
}

#[test]
fn test_circleci_job_environment() {
    let source = r#"version: 2.1
jobs:
  build:
    docker:
      - image: node:18
    environment:
      NODE_ENV: production
    steps:
      - run: echo hi
"#;
    let pipeline = pipechecker::parsers::circleci::parse(source).unwrap();
    let build_job = pipeline.jobs.iter().find(|j| j.id == "build").unwrap();
    assert_eq!(build_job.env.len(), 1);
    assert_eq!(build_job.env[0].key, "NODE_ENV");
}

// =============================================================================
// A9: GitHub parser tests
// =============================================================================

#[test]
fn test_github_workflow_call() {
    let source = r#"on:
  workflow_call:
    inputs:
      environment:
        type: string
    secrets:
      deploy_token:
        required: true
jobs:
  build:
    runs-on: ubuntu
    steps:
      - run: echo hi
"#;
    let pipeline = pipechecker::parsers::github::parse(source).unwrap();
    assert!(pipeline.is_reusable);
    assert!(pipeline
        .workflow_call_inputs
        .contains(&"environment".to_string()));
    assert!(pipeline
        .workflow_call_secrets
        .contains(&"deploy_token".to_string()));
}

#[test]
fn test_github_workflow_call_no_config() {
    let source = r#"on:
  workflow_call:
jobs:
  build:
    runs-on: ubuntu
    steps:
      - run: echo hi
"#;
    let pipeline = pipechecker::parsers::github::parse(source).unwrap();
    assert!(pipeline.is_reusable);
    assert!(pipeline.workflow_call_inputs.is_empty());
    assert!(pipeline.workflow_call_secrets.is_empty());
}

#[test]
fn test_github_container_image_mapping() {
    let source = r#"on: push
jobs:
  build:
    container:
      image:
        name: node:18-alpine
    steps:
      - run: echo hi
"#;
    let pipeline = pipechecker::parsers::github::parse(source).unwrap();
    let build_job = pipeline.jobs.iter().find(|j| j.id == "build").unwrap();
    assert_eq!(
        build_job.container_image,
        Some("node:18-alpine".to_string())
    );
}

#[test]
fn test_github_container_string() {
    let source = r#"on: push
jobs:
  build:
    container: node:20
    steps:
      - run: echo hi
"#;
    let pipeline = pipechecker::parsers::github::parse(source).unwrap();
    let build_job = pipeline.jobs.iter().find(|j| j.id == "build").unwrap();
    assert_eq!(build_job.container_image, Some("node:20".to_string()));
}

#[test]
fn test_github_services() {
    let source = r#"on: push
jobs:
  build:
    runs-on: ubuntu
    services:
      db:
        image: postgres:15
      redis:
        image: redis:7
    steps:
      - run: echo hi
"#;
    let pipeline = pipechecker::parsers::github::parse(source).unwrap();
    let build_job = pipeline.jobs.iter().find(|j| j.id == "build").unwrap();
    assert_eq!(build_job.service_images.len(), 2);
    assert!(build_job
        .service_images
        .contains(&"postgres:15".to_string()));
    assert!(build_job.service_images.contains(&"redis:7".to_string()));
}

#[test]
fn test_github_timeout_minutes() {
    let source = r#"on: push
jobs:
  build:
    runs-on: ubuntu
    timeout-minutes: 30
    steps:
      - run: echo hi
"#;
    let pipeline = pipechecker::parsers::github::parse(source).unwrap();
    let build_job = pipeline.jobs.iter().find(|j| j.id == "build").unwrap();
    assert_eq!(build_job.timeout_minutes, Some(30));
}

#[test]
fn test_github_env_secret_ref() {
    let source = r#"on: push
env:
  MY_SECRET: ${{ secrets.MY_SECRET }}
jobs:
  build:
    runs-on: ubuntu
    steps:
      - run: echo hi
"#;
    let pipeline = pipechecker::parsers::github::parse(source).unwrap();
    assert!(pipeline.env.iter().any(|e| e.is_secret));
}

#[test]
fn test_github_needs_string() {
    let source = r#"on: push
jobs:
  build:
    runs-on: ubuntu
    steps:
      - run: echo build
  test:
    runs-on: ubuntu
    needs: build
    steps:
      - run: echo test
"#;
    let pipeline = pipechecker::parsers::github::parse(source).unwrap();
    let test_job = pipeline.jobs.iter().find(|j| j.id == "test").unwrap();
    assert!(test_job.depends_on.contains(&"build".to_string()));
}

#[test]
fn test_github_needs_array() {
    let source = r#"on: push
jobs:
  build:
    runs-on: ubuntu
    steps:
      - run: echo build
  lint:
    runs-on: ubuntu
    steps:
      - run: echo lint
  test:
    runs-on: ubuntu
    needs: [build, lint]
    steps:
      - run: echo test
"#;
    let pipeline = pipechecker::parsers::github::parse(source).unwrap();
    let test_job = pipeline.jobs.iter().find(|j| j.id == "test").unwrap();
    assert_eq!(test_job.depends_on.len(), 2);
}

// =============================================================================
// A10: Config tests
// =============================================================================

#[test]
fn test_config_load_from_valid() {
    let dir = std::env::temp_dir().join("pipechecker_test_config_valid");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("config.yml");
    std::fs::write(
        &path,
        "ignore:\n  - old.yml\nrules:\n  circular_dependencies: false\n",
    )
    .unwrap();
    let config = pipechecker::config::load_from(path.to_str().unwrap());
    assert!(config.ignore.contains(&"old.yml".to_string()));
    assert!(!config.rules.circular_dependencies);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_config_load_from_nonexistent() {
    let config = pipechecker::config::load_from("/nonexistent/path/config.yml");
    assert!(config.ignore.is_empty());
    assert!(config.rules.circular_dependencies);
}

#[test]
fn test_config_load_from_invalid_yaml() {
    let dir = std::env::temp_dir().join("pipechecker_test_config_invalid");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("config.yml");
    std::fs::write(&path, "not: [valid: yaml: {").unwrap();
    let config = pipechecker::config::load_from(path.to_str().unwrap());
    // invalid yaml returns default
    assert!(config.ignore.is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}

// =============================================================================
// A11: Fix tests
// =============================================================================

#[test]
fn test_fix_dry_run() {
    let dir = std::env::temp_dir().join("pipechecker_test_fix_dry_run");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("ci.yml");
    std::fs::write(
        &path,
        "name: CI\non: push\njobs:\n  build:\n    runs-on: ubuntu\n    steps:\n      - uses: actions/checkout\n",
    )
    .unwrap();
    let result = pipechecker::fix::fix_file(path.to_str().unwrap(), true).unwrap();
    assert_eq!(result.fixed, 1);
    // file should be unchanged
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("uses: actions/checkout\n") && !content.contains("@v4"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_fix_dry_run_false() {
    let dir = std::env::temp_dir().join("pipechecker_test_fix_write");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("ci.yml");
    std::fs::write(
        &path,
        "name: CI\non: push\njobs:\n  build:\n    runs-on: ubuntu\n    steps:\n      - uses: actions/checkout\n",
    )
    .unwrap();
    let result = pipechecker::fix::fix_file(path.to_str().unwrap(), false).unwrap();
    assert_eq!(result.fixed, 1);
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("actions/checkout@v4"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_fix_unknown_action() {
    let input = "      - uses: unknown-org/unknown-action\n";
    let result = pipechecker::fix::fix_content(input);
    assert_eq!(result.fixed, 0);
    assert!(result.changes.iter().any(|c| c.contains("Unknown action")));
}

#[test]
fn test_fix_docker_image_prefix() {
    let input = "    - image: node:latest\n";
    let result = pipechecker::fix::fix_content(input);
    assert_eq!(result.fixed, 1);
    assert!(result.changes.iter().any(|c| c.contains("node:20-alpine")));
}

#[test]
fn test_fix_trailing_newline_preserved() {
    let dir = std::env::temp_dir().join("pipechecker_test_fix_newline");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("ci.yml");
    std::fs::write(
        &path,
        "name: CI\non: push\njobs:\n  build:\n    runs-on: ubuntu\n    steps:\n      - uses: actions/checkout\n",
    )
    .unwrap();
    pipechecker::fix::fix_file(path.to_str().unwrap(), false).unwrap();
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.ends_with('\n'));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_fix_no_trailing_newline() {
    let input = "      - uses: actions/checkout";
    let result = pipechecker::fix::fix_content(input);
    assert_eq!(result.fixed, 1);
    assert!(result.changes.iter().any(|c| c.contains("@v4")));
}

// =============================================================================
// A12: lib.rs config toggle tests
// =============================================================================

#[test]
fn test_audit_content_toggle_circular_false() {
    let content = r#"name: CI
on: push
jobs:
  a:
    runs-on: ubuntu
    needs: b
    steps: [{run: echo}]
  b:
    runs-on: ubuntu
    needs: a
    steps: [{run: echo}]
"#;
    let opts = AuditOptions {
        rules: Some(Rules {
            circular_dependencies: false,
            ..Rules::default()
        }),
        ..Default::default()
    };
    let result = audit_content(content, opts).unwrap();
    assert!(!result
        .issues
        .iter()
        .any(|i| i.rule_code == Some(pipechecker::rule_codes::CIRCULAR_DEPENDENCY)));
}

#[test]
fn test_audit_content_toggle_secrets_false() {
    let content = r#"name: CI
on: push
jobs:
  build:
    runs-on: ubuntu
    env:
      API_KEY: abc123secretkeyvalue
    steps:
      - run: echo ${{ secrets.MY_SECRET }}
"#;
    let opts = AuditOptions {
        rules: Some(Rules {
            missing_secrets: false,
            ..Rules::default()
        }),
        ..Default::default()
    };
    let result = audit_content(content, opts).unwrap();
    assert!(!result
        .issues
        .iter()
        .any(|i| i.rule_code == Some(pipechecker::rule_codes::HARDCODED_SECRET)));
}

#[test]
fn test_audit_content_toggle_timeout_false() {
    let content = r#"name: CI
on: push
jobs:
  build:
    runs-on: ubuntu
    steps:
      - run: echo
"#;
    let opts = AuditOptions {
        rules: Some(Rules {
            timeout_validation: false,
            ..Rules::default()
        }),
        ..Default::default()
    };
    let result = audit_content(content, opts).unwrap();
    assert!(!result
        .issues
        .iter()
        .any(|i| i.rule_code == Some(pipechecker::rule_codes::MISSING_TIMEOUT)));
}

#[test]
fn test_audit_content_toggle_permissions_false() {
    let content = r#"name: CI
on: push
jobs:
  build:
    runs-on: ubuntu
    steps:
      - run: echo
"#;
    let opts = AuditOptions {
        rules: Some(Rules {
            permissions_check: false,
            ..Rules::default()
        }),
        ..Default::default()
    };
    let result = audit_content(content, opts).unwrap();
    assert!(!result
        .issues
        .iter()
        .any(|i| i.rule_code == Some(pipechecker::rule_codes::MISSING_PERMISSIONS)));
}

#[test]
fn test_audit_content_toggle_schema_false() {
    let content = r#"on: push
jobs:
  build:
    runs-on: ubuntu
    steps:
      - run: echo
"#;
    let opts = AuditOptions {
        rules: Some(Rules {
            schema_validation: false,
            ..Rules::default()
        }),
        ..Default::default()
    };
    let result = audit_content(content, opts).unwrap();
    // schema validation disabled → no MISSING_TRIGGER warning
    assert!(!result
        .issues
        .iter()
        .any(|i| i.rule_code == Some(pipechecker::rule_codes::MISSING_TRIGGER)));
}

#[test]
fn test_audit_content_toggle_concurrency_false() {
    let content = r#"name: CI
on: push
concurrency:
  group: deploy
jobs:
  build:
    runs-on: ubuntu
    steps:
      - run: echo
"#;
    let opts = AuditOptions {
        rules: Some(Rules {
            concurrency_validation: false,
            ..Rules::default()
        }),
        ..Default::default()
    };
    let result = audit_content(content, opts).unwrap();
    assert!(!result
        .issues
        .iter()
        .any(|i| i.rule_code == Some(pipechecker::rule_codes::CONCURRENCY_CANCEL_MISSING)));
}

#[test]
fn test_audit_content_toggle_deprecated_false() {
    let content = r#"name: CI
on: push
jobs:
  build:
    runs-on: ubuntu
    steps:
      - run: echo "::set-output=name=foo::bar"
"#;
    let opts = AuditOptions {
        rules: Some(Rules {
            deprecated_feature_check: false,
            ..Rules::default()
        }),
        ..Default::default()
    };
    let result = audit_content(content, opts).unwrap();
    assert!(!result
        .issues
        .iter()
        .any(|i| i.rule_code == Some(pipechecker::rule_codes::DEPRECATED_ACTION)));
}

#[test]
fn test_audit_content_toggle_cost_false() {
    let content = r#"name: CI
on: push
jobs:
  build:
    runs-on: ubuntu
    timeout-minutes: 9999
    steps:
      - run: echo
"#;
    let opts = AuditOptions {
        rules: Some(Rules {
            cost_efficiency_check: false,
            ..Rules::default()
        }),
        ..Default::default()
    };
    let result = audit_content(content, opts).unwrap();
    assert!(!result
        .issues
        .iter()
        .any(|i| i.rule_code == Some(pipechecker::rule_codes::EXCESSIVE_TIMEOUT)));
}

#[test]
fn test_audit_content_docker_images_enabled() {
    let content = r#"name: CI
on: push
jobs:
  build:
    runs-on: ubuntu
    container: node:latest
    steps:
      - run: echo
"#;
    let opts = AuditOptions {
        check_docker_images: true,
        rules: Some(Rules {
            docker_latest_tag: true,
            ..Rules::default()
        }),
        ..Default::default()
    };
    // Should complete without error regardless of network feature
    let _result = audit_content(content, opts).unwrap();
}

#[test]
fn test_audit_content_docker_images_disabled() {
    let content = r#"name: CI
on: push
jobs:
  build:
    runs-on: ubuntu
    container: node:latest
    steps:
      - run: echo
"#;
    let opts = AuditOptions {
        check_docker_images: false,
        ..Default::default()
    };
    let result = audit_content(content, opts).unwrap();
    assert!(!result
        .issues
        .iter()
        .any(|i| i.message.contains("network feature")));
}

// =============================================================================
// A13: parsers/mod.rs tests
// =============================================================================

#[test]
fn test_detect_provider_unknown() {
    let result = pipechecker::parsers::detect_provider("just some random text\nwith no yaml keys");
    assert!(result.is_err());
}

#[test]
fn test_detect_provider_invalid_yaml() {
    let result = pipechecker::parsers::detect_provider("not: [valid: yaml: {");
    assert!(result.is_err());
}
