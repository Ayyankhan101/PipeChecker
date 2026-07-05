//! Auto-fix module - applies automatic fixes to workflow files
//!
//! Currently supported fixes:
//! - Pin unpinned GitHub Actions to their latest known SHA/version

use std::collections::HashMap;
use std::fs;

/// Known action version mappings (action_name -> pinned reference)
const KNOWN_ACTIONS: &[(&str, &str)] = &[
    ("actions/checkout", "actions/checkout@v4"),
    ("actions/setup-node", "actions/setup-node@v4"),
    ("actions/setup-python", "actions/setup-python@v5"),
    ("actions/upload-artifact", "actions/upload-artifact@v4"),
    ("actions/download-artifact", "actions/download-artifact@v4"),
    ("actions/cache", "actions/cache@v4"),
    (
        "actions/dependency-review-action",
        "actions/dependency-review-action@v4",
    ),
    ("actions/stale", "actions/stale@v9"),
    ("actions/labeler", "actions/labeler@v5"),
    ("actions/setup-java", "actions/setup-java@v4"),
    ("actions/setup-go", "actions/setup-go@v5"),
    ("actions/setup-dotnet", "actions/setup-dotnet@v4"),
    ("actions/github-script", "actions/github-script@v7"),
    ("actions/delete-artifact", "actions/delete-artifact@v2"),
    (
        "actions/upload-pages-artifact",
        "actions/upload-pages-artifact@v3",
    ),
    (
        "actions/download-pages-artifact",
        "actions/download-pages-artifact@v3",
    ),
    (
        "actions/configure-aws-credentials",
        "actions/configure-aws-credentials@v4",
    ),
    (
        "aws-actions/configure-credentials",
        "aws-actions/configure-credentials@v4",
    ),
    ("azure/login", "azure/login@v2"),
    (
        "google-github-actions/auth",
        "google-github-actions/auth@v2",
    ),
    (
        "google-github-actions/setup-gcloud",
        "google-github-actions/setup-gcloud@v2",
    ),
    ("hashicorp/setup-terraform", "hashicorp/setup-terraform@v3"),
    (
        "slackapi/slack-github-action",
        "slackapi/slack-github-action@v2",
    ),
    (
        "peter-evans/create-pull-request",
        "peter-evans/create-pull-request@v7",
    ),
    (
        "peter-evans/commit-comment",
        "peter-evans/commit-comment@v1",
    ),
    ("EndBug/add-and-commit", "EndBug/add-and-commit@v9"),
    (
        "peaceiris/actions-gh-pages",
        "peaceiris/actions-gh-pages@v4",
    ),
    (
        "JamesIves/github-pages-deploy-action",
        "JamesIves/github-pages-deploy-action@v4",
    ),
    ("ruby/setup-ruby", "ruby/setup-ruby@v1"),
    ("erlef/setup-beam", "erlef/setup-beam@v1"),
    ("dtolnay/rust-toolchain", "dtolnay/rust-toolchain@stable"),
    ("arduino/setup-arduino-cli", "arduino/setup-arduino-cli@v2"),
    (
        "cloudposse/github-action-setup-tf",
        "cloudposse/github-action-setup-tf@v1",
    ),
    (
        "aws-actions/amazon-ecr-login",
        "aws-actions/amazon-ecr-login@v2",
    ),
    ("docker/setup-qemu-action", "docker/setup-qemu-action@v3"),
    ("docker/metadata-action", "docker/metadata-action@v5"),
    (
        "docker/setup-buildx-action",
        "docker/setup-buildx-action@v3",
    ),
    ("docker/build-push-action", "docker/build-push-action@v6"),
    ("docker/login-action", "docker/login-action@v3"),
    ("github/codeql-action/init", "github/codeql-action/init@v3"),
    (
        "github/codeql-action/analyze",
        "github/codeql-action/analyze@v3",
    ),
    (
        "softprops/action-gh-release",
        "softprops/action-gh-release@v2",
    ),
];

/// Result of a fix operation
#[derive(Debug)]
pub struct FixResult {
    pub fixed: usize,
    pub changes: Vec<String>,
}

/// Known Docker image mappings (image:latest -> pinned reference)
const KNOWN_DOCKER_IMAGES: &[(&str, &str)] = &[
    ("node:latest", "node:20-alpine"),
    ("python:latest", "python:3.12-slim"),
    ("ruby:latest", "ruby:3.3-slim"),
    ("nginx:latest", "nginx:1.25-alpine"),
    ("postgres:latest", "postgres:16-alpine"),
    ("redis:latest", "redis:7-alpine"),
    ("mysql:latest", "mysql:8.0"),
    ("ubuntu:latest", "ubuntu:22.04"),
    ("alpine:latest", "alpine:3.19"),
    ("golang:latest", "golang:1.22-alpine"),
    ("rust:latest", "rust:1.75-slim"),
    ("maven:latest", "maven:3.9-eclipse-temurin-21"),
    ("gradle:latest", "gradle:8.6-jdk21"),
];

