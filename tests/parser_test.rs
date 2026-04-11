//! Integration tests for pipechecker parsers

use pipechecker::models::Provider;
use pipechecker::parsers::{detect_provider, parse};

#[test]
fn test_detect_provider_github_actions() {
    let yaml = r#"
name: CI
on:
  push:
    branches: [main]
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
"#;

    let provider = detect_provider(yaml).unwrap();
    assert_eq!(provider, Provider::GitHubActions);
}

#[test]
fn test_detect_provider_gitlab_ci() {
    let yaml = r#"
stages:
  - build
  - test

build:
  stage: build
  script:
    - echo "Building"
"#;

    let provider = detect_provider(yaml).unwrap();
    assert_eq!(provider, Provider::GitLabCI);
}

#[test]
fn test_detect_provider_circleci() {
    let yaml = r#"
version: 2.1

workflows:
  build-test:
    jobs:
      - build

jobs:
  build:
    docker:
      - image: cimg/node:18.0
    steps:
      - checkout
"#;

    let provider = detect_provider(yaml).unwrap();
    assert_eq!(provider, Provider::CircleCI);
}

#[test]
fn test_detect_provider_unknown() {
    let yaml = r#"
some_random_key: value
another_key: 123
"#;

    assert!(detect_provider(yaml).is_err());
}

#[test]
fn test_parse_github_actions_valid() {
    let yaml = r#"
name: Test Workflow
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build
        run: cargo build
  test:
    runs-on: ubuntu-latest
    needs: build
    steps:
      - uses: actions/checkout@v4
      - name: Test
        run: cargo test
"#;

    let pipeline = parse(yaml, Provider::GitHubActions).unwrap();
    assert_eq!(pipeline.provider, Provider::GitHubActions);
    assert_eq!(pipeline.jobs.len(), 2);

    // Check job IDs
    let job_ids: Vec<_> = pipeline.jobs.iter().map(|j| j.id.as_str()).collect();
    assert!(job_ids.contains(&"build"));
    assert!(job_ids.contains(&"test"));
}

#[test]
fn test_parse_github_actions_with_env() {
    let yaml = r#"
name: Test Workflow
on: push
env:
  GLOBAL_VAR: global_value
  SECRET_TOKEN: ${{ secrets.TOKEN }}
jobs:
  build:
    runs-on: ubuntu-latest
    env:
      JOB_VAR: job_value
    steps:
      - uses: actions/checkout@v4
      - name: Build
        run: cargo build
        env:
          STEP_VAR: step_value
"#;

    let pipeline = parse(yaml, Provider::GitHubActions).unwrap();

    // Check workflow-level env
    assert_eq!(pipeline.env.len(), 2);
    assert!(pipeline.env.iter().any(|e| e.key == "GLOBAL_VAR"));
    assert!(pipeline
        .env
        .iter()
        .any(|e| e.key == "SECRET_TOKEN" && e.is_secret));

    // Check job-level env
    let build_job = pipeline.jobs.iter().find(|j| j.id == "build").unwrap();
    assert_eq!(build_job.env.len(), 1);
    assert_eq!(build_job.env[0].key, "JOB_VAR");
}

#[test]
fn test_parse_github_actions_needs_array() {
    let yaml = r#"
name: Test Workflow
on: push
jobs:
  job-a:
    runs-on: ubuntu-latest
    steps:
      - run: echo "A"
  job-b:
    runs-on: ubuntu-latest
    steps:
      - run: echo "B"
  job-c:
    runs-on: ubuntu-latest
    needs: [job-a, job-b]
    steps:
      - run: echo "C"
"#;

    let pipeline = parse(yaml, Provider::GitHubActions).unwrap();
    let job_c = pipeline.jobs.iter().find(|j| j.id == "job-c").unwrap();
    assert_eq!(job_c.depends_on.len(), 2);
    assert!(job_c.depends_on.contains(&"job-a".to_string()));
    assert!(job_c.depends_on.contains(&"job-b".to_string()));
}

#[test]
fn test_parse_github_actions_needs_single() {
    let yaml = r#"
name: Test Workflow
on: push
jobs:
  job-a:
    runs-on: ubuntu-latest
    steps:
      - run: echo "A"
  job-b:
    runs-on: ubuntu-latest
    needs: job-a
    steps:
      - run: echo "B"
"#;

    let pipeline = parse(yaml, Provider::GitHubActions).unwrap();
    let job_b = pipeline.jobs.iter().find(|j| j.id == "job-b").unwrap();
    assert_eq!(job_b.depends_on, vec!["job-a"]);
}

#[test]
fn test_parse_gitlab_ci_valid() {
    let yaml = r#"
stages:
  - build
  - test

variables:
  GLOBAL_VAR: global_value

build:
  stage: build
  image: rust:1.75
  script:
    - cargo build
  variables:
    JOB_VAR: job_value

test:
  stage: test
  needs:
    - job: build
  script:
    - cargo test
"#;

    let pipeline = parse(yaml, Provider::GitLabCI).unwrap();
    assert_eq!(pipeline.provider, Provider::GitLabCI);
    assert_eq!(pipeline.jobs.len(), 2);

    let job_ids: Vec<_> = pipeline.jobs.iter().map(|j| j.id.as_str()).collect();
    assert!(job_ids.contains(&"build"));
    assert!(job_ids.contains(&"test"));

    // Check that test job depends on build
    let test_job = pipeline.jobs.iter().find(|j| j.id == "test").unwrap();
    assert!(test_job.depends_on.contains(&"build".to_string()));
}

#[test]
fn test_parse_circleci_valid() {
    let yaml = r#"
version: 2.1

jobs:
  build:
    docker:
      - image: cimg/node:18.0
    steps:
      - checkout
      - run: npm install
  test:
    docker:
      - image: cimg/node:18.0
    steps:
      - run: npm test

workflows:
  build-test:
    jobs:
      - build
      - test:
          requires:
            - build
"#;

    let pipeline = parse(yaml, Provider::CircleCI).unwrap();
    assert_eq!(pipeline.provider, Provider::CircleCI);
    assert_eq!(pipeline.jobs.len(), 2);

    let job_ids: Vec<_> = pipeline.jobs.iter().map(|j| j.id.as_str()).collect();
    assert!(job_ids.contains(&"build"));
    assert!(job_ids.contains(&"test"));

    // Check that test job depends on build (from workflow)
    let test_job = pipeline.jobs.iter().find(|j| j.id == "test").unwrap();
    assert!(test_job.depends_on.contains(&"build".to_string()));
}
