use crate::error::Result;
use crate::models::{Issue, Pipeline, Severity};

pub fn audit(pipeline: &Pipeline) -> Result<Vec<Issue>> {
    let mut issues = Vec::new();
    
    for job in &pipeline.jobs {
        for step in &job.steps {
            if let Some(uses) = &step.uses {
                // Check for :latest tag
                if uses.contains(":latest") {
                    issues.push(Issue {
                        severity: Severity::Warning,
                        message: format!("Job '{}' uses :latest tag: {}", job.id, uses),
                        location: None,
                        suggestion: Some("Pin to a specific version for reproducible builds".to_string()),
                    });
                }
                
                // Check for missing version
                if !uses.contains('@') && !uses.contains(':') {
                    issues.push(Issue {
                        severity: Severity::Warning,
                        message: format!("Job '{}' uses action without version: {}", job.id, uses),
                        location: None,
                        suggestion: Some("Specify a version with @v1 or @commit-sha".to_string()),
                    });
                }
            }
        }
    }
    
    Ok(issues)
}
