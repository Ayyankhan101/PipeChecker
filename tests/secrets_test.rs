//! Tests for secrets auditor

use pipechecker::auditors::secrets;
use pipechecker::parsers::{detect_provider, parse};

#[test]
fn test_detect_api_key_in_env() {
    let yaml = r#"
name: CI
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    env:
      API_KEY: apikey12345
      SECRET: secretkey789
    steps:
      - run: echo test
"#;
    let provider = detect_provider(yaml).unwrap();
    let pipeline = parse(yaml, provider).unwrap();
    let issues = secrets::audit(&pipeline).unwrap();

    let has_secrets = issues
        .iter()
        .any(|i| i.message.contains("API_KEY") || i.message.contains("SECRET"));
    assert!(has_secrets, "Should detect hardcoded secrets");
}

#[test]
fn test_allow_secret_references() {
    let yaml = r#"
name: CI
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    env:
      API_KEY: ${{ secrets.API_KEY }}
    steps:
      - run: echo test
"#;
    let provider = detect_provider(yaml).unwrap();
    let pipeline = parse(yaml, provider).unwrap();
    let issues = secrets::audit(&pipeline).unwrap();

    // Should NOT flag secret references
    let has_secrets_ref = issues
        .iter()
        .any(|i| i.message.contains("sk_") || i.message.contains("secret"));
    assert!(!has_secrets_ref, "Should allow secret references");
}

#[test]
fn test_detect_password_in_steps() {
    let yaml = r#"
name: CI
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - name: Deploy
        env:
          PASSWORD: password123
        run: deploy
"#;
    let provider = detect_provider(yaml).unwrap();
    let pipeline = parse(yaml, provider).unwrap();
    let issues = secrets::audit(&pipeline).unwrap();

    let has_password = issues
        .iter()
        .any(|i| i.message.to_lowercase().contains("password"));
    assert!(has_password, "Should detect password in steps");
}

#[test]
fn test_detect_github_token() {
    let yaml = r#"
name: CI
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    env:
      GITHUB_TOKEN: ghp_1234567890abcdefghijklmnopqrstuvwxyz
    steps:
      - run: echo test
"#;
    let provider = detect_provider(yaml).unwrap();
    let pipeline = parse(yaml, provider).unwrap();
    let issues = secrets::audit(&pipeline).unwrap();

    let has_token = issues
        .iter()
        .any(|i| i.message.contains("GITHUB_TOKEN") || i.message.contains("ghp_"));
    assert!(has_token, "Should detect GitHub token");
}

#[test]
fn test_detect_aws_access_key() {
    let yaml = r#"
name: CI
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    env:
      AWS_ACCESS_KEY_ID: myaccesskey12345
    steps:
      - run: aws s3 ls
"#;
    let provider = detect_provider(yaml).unwrap();
    let pipeline = parse(yaml, provider).unwrap();
    let issues = secrets::audit(&pipeline).unwrap();

    let has_aws = issues
        .iter()
        .any(|i| i.message.contains("AWS") || i.message.contains("AKIA"));
    assert!(has_aws, "Should detect AWS access key");
}

#[test]
fn test_gitlab_secrets_detection() {
    let yaml = r#"
image: node:20

stages:
  - build

build:
  stage: build
  variables:
    API_KEY: "sk_test_12345678"
  script:
    - npm install
"#;
    let provider = detect_provider(yaml).unwrap();
    let pipeline = parse(yaml, provider).unwrap();
    let issues = secrets::audit(&pipeline).unwrap();

    // GitLab CI secrets should also be detected
    assert!(!issues.is_empty());
}

#[test]
fn test_circleci_secrets_detection() {
    let yaml = r#"
version: 2.1

jobs:
  build:
    docker:
      - image: node:20
    environment:
      API_KEY: apikey12345
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
    let issues = secrets::audit(&pipeline).unwrap();

    let has_secret = issues.iter().any(|i| i.message.contains("API_KEY"));
    assert!(has_secret, "Should detect CircleCI environment secrets");
}
