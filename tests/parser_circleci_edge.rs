//! Extended parser tests for CircleCI — edge cases

use pipechecker::parsers::{detect_provider, parse};

#[test]
fn test_named_executor_not_defined() {
    let yaml = r#"
version: 2.1

executors:
  my-executor:
    docker:
      - image: node:20

jobs:
  build:
    executor: my-executor
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
    // Parser captures as "executor:my-executor" marker string
    assert_eq!(
        pipeline.jobs[0].container_image,
        Some("executor:my-executor".to_string())
    );
}

#[test]
fn test_orb_with_parameters() {
    let yaml = r#"
version: 2.1

jobs:
  build:
    docker:
      - image: node:20
    steps:
      - my-orb/command@v1:
          input1: value1
          input2: value2

workflows:
  version: 2
  build:
    jobs:
      - build
"#;
    let provider = detect_provider(yaml).unwrap();
    let pipeline = parse(yaml, provider).unwrap();
    let step = &pipeline.jobs[0].steps[0];
    assert_eq!(step.name, Some("my-orb/command@v1".to_string()));
    assert_eq!(step.uses, Some("my-orb/command@v1".to_string()));
    assert!(step.with_inputs.is_some());
}

#[test]
fn test_approval_step() {
    let yaml = r#"
version: 2.1

jobs:
  build:
    docker:
      - image: node:20
    steps:
      - type: approval
        when: << pipeline.parameters.run_approval >>

workflows:
  version: 2
  build:
    jobs:
      - build
"#;
    let provider = detect_provider(yaml).unwrap();
    let pipeline = parse(yaml, provider).unwrap();
    // Approval step treated as custom orb-like step
    let step = &pipeline.jobs[0].steps[0];
    assert_eq!(step.name, Some("type".to_string()));
}

#[test]
fn test_workflow_requires_on_job() {
    let yaml = r#"
version: 2.1

jobs:
  test:
    docker:
      - image: node:20
    steps:
      - checkout
  build:
    docker:
      - image: node:20
    steps:
      - checkout
workflows:
  version: 2
  build:
    jobs:
      - test
      - build:
          requires:
            - test
"#;
    let provider = detect_provider(yaml).unwrap();
    let pipeline = parse(yaml, provider).unwrap();
    // Job-level requires should set depends_on for 'build'
    let build_job = pipeline.jobs.iter().find(|j| j.id == "build").unwrap();
    assert!(build_job.depends_on.contains(&"test".to_string()));
}

#[test]
fn test_step_with_context() {
    let yaml = r#"
version: 2.1

jobs:
  build:
    docker:
      - image: node:20
    steps:
      - attach_workspace:
          at: ~/project

workflows:
  version: 2
  build:
    jobs:
      - build
"#;
    let provider = detect_provider(yaml).unwrap();
    let pipeline = parse(yaml, provider).unwrap();
    let step = &pipeline.jobs[0].steps[0];
    assert_eq!(step.name, Some("attach_workspace".to_string()));
    assert!(step.with_inputs.is_some());
}

#[test]
fn test_step_with_type_and_name() {
    let yaml = r#"
version: 2.1

jobs:
  build:
    docker:
      - image: node:20
    steps:
      - say-hello:
          when: << pipeline.parameters.run_hello >>

workflows:
  version: 2
  build:
    jobs:
      - build
"#;
    let provider = detect_provider(yaml).unwrap();
    let pipeline = parse(yaml, provider).unwrap();
    let step = &pipeline.jobs[0].steps[0];
    assert_eq!(step.name, Some("say-hello".to_string()));
}

#[test]
fn test_multiple_docker_images_in_job() {
    let yaml = r#"
version: 2.1

jobs:
  build:
    docker:
      - image: node:20
      - image: postgres:15
        alias: db
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
    let job = &pipeline.jobs[0];
    assert_eq!(job.container_image, Some("node:20".to_string()));
    assert_eq!(job.service_images, vec!["postgres:15"]);
}

#[test]
fn test_job_without_workflow() {
    let yaml = r#"
version: 2.1

jobs:
  build:
    docker:
      - image: node:20
    steps:
      - checkout
"#;
    let provider = detect_provider(yaml).unwrap();
    let pipeline = parse(yaml, provider).unwrap();
    // Job should be parsed even if not referenced in any workflow
    assert_eq!(pipeline.jobs.len(), 1);
    assert_eq!(pipeline.jobs[0].id, "build");
}

#[test]
fn test_circleci_named_executor_reference() {
    let yaml = r#"
version: 2.1

executors:
  my-executor:
    docker:
      - image: node:20

jobs:
  build:
    executor: my-executor
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
    // Parser uses executor as marker
    assert_eq!(
        pipeline.jobs[0].container_image,
        Some("executor:my-executor".to_string())
    );
}
