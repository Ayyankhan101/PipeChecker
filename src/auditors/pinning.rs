//! Action pinning and Docker image auditor
//!
//! This module validates action references and Docker image tags:
//! - Warns about actions using `:latest` tags (reproducibility concern)
//! - Warns about actions without version specifications
//! - Warns about Docker images using `:latest` tag in `container:` blocks
//! - Warns about Docker images using `:latest` tag in `services:` blocks

use crate::error::Result;
use crate::models::{Issue, Pipeline, Severity};

/// Audit a pipeline for action pinning and Docker image issues
///
/// Checks for:
/// - Actions using :latest tag (reproducibility concern)
/// - Actions without version pinning
/// - Docker images in `container:` using :latest
/// - Docker images in `services:` using :latest
pub fn audit(pipeline: &Pipeline) -> Result<Vec<Issue>> {
    let mut issues = Vec::new();

    for job in &pipeline.jobs {
        // Check job-level container image
        if let Some(image) = &job.container_image {
            let (line, col) = pipeline.find_job_line(&job.id, "container");
            if image.ends_with(":latest") {
                issues.push(Issue::for_job(
                    Severity::Warning,
                    &format!(
                        "Job '{}' uses :latest Docker image in container: {}",
                        job.id, image
                    ),
                    &job.id,
                    line,
                    col,
                    Some("Pin to a specific image tag for reproducible builds".to_string()),
                ));
            } else if !image.contains(':') {
                issues.push(Issue::for_job(
                    Severity::Warning,
                    &format!(
                        "Job '{}' uses Docker image without explicit tag in container: {}",
                        job.id, image
                    ),
                    &job.id,
                    line,
                    col,
                    Some("Specify an explicit image tag (e.g. node:18-alpine)".to_string()),
                ));
            }
        }

        // Check service images
        for image in &job.service_images {
            let (line, col) = pipeline.find_job_line(&job.id, "services");
            if image.ends_with(":latest") {
                issues.push(Issue::for_job(
                    Severity::Warning,
                    &format!(
                        "Job '{}' uses :latest Docker image in services: {}",
                        job.id, image
                    ),
                    &job.id,
                    line,
                    col,
                    Some("Pin to a specific image tag for reproducible builds".to_string()),
                ));
            } else if !image.contains(':') {
                issues.push(Issue::for_job(
                    Severity::Warning,
                    &format!(
                        "Job '{}' uses Docker service image without explicit tag: {}",
                        job.id, image
                    ),
                    &job.id,
                    line,
                    col,
                    Some("Specify an explicit image tag".to_string()),
                ));
            }
        }

        // Check step actions
        for step in &job.steps {
            if let Some(uses) = &step.uses {
                let (line, col) = pipeline.find_job_line(&job.id, "uses");
                // Check for :latest tag
                if uses.contains(":latest") {
                    issues.push(Issue::for_job(
                        Severity::Warning,
                        &format!("Job '{}' uses :latest tag: {}", job.id, uses),
                        &job.id,
                        line,
                        col,
                        Some("Pin to a specific version for reproducible builds".to_string()),
                    ));
                }

                // Check for missing version in action references
                if !uses.contains('@') && !uses.contains(':') {
                    issues.push(Issue::for_job(
                        Severity::Warning,
                        &format!("Job '{}' uses action without version: {}", job.id, uses),
                        &job.id,
                        line,
                        col,
                        Some("Specify a version with @v1 or @commit-sha".to_string()),
                    ));
                }
            }
        }
    }

    Ok(issues)
}
