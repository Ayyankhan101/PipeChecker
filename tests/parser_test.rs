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

// --- Edge case parser tests ---

#[test]
fn test_parse_github_actions_container_image_string() {
    let yaml = r#"
name: CI
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    container: node:18-alpine
    steps:
      - run: echo "hello"
"#;

    let pipeline = parse(yaml, Provider::GitHubActions).unwrap();
    let build_job = pipeline.jobs.iter().find(|j| j.id == "build").unwrap();
    assert_eq!(build_job.container_image, Some("node:18-alpine".to_string()));
}

#[test]
fn test_parse_github_actions_container_image_object() {
    let yaml = r#"
name: CI
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    container:
      image: myregistry.io/myimage:v1.2.3
      credentials:
        username: user
        password: pass
    steps:
      - run: echo "hello"
"#;

    let pipeline = parse(yaml, Provider::GitHubActions).unwrap();
    let build_job = pipeline.jobs.iter().find(|j| j.id == "build").unwrap();
    assert_eq!(
        build_job.container_image,
        Some("myregistry.io/myimage:v1.2.3".to_string())
    );
}

#[test]
fn test_parse_github_actions_service_images() {
    let yaml = r#"
name: CI
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: postgres
      redis:
        image: redis:7-alpine
    steps:
      - run: echo "hello"
"#;

    let pipeline = parse(yaml, Provider::GitHubActions).unwrap();
    let build_job = pipeline.jobs.iter().find(|j| j.id == "build").unwrap();
    assert_eq!(build_job.service_images.len(), 2);
    assert!(build_job.service_images.contains(&"postgres:15".to_string()));
    assert!(build_job
        .service_images
        .contains(&"redis:7-alpine".to_string()));
}

#[test]
fn test_parse_github_actions_with_inputs_captured() {
    let yaml = r#"
name: CI
on: push
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: some/action@v1
        with:
          token: ${{ secrets.TOKEN }}
          environment: production
"#;

    let pipeline = parse(yaml, Provider::GitHubActions).unwrap();
    let deploy_job = pipeline.jobs.iter().find(|j| j.id == "deploy").unwrap();
    let step = deploy_job.steps.iter().find(|s| s.uses.is_some()).unwrap();
    assert!(step.with_inputs.is_some());
}

#[test]
fn test_parse_github_actions_no_container_no_services() {
    let yaml = r#"
name: CI
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "hello"
"#;

    let pipeline = parse(yaml, Provider::GitHubActions).unwrap();
    let build_job = pipeline.jobs.iter().find(|j| j.id == "build").unwrap();
    assert!(build_job.container_image.is_none());
    assert!(build_job.service_images.is_empty());
}

#[test]
fn test_parse_gitlab_trigger_step() {
    let yaml = r#"
stages:
  - deploy

deploy-pipeline:
  stage: deploy
  trigger:
    project: myorg/infra-pipeline
    branch: main
"#;

    let pipeline = parse(yaml, Provider::GitLabCI).unwrap();
    assert_eq!(pipeline.jobs.len(), 1);
    let deploy_job = &pipeline.jobs[0];
    assert_eq!(deploy_job.id, "deploy-pipeline");
    assert!(deploy_job
        .steps
        .iter()
        .any(|s| s.name.as_deref() == Some("trigger")));
    assert!(deploy_job.steps[0]
        .run
        .as_deref()
        .unwrap()
        .contains("myorg/infra-pipeline"));
}

#[test]
fn test_parse_gitlab_trigger_string() {
    let yaml = r#"
stages:
  - deploy

deploy:
  stage: deploy
  trigger: myorg/downstream
"#;

    let pipeline = parse(yaml, Provider::GitLabCI).unwrap();
    let deploy_job = &pipeline.jobs[0];
    let trigger_step = deploy_job.steps.iter().find(|s| s.name.as_deref() == Some("trigger"));
    assert!(trigger_step.is_some());
}

#[test]
fn test_parse_gitlab_dependencies_keyword() {
    let yaml = r#"
stages:
  - build
  - test

build:
  stage: build
  script:
    - cargo build

test:
  stage: test
  dependencies:
    - build
  script:
    - cargo test
"#;

    let pipeline = parse(yaml, Provider::GitLabCI).unwrap();
    let test_job = pipeline.jobs.iter().find(|j| j.id == "test").unwrap();
    assert!(test_job.depends_on.contains(&"build".to_string()));
}

