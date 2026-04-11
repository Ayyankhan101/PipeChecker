//! Dependency graph auditor - detects circular dependencies and missing job references
//!
//! This module analyzes job dependencies using a directed graph:
//! - Builds a dependency graph from job `needs` relationships
//! - Uses Tarjan's SCC algorithm to detect cycles
//! - Reports missing dependencies (references to non-existent jobs)

use crate::error::Result;
use crate::models::{Issue, Pipeline, Severity};
use petgraph::algo::tarjan_scc;
use petgraph::graph::DiGraph;
use std::collections::HashMap;

/// Audit a pipeline for dependency-related issues
///
/// Checks for:
/// - Circular dependencies between jobs
/// - References to non-existent jobs in `needs` clauses
pub fn audit(pipeline: &Pipeline) -> Result<Vec<Issue>> {
    let mut issues = Vec::new();

    // Build dependency graph
    let mut graph = DiGraph::new();
    let mut job_indices = HashMap::new();

    // Add nodes
    for job in &pipeline.jobs {
        let idx = graph.add_node(job.id.clone());
        job_indices.insert(job.id.clone(), idx);
    }

    // Add edges
    for job in &pipeline.jobs {
        let from_idx = job_indices[&job.id];
        for dep in &job.depends_on {
            if let Some(&to_idx) = job_indices.get(dep) {
                graph.add_edge(from_idx, to_idx, ());
            } else {
                let (line, col) = pipeline.find_job_line(&job.id, "needs");
                issues.push(Issue::for_job(
                    Severity::Error,
                    &format!("Job '{}' depends on non-existent job '{}'", job.id, dep),
                    &job.id,
                    line,
                    col,
                    Some(format!("Remove dependency or add job '{}'", dep)),
                ));
            }
        }
    }

    // Detect cycles using Tarjan's algorithm
    let sccs = tarjan_scc(&graph);
    for scc in sccs {
        if scc.len() > 1 {
            let job_names: Vec<_> = scc.iter().map(|&idx| graph[idx].clone()).collect();
            // Find line of first job in the cycle for location
            let first_job = &job_names[0];
            let (line, col) = pipeline.find_job_line(first_job, "runs-on");
            issues.push(Issue::for_job(
                Severity::Error,
                &format!("Circular dependency detected: {}", job_names.join(" -> ")),
                first_job,
                line,
                col,
                Some("Remove one of the dependencies to break the cycle".to_string()),
            ));
        }
    }

    Ok(issues)
}
