//! Matrix strategy auditor for GitHub Actions
//!
//! Checks for:
//! - Large matrix strategies (>9 combinations) without `fail-fast: false` → Warning, PC022
//! - Empty `matrix: include:` entries → Info

use crate::error::Result;
use crate::models::{rule_codes, Issue, Pipeline, Provider, Severity};
use serde_yaml::Value;

/// Audit a pipeline for matrix strategy configurations.
pub fn audit(pipeline: &Pipeline) -> Result<Vec<Issue>> {
    let mut issues = Vec::new();

    // Matrix is a GitHub Actions feature
    if pipeline.provider != Provider::GitHubActions {
        return Ok(issues);
    }

    let doc: Value = match serde_yaml::from_str(&pipeline.source) {
        Ok(d) => d,
        Err(_) => return Ok(issues),
    };

    if let Some(jobs) = doc.get("jobs").and_then(|j| j.as_mapping()) {
        for (job_id_val, job_val) in jobs {
            let job_id = match job_id_val.as_str() {
                Some(s) => s,
                None => continue,
            };

            let job_map = match job_val.as_mapping() {
                Some(m) => m,
                None => continue,
            };

            // Check for strategy.matrix
            if let Some(strategy) = job_map.get("strategy").and_then(|v| v.as_mapping()) {
                if let Some(matrix) = strategy.get("matrix").and_then(|v| v.as_mapping()) {
                    // Count matrix combinations
                    let combination_count = count_matrix_combinations(matrix);

                    // Check for fail-fast: false
                    let has_fail_fast_false =
                        strategy.get("fail-fast").and_then(|v| v.as_bool()) == Some(false);

                    // PC022: Large matrix without fail-fast: false
                    if combination_count > 9 && !has_fail_fast_false {
                        let (line, col) = pipeline.find_job_line(job_id, "matrix");
                        issues.push(Issue::for_job_with_code(
                            Severity::Warning,
                            &format!(
                                "Job '{}' has a matrix with {} combinations but no 'fail-fast: false'",
                                job_id, combination_count
                            ),
                            job_id,
                            line,
                            col,
                            Some(
                                "Add 'fail-fast: false' to the strategy to prevent cancelling other matrix jobs on failure"
                                    .to_string(),
                            ),
                            rule_codes::LARGE_MATRIX_NO_FAILFAST,
                        ));
                    }

                    // Check for empty matrix: include entries
                    if let Some(include) = matrix.get("include").and_then(|v| v.as_sequence()) {
                        if include.is_empty() {
                            let (line, col) = pipeline.find_job_line(job_id, "include");
                            issues.push(Issue::for_job_with_code(
                                Severity::Info,
                                &format!("Job '{}' has an empty 'matrix: include:' list", job_id),
                                job_id,
                                line,
                                col,
                                None,
                                rule_codes::LARGE_MATRIX_NO_FAILFAST,
                            ));
                        }
                    }
                }
            }
        }
    }

    Ok(issues)
}

/// Count the total number of matrix combinations by computing the product of all dimensions.
/// If no dimensions are found (e.g., only `include:`), returns 0.
fn count_matrix_combinations(matrix: &serde_yaml::Mapping) -> usize {
    let mut count: usize = 1;

    for (key, value) in matrix {
        let key_str = match key.as_str() {
            Some(s) => s,
            None => continue,
        };

        // Skip special keys
        if key_str == "include" || key_str == "exclude" {
            continue;
        }

        if let Some(arr) = value.as_sequence() {
            let len = arr.len();
            if len == 0 {
                return 0;
            }
            count = count.saturating_mul(len);
        }
    }

    // If no dimensions were found (count still 1 and no dimension keys exist)
    if count == 1
        && !matrix.keys().any(|k| {
            k.as_str()
                .map(|s| s != "include" && s != "exclude")
                .unwrap_or(false)
        })
    {
        return 0;
    }

    count
}
