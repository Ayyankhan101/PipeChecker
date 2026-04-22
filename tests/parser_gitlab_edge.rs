//! Extended parser tests for GitLab CI — edge cases

use pipechecker::parsers::{detect_provider, parse};

#[test]
fn test_image_object_with_entrypoint() {
    let yaml = r#"
image:
  name: node:20
  entrypoint: ["/bin/bash"]

build:
  stage: build
  script:
    - npm test
"#;
    let provider = detect_provider(yaml).unwrap();
    let pipeline = parse(yaml, provider).unwrap();
    assert_eq!(
        pipeline.jobs[0].container_image,
        Some("node:20".to_string())
    );
}

#[test]
fn test_services_full_object_with_alias() {
    let yaml = r#"
image: node:20

build:
  stage: build
  services:
    - name: postgres:15
      alias: db
      entrypoint: ["docker-entrypoint.sh"]
  script:
    - npm test
"#;
    let provider = detect_provider(yaml).unwrap();
    let pipeline = parse(yaml, provider).unwrap();
    let job = &pipeline.jobs[0];
    assert_eq!(job.service_images, vec!["postgres:15"]);
}

#[test]
fn test_trigger_project_branch_syntax() {
    let yaml = r#"
image: node:20

stages:
  - build

build:
  stage: build
  script:
    - npm test
  trigger:
    project: group/other-project
    branch: main
"#;
    let provider = detect_provider(yaml).unwrap();
    let pipeline = parse(yaml, provider).unwrap();
    // The parsing should succeed; trigger details are ignored for now.
    assert!(!pipeline.jobs.is_empty());
}

#[test]
fn test_before_script_after_script_array() {
    let yaml = r#"
image: node:20

before_script:
  - apt-get update
  - apt-get install -y build-essential

build:
  stage: build
  script:
    - npm test
  after_script:
    - echo done
"#;
    let provider = detect_provider(yaml).unwrap();
    let pipeline = parse(yaml, provider).unwrap();
    // Parser currently does not convert before_script/after_script into steps, but they should not crash.
    assert!(!pipeline.jobs.is_empty());
}

#[test]
fn test_needs_and_dependencies_both() {
    let yaml = r#"
image: node:20

stages:
  - build

build:
  stage: build
  needs: [test]
  dependencies: [test]
  script:
    - npm test
"#;
    let provider = detect_provider(yaml).unwrap();
    let pipeline = parse(yaml, provider).unwrap();
    let job = &pipeline.jobs[0];
    assert!(job.depends_on.contains(&"test".to_string()));
}

#[test]
fn test_invalid_reserved_key_as_job() {
    // Top-level 'stages' is reserved; should not be parsed as a job
    let yaml = r#"
stages:
  - build
  - test

build:
  stage: build
  script:
    - echo build
"#;
    let provider = detect_provider(yaml).unwrap();
    let pipeline = parse(yaml, provider).unwrap();
    // 'stages' should not be a job; only 'build' should be job
    assert_eq!(pipeline.jobs.len(), 1);
    assert_eq!(pipeline.jobs[0].id, "build");
}

#[test]
fn test_job_with_variables_and_script() {
    let yaml = r#"
image: node:20

variables:
  NODE_OPTIONS: --max-old-space-size=4096

build:
  stage: build
  variables:
    DEBUG: true
  script:
    - npm test
"#;
    let provider = detect_provider(yaml).unwrap();
    let pipeline = parse(yaml, provider).unwrap();
    assert_eq!(pipeline.jobs.len(), 1);
    // global env should include NODE_OPTIONS
    assert!(pipeline.env.iter().any(|e| e.key == "NODE_OPTIONS"));
    // job env includes DEBUG
    let job = &pipeline.jobs[0];
    assert!(job.env.iter().any(|e| e.key == "DEBUG"));
}

#[test]
fn test_gitlab_trigger_with_strategy() {
    let yaml = r#"
image: node:20

stages:
  - build

build:
  stage: build
  script:
    - npm test
  trigger:
    project: group/other
    branch: main
    strategy: depend
"#;
    let provider = detect_provider(yaml).unwrap();
    let pipeline = parse(yaml, provider).unwrap();
    // Parsing should succeed; trigger details not modeled yet
    assert!(!pipeline.jobs.is_empty());
}
