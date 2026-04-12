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
    (
        "docker/setup-buildx-action",
        "docker/setup-buildx-action@v3",
    ),
    ("docker/login-action", "docker/login-action@v3"),
    ("docker/build-push-action", "docker/build-push-action@v6"),
    ("github/codeql-action/init", "github/codeql-action/init@v3"),
    (
        "github/codeql-action/analyze",
        "github/codeql-action/analyze@v3",
    ),
    (
        "peaceiris/actions-gh-pages",
        "peaceiris/actions-gh-pages@v4",
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

/// Attempt to auto-fix a workflow file
///
/// Returns the number of fixes applied and a description of each change.
pub fn fix_file(path: &str) -> std::io::Result<FixResult> {
    let content = fs::read_to_string(path)?;
    let result = fix_content(&content);

    if result.fixed > 0 {
        // Exclude warning messages from being written back to the file.
        let cleaned: String = result
            .changes
            .iter()
            .filter(|line| !line.trim_start().starts_with("⚠️"))
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(path, cleaned)?;
    }

    Ok(result)
}

/// Attempt to auto-fix workflow content
fn fix_content(content: &str) -> FixResult {
    let mut changes = Vec::new();
    let mut fixed = 0;

    // Build a lookup map for known actions
    let action_map: HashMap<&str, &str> = KNOWN_ACTIONS.iter().cloned().collect();

    for line in content.lines() {
        let trimmed = line.trim_start();
        // Check for unpinned action references (handles both `- uses:` and `uses:` forms)
        let uses_stripped = trimmed.strip_prefix("- ").unwrap_or(trimmed);
        if uses_stripped.starts_with("uses:") {
            let uses_value = uses_stripped
                .trim_start_matches("uses:")
                .trim()
                .trim_matches('"')
                .trim_matches('\'');

            // Check if this is a GitHub Action without version pinning
            if !uses_value.contains('@')
                && !uses_value.contains(':')
                && !uses_value.starts_with("./")
            {
                if let Some(pinned) = action_map.get(uses_value) {
                    // Replace the line with the pinned version
                    let indent = line.len() - line.trim_start().len();
                    let new_line = format!("{}uses: {}", " ".repeat(indent), pinned);
                    changes.push(new_line);
                    fixed += 1;
                } else {
                    changes.push(format!(
                        "  ⚠️  Unknown action (no auto-fix available): {}",
                        uses_value
                    ));
                }
            } else {
                changes.push(line.to_string());
            }
        } else {
            changes.push(line.to_string());
        }
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
}
