use pipechecker::config::load;
use std::fs;
use std::path::Path;

#[test]
fn test_config_file_loading() {
    let config_path = ".pipecheckerrc.yml";
    let backup_path = ".pipecheckerrc.yml.bak";

    // Backup existing config if it exists
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

    // Cleanup
    fs::remove_file(config_path).unwrap();
    if Path::new(backup_path).exists() {
        fs::rename(backup_path, config_path).unwrap();
    }

    // Assertions
    assert!(config.should_ignore("skip-me.yml"));
    assert!(!config.rules.circular_dependencies);
    assert!(config.rules.missing_secrets);
    assert!(!config.rules.docker_latest_tag);
    assert!(!config.rules.timeout_validation);
}
