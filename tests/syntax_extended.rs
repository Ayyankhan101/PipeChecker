//! Extended syntax auditor tests

use pipechecker::auditors::syntax;
use pipechecker::models::{Job, Pipeline, Provider, Severity, Step};

fn create_test_step(name: &str, run: &str) -> Step {
    Step {
        name: Some(name.to_string()),
        uses: None,
        run: Some(run.to_string()),
        env: vec![],
        with_inputs: None,
    }
}

fn create_test_job(id: &str, needs: Vec<String>) -> Job {
    Job {
        id: id.to_string(),
        name: Some(format!("Job {}", id)),
        depends_on: needs,
        steps: vec![create_test_step("step1", "echo hello")],
        env: vec![],
        container_image: None,
        service_images: vec![],
        timeout_minutes: None,
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
fn test_syntax_large_number_of_jobs() {
    let mut jobs = Vec::new();
    for i in 0..100 {
        jobs.push(create_test_job(&format!("job-{}", i), vec![]));
    }
    let pipeline = make_pipeline(jobs);

    let issues = syntax::audit(&pipeline).unwrap();
    assert!(issues.is_empty());
}

#[test]
fn test_syntax_long_job_id() {
    let long_id = "a".repeat(256);
    let jobs = vec![create_test_job(&long_id, vec![])];
    let pipeline = make_pipeline(jobs);

    let issues = syntax::audit(&pipeline).unwrap();
    assert!(issues.is_empty());
}

#[test]
fn test_syntax_many_dependencies() {
    let mut jobs = Vec::new();
    let mut needs = Vec::new();

    // Create 20 prerequisite jobs
    for i in 0..20 {
        let id = format!("pre-{}", i);
        jobs.push(create_test_job(&id, vec![]));
        needs.push(id);
    }

    // One job depending on all of them
    jobs.push(create_test_job("final-job", needs));

    let pipeline = make_pipeline(jobs);
    let issues = syntax::audit(&pipeline).unwrap();
    assert!(issues.is_empty());
}

#[test]
fn test_syntax_job_id_with_special_chars() {
    let special_ids = vec!["job_123", "job-abc", "JOB.123", "12345"];
    let mut jobs = Vec::new();
    for id in special_ids {
        jobs.push(create_test_job(id, vec![]));
    }

    let pipeline = make_pipeline(jobs);
    let issues = syntax::audit(&pipeline).unwrap();
    assert!(issues.is_empty());
}

#[test]
fn test_syntax_missing_dependency_reported_as_error() {
    let jobs = vec![create_test_job("job-a", vec!["missing-job".to_string()])];
    let pipeline = make_pipeline(jobs);

    let issues = syntax::audit(&pipeline).unwrap();
    let errors: Vec<_> = issues
        .iter()
        .filter(|i| i.severity == Severity::Error)
        .collect();
    assert_eq!(errors.len(), 1);
    assert!(errors[0]
        .message
        .contains("depends on non-existent job 'missing-job'"));
}

#[test]
fn test_syntax_multiple_duplicate_job_ids() {
    let jobs = vec![
        create_test_job("dup", vec![]),
        create_test_job("dup", vec![]),
        create_test_job("other", vec![]),
        create_test_job("other", vec![]),
    ];
    let pipeline = make_pipeline(jobs);

    let issues = syntax::audit(&pipeline).unwrap();
    let duplicates: Vec<_> = issues
        .iter()
        .filter(|i| i.message.contains("Duplicate job ID"))
        .collect();
    // It reports each duplicate once it finds it again
    assert_eq!(duplicates.len(), 2);
}
