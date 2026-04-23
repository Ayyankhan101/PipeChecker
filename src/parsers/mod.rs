//! CI/CD provider parsers - converts provider-specific YAML to common Pipeline model
//!
//! # Providers
//! - GitHub Actions (implemented)
//! - GitLab CI (implemented, with include: support)
//! - CircleCI (implemented)

pub mod circleci;
pub mod github;
pub mod gitlab;

use crate::error::{PipecheckError, Result};
use crate::models::{Pipeline, Provider};
use serde_yaml::Value;

/// Detect CI/CD provider from YAML content
pub fn detect_provider(content: &str) -> Result<Provider> {
    let yaml: Value = match serde_yaml::from_str(content) {
        Ok(v) => v,
        Err(_) => {
            return Err(PipecheckError::UnknownProvider(
                "Not a valid YAML file".to_string(),
            ))
        }
    };

    if let Some(map) = yaml.as_mapping() {
        // GitHub Actions: 'on' and ('jobs' or 'runs-on')
        if map.contains_key("on") && (map.contains_key("jobs") || map.contains_key("runs-on")) {
            return Ok(Provider::GitHubActions);
        }

        // GitLab CI
        if (map.contains_key("stages")
            || map.contains_key("image")
            || map.contains_key("before_script"))
            && !map.contains_key("on")
            && !map.contains_key("workflows")
        {
            return Ok(Provider::GitLabCI);
        }

        // CircleCI: 'version' and ('workflows' or 'jobs')
        if map.contains_key("version")
            && (map.contains_key("workflows") || map.contains_key("jobs"))
            && !map.contains_key("on")
        {
            return Ok(Provider::CircleCI);
        }
    }

    // Fallback to naive string matching
    if content.contains("on:") && (content.contains("jobs:") || content.contains("runs-on:")) {
        Ok(Provider::GitHubActions)
    } else if content.contains("stages:") && content.contains("script:") {
        Ok(Provider::GitLabCI)
    } else if content.contains("version:")
        && (content.contains("workflows:") || content.contains("jobs:"))
    {
        Ok(Provider::CircleCI)
    } else {
        Err(PipecheckError::UnknownProvider(
            "Could not detect CI/CD provider".to_string(),
        ))
    }
}

/// Parse pipeline configuration based on provider
pub fn parse(content: &str, provider: Provider) -> Result<Pipeline> {
    match provider {
        Provider::GitHubActions => github::parse(content),
        Provider::GitLabCI => gitlab::parse(content),
        Provider::CircleCI => circleci::parse(content),
    }
}
