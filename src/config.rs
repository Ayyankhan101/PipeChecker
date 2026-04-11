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
