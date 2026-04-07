use crate::error::Result;
use crate::models::{Issue, Pipeline, Severity};
use regex::Regex;
use std::collections::HashSet;

pub fn audit(pipeline: &Pipeline) -> Result<Vec<Issue>> {
    let mut issues = Vec::new();
    
    let secret_pattern = Regex::new(r"\$\{\{\s*secrets\.(\w+)\s*\}\}").unwrap();
    let env_pattern = Regex::new(r"\$\{\{\s*env\.(\w+)\s*\}\}").unwrap();
    
    let mut declared_env: HashSet<String> = pipeline.env.iter()
        .map(|e| e.key.clone())
        .collect();
    
    for job in &pipeline.jobs {
        // Add job-level env vars
        for env_var in &job.env {
            declared_env.insert(env_var.key.clone());
        }
        
        for step in &job.steps {
            // Check run commands for secret references
            if let Some(run) = &step.run {
                for cap in secret_pattern.captures_iter(run) {
                    let secret_name = &cap[1];
                    issues.push(Issue {
                        severity: Severity::Info,
                        message: format!("Job '{}' uses secret: {}", job.id, secret_name),
                        location: None,
                        suggestion: Some("Ensure this secret is configured in repository settings".to_string()),
                    });
                }
                
                for cap in env_pattern.captures_iter(run) {
                    let env_name = &cap[1];
                    if !declared_env.contains(env_name) {
                        issues.push(Issue {
                            severity: Severity::Warning,
                            message: format!("Job '{}' references undeclared env var: {}", job.id, env_name),
                            location: None,
                            suggestion: Some(format!("Declare '{}' in env section", env_name)),
                        });
                    }
                }
            }
        }
    }
    
    Ok(issues)
}