/// Attempt to auto-fix a workflow file
///
/// When `dry_run` is true, reports what would change without writing to disk.
/// Returns the number of fixes applied and a description of each change.
pub fn fix_file(path: &str, dry_run: bool) -> std::io::Result<FixResult> {
    let content = fs::read_to_string(path)?;
    let has_trailing_newline = content.ends_with('\n');
    let result = fix_content(&content);

    if !dry_run && result.fixed > 0 {
        // Exclude warning messages from being written back to the file.
        let mut cleaned: String = result
            .changes
            .iter()
            .filter(|line| !line.trim_start().starts_with("⚠️"))
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");

        if has_trailing_newline && !cleaned.ends_with('\n') {
            cleaned.push('\n');
        }

        fs::write(path, cleaned)?;
    }

    Ok(result)
}

/// Attempt to auto-fix workflow content
pub fn fix_content(content: &str) -> FixResult {
    let mut changes = Vec::new();
    let mut fixed = 0;

    // Build a lookup map for known actions
    let action_map: HashMap<&str, &str> = KNOWN_ACTIONS.iter().cloned().collect();
    // Build a lookup map for known Docker images
    let docker_map: HashMap<&str, &str> = KNOWN_DOCKER_IMAGES.iter().cloned().collect();

    for line in content.lines() {
        let trimmed = line.trim_start();
        let indent = line.len() - line.trim_start().len();

        // Check for unpinned action references
        let uses_stripped = trimmed.strip_prefix("- ").unwrap_or(trimmed);
        if uses_stripped.starts_with("uses:") {
            let uses_value = uses_stripped
                .trim_start_matches("uses:")
                .trim()
                .trim_matches('"')
                .trim_matches('\'');

            if !uses_value.contains('@')
                && !uses_value.contains(':')
                && !uses_value.starts_with("./")
            {
                if let Some(pinned) = action_map.get(uses_value) {
                    let new_line = format!("{}uses: {}", " ".repeat(indent), pinned);
                    changes.push(new_line);
                    fixed += 1;
                    continue;
                } else {
                    changes.push(format!(
                        "  ⚠️  Unknown action (no auto-fix available): {}",
                        uses_value
                    ));
                    continue;
                }
            }
        }

        // Check for Docker :latest tags in image: or container: lines
        if trimmed.starts_with("image:")
            || trimmed.starts_with("- image:")
            || trimmed.starts_with("container:")
        {
            // For container: the image is usually on the next line, skip here
            if !trimmed.starts_with("container:") {
                let image_val = trimmed
                    .trim_start_matches("image:")
                    .trim_start_matches("- image:")
                    .trim();
                if let Some(pinned) = docker_map.get(image_val) {
                    let prefix = if trimmed.starts_with("- image:") {
                        "- image:"
                    } else {
                        "image:"
                    };
                    let new_line = format!("{}{} {}", " ".repeat(indent), prefix, pinned);
                    changes.push(new_line);
                    fixed += 1;
                    continue;
                }
            }
        }

        changes.push(line.to_string());
    }

    FixResult { fixed, changes }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_unpinned_action() {
        let input = r#"name: CI
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout
      - uses: actions/setup-node
"#;
        let result = fix_content(input);
        assert_eq!(result.fixed, 2);
        assert!(result
            .changes
            .iter()
            .any(|c| c.contains("actions/checkout@v4")));
        assert!(result
            .changes
            .iter()
            .any(|c| c.contains("actions/setup-node@v4")));
    }

    #[test]
    fn test_skip_already_pinned() {
        let input = r#"      - uses: actions/checkout@v4
"#;
        let result = fix_content(input);
        assert_eq!(result.fixed, 0);
    }

    #[test]
    fn test_skip_local_actions() {
        let input = r#"      - uses: ./scripts/my-action
"#;
        let result = fix_content(input);
        assert_eq!(result.fixed, 0);
    }

    #[test]
    fn test_fix_docker_latest() {
        let input = r#"name: CI
on: push
jobs:
  build:
    container:
      image: node:latest
    services:
      db:
        image: postgres:latest
"#;
        let result = fix_content(input);
        assert_eq!(result.fixed, 2);
        assert!(result.changes.iter().any(|c| c.contains("node:20-alpine")));
        assert!(result
            .changes
            .iter()
            .any(|c| c.contains("postgres:16-alpine")));
    }

    #[test]
    fn test_skip_already_pinned_docker() {
        let input = r#"    container:
      image: node:20-alpine
"#;
        let result = fix_content(input);
        assert_eq!(result.fixed, 0);
    }
}
