# Pipechecker User Guide

## Overview

Pipechecker is a CLI tool that validates CI/CD pipeline configurations locally. It catches errors before you push — saving CI minutes and developer frustration.

### Supported Platforms

- **GitHub Actions** — `.github/workflows/*.yml`
- **GitLab CI** — `.gitlab-ci.yml`
- **CircleCI** — `.circleci/config.yml`

## Installation

### From Source (Cargo)

```bash
cargo install pipechecker
```

### From npm

```bash
npm install -g pipechecker
```

## Usage

### Basic

```bash
pipechecker
```

Auto-detects the first workflow file in your project and runs all auditors.

### Check All Workflows

```bash
pipechecker --all
```

Discovers all workflow files and audits every one.

### Check Specific File

```bash
pipechecker .github/workflows/deploy.yml
```

### Diff Mode

Check only files changed since a base branch:

```bash
pipechecker --diff --diff-branch main
```

## Flags

| Flag | Description |
|------|-------------|
| `--all` | Check all discovered workflows |
| `--strict` | Treat warnings as errors |
| `--ci` | CI mode: `--quiet --strict --format json` |
| `--quiet` | Only show errors |
| `--verbose` | Show diagnostic info & auditor list |
| `--format json` | JSON output |
| `--diff` | Only check changed files |
| `--diff-branch <name>` | Base branch for diff (default: main) |
| `--watch` | Watch for changes and re-check |
| `--tui` | Interactive terminal UI |
| `--fix` | Auto-fix unpinned actions / Docker images |
| `--no-pinning` | Skip action pinning checks |
| `--no-permissions` | Skip permissions checks |
| `--no-schema` | Skip schema validation |
| `--explain <CODE>` | Show detailed explanation for a rule |
| `--init --template <name>` | Scaffold a workflow from template |
| `--install-hook` | Install git pre-commit hook |

## Auditors

### 1. Syntax Auditor (PC002-PC005)
Checks for structural issues: empty pipelines, duplicate job IDs, jobs without steps, and missing dependency targets.

### 2. DAG Auditor (PC001)
Uses Tarjan's algorithm to detect circular job dependencies. Reports the exact cycle path with job names.

### 3. Secrets Auditor (PC006-PC008)
Scans environment variables at all levels (workflow, job, step) for hardcoded secrets, suspicious key names, and undeclared `${{ env.* }}` references.

### 4. Pinning Auditor (PC009-PC010)
Warns about GitHub Actions without a version pin (`@v4`, `@v3`, etc.) and Docker images using the `:latest` tag.

### 5. Timeout Auditor (PC011)
Warns when jobs lack `timeout-minutes` (GitHub), `timeout` (GitLab), or `max_time` (CircleCI).

### 6. Permissions Auditor (PC014)
Checks that GitHub Actions jobs have explicit `permissions:` blocks instead of inheriting repo-level defaults.

### 7. Schema Auditor (PC012-PC013, PC015)
Validates YAML structure against provider-specific expectations: required keys, valid job structures, and known keywords.

### 8. Concurrency Auditor (PC016)
Checks that jobs using `concurrency:` groups have `cancel-in-progress: true`.

### 9. Artifacts Auditor (PC017)
Checks for artifact retention configuration in GitHub Actions upload-artifact steps.

## Configuration File

Place `.pipecheckerrc.yml` in your project root:

```yaml
# Files/patterns to ignore
ignore:
  - .github/workflows/old-*.yml
  - templates/

# Rule toggles (all default to true)
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

## Auto-Fix

The `--fix` command pins unpinned GitHub Actions and Docker `:latest` tags to known safe versions:

- `actions/checkout` → `actions/checkout@v4`
- `node:latest` → `node:20-alpine`
- `postgres:latest` → `postgres:16-alpine`
- And 20+ more mappings

## Pre-commit Hook

```bash
pipechecker --install-hook
```

Installs a hook that runs `pipechecker --all --strict` on staged workflow files before every commit.

## Release Process

### Creating a New Release

1. Update version in `Cargo.toml` and `package.json`
2. Update `CHANGELOG.md`
3. Commit and push:
   ```bash
   git add .
   git commit -m "chore: bump to v0.X.Y"
   git tag v0.X.Y
   git push origin main --tags
   ```
4. GitHub Actions builds binaries, creates release, publishes to crates.io
5. If npm publish fails, ensure `NPM_TOKEN` is an Automation token (2FA bypass)

### Required GitHub Secrets

- `NPM_TOKEN` — npm Automation token
- `CARGO_TOKEN` — crates.io API token

## Troubleshooting

**`pipechecker` not found after npm install?**
Ensure the npm global bin directory is in your `$PATH`.

**Build fails?**
```bash
cargo clean
cargo build --release
```

**Permission denied on pre-commit hook?**
The hook is automatically made executable on Unix. On Windows, manually set permissions.

---

## Further Reading

- `QUICK_REFERENCE.md` — Cheat sheet
- `TUI_GUIDE.md` — Terminal UI usage
- `docs/` — Full documentation
- `pipechecker --help` — CLI reference
