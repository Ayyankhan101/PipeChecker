use crate::error::Result;
use crate::models::{Issue, Pipeline, Severity};
use petgraph::graph::DiGraph;
use petgraph::algo::tarjan_scc;
use std::collections::HashMap;

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
                issues.push(Issue {
                    severity: Severity::Error,
                    message: format!("Job '{}' depends on non-existent job '{}'", job.id, dep),
                    location: None,
                    suggestion: Some(format!("Remove dependency or add job '{}'", dep)),
                });
            }
        }
    }
    
    // Detect cycles using Tarjan's algorithm
    let sccs = tarjan_scc(&graph);
    for scc in sccs {
        if scc.len() > 1 {
            let job_names: Vec<_> = scc.iter()
                .map(|&idx| graph[idx].clone())
                .collect();
            issues.push(Issue {
                severity: Severity::Error,
                message: format!("Circular dependency detected: {}", job_names.join(" -> ")),
                location: None,
                suggestion: Some("Remove one of the dependencies to break the cycle".to_string()),
            });
        }
    }
    
    Ok(issues)
}
