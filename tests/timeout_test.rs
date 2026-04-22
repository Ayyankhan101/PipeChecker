//! Tests for timeout auditor

use pipechecker::auditors::timeout;
use pipechecker::parsers::{detect_provider, parse};

#[test]
fn test_github_timeout_detected() {
    let yaml = r#"
name: CI
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    timeout-minutes: 30
    steps:
      - run: echo test
"#;
    let provider = detect_provider(yaml).unwrap();
    let pipeline = parse(yaml, provider).unwrap();
    let issues = timeout::audit(&pipeline).unwrap();

    // Should NOT warn when timeout is set
    let has_timeout_warning = issues.iter().any(|i| i.message.contains("timeout-minutes"));
    assert!(!has_timeout_warning, "Should not warn when timeout is set");
}

#[test]
fn test_github_missing_timeout() {
    let yaml = r#"
name: CI
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: npm test
"#;
    let provider = detect_provider(yaml).unwrap();
    let pipeline = parse(yaml, provider).unwrap();
    let issues = timeout::audit(&pipeline).unwrap();

    // Should warn when no timeout
    let has_no_timeout = issues.iter().any(|i| i.message.contains("timeout-minutes"));
    assert!(has_no_timeout, "Should warn when timeout is missing");
}

#[test]
fn test_gitlab_timeout_detected() {
    let yaml = r#"
image: node:20

build:
  stage: build
  timeout: 30
  script:
    - npm test
"#;
    let provider = detect_provider(yaml).unwrap();
    let pipeline = parse(yaml, provider).unwrap();
    let issues = timeout::audit(&pipeline).unwrap();

    // When GitLab has timeout set, should not warn
    let has_warning = issues
        .iter()
        .any(|i| i.message.to_lowercase().contains("timeout"));
    assert!(!has_warning, "Should not warn when gitlab timeout is set");
}

#[test]
fn test_gitlab_missing_timeout() {
    let yaml = r#"
image: node:20

build:
  stage: build
  script:
    - npm test
"#;
    let provider = detect_provider(yaml).unwrap();
    let pipeline = parse(yaml, provider).unwrap();
    let issues = timeout::audit(&pipeline).unwrap();

    let has_warning = issues
        .iter()
        .any(|i| i.message.to_lowercase().contains("timeout"));
    assert!(has_warning, "Should warn when gitlab timeout is missing");
}

#[test]
fn test_circleci_timeout_detected() {
    let yaml = r#"
version: 2.1

jobs:
  build:
    docker:
      - image: node:20
    steps:
      - checkout
    max_time: 30

workflows:
  version: 2
  build:
    jobs:
      - build
"#;
    let provider = detect_provider(yaml).unwrap();
    let pipeline = parse(yaml, provider).unwrap();
    let issues = timeout::audit(&pipeline).unwrap();

    // Should not warn when CircleCI timeout is set
    let has_warning = issues
        .iter()
        .any(|i| i.message.to_lowercase().contains("max_time"));
    assert!(!has_warning, "Should not warn when CircleCI timeout is set");
}

#[test]
fn test_circleci_missing_timeout() {
    let yaml = r#"
version: 2.1

jobs:
  build:
    docker:
      - image: node:20
    steps:
      - checkout

workflows:
  version: 2
  build:
    jobs:
      - build
"#;
    let provider = detect_provider(yaml).unwrap();
    let pipeline = parse(yaml, provider).unwrap();
    let issues = timeout::audit(&pipeline).unwrap();

    let has_warning = issues
        .iter()
        .any(|i| i.message.to_lowercase().contains("max_time"));
    assert!(has_warning, "Should warn when CircleCI timeout is missing");
}

