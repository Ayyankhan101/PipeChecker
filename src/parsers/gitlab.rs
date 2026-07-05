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
//! - `include` block parsing (local + remote detection)
//! - `extends:` support with shallow merge semantics
//! - Hidden job filtering (keys starting with `.`)
//! - `rules:` block parsing per job
//! - Top-level `workflow:rules:` parsing

use crate::error::Result;
use crate::models::{EnvVar, Job, Pipeline, Provider, RuleCondition, Step};
use serde_yaml::Value;

/// Information about included files from `include:` blocks
#[derive(Debug, Clone, Default)]
pub struct IncludeInfo {
    /// Local file paths included
    pub local: Vec<String>,
    /// Remote URLs included
    pub remote: Vec<String>,
    /// Project-based includes (project::path)
    pub project: Vec<String>,
}

/// Resolve `extends:` — merge parent job fields into child job (task 2.1)
///
/// Handles both `extends: .base` (single string) and `extends: [.base, .deploy]` (list).
/// Performs shallow merge per GitLab semantics: child fields override parent fields.
fn resolve_extends(mapping: &serde_yaml::Mapping) -> serde_yaml::Mapping {
    let mut resolved = serde_yaml::Mapping::new();

    // First pass: clone all entries
    for (k, v) in mapping {
        resolved.insert(k.clone(), v.clone());
    }

    // Second pass: resolve extends for each job
    for (key, value) in mapping.clone() {
        if let Value::Mapping(job_map) = value {
            let mut merged = job_map.clone();
            resolve_extends_for_job(&mut merged, mapping);
            resolved.insert(key, Value::Mapping(merged));
        }
    }

    resolved
}

/// Resolve extends for a single job by merging parent fields
fn resolve_extends_for_job(job_map: &mut serde_yaml::Mapping, all_entries: &serde_yaml::Mapping) {
    let extends_val = match job_map.get(Value::String("extends".to_string())) {
        Some(v) => v.clone(),
        None => return,
    };

    // Remove extends from the child
    job_map.remove(Value::String("extends".to_string()));

    // Get the list of parent names
    let parent_names: Vec<String> = match &extends_val {
        Value::String(s) => vec![s.clone()],
        Value::Sequence(seq) => seq
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect(),
        _ => return,
    };

    // Merge each parent (later parents override earlier ones, child overrides all)
    for parent_name in &parent_names {
        let parent_val = match all_entries.get(Value::String(parent_name.clone())) {
            Some(Value::Mapping(m)) => m.clone(),
            _ => continue,
        };

        // Insert parent fields only if child doesn't already have them
        for (pk, pv) in parent_val {
            if !job_map.contains_key(&pk) {
                job_map.insert(pk, pv);
            }
        }
    }
}

/// Parse top-level `workflow:rules:` block (task 2.4)
fn parse_workflow_rules(mapping: &serde_yaml::Mapping) -> Vec<RuleCondition> {
    let workflow_val = match mapping.get(Value::String("workflow".to_string())) {
        Some(v) => v,
        None => return Vec::new(),
    };

    let workflow_map = match workflow_val.as_mapping() {
        Some(m) => m,
        None => return Vec::new(),
    };

    let rules_val = match workflow_map.get(Value::String("rules".to_string())) {
        Some(Value::Sequence(seq)) => seq,
        _ => return Vec::new(),
    };

    rules_val.iter().filter_map(parse_rule_condition).collect()
}

/// Parse a `rules:` block for a job (task 2.3)
fn parse_rules(mapping: &serde_yaml::Mapping) -> Vec<RuleCondition> {
    let rules_val = match mapping.get(Value::String("rules".to_string())) {
        Some(Value::Sequence(seq)) => seq,
        _ => return Vec::new(),
    };

    rules_val.iter().filter_map(parse_rule_condition).collect()
}

