# 📋 Pipecheck Quick Reference

## Installation

```bash
# Via Cargo
cargo install pipecheck

# Via npm (once published)
npm install -g pipecheck
```

## Basic Commands

| Command | Description |
|---------|-------------|
| `pipecheck` | Auto-detect and check workflow |
| `pipecheck file.yml` | Check specific file |
| `pipecheck --all` | Check all workflows |
| `pipecheck --tui` | Interactive terminal UI |
| `pipecheck --version` | Show version |
| `pipecheck --help` | Show help |

## Interactive Features

| Command | Description |
|---------|-------------|
| `pipecheck --install-hook` | Install pre-commit hook |
| `pipecheck --watch` | Watch for file changes |
| `pipecheck --tui` | Interactive TUI mode |

## Output Options

| Command | Description |
|---------|-------------|
| `pipecheck --format json` | JSON output |
| `pipecheck --strict` | Warnings as errors |
| `pipecheck --no-docker` | Skip Docker checks |

## TUI Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `↑` / `k` | Move up |
| `↓` / `j` | Move down |
| `Enter` / `Space` | Toggle details |
| `q` / `Esc` | Quit |

## Configuration File

Create `.pipecheckrc.yml`:

```yaml
ignore:
  - .github/workflows/old-*.yml
  - .github/workflows/experimental/

rules:
  circular_dependencies: true
  missing_secrets: true
  docker_latest_tag: true
```

## Common Workflows

### Quick Check
```bash
pipecheck
```

### Check All Before Commit
```bash
pipecheck --all --strict
```

### Interactive Exploration
```bash
pipecheck --tui
```

### Development with Auto-reload
```bash
pipecheck --watch
```

### CI Integration
```bash
pipecheck --all --format json --strict
```

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | No errors |
| `1` | Errors found |
| `1` | Warnings found (in strict mode) |

## Status Indicators

| Symbol | Meaning |
|--------|---------|
| ✅ | No issues |
| ⚠️ | Warnings |
| ❌ | Errors |
| ℹ️ | Info |

## Supported Platforms

- ✅ GitHub Actions (`.github/workflows/*.yml`)
- ✅ GitLab CI (`.gitlab-ci.yml`)
- ✅ CircleCI (`.circleci/config.yml`)

## Examples

### Example 1: Quick Validation
```bash
$ pipecheck
✓ Auto-detected: .github/workflows/ci.yml
Provider: GitHubActions
0 errors, 0 warnings
```

### Example 2: Multiple Files
```bash
$ pipecheck --all
Checking 3 workflow file(s)...

📄 .github/workflows/ci.yml
   ✅ No issues found

📄 .github/workflows/deploy.yml
   ✅ No issues found

Total: 0 errors, 0 warnings across 3 files
```

### Example 3: Error Detection
```bash
$ pipecheck broken.yml
Provider: GitHubActions

1 errors, 0 warnings

❌ ERROR: Circular dependency detected: job-a -> job-b -> job-c
   💡 Remove one of the dependencies to break the cycle
```

### Example 4: JSON Output
```bash
$ pipecheck --format json
{
  "provider": "GitHubActions",
  "issues": [],
  "summary": "0 errors, 0 warnings"
}
```

## Tips

1. **Install pre-commit hook** for automatic validation
   ```bash
   pipecheck --install-hook
   ```

2. **Use watch mode** during development
   ```bash
   pipecheck --watch
   ```

3. **Use TUI** for exploring multiple workflows
   ```bash
   pipecheck --tui
   ```

4. **Add config file** for team standards
   ```bash
   echo "ignore: [.github/workflows/old-*.yml]" > .pipecheckrc.yml
   ```

5. **Use strict mode** in CI
   ```bash
   pipecheck --all --strict
   ```

## Getting Help

- Documentation: See `README.md`
- TUI Guide: See `TUI_GUIDE.md`
- Issues: https://github.com/Ayyankhan101/PipeCheck/issues
- Help: `pipecheck --help`

---

**Pipecheck v0.2.0 - Catch CI/CD errors before you push! 🚀**
