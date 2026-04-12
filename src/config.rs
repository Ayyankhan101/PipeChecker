//! Configuration file support for pipechecker
//!
//! Supports `.pipecheckerrc.yml`, `.pipecheckerrc.yaml`, or `.pipechecker.yml`
//! configuration files for specifying ignore patterns and rule settings.
//!
//! # Example
//! ```no_run
//! use pipechecker::config::load;
//!
//! let config = load();
//! assert!(config.should_ignore(".github/workflows/old-test.yml"));
//! ```

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Pipechecker configuration loaded from a `.pipecheckerrc.yml` file.
///
/// Supports glob patterns for ignoring files and toggling individual rules.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub ignore: Vec<String>,

    #[serde(default)]
    pub rules: Rules,
}

/// Rule configuration toggling individual audit checks on or off.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rules {
    #[serde(default = "default_true")]
    pub circular_dependencies: bool,

    #[serde(default = "default_true")]
    pub missing_secrets: bool,

    #[serde(default = "default_true")]
    pub docker_latest_tag: bool,
}

fn default_true() -> bool {
    true
}

#[allow(clippy::derivable_impls)]
impl Default for Rules {
    fn default() -> Self {
        Rules {
            circular_dependencies: true,
            missing_secrets: true,
            docker_latest_tag: true,
        }
    }
}

/// Load configuration from file or return defaults
///
/// Searches for `.pipecheckerrc.yml`, `.pipecheckerrc.yaml`, or `.pipechecker.yml`
/// in the current directory. Returns default config if none found.
pub fn load() -> Config {
    let paths = [
        ".pipecheckerrc.yml",
        ".pipecheckerrc.yaml",
        ".pipechecker.yml",
    ];

    for path in &paths {
        if Path::new(path).exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(config) = serde_yaml::from_str(&content) {
                    return config;
                }
            }
        }
    }

    Config::default()
}

impl Config {
    /// Check if a file should be ignored based on configured patterns
    pub fn should_ignore(&self, file: &str) -> bool {
        self.ignore.iter().any(|pattern| {
            if pattern.contains('*') {
                let re = pattern.replace("*", ".*");
                regex::Regex::new(&re)
                    .map(|r| r.is_match(file))
                    .unwrap_or(false)
            } else {
                file.contains(pattern)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- should_ignore tests ---

    #[test]
    fn test_should_ignore_empty_config() {
        let config = Config::default();
        assert!(!config.should_ignore(".github/workflows/ci.yml"));
        assert!(!config.should_ignore("any/path.yml"));
    }

    #[test]
    fn test_should_ignore_exact_match() {
        let config = Config {
            ignore: vec![".github/workflows/old.yml".to_string()],
            rules: Rules::default(),
        };
        assert!(config.should_ignore(".github/workflows/old.yml"));
        assert!(!config.should_ignore(".github/workflows/new.yml"));
    }

    #[test]
    fn test_should_ignore_contains_match() {
        let config = Config {
            ignore: vec!["old".to_string()],
            rules: Rules::default(),
        };
        assert!(config.should_ignore(".github/workflows/old-test.yml"));
        assert!(config.should_ignore("old-deploy.yml"));
        assert!(!config.should_ignore(".github/workflows/new.yml"));
    }

    #[test]
    fn test_should_ignore_wildcard_github_style() {
        let config = Config {
            ignore: vec!["*.yml".to_string()],
            rules: Rules::default(),
        };
        assert!(config.should_ignore("ci.yml"));
        assert!(config.should_ignore(".github/workflows/deploy.yml"));
        assert!(!config.should_ignore("script.sh"));
    }

    #[test]
    fn test_should_ignore_wildcard_path_pattern() {
        let config = Config {
            ignore: vec![".*test.*".to_string()],
            rules: Rules::default(),
        };
        assert!(config.should_ignore(".github/workflows/test-ci.yml"));
        assert!(config.should_ignore("my-test-file.yml"));
        assert!(!config.should_ignore(".github/workflows/ci.yml"));
    }

    #[test]
    fn test_should_ignore_multiple_patterns() {
        let config = Config {
            ignore: vec![
                "old".to_string(),
                "*.bak.yml".to_string(),
                "temp".to_string(),
            ],
            rules: Rules::default(),
        };
        assert!(config.should_ignore("old-pipeline.yml"));
        assert!(config.should_ignore("backup.bak.yml"));
        assert!(config.should_ignore("temp-deploy.yml"));
        assert!(!config.should_ignore("production.yml"));
    }

    #[test]
    fn test_should_ignore_wildcard_complex_path() {
        let config = Config {
            ignore: vec![".github/workflows/*draft*".to_string()],
            rules: Rules::default(),
        };
        assert!(config.should_ignore(".github/workflows/draft-pr.yml"));
        assert!(config.should_ignore(".github/workflows/my-draft-pipeline.yml"));
        assert!(!config.should_ignore(".github/workflows/ci.yml"));
    }

    #[test]
    fn test_should_ignore_invalid_regex() {
        // Invalid regex pattern should return false (not panic)
        let config = Config {
            ignore: vec!["[invalid".to_string()],
            rules: Rules::default(),
        };
        assert!(!config.should_ignore("any-file.yml"));
    }

    // --- Rules defaults ---

    #[test]
    fn test_rules_default_values() {
        let rules = Rules::default();
        assert!(rules.circular_dependencies);
        assert!(rules.missing_secrets);
        assert!(rules.docker_latest_tag);
    }

    // --- Config default ---

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.ignore.is_empty());
        assert!(config.rules.circular_dependencies);
        assert!(config.rules.missing_secrets);
        assert!(config.rules.docker_latest_tag);
    }
}
