# Getting Started with Pipechecker

Pipechecker validates your CI/CD pipeline configuration **before** you push — catching circular dependencies, missing secrets, unpinned actions, and other issues early.

## Quick Start

```bash
# Auto-detect and check your workflow
pipechecker

# Check all workflows
pipechecker --all

# Detailed output
pipechecker --verbose
```

## Installation

```bash
# Via Cargo (recommended)
cargo install pipechecker

# Via npm
npm install -g pipechecker
```

## What It Checks

| Auditor | Rules | Description |
|---------|-------|-------------|
| Syntax | PC002-PC005 | Empty pipelines, duplicate jobs, missing deps |
| DAG | PC001 | Circular job dependencies |
| Secrets | PC006-PC008 | Hardcoded secrets in env vars |
| Pinning | PC009-PC010 | Unpinned actions, Docker :latest |
| Timeout | PC011 | Missing job timeout-minutes |
| Permissions | PC014 | Missing permissions block (GitHub) |
| Schema | PC012-PC013, PC015 | Missing keys, invalid structure |
| Concurrency | PC016 | Missing cancel-in-progress |
| Artifacts | PC017 | Missing artifact retention config |

## Examples

```bash
# Auto-detect and audit
pipechecker

# Check specific file
pipechecker .github/workflows/ci.yml

# Check all workflows with strict rules
pipechecker --all --strict

# Check only files changed since main
pipechecker --diff --diff-branch main

# Interactive terminal UI
pipechecker --tui

# Auto-fix unpinned actions
pipechecker --fix

# CI integration
pipechecker --ci

# Explain a rule
pipechecker --explain PC005

# Watch for changes
pipechecker --watch
```

## Configuration

Create `.pipecheckerrc.yml` in your project root to customize checks:

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

## Pre-commit Hook

```bash
pipechecker --install-hook
```

Runs `pipechecker --all --strict` before every commit.

## Supported Platforms

- GitHub Actions (`.github/workflows/*.yml`)
- GitLab CI (`.gitlab-ci.yml`)
- CircleCI (`.circleci/config.yml`)

---

Need details? See `docs/COMPLETE_GUIDE.md` or `pipechecker --help`.
