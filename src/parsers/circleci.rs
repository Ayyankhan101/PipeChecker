//! CircleCI parser - converts CircleCI config YAML to common Pipeline model
//!
//! Parses CircleCI configuration files into the common Pipeline representation.
//! Supports:
//! - `version` field
//! - `workflows` with job orchestration
//! - `jobs` definitions with `steps`
//! - `executors` for Docker images
//! - Environment variables at workflow, job, and step levels
//! - Docker executor image references

use crate::error::Result;
use crate::models::{EnvVar, Job, Pipeline, Provider, Step};
use serde_yaml::Value;

/// Parse CircleCI configuration YAML content
///
/// Converts CircleCI syntax to the common Pipeline model.
pub fn parse(content: &str) -> Result<Pipeline> {
    let yaml: Value = serde_yaml::from_str(content)?;
    let mapping = yaml.as_mapping().ok_or_else(|| {
        crate::error::PipecheckError::InvalidPipeline("Expected YAML mapping".to_string())
    })?;

    // Parse global env vars (from setup commands or environment key)
    let env = Vec::new();

    // Parse workflows to understand job ordering
    let mut workflow_deps: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    if let Some(workflows_val) = mapping.get("workflows") {
        if let Some(workflows_map) = workflows_val.as_mapping() {
            for (_wf_name, wf_val) in workflows_map {
                if let Some(Value::Sequence(job_list)) = wf_val.get("jobs") {
                    for job_entry in job_list {
                        match job_entry {
                            Value::String(s) => {
                                workflow_deps.entry(s.clone()).or_default();
                            }
                            Value::Mapping(m) => {
                                // Could be job with requires or job name as key
                                if let Some(Value::String(job_name)) = m.keys().next() {
                                    workflow_deps.entry(job_name.clone()).or_default();
                                    if let Some(config_map) = m
                                        .get(Value::String(job_name.clone()))
                                        .and_then(|v| v.as_mapping())
                                    {
                                        if let Some(Value::Sequence(reqs)) =
                                            config_map.get("requires")
                                        {
                                            let deps: Vec<String> = reqs
                                                .iter()
                                                .filter_map(|r| {
                                                    if let Value::String(s) = r {
                                                        Some(s.clone())
                                                    } else {
                                                        None
                                                    }
                                                })
                                                .collect();
                                            workflow_deps.insert(job_name.clone(), deps);
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    // Parse jobs
    let mut jobs = Vec::new();
    if let Some(jobs_val) = mapping.get("jobs") {
        if let Some(jobs_map) = jobs_val.as_mapping() {
            for (key, value) in jobs_map {
                let id = match key.as_str() {
                    Some(s) => s.to_string(),
                    None => continue,
                };

                let job_map = match value.as_mapping() {
                    Some(m) => m,
                    None => continue,
                };

                let job = parse_job(&id, job_map, &workflow_deps)?;
                jobs.push(job);
            }
        }
    }

    Ok(Pipeline {
        provider: Provider::CircleCI,
        jobs,
        env,
        source: content.to_string(),
    })
}

fn parse_job(
    id: &str,
    map: &serde_yaml::Mapping,
    workflow_deps: &std::collections::HashMap<String, Vec<String>>,
) -> Result<Job> {
    let mut steps = Vec::new();
    let mut env = Vec::new();
    let mut container_image: Option<String> = None;

    // Parse executor (Docker image)
    if let Some(executor_val) = map.get("executor") {
        if let Value::Mapping(m) = executor_val {
            if let Some(Value::String(s)) = m.get(Value::String("name".to_string())) {
                container_image = Some(s.clone());
            }
        } else if let Value::String(s) = executor_val {
            // Named executor reference - check top-level executors
            container_image = Some(format!("executor:{}", s));
        }
    }

    // Parse docker executor (inline)
    if let Some(Value::Sequence(docker_list)) = map.get("docker") {
        if let Some(Value::Mapping(m)) = docker_list.first() {
            if let Some(Value::String(s)) = m.get(Value::String("image".to_string())) {
                container_image = Some(s.clone());
            }
        }
    }

    // Parse job-level environment
    if let Some(env_val) = map.get("environment") {
        if let Some(env_map) = env_val.as_mapping() {
            for (k, v) in env_map {
                if let Some(key) = k.as_str() {
                    let value = match v {
                        Value::String(s) => s.clone(),
                        other => format!("{:?}", other),
                    };
                    env.push(EnvVar {
                        key: key.to_string(),
                        value,
                        is_secret: false,
                    });
                }
            }
        }
    }

    // Parse steps
    if let Value::Sequence(step_list) = map.get("steps").unwrap_or(&Value::Null) {
        for (idx, step_val) in step_list.iter().enumerate() {
            match step_val {
                Value::String(s) => {
                    steps.push(Step {
                        name: Some(format!("step-{}", idx)),
                        uses: None,
                        run: Some(s.clone()),
                        env: Vec::new(),
                        with_inputs: None,
                    });
                }
                Value::Mapping(m) => {
                    if let Some(Value::String(step_name)) = m.keys().next() {
                        // Check for special step types
                        if step_name == "run" {
                            let run = match m.get(Value::String(step_name.clone())) {
                                Some(Value::String(s)) => s.clone(),
                                Some(Value::Mapping(run_map)) => {
                                    if let Some(cmd) =
                                        run_map.get(Value::String("command".to_string()))
                                    {
                                        match cmd {
                                            Value::String(s) => s.clone(),
                                            other => format!("{:?}", other),
                                        }
                                    } else {
                                        format!("{:?}", m)
                                    }
                                }
                                Some(other) => format!("{:?}", other),
                                None => format!("{:?}", m),
                            };
                            steps.push(Step {
                                name: Some(format!("step-{}", idx)),
                                uses: None,
                                run: Some(run),
                                env: Vec::new(),
                                with_inputs: None,
                            });
                        } else if step_name == "checkout" {
                            steps.push(Step {
                                name: Some("checkout".to_string()),
                                uses: Some("circleci/checkout".to_string()),
                                run: None,
                                env: Vec::new(),
                                with_inputs: None,
                            });
                        } else if step_name == "save_cache"
                            || step_name == "restore_cache"
                            || step_name == "store_artifacts"
                            || step_name == "store_test_results"
                        {
                            steps.push(Step {
                                name: Some(step_name.clone()),
                                uses: Some(format!("circleci/{}", step_name)),
                                run: None,
                                env: Vec::new(),
                                with_inputs: Some(step_val.clone()),
                            });
                        } else {
                            // Custom orb command or step
                            let step_config = m.get(Value::String(step_name.clone()));
                            steps.push(Step {
                                name: Some(step_name.clone()),
                                uses: Some(step_name.clone()),
                                run: step_config.map(|v| format!("{:?}", v)),
                                env: Vec::new(),
                                with_inputs: step_config.cloned(),
                            });
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Get dependencies from workflow definition
    let depends_on = workflow_deps.get(id).cloned().unwrap_or_default();

    Ok(Job {
        id: id.to_string(),
        name: None,
        depends_on,
        steps,
        env,
        container_image,
        service_images: Vec::new(),
    })
}
