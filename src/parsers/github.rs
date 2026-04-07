use crate::error::Result;
use crate::models::{EnvVar, Job, Pipeline, Provider, Step};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
struct GitHubWorkflow {
    name: Option<String>,
    on: serde_yaml::Value,
    jobs: HashMap<String, GitHubJob>,
    env: Option<HashMap<String, serde_yaml::Value>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct GitHubJob {
    name: Option<String>,
    #[serde(rename = "runs-on")]
    runs_on: serde_yaml::Value,
    needs: Option<serde_yaml::Value>,
    steps: Vec<GitHubStep>,
    env: Option<HashMap<String, serde_yaml::Value>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct GitHubStep {
    name: Option<String>,
    uses: Option<String>,
    run: Option<String>,
    env: Option<HashMap<String, serde_yaml::Value>>,
}

pub fn parse(content: &str) -> Result<Pipeline> {
    let workflow: GitHubWorkflow = serde_yaml::from_str(content)?;

    let mut jobs = Vec::new();

    for (job_id, job_data) in workflow.jobs {
        let depends_on = match job_data.needs {
            Some(serde_yaml::Value::String(s)) => vec![s],
            Some(serde_yaml::Value::Sequence(seq)) => seq
                .into_iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect(),
            _ => Vec::new(),
        };

        let steps = job_data
            .steps
            .into_iter()
            .map(|s| Step {
                name: s.name,
                uses: s.uses,
                run: s.run,
                env: parse_env_vars(s.env),
            })
            .collect();

        jobs.push(Job {
            id: job_id,
            name: job_data.name,
            depends_on,
            steps,
            env: parse_env_vars(job_data.env),
        });
    }

    Ok(Pipeline {
        provider: Provider::GitHubActions,
        jobs,
        env: parse_env_vars(workflow.env),
    })
}

fn parse_env_vars(env: Option<HashMap<String, serde_yaml::Value>>) -> Vec<EnvVar> {
    env.unwrap_or_default()
        .into_iter()
        .map(|(key, value)| {
            let value_str = match value {
                serde_yaml::Value::String(s) => s,
                _ => format!("{:?}", value),
            };
            let is_secret = value_str.contains("secrets.");
            EnvVar {
                key,
                value: value_str,
                is_secret,
            }
        })
        .collect()
}