#[test]
fn test_multiple_jobs_timeout() {
    let yaml = r#"
name: CI
on: push
jobs:
  lint:
    runs-on: ubuntu-latest
    timeout-minutes: 10
    steps:
      - run: npm lint
  
  test:
    runs-on: ubuntu-latest
    steps:
      - run: npm test
  
  build:
    runs-on: ubuntu-latest
    timeout-minutes: 20
    steps:
      - run: npm run build
"#;
    let provider = detect_provider(yaml).unwrap();
    let pipeline = parse(yaml, provider).unwrap();
    let issues = timeout::audit(&pipeline).unwrap();

    // Should warn only for jobs without timeout
    let job_names = issues
        .iter()
        .filter_map(|i| i.location.as_ref()?.job.as_ref())
        .collect::<Vec<_>>();

    // Only 'test' job should have warning
    assert!(job_names.iter().any(|j| *j == "test"));
    assert!(!job_names.iter().any(|j| *j == "lint" || *j == "build"));
}

// --- Edge cases for timeout handling ---

#[test]
fn test_gitlab_timeout_non_numeric_warns() {
    // Non-numeric timeout like "30m" should be treated as missing/invalid and produce warning
    let yaml = r#"
image: node:20

build:
  stage: build
  timeout: "30m"
  script:
    - npm test
"#;
    let provider = detect_provider(yaml).unwrap();
    let pipeline = parse(yaml, provider).unwrap();
    let issues = timeout::audit(&pipeline).unwrap();

    let has_warning = issues
        .iter()
        .any(|i| i.message.to_lowercase().contains("timeout"));
    assert!(
        has_warning,
        "Should warn when gitlab timeout is non-numeric"
    );
}

#[test]
fn test_circleci_max_time_non_numeric_warns() {
    // Non-numeric max_time should warn
    let yaml = r#"
version: 2.1

jobs:
  build:
    docker:
      - image: node:20
    steps:
      - checkout
    max_time: "30 minutes"

workflows:
  version: 2
  build:
    jobs:
      - build
"#;
    let provider = detect_provider(yaml).unwrap();
    let pipeline = parse(yaml, provider).unwrap();
    let issues = timeout::audit(&pipeline).unwrap();

    let has_warning = issues
        .iter()
        .any(|i| i.message.to_lowercase().contains("max_time"));
    assert!(
        has_warning,
        "Should warn when CircleCI max_time is non-numeric"
    );
}

#[test]
fn test_github_timeout_zero_allowed() {
    // GitHub allows timeout-minutes: 0 (edge: might be instant timeout, but not flagged)
    let yaml = r#"
name: CI
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    timeout-minutes: 0
    steps:
      - run: echo test
"#;
    let provider = detect_provider(yaml).unwrap();
    let pipeline = parse(yaml, provider).unwrap();
    let issues = timeout::audit(&pipeline).unwrap();

    let warns_about_timeout = issues.iter().any(|i| i.message.contains("timeout-minutes"));
    assert!(
        !warns_about_timeout,
        "Zero timeout should not trigger missing timeout warning"
    );
}

#[test]
fn test_gitlab_timeout_zero_allowed() {
    let yaml = r#"
image: node:20

build:
  stage: build
  timeout: 0
  script:
    - npm test
"#;
    let provider = detect_provider(yaml).unwrap();
    let pipeline = parse(yaml, provider).unwrap();
    let issues = timeout::audit(&pipeline).unwrap();

    let warns_about_timeout = issues
        .iter()
        .any(|i| i.message.to_lowercase().contains("timeout"));
    assert!(!warns_about_timeout, "Zero timeout should be accepted");
}

#[test]
fn test_circleci_max_time_zero_allowed() {
    let yaml = r#"
version: 2.1

jobs:
  build:
    docker:
      - image: node:20
    steps:
      - checkout
    max_time: 0

workflows:
  version: 2
  build:
    jobs:
      - build
"#;
    let provider = detect_provider(yaml).unwrap();
    let pipeline = parse(yaml, provider).unwrap();
    let issues = timeout::audit(&pipeline).unwrap();

    let warns_about_timeout = issues
        .iter()
        .any(|i| i.message.to_lowercase().contains("max_time"));
    assert!(!warns_about_timeout, "Zero max_time should be accepted");
}
