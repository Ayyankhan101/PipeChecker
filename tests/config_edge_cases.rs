//! Config edge case tests

use pipechecker::config::{Config, Rules};

#[test]
fn test_config_deserialize_empty_ignore() {
    let yaml = "ignore: []\nrules:\n  missing_secrets: false";
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    assert!(config.ignore.is_empty());
    assert!(!config.rules.missing_secrets);
    assert!(config.rules.circular_dependencies); // Default should be true
}

#[test]
fn test_config_deserialize_partial_rules() {
    let yaml = "rules:\n  docker_latest_tag: false";
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    assert!(!config.rules.docker_latest_tag);
    assert!(config.rules.missing_secrets); // Default true
    assert!(config.rules.circular_dependencies); // Default true
}

#[test]
fn test_should_ignore_with_regex_special_chars() {
    let mut config = Config::default();
    config.ignore.push("test(123).yml".to_string());
    
    // contains check
    assert!(config.should_ignore("some/test(123).yml"));
    assert!(!config.should_ignore("test123.yml"));
}

#[test]
fn test_should_ignore_with_leading_wildcard() {
    let mut config = Config::default();
    config.ignore.push("*.bak".to_string());
    
    assert!(config.should_ignore("pipeline.bak"));
    assert!(config.should_ignore("path/to/file.bak"));
    assert!(!config.should_ignore("file.bak.yml"));
}

#[test]
fn test_should_ignore_with_middle_wildcard() {
    let mut config = Config::default();
    config.ignore.push("jobs/*/config.yml".to_string());
    
    // Pattern "jobs/*/config.yml" becomes "jobs/.*/config.yml"
    assert!(config.should_ignore("jobs/build/config.yml"));
    assert!(config.should_ignore("jobs/test/deploy/config.yml"));
    assert!(!config.should_ignore("jobs/config.yml"));
}

#[test]
fn test_should_ignore_no_wildcard_is_literal() {
    let mut config = Config::default();
    config.ignore.push("[a-z].yml".to_string());
    
    assert!(!config.should_ignore("x.yml"));
    assert!(config.should_ignore("path/[a-z].yml"));
}

#[test]
fn test_rules_all_disabled() {
    let rules = Rules {
        circular_dependencies: false,
        missing_secrets: false,
        docker_latest_tag: false,
        timeout_validation: false,
    };
    assert!(!rules.circular_dependencies);
    assert!(!rules.missing_secrets);
    assert!(!rules.docker_latest_tag);
    assert!(!rules.timeout_validation);
}

#[test]
fn test_config_serialization_roundtrip() {
    let mut config = Config::default();
    config.ignore.push("node_modules".to_string());
    config.rules.missing_secrets = false;
    
    let serialized = serde_yaml::to_string(&config).unwrap();
    let deserialized: Config = serde_yaml::from_str(&serialized).unwrap();
    
    assert_eq!(deserialized.ignore, config.ignore);
    assert_eq!(deserialized.rules.missing_secrets, config.rules.missing_secrets);
    assert_eq!(deserialized.rules.circular_dependencies, config.rules.circular_dependencies);
}
