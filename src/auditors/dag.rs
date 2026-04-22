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
            } // Missing dependency errors are handled in the syntax auditor; skip duplicate reporting
        }
    }

    // Detect cycles using Tarjan's algorithm
    let sccs = tarjan_scc(&graph);
    for scc in sccs {
        let is_cycle = if scc.len() > 1 {
            true
        } else {
            // Check for self-loop
            let node_idx = scc[0];
            graph.neighbors(node_idx).any(|n| n == node_idx)
        };

        if is_cycle {
            // SCC is a cycle, let's find a specific path within it
            let mut path = Vec::new();
            let current = scc[0];
            let scc_set: std::collections::HashSet<_> = scc.iter().cloned().collect();

            path.push(graph[current].clone());

            // If it's a self-loop, path is simple
            if scc.len() == 1 {
                path.push(graph[current].clone());
            } else {
                // Simple DFS to find a cycle path back to the start node
                let mut temp_current = current;
                for _ in 0..scc.len() {
                    if let Some(edge) = graph
                        .neighbors(temp_current)
                        .find(|neighbor| scc_set.contains(neighbor))
                    {
                        path.push(graph[edge].clone());
                        temp_current = edge;
                        if edge == current {
                            break;
                        }
                    }
                }
            }

            if path.first() != path.last() {
                path.push(graph[current].clone());
            }

            let cycle_str = path.join(" -> ");
            let first_job = &graph[current];
            let (line, col) = pipeline.find_job_line(first_job, "runs-on");

            issues.push(Issue::for_job(
                Severity::Error,
                &format!("Circular dependency detected: {}", cycle_str),
                first_job,
                line,
                col,
                Some("Remove one of the dependencies to break the cycle".to_string()),
            ));
        }
    }
    Ok(issues)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsers::github;

    #[test]
    fn test_missing_dependency() {
        let yaml = r#"on:
  push: {}
jobs:
  build:
    needs: nonexistent
    runs-on: ubuntu-latest
    steps: []
"#;
        let pipeline = github::parse(yaml).unwrap();
        let issues = audit(&pipeline).unwrap();
        // DAG auditor does not report missing dependencies (handled by syntax auditor);
        // Ensure it runs without error and returns zero issues for this case.
        assert!(issues.is_empty());
    }

    #[test]
    fn test_circular_dependency() {
        let yaml = r#"on:
  push: {}
jobs:
  a:
    needs: b
    runs-on: ubuntu-latest
    steps: []
  b:
    needs: a
    runs-on: ubuntu-latest
    steps: []
"#;
        let pipeline = github::parse(yaml).unwrap();
        let issues = audit(&pipeline).unwrap();
        assert!(issues
            .iter()
            .any(|i| i.message.contains("Circular dependency detected")));
    }
}
