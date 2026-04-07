pub mod github;

use crate::error::{PipecheckError, Result};
use crate::models::{Pipeline, Provider};

/// Detect CI/CD provider from YAML content
pub fn detect_provider(content: &str) -> Result<Provider> {
    if content.contains("on:") && (content.contains("jobs:") || content.contains("runs-on:")) {
        Ok(Provider::GitHubActions)
    } else if content.contains("stages:") && content.contains("script:") {
        Ok(Provider::GitLabCI)
    } else if content.contains("version:") && content.contains("workflows:") {
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
        Provider::GitLabCI => todo!("GitLab CI parser not yet implemented"),
        Provider::CircleCI => todo!("CircleCI parser not yet implemented"),
    }
}
