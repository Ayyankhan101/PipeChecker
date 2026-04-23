//! Extended parser tests for GitHub Actions — edge cases

use pipechecker::parsers::{detect_provider, parse};

#[test]
fn test_container_with_credentials_object() {
    let yaml = r#"
name: CI
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    container:
      image:
        name: node:20
        credentials:
          username: myuser
          password: ${{ secrets.PASSWORD }}
    steps:
      - run: echo test
"#;
    let provider = detect_provider(yaml).unwrap();
    let pipeline = parse(yaml, provider).unwrap();
    assert_eq!(pipeline.jobs.len(), 1);
    let job = &pipeline.jobs[0];
    assert_eq!(job.container_image, Some("node:20".to_string()));
    // credentials are ignored; no error
}

#[test]
fn test_services_with_alias() {
    let yaml = r#"
name: CI
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    services:
      db:
        image: postgres:15
        alias: database
    steps:
      - run: echo test
"#;
    let provider = detect_provider(yaml).unwrap();
    let pipeline = parse(yaml, provider).unwrap();
    let job = &pipeline.jobs[0];
    assert_eq!(job.service_images, vec!["postgres:15"]);
}

#[test]
fn test_step_with_only_uses_and_with_no_run() {
    let yaml = r#"
name: CI
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          submodules: true
"#;
    let provider = detect_provider(yaml).unwrap();
    let pipeline = parse(yaml, provider).unwrap();
    let step = &pipeline.jobs[0].steps[0];
    assert_eq!(step.uses, Some("actions/checkout@v4".to_string()));
    assert!(step.run.is_none());
    assert!(step.with_inputs.is_some());
}

#[test]
fn test_step_with_continue_on_error() {
    let yaml = r#"
name: CI
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: npm test
        continue-on-error: true
"#;
    let provider = detect_provider(yaml).unwrap();
    let pipeline = parse(yaml, provider).unwrap();
    // Should parse without error; continue-on-error is ignored by the parser (not part of Step model)
    assert_eq!(pipeline.jobs[0].steps[0].run, Some("npm test".to_string()));
}

#[test]
fn test_job_container_full_object() {
    let yaml = r#"
name: CI
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    container:
      image: node:20
      env:
        NODE_ENV: test
      ports:
        - 8080
    steps:
      - run: echo test
"#;
    let provider = detect_provider(yaml).unwrap();
    let pipeline = parse(yaml, provider).unwrap();
    let job = &pipeline.jobs[0];
    assert_eq!(job.container_image, Some("node:20".to_string()));
    // env inside container is not currently captured into job.env, that's fine
}

#[test]
fn test_malformed_yaml_invalid_syntax() {
    let yaml = "name: CI\non: push\njobs:\n  build:\n    runs-on: [";
    let result = detect_provider(yaml);
    assert!(result.is_err());
}

#[test]
fn test_duplicate_job_ids_parse_error() {
    let yaml = r#"
name: CI
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo A
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo B
"#;
    let result = detect_provider(yaml);
    // detect_provider should fail on duplicate keys in YAML
    assert!(result.is_err());
}

#[test]
fn test_step_with_uses_and_run_both() {
    let yaml = r#"
name: CI
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        run: echo check it out
"#;
    let provider = detect_provider(yaml).unwrap();
    let pipeline = parse(yaml, provider).unwrap();
    let step = &pipeline.jobs[0].steps[0];
    assert_eq!(step.uses, Some("actions/checkout@v4".to_string()));
    assert_eq!(step.run, Some("echo check it out".to_string()));
}

#[test]
fn test_github_actions_with_matrix() {
    // Matrix is just a string/value; parser ignores but should not crash
    let yaml = r#"
name: CI
on: push
jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        node: [18, 20]
    steps:
      - run: echo ${{ matrix.node }}
"#;
    let provider = detect_provider(yaml).unwrap();
    let pipeline = parse(yaml, provider).unwrap();
    assert_eq!(pipeline.jobs.len(), 1);
}
