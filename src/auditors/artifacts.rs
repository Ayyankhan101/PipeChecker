use crate::error::Result;
use crate::models::{rule_codes, Issue, Pipeline, Provider, Severity};

/// Audit a pipeline for artifact and cache best practices.
pub fn audit(pipeline: &Pipeline) -> Result<Vec<Issue>> {
    let mut issues = Vec::new();

    if pipeline.provider != Provider::GitHubActions {
        return Ok(issues);
    }

    for job in &pipeline.jobs {
        for step in &job.steps {
            if let Some(uses) = &step.uses {
                // Check actions/cache
                if uses.starts_with("actions/cache") {
                    if let Some(with) = &step.with_inputs {
                        if let Some(key) = with.get("key") {
                            if key.as_str().is_none_or(|s| !s.contains("hashFiles")) {
                                let (line, col) = pipeline.find_job_line(&job.id, "uses:");
                                issues.push(Issue::for_job_with_code(
                                    Severity::Warning,
                                    "Cache action uses a static key without 'hashFiles'",
                                    &job.id,
                                    line,
                                    col,
                                    Some("Include '${{ hashFiles(...) }}' in your cache key to automatically invalidate the cache when files change.".to_string()),
                                    rule_codes::CACHE_STATIC_KEY,
                                ));
                            }
                        }
                    }
                }

                // Check actions/upload-artifact
                if uses.starts_with("actions/upload-artifact") {
                    let has_retention = step
                        .with_inputs
                        .as_ref()
                        .and_then(|w| w.as_mapping())
                        .is_some_and(|m| m.contains_key("retention-days"));

                    if !has_retention {
                        let (line, col) = pipeline.find_job_line(&job.id, "uses:");
                        issues.push(Issue::for_job_with_code(
                            Severity::Info,
                            "Upload-artifact action is missing 'retention-days'",
                            &job.id,
                            line,
                            col,
                            Some("Add 'retention-days: 7' (or appropriate value) to avoid retaining artifacts for the default 90 days and consuming storage.".to_string()),
                            rule_codes::ARTIFACT_NO_RETENTION,
                        ));
                    }
                }
            }
        }
    }

    Ok(issues)
}