/// Parse a single rule condition from a YAML mapping
fn parse_rule_condition(rule_val: &Value) -> Option<RuleCondition> {
    let rule_map = rule_val.as_mapping()?;

    let when = rule_map
        .get(Value::String("when".to_string()))
        .and_then(|v| v.as_str())
        .map(String::from);

    let if_condition = rule_map
        .get(Value::String("if".to_string()))
        .and_then(|v| v.as_str())
        .map(String::from);

    let exists = rule_map
        .get(Value::String("exists".to_string()))
        .and_then(|v| {
            if let Value::Sequence(seq) = v {
                let paths: Vec<String> = seq
                    .iter()
                    .filter_map(|p| p.as_str().map(String::from))
                    .collect();
                if paths.is_empty() {
                    None
                } else {
                    Some(paths)
                }
            } else {
                None
            }
        });

    let allow_failure = rule_map
        .get(Value::String("allow_failure".to_string()))
        .and_then(|v| v.as_bool());

    if when.is_some() || if_condition.is_some() || exists.is_some() || allow_failure.is_some() {
        Some(RuleCondition {
            when,
            if_condition,
            exists,
            allow_failure,
        })
    } else {
        None
    }
}

/// Parse GitLab CI configuration YAML content
///
/// Converts GitLab CI syntax to the common Pipeline model.
/// Note: `include:` blocks are parsed for detection but external files are NOT loaded.
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

    // Parse top-level workflow:rules (task 2.4)
    let workflow_rules = parse_workflow_rules(mapping);

    // Resolve extends: merge parent fields into child jobs (task 2.1)
    let resolved = resolve_extends(mapping);

    // Filter out hidden jobs (task 2.2) — keys starting with '.'
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

    for (key, value) in &resolved {
        let key_str = match key.as_str() {
            Some(s) => s,
            None => continue,
        };

        // Skip reserved top-level keys
        if reserved.contains(&key_str) || key_str == "workflow" {
            continue;
        }

        // Skip hidden jobs (task 2.2) — keys starting with '.'
        if key_str.starts_with('.') {
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
        is_reusable: false,
        workflow_call_inputs: Vec::new(),
        workflow_call_secrets: Vec::new(),
        workflow_rules,
    })
}

/// Parse include blocks to extract referenced files (without loading external content)
///
/// Returns IncludeInfo with detected include references.
/// This does NOT resolve/load external files - just extracts paths for auditing.
pub fn parse_includes(content: &str) -> Result<IncludeInfo> {
    let yaml: Value = serde_yaml::from_str(content)?;
    let mapping = yaml.as_mapping().ok_or_else(|| {
        crate::error::PipecheckError::InvalidPipeline("Expected YAML mapping".to_string())
    });

    let mut info = IncludeInfo::default();

    let include_val = match mapping {
        Ok(m) => m.get("include"),
        Err(e) => return Err(e),
    };

    let Some(include_block) = include_val else {
        return Ok(info);
    };

    let Value::Sequence(includes) = include_block else {
        if let Value::String(s) = include_block {
            classify_include(s, &mut info);
        }
        return Ok(info);
    };

    for item in includes {
        match item {
            Value::String(s) => {
                classify_include(s, &mut info);
            }
            Value::Mapping(m) => {
                if let Some(Value::String(s)) = m.get("local") {
                    info.local.push(s.clone());
                }
                if let Some(Value::String(s)) = m.get("remote") {
                    info.remote.push(s.clone());
                }
                if let Some(Value::String(s)) = m.get("project") {
                    if let Some(Value::String(path)) = m.get("file") {
                        info.project.push(format!("{}:{}", s, path));
                    } else {
                        info.project.push(s.clone());
                    }
                }
            }
            _ => {}
        }
    }

    Ok(info)
}

fn classify_include(s: &str, info: &mut IncludeInfo) {
    if s.starts_with("https://") || s.starts_with("http://") {
        info.remote.push(s.to_string());
    } else if s.contains("::") {
        info.project.push(s.to_string());
    } else {
        info.local.push(s.to_string());
    }
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

    // Parse trigger
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

    // Parse rules (task 2.3)
    let rules = parse_rules(map);

    Ok(Job {
        id: id.to_string(),
        name: None,
        depends_on,
        steps,
        env,
        container_image,
        service_images,
        timeout_minutes,
        rules,
    })
}
