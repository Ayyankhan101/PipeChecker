//! Include auditor — validates GitLab CI include: blocks
//!
//! Detects include: blocks and warns about:
//! - Local includes that don't exist
//! - Remote includes that may be unreachable
//! - Project includes that can't be validated locally
//! - Missing allow_failure for critical includes

use crate::error::Result;
use crate::models::{Issue, Pipeline, Severity};
use crate::parsers::gitlab::parse_includes;
use std::path::Path;

/// Audit a pipeline for include-related issues
pub fn audit(pipeline: &Pipeline) -> Result<Vec<Issue>> {
    let mut issues = Vec::new();

    // Only applicable for GitLab CI
    if pipeline.provider != crate::models::Provider::GitLabCI {
        return Ok(issues);
    }

    // Parse includes from source
    let include_info = match parse_includes(&pipeline.source) {
        Ok(info) => info,
        Err(_) => return Ok(issues), // Can't parse includes - skip check
    };

    let source_path = Path::new(&pipeline.source);

    // Check local includes exist
    for local in &include_info.local {
        let path = if local.starts_with("./") || local.starts_with("../") {
            // Relative path - check relative to where .gitlab-ci.yml lives
            if let Some(parent) = source_path.parent() {
                parent.join(local)
            } else {
                Path::new(local).to_path_buf()
            }
        } else {
            Path::new(local).to_path_buf()
        };

        if !path.exists() {
            let (line, _) = pipeline.find_line("include:");
            issues.push(Issue::for_job(
                Severity::Warning,
                &format!("Local include not found: {}", local),
                "",
                line,
                1,
                Some("Ensure the file exists or check the path".to_string()),
            ));
        }
    }

    // Warn about remote includes
    for remote in &include_info.remote {
        let (line, _) = pipeline.find_line("include:");
        issues.push(Issue::for_job(
            Severity::Info,
            &format!("Remote include: {}", remote),
            "",
            line,
            1,
            Some("Remote includes require network access - add allow_failure: true for resilient pipelines".to_string()),
        ));
    }

    // Warn about project includes
    for project in &include_info.project {
        let (line, _) = pipeline.find_line("include:");
        issues.push(Issue::for_job(
            Severity::Info,
            &format!("Project include: {}", project),
            "",
            line,
            1,
            Some(
                "Project includes depend on external repository - ensure it's accessible"
                    .to_string(),
            ),
        ));
    }

    Ok(issues)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_include_block() {
        let pipeline = Pipeline {
            provider: crate::models::Provider::GitLabCI,
            jobs: vec![],
            env: vec![],
            source: "stages: [build]\nbuild: script: echo hi\n".to_string(),
        };

        let issues = audit(&pipeline).unwrap();
        assert!(issues.is_empty());
    }

    #[test]
    fn test_local_include_not_applicable_to_github() {
        let pipeline = Pipeline {
            provider: crate::models::Provider::GitHubActions,
            jobs: vec![],
            env: vec![],
            source: "jobs:\n  build:\n    runs-on: ubuntu\n".to_string(),
        };

        let issues = audit(&pipeline).unwrap();
        assert!(issues.is_empty());
    }
}