#[test]
fn test_parse_gitlab_complex_image() {
    let yaml = r#"
stages:
  - build

build:
  stage: build
  image:
    name: gcr.io/distroless/base
    entrypoint: [""]
  services:
    - postgres:15
    - name: redis:7
      alias: cache
  script:
    - echo "build"
"#;

    let pipeline = parse(yaml, Provider::GitLabCI).unwrap();
    let build_job = &pipeline.jobs[0];
    assert_eq!(
        build_job.container_image,
        Some("gcr.io/distroless/base".to_string())
    );
    assert_eq!(build_job.service_images.len(), 2);
    assert!(build_job.service_images.contains(&"postgres:15".to_string()));
    assert!(build_job.service_images.contains(&"redis:7".to_string()));
}

#[test]
fn test_parse_gitlab_before_after_script() {
    let yaml = r#"
stages:
  - test

test:
  stage: test
  before_script:
    - echo "setup"
  script:
    - cargo test
  after_script:
    - echo "cleanup"
"#;

    let pipeline = parse(yaml, Provider::GitLabCI).unwrap();
    let test_job = &pipeline.jobs[0];
    assert!(test_job.steps.iter().any(|s| s.name.as_deref() == Some("before_script")));
    assert!(test_job.steps.iter().any(|s| s.name.as_deref() == Some("script")));
    assert!(test_job.steps.iter().any(|s| s.name.as_deref() == Some("after_script")));
}

#[test]
fn test_parse_circleci_named_executor() {
    let yaml = r#"
version: 2.1

executors:
  node-exec:
    docker:
      - image: node:18

jobs:
  build:
    executor: node-exec
    steps:
      - checkout
      - run: npm install

workflows:
  main:
    jobs:
      - build
"#;

    let pipeline = parse(yaml, Provider::CircleCI).unwrap();
    let build_job = pipeline.jobs.iter().find(|j| j.id == "build").unwrap();
    assert_eq!(
        build_job.container_image,
        Some("executor:node-exec".to_string())
    );
}

#[test]
fn test_parse_circleci_docker_inline() {
    let yaml = r#"
version: 2.1

jobs:
  build:
    docker:
      - image: cimg/python:3.11
    steps:
      - run: python --version

workflows:
  main:
    jobs:
      - build
"#;

    let pipeline = parse(yaml, Provider::CircleCI).unwrap();
    let build_job = pipeline.jobs.iter().find(|j| j.id == "build").unwrap();
    assert_eq!(
        build_job.container_image,
        Some("cimg/python:3.11".to_string())
    );
}

#[test]
fn test_parse_circleci_cache_steps() {
    let yaml = r#"
version: 2.1

jobs:
  build:
    docker:
      - image: cimg/node:18
    steps:
      - checkout
      - restore_cache:
          keys:
            - npm-deps-{{ checksum "package-lock.json" }}
      - run: npm install
      - save_cache:
          paths:
            - node_modules
          key: npm-deps-{{ checksum "package-lock.json" }}

workflows:
  main:
    jobs:
      - build
"#;

    let pipeline = parse(yaml, Provider::CircleCI).unwrap();
    let build_job = pipeline.jobs.iter().find(|j| j.id == "build").unwrap();
    assert!(build_job
        .steps
        .iter()
        .any(|s| s.name.as_deref() == Some("restore_cache")));
    assert!(build_job
        .steps
        .iter()
        .any(|s| s.name.as_deref() == Some("save_cache")));
    // Cache steps should have with_inputs
    let restore = build_job
        .steps
        .iter()
        .find(|s| s.name.as_deref() == Some("restore_cache"));
    assert!(restore.unwrap().with_inputs.is_some());
}

#[test]
fn test_parse_circleci_environment() {
    let yaml = r#"
version: 2.1

jobs:
  build:
    docker:
      - image: node:18
    environment:
      NODE_ENV: production
      LOG_LEVEL: debug
    steps:
      - run: node server.js

workflows:
  main:
    jobs:
      - build
"#;

    let pipeline = parse(yaml, Provider::CircleCI).unwrap();
    let build_job = pipeline.jobs.iter().find(|j| j.id == "build").unwrap();
    assert!(build_job.env.iter().any(|e| e.key == "NODE_ENV"));
    assert!(build_job.env.iter().any(|e| e.key == "LOG_LEVEL"));
}

#[test]
fn test_parse_circleci_multiple_workflow_deps() {
    let yaml = r#"
version: 2.1

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
  deploy:
    docker:
      - image: node:18
    steps:
      - run: echo deploy

workflows:
  build-test:
    jobs:
      - build
      - test:
          requires:
            - build
  deploy-wf:
    jobs:
      - deploy:
          requires:
            - test
"#;

    let pipeline = parse(yaml, Provider::CircleCI).unwrap();
    let test_job = pipeline.jobs.iter().find(|j| j.id == "test").unwrap();
    assert!(test_job.depends_on.iter().any(|d| d == "build"));
}
