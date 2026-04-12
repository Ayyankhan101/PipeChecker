//! GitHub Actions workflow parser
//!
//! Parses GitHub Actions workflow YAML files into the common Pipeline representation.
//! Supports:
//! - Job definitions with steps
//! - Dependency chains via `needs`
//! - Environment variables at workflow, job, and step levels
//! - Docker container and service image references
//! - `with:` input blocks for secret scanning

use crate::error::Result;
use crate::models::{EnvVar, Job, Pipeline, Provider, Step};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::collections::HashMap;

/// Represents a GitHub Actions workflow file structure
#[derive(Debug, Deserialize, Serialize)]
struct GitHubWorkflow {
    name: Option<String>,
    #[serde(rename = "on")]
    on: serde_yaml::Value,
    jobs: HashMap<String, serde_yaml::Value>,
    env: Option<HashMap<String, serde_yaml::Value>>,
}

/// Parse GitHub Actions workflow YAML content
///
/// Converts GitHub Actions syntax to the common Pipeline model.
/// Handles both single and array `needs` specifications.
/// Populates `container_image`, `service_images`, and `with_inputs` fields.
pub fn parse(content: &str) -> Result<Pipeline> {
    let workflow: GitHubWorkflow = serde_yaml::from_str(content)?;

    let mut jobs = Vec::new();

    for (job_id, job_val) in workflow.jobs {
        let job_map = job_val
            .as_mapping()
            .cloned()
            .unwrap_or_else(serde_yaml::Mapping::new);

        let job_name = job_map
            .get(Value::String("name".into()))
            .and_then(|v| v.as_str())
            .map(String::from);

        // Parse needs
        let depends_on = match job_map.get(Value::String("needs".into())) {
            Some(Value::String(s)) => vec![s.clone()],
            Some(Value::Sequence(seq)) => seq
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect(),
            _ => Vec::new(),
        };

        // Parse container image
        let container_image = job_map
            .get(Value::String("container".into()))
            .and_then(|v| {
                if let Some(container_map) = v.as_mapping() {
                    container_map
                        .get(Value::String("image".into()))
                        .and_then(|v| v.as_str().map(String::from))
                } else {
                    v.as_str().map(String::from)
                }
            });

        // Parse service images
        let mut service_images = Vec::new();
        if let Some(services) = job_map.get(Value::String("services".into())) {
            if let Some(services_map) = services.as_mapping() {
                for (_key, val) in services_map {
                    if let Some(svc_map) = val.as_mapping() {
                        if let Some(img) = svc_map
                            .get(Value::String("image".into()))
                            .and_then(|v| v.as_str())
                        {
                            service_images.push(img.to_string());
                        }
                    }
                }
            }
        }

        // Parse steps
        let steps = match job_map.get(Value::String("steps".into())) {
            Some(Value::Sequence(steps_seq)) => steps_seq
                .iter()
                .map(|s| {
                    let step_map = s
                        .as_mapping()
                        .cloned()
                        .unwrap_or_else(serde_yaml::Mapping::new);

                    let step_name = step_map
                        .get(Value::String("name".into()))
                        .and_then(|v| v.as_str())
                        .map(String::from);

                    let uses = step_map
                        .get(Value::String("uses".into()))
                        .and_then(|v| v.as_str())
                        .map(String::from);

                    let run = step_map
                        .get(Value::String("run".into()))
                        .and_then(|v| v.as_str())
                        .map(String::from);

                    let with_inputs = step_map.get(Value::String("with".into())).cloned();

                    Step {
                        name: step_name,
                        uses,
                        run,
                        env: parse_env_map(step_map.get(Value::String("env".into()))),
                        with_inputs,
                    }
                })
                .collect(),
            _ => Vec::new(),
        };

        // Parse timeout-minutes
        let timeout_minutes = job_map
            .get(Value::String("timeout-minutes".into()))
            .and_then(|v| v.as_u64());

        let job_env = parse_env_map(job_map.get(Value::String("env".into())));

        jobs.push(Job {
            id: job_id,
            name: job_name,
            depends_on,
            steps,
            env: job_env,
            container_image,
            service_images,
            timeout_minutes,
        });
    }

    Ok(Pipeline {
        provider: Provider::GitHubActions,
        jobs,
        env: parse_env_map(
            workflow
                .env
                .as_ref()
                .map(|m| serde_yaml::to_value(m).unwrap_or(Value::Null))
                .as_ref(),
        ),
        source: content.to_string(),
    })
}

/// Parse environment variables from an optional YAML value
fn parse_env_map(val: Option<&Value>) -> Vec<EnvVar> {
    let map = match val.and_then(|v| v.as_mapping()) {
        Some(m) => m,
        None => return Vec::new(),
    };

    map.iter()
        .filter_map(|(k, v)| {
            let key = k.as_str()?.to_string();
            let value_str = match v {
                Value::String(s) => s.clone(),
                _ => format!("{:?}", v),
            };
            let is_secret = value_str.contains("secrets.");
            Some(EnvVar {
                key,
                value: value_str,
                is_secret,
            })
        })
        .collect()
}
