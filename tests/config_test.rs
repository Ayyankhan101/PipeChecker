use pipechecker::config::{load, Rules};
use std::fs;
use std::path::Path;

#[test]
fn test_config_file_loading_and_patterns() {
    // --- Part 1: load from file and rule toggles ---
    let config_path = ".pipecheckerrc.yml";
    let backup_path = ".pipecheckerrc.yml.bak";

    if Path::new(config_path).exists() {
        fs::rename(config_path, backup_path).unwrap();
    }

    let config_content = r#"
ignore:
  - "skip-me.yml"
rules:
  circular_dependencies: false
  missing_secrets: true
  docker_latest_tag: false
  timeout_validation: false
"#;

    fs::write(config_path, config_content).unwrap();
    let config = load();

    // Cleanup immediately to reduce collision window
    fs::remove_file(config_path).unwrap();
    if Path::new(backup_path).exists() {
        fs::rename(backup_path, config_path).unwrap();
    }

    assert!(config.should_ignore("skip-me.yml"));
    assert!(!config.rules.circular_dependencies);
    assert!(config.rules.missing_secrets);
    assert!(!config.rules.docker_latest_tag);
    assert!(!config.rules.timeout_validation);

    // --- Part 2: glob pattern matching ---
    // Use a different config filename to avoid conflict with Part 1
    let config_path2 = ".pipecheckerrc.yaml";
    let backup_path2 = ".pipecheckerrc.yaml.bak";

    if Path::new(config_path2).exists() {
        fs::rename(config_path2, backup_path2).unwrap();
    }

    let config_content2 = r#"
ignore:
  - "test*.yml"
  - "old.yml"
  - "deploy.yml"
  - ".github/workflows/old/**"
"#;
    fs::write(config_path2, config_content2).unwrap();
    let config2 = load();

    // Cleanup
    fs::remove_file(config_path2).unwrap();
    if Path::new(backup_path2).exists() {
        fs::rename(backup_path2, config_path2).unwrap();
    }

    assert!(config2.should_ignore("test.yml"));
    assert!(config2.should_ignore("old.yml"));
    assert!(config2.should_ignore("deploy.yml"));
    assert!(!config2.should_ignore("main.yml"));
}

#[test]
fn test_default_rules() {
    let rules = Rules::default();

    assert!(rules.circular_dependencies);
    assert!(rules.missing_secrets);
    assert!(rules.docker_latest_tag);
    assert!(rules.timeout_validation);
}

#[test]
fn test_rules_from_yaml() {
    let yaml = r#"
circular_dependencies: false
missing_secrets: false
docker_latest_tag: true
timeout_validation: true
"#;

    let rules: Rules = serde_yaml::from_str(yaml).unwrap();

    assert!(!rules.circular_dependencies);
    assert!(!rules.missing_secrets);
    assert!(rules.docker_latest_tag);
    assert!(rules.timeout_validation);
}

#[test]
fn test_json_config_support() {
    let config_content = r#"{
  "ignore": ["*.yml"],
  "rules": {
    "circular_dependencies": false,
    "missing_secrets": true
  }
}"#;

    let config: pipechecker::config::Config = serde_json::from_str(config_content).unwrap();

    assert!(config.should_ignore("test.yml"));
    assert!(!config.rules.circular_dependencies);
}
