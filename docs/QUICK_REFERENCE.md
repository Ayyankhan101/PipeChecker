# Pipechecker Quick Reference

## Installation

```bash
# Via Cargo
cargo install pipechecker

# Via npm
npm install -g pipechecker
```

## Basic Commands

| Command | Description |
|---------|-------------|
| `pipechecker` | Auto-detect and check workflow |
| `pipechecker <file>` | Check specific file |
| `pipechecker --all` | Check all workflows |
| `pipechecker --tui` | Interactive terminal UI |
| `pipechecker --version` | Show version |
| `pipechecker --help` | Show help |

## Interactive & Dev Features

| Command | Description |
|---------|-------------|
| `pipechecker --install-hook` | Install pre-commit hook |
| `pipechecker --watch` | Watch for file changes |
| `pipechecker --tui` | Interactive TUI mode |

## Output Options

| Command | Description |
|---------|-------------|
| `pipechecker --format json` | JSON output |
| `pipechecker --strict` | Warnings as errors |
| `pipechecker --ci` | CI mode (--quiet --strict --format json) |
| `pipechecker --quiet` | Suppress warnings/info |
| `pipechecker --verbose` | Show diagnostic info |
| `pipechecker --no-pinning` | Skip Docker/action pin checks |
| `pipechecker --no-permissions` | Skip permissions checks |
| `pipechecker --no-schema` | Skip schema validation |
| `pipechecker --explain PC005` | Explain a rule code |

## Diff & Init

| Command | Description |
|---------|-------------|
| `pipechecker --diff` | Check files changed since base branch |
| `pipechecker --diff --diff-branch main` | Compare against main |
| `pipechecker --init --template rust` | Scaffold a workflow template |
| `pipechecker --fix` | Auto-fix pinning issues |

## TUI Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `up` / `k` | Move up |
| `down` / `j` | Move down |
| `Enter` / `Space` | Toggle details |
| `q` / `Esc` | Quit |

## Configuration File

Create `.pipecheckerrc.yml`:

```yaml
ignore:
  - .github/workflows/old-*.yml

rules:
  circular_dependencies: true
  missing_secrets: true
  docker_latest_tag: true
  timeout_validation: true
  permissions_check: true
  schema_validation: true
  concurrency_validation: true
  artifacts_check: true
```

## Common Workflows

### Quick Check
```bash
pipechecker
```

### Check All Before Commit
```bash
pipechecker --all --strict
```

### Interactive Exploration
```bash
pipechecker --tui
```

### Development with Auto-reload
```bash
pipechecker --watch
```

### CI Integration
```bash
pipechecker --ci
```

### Check Only Changed Files
```bash
pipechecker --diff
```

## Rule Codes

| Code | Auditor | Description |
|------|---------|-------------|
| PC001 | DAG | Circular dependency detected |
| PC002 | Syntax | Empty pipeline (no jobs) |
| PC003 | Syntax | Duplicate job ID |
| PC004 | Syntax | Job with no steps |
| PC005 | Syntax | Missing dependency target |
| PC006 | Secrets | Hardcoded secret in env |
| PC007 | Secrets | Undeclared env var reference |
| PC008 | Secrets | Suspicious key-value pair |
| PC009 | Pinning | Unpinned action |
| PC010 | Pinning | Docker image using :latest |
| PC011 | Timeout | Job missing timeout-minutes |
| PC012 | Schema | Missing required top-level key |
| PC013 | Schema | Invalid job structure |
| PC014 | Permissions | Missing permissions block |
| PC015 | Schema | Unknown top-level key |
| PC016 | Concurrency | Missing cancel-in-progress |
| PC017 | Artifacts | Missing artifact retention config |
| PC018 | Include | Uses GitLab include blocks |

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | No errors |
| `1` | Errors found (or warnings in strict mode) |

## Supported Platforms

- GitHub Actions (`.github/workflows/*.yml`)
- GitLab CI (`.gitlab-ci.yml`)
- CircleCI (`.circleci/config.yml`)

## Tips

1. **Install pre-commit hook** for automatic validation
   ```bash
   pipechecker --install-hook
   ```

2. **Use watch mode** during development
   ```bash
   pipechecker --watch
   ```

3. **Use TUI** for exploring multiple workflows
   ```bash
   pipechecker --tui
   ```

4. **Use strict mode** in CI
   ```bash
   pipechecker --ci
   ```

5. **Check only changed files** against main
   ```bash
   pipechecker --diff --diff-branch main
   ```

## Getting Help

- Documentation: See `docs/`
- Issues: https://github.com/Ayyankhan101/PipeCheck/issues
- Help: `pipechecker --help`

---

**Pipechecker v0.3.0 — Catch CI/CD errors before you push!**
