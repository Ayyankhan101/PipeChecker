//! Extended tests for DAG auditor — multi-node cycles, self-loops, complex graphs

use pipechecker::auditors::dag;
use pipechecker::models::{Job, Pipeline, Provider};

#[test]
fn test_self_loop() {
    // Job depends on itself → cycle
    let pipeline = Pipeline {
        provider: Provider::GitHubActions,
        jobs: vec![Job {
            id: "A".to_string(),
            name: None,
            depends_on: vec!["A".to_string()], // self依赖
            steps: vec![],
            env: vec![],
            container_image: None,
            service_images: vec![],
            timeout_minutes: None,
        }],
        env: vec![],
        source: "jobs:\n  A:\n    needs: [A]".to_string(),
    };

    let issues = dag::audit(&pipeline).unwrap();
    assert_eq!(issues.len(), 1);
    assert!(issues[0].message.contains("Circular"));
    assert_eq!(
        issues[0].location.as_ref().unwrap().job,
        Some("A".to_string())
    );
}

#[test]
fn test_three_node_cycle() {
    // A → B → C → A
    let pipeline = Pipeline {
        provider: Provider::GitHubActions,
        jobs: vec![
            Job {
                id: "A".to_string(),
                name: None,
                depends_on: vec!["B".to_string()],
                steps: vec![],
                env: vec![],
                container_image: None,
                service_images: vec![],
                timeout_minutes: None,
            },
            Job {
                id: "B".to_string(),
                name: None,
                depends_on: vec!["C".to_string()],
                steps: vec![],
                env: vec![],
                container_image: None,
                service_images: vec![],
                timeout_minutes: None,
            },
            Job {
                id: "C".to_string(),
                name: None,
                depends_on: vec!["A".to_string()],
                steps: vec![],
                env: vec![],
                container_image: None,
                service_images: vec![],
                timeout_minutes: None,
            },
        ],
        env: vec![],
        source: "".to_string(),
    };

    let issues = dag::audit(&pipeline).unwrap();
    // cycle detected, at least one issue
    assert!(!issues.is_empty());
    // The cycle should be reported involving one of the jobs
    let jobs_in_issues: Vec<_> = issues
        .iter()
        .filter_map(|i| i.location.as_ref()?.job.as_ref())
        .collect();
    assert!(jobs_in_issues
        .iter()
        .any(|j| j == &&"A".to_string() || j == &&"B".to_string() || j == &&"C".to_string()));
}

#[test]
fn test_four_node_cycle() {
    // A → B → C → D → A
    let pipeline = Pipeline {
        provider: Provider::GitHubActions,
        jobs: vec![
            Job {
                id: "A".to_string(),
                depends_on: vec!["B".to_string()],
                /* other fields blank */ ..Default::default()
            },
            Job {
                id: "B".to_string(),
                depends_on: vec!["C".to_string()],
                ..Default::default()
            },
            Job {
                id: "C".to_string(),
                depends_on: vec!["D".to_string()],
                ..Default::default()
            },
            Job {
                id: "D".to_string(),
                depends_on: vec!["A".to_string()],
                ..Default::default()
            },
        ],
        env: vec![],
        source: "".to_string(),
    };

    let issues = dag::audit(&pipeline).unwrap();
    assert!(!issues.is_empty());
    // Ensure all four jobs are part of the reported cycle? Not necessarily, but at least some are reported.
    // We can check that there are 4 jobs in the cycle? The auditor might report one issue per cycle or per node.
    // Current implementation: pushes an issue per node in a cycle? Let's check dag::audit logic later. For now ensure non-empty.
    assert!(issues.len() >= 1);
}

