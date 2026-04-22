//! GitLab CI parser - converts GitLab CI YAML to common Pipeline model
//!
//! Parses GitLab CI configuration files into the common Pipeline representation.
//! Supports:
//! - `stages` definitions
//! - Job definitions with `script`, `before_script`, `after_script`
//! - Dependency chains via `needs` or `dependencies` keywords
//! - Environment variables at global and job levels
//! - Docker image references via `image` keyword
//! - Service definitions

use crate::error::Result;
use crate::models::{EnvVar, Job, Pipeline, Provider, Step};
use serde_yaml::Value;

/// Parse GitLab CI configuration YAML content
///
/// Converts GitLab CI syntax to the common Pipeline model.
pub fn parse(content: &str) -> Result<Pipeline> {
    let yaml: Value = serde_yaml::from_str(content)?;
    let mapping = yaml.as_mapping().ok_or_else(|| {
        crate::error::PipecheckError::InvalidPipeline("Expected YAML mapping".to_string())
    })?;

    // Parse global env vars
    let mut env = Vec::new();
    if let Some(env_val) = mapping.get("variables") {
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

    // Parse global image
    let global_image = mapping.get("image").and_then(|v| {
        if let Value::String(s) = v {
            Some(s.clone())
        } else if let Value::Mapping(m) = v {
            if let Some(Value::String(s)) = m.get(Value::String("name".to_string())) {
                Some(s.clone())
            } else {
                None
            }
        } else {
            None
        }
    });

    // Parse jobs (everything under top-level keys that aren't reserved keywords)
    let reserved = vec![
        "stages",
        "variables",
        "image",
        "before_script",
        "after_script",
        "cache",
        "services",
        "include",
        "default",
    ];

    let mut jobs = Vec::new();

    for (key, value) in mapping {
        let key_str = match key.as_str() {
            Some(s) => s,
            None => continue,
        };

        // Skip reserved top-level keys and stage names
        if reserved.contains(&key_str) || key_str == "workflow" {
            continue;
        }

        // Skip stage names (they're just strings)
        if key_str == "stages" {
            continue;
        }

        // Each other top-level key is a job
        if let Some(job_map) = value.as_mapping() {
            let mut job = parse_job(key_str, job_map)?;
            // If job doesn't have an image, use global default
            if job.container_image.is_none() {
                job.container_image = global_image.clone();
            }
            jobs.push(job);
        }
    }

    Ok(Pipeline {
        provider: Provider::GitLabCI,
        jobs,
        env,
        source: content.to_string(),
    })
}

fn parse_job(id: &str, map: &serde_yaml::Mapping) -> Result<Job> {
    let mut steps = Vec::new();
    let mut depends_on = Vec::new();
    let mut env = Vec::new();
    let mut container_image: Option<String> = None;
    let mut service_images: Vec<String> = Vec::new();
    let mut timeout_minutes: Option<u64> = None;

    // Parse image
    if let Some(image_val) = map.get("image") {
        if let Value::String(s) = image_val {
            container_image = Some(s.clone());
        } else if let Value::Mapping(m) = image_val {
            if let Some(Value::String(s)) = m.get(Value::String("name".to_string())) {
                container_image = Some(s.clone());
            }
        }
    }

    // Parse timeout
    if let Some(timeout_val) = map.get("timeout") {
        if let Some(n) = timeout_val.as_u64() {
            timeout_minutes = Some(n);
        }
    }

    // Parse services
    if let Some(Value::Sequence(services)) = map.get("services") {
        for svc in services {
            if let Value::String(s) = svc {
                service_images.push(s.clone());
            } else if let Value::Mapping(m) = svc {
                if let Some(Value::String(s)) = m.get(Value::String("name".to_string())) {
                    service_images.push(s.clone());
                }
            }
        }
    }

    // Parse job-level variables
    if let Some(vars_val) = map.get("variables") {
        if let Some(vars_map) = vars_val.as_mapping() {
            for (k, v) in vars_map {
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

    // Parse dependencies / needs
    if let Some(needs_val) = map.get("needs") {
        if let Value::Sequence(seq) = needs_val {
            for item in seq {
                match item {
                    Value::String(s) => depends_on.push(s.clone()),
                    Value::Mapping(m) => {
                        if let Some(Value::String(s)) = m.get(Value::String("job".to_string())) {
                            depends_on.push(s.clone());
                        }
                    }
                    _ => {}
                }
            }
        }
    } else if let Value::Sequence(seq) = map.get("dependencies").unwrap_or(&Value::Null) {
        for item in seq {
            if let Value::String(s) = item {
                depends_on.push(s.clone());
            }
        }
    }

    // Parse script steps
    // GitLab CI can have script, before_script, after_script
    for script_key in &["before_script", "script", "after_script"] {
        if let Some(script_val) = map.get(*script_key) {
            let run = match script_val {
                Value::String(s) => s.clone(),
                Value::Sequence(seq) => seq
                    .iter()
                    .map(|v| match v {
                        Value::String(s) => s.clone(),
                        other => format!("{:?}", other),
                    })
                    .collect::<Vec<_>>()
                    .join("\n"),
                other => format!("{:?}", other),
            };

            steps.push(Step {
                name: Some(script_key.to_string()),
                uses: None,
                run: Some(run),
                env: Vec::new(),
                with_inputs: None,
            });
        }
    }

    // Parse trigger (for multi-project pipelines)
    if let Some(trigger_val) = map.get("trigger") {
        let trigger_str = match trigger_val {
            Value::String(s) => s.clone(),
            Value::Mapping(m) => {
                if let Some(Value::String(s)) = m.get(Value::String("project".to_string())) {
                    format!("project: {}", s)
                } else {
                    format!("{:?}", trigger_val)
                }
            }
            other => format!("{:?}", other),
        };
        steps.push(Step {
            name: Some("trigger".to_string()),
            uses: None,
            run: Some(format!("trigger: {}", trigger_str)),
            env: Vec::new(),
            with_inputs: None,
        });
    }

    Ok(Job {
        id: id.to_string(),
        name: None,
        depends_on,
        steps,
        env,
        container_image,
        service_images,
        timeout_minutes,
    })
}