#[test]
fn test_multiple_independent_cycles() {
    // Two independent cycles: A→A (self-loop) and B→C→B
    let pipeline = Pipeline {
        provider: Provider::GitHubActions,
        jobs: vec![
            Job {
                id: "A".to_string(),
                depends_on: vec!["A".to_string()],
                ..Default::default()
            },
            Job {
                id: "B".to_string(),
                depends_on: vec!["C".to_string()],
                ..Default::default()
            },
            Job {
                id: "C".to_string(),
                depends_on: vec!["B".to_string()],
                ..Default::default()
            },
        ],
        env: vec![],
        source: "".to_string(),
    };

    let issues = dag::audit(&pipeline).unwrap();
    // Should detect at least 2 issues (two cycles). The algorithm might flag each node in a cycle; but at least 2.
    assert!(issues.len() >= 2);
    let messages: Vec<_> = issues.iter().map(|i| i.message.clone()).collect();
    // Both cycles should be reported
    assert!(messages.iter().any(|m| m.contains("A")));
    assert!(messages.iter().any(|m| m.contains("B") || m.contains("C")));
}

#[test]
fn test_large_dag_no_cycles() {
    // 20 jobs, linear chain A→B→C→... no cycles
    let mut jobs = Vec::new();
    let mut prev: Option<String> = None;
    for i in 0..20 {
        let id = format!("job{}", i);
        let depends_on = if let Some(p) = prev.take() {
            vec![p]
        } else {
            vec![]
        };
        prev = Some(id.clone());
        jobs.push(Job {
            id,
            name: None,
            depends_on,
            steps: vec![],
            env: vec![],
            container_image: None,
            service_images: vec![],
            timeout_minutes: None,
        });
    }
    let pipeline = Pipeline {
        provider: Provider::GitHubActions,
        jobs,
        env: vec![],
        source: "".to_string(),
    };

    let issues = dag::audit(&pipeline).unwrap();
    assert!(
        issues.is_empty(),
        "Large DAG with no cycles should produce no issues"
    );
}

#[test]
fn test_indirect_dependency_cycle() {
    // A → B → C → A (indirect)
    let pipeline = Pipeline {
        provider: Provider::GitHubActions,
        jobs: vec![
            Job {
                id: "A".to_string(),
                depends_on: vec!["B".to_string()],
                ..Default::default()
            },
            Job {
                id: "B".to_string(),
                depends_on: vec!["C".to_string()],
                ..Default::default()
            },
            Job {
                id: "C".to_string(),
                depends_on: vec!["A".to_string()],
                ..Default::default()
            },
        ],
        env: vec![],
        source: "".to_string(),
    };

    let issues = dag::audit(&pipeline).unwrap();
    assert!(!issues.is_empty());
}

#[test]
fn test_cycle_location_points_to_job_in_cycle() {
    // Ensure that the reported location's job is one of the jobs in the cycle.
    let pipeline = Pipeline {
        provider: Provider::GitHubActions,
        jobs: vec![
            Job {
                id: "A".to_string(),
                depends_on: vec!["B".to_string()],
                ..Default::default()
            },
            Job {
                id: "B".to_string(),
                depends_on: vec!["A".to_string()],
                ..Default::default()
            },
        ],
        env: vec![],
        source: "jobs:\n  A:\n    needs: [B]\n  B:\n    needs: [A]".to_string(),
    };

    let issues = dag::audit(&pipeline).unwrap();
    assert!(!issues.is_empty());
    let loc_job = issues[0].location.as_ref().unwrap().job.as_ref().unwrap();
    assert!(loc_job == "A" || loc_job == "B");
}

#[test]
fn test_no_cycle_with_diamond_dependency() {
    // Diamond shape: A → B, A → C, B → D, C → D — no cycle
    let pipeline = Pipeline {
        provider: Provider::GitHubActions,
        jobs: vec![
            Job {
                id: "A".to_string(),
                depends_on: vec![],
                ..Default::default()
            },
            Job {
                id: "B".to_string(),
                depends_on: vec!["A".to_string()],
                ..Default::default()
            },
            Job {
                id: "C".to_string(),
                depends_on: vec!["A".to_string()],
                ..Default::default()
            },
            Job {
                id: "D".to_string(),
                depends_on: vec!["B".to_string(), "C".to_string()],
                ..Default::default()
            },
        ],
        env: vec![],
        source: "".to_string(),
    };

    let issues = dag::audit(&pipeline).unwrap();
    assert!(
        issues.is_empty(),
        "Diamond dependency DAG should have no cycles"
    );
}
