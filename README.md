# PipeChecker

> **Catch CI/CD pipeline errors before you push — not after CI fails.**

[![CI](https://github.com/Ayyankhan101/PipeChecker/actions/workflows/ci.yml/badge.svg)](https://github.com/Ayyankhan101/PipeChecker/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/pipechecker.svg)](https://crates.io/crates/pipechecker)
[![License](https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-blue)](LICENSE-MIT)
[![Test Coverage](https://img.shields.io/badge/tests-134%20passing-brightgreen)]()
[![Rust](https://img.shields.io/badge/rust-2021-orange)](Cargo.toml)

---

## What Problem Does This Solve?

Every developer has been here:

```
💀 You push a small change → CI fails 10 minutes later →
   you fix it → push again → CI fails again → repeat 3 more times
```

**PipeChecker runs locally** and validates your CI/CD workflows **before** you commit, so you catch:

| Catches | Example |
|---------|---------|
| ❌ **Circular dependencies** | Job A → Job B → Job A |
| ❌ **Missing job references** | `needs: [build]` but no `build` job exists |
| ❌ **Empty pipelines** | No jobs or steps defined |
| ⚠️ **Hardcoded secrets** | `API_KEY=sk_live_abc123` in env vars |
| ⚠️ **Undeclared env vars** | `${{ env.UNKNOWN }}` never defined |
| ⚠️ **Unpinned actions** | `uses: actions/checkout` without `@v4` |
| ⚠️ **Docker `:latest` tags** | `image: nginx:latest` (unreproducible builds) |
| ⚠️ **Missing job timeouts** | No `timeout-minutes` set — jobs can run forever |

---

## Visual Overview

```
┌─────────────────────────────────────────────────────────┐
│                    YOUR WORKFLOW FILE                    │
│              (.github/workflows/ci.yml)                  │
└────────────────────────┬────────────────────────────────┘
                         │
                         ▼
              ┌──────────────────────┐
              │    PIPECHECKER       │
              │                      │
              │  ┌────────────────┐  │
              │  │  YAML Parser   │  │
              │  │ GitHub/GitLab  │  │
              │  │   CircleCI     │  │
              │  └───────┬────────┘  │
              │          │           │
              │  ┌───────▼────────┐  │
              │  │    Auditors    │  │
              │  │                │  │
              │  │  📋 Syntax     │  │
              │  │  🔗 DAG/Cycle  │  │
              │  │  🔒 Secrets    │  │
              │  │  🐳 Docker     │  │
              │  │  📌 Pinning    │  │
              │  └───────┬────────┘  │
              │          │           │
              └──────────┼───────────┘
                         │
          ┌──────────────┼──────────────┐
          ▼              ▼              ▼
     ✅ PASS       ⚠️ WARNINGS    ❌ ERRORS
   No issues     Fix before       Must fix
   found!        production       before push
```

---

## Documentation

Comprehensive documentation can be found in the [docs/](docs/index.md) folder:
- [Start Here](docs/START_HERE.md)
- [Complete Guide](docs/COMPLETE_GUIDE.md)
- [TUI Guide](docs/TUI_GUIDE.md)
- [Contributing](docs/CONTRIBUTING.md)

---

## Supported Platforms

| Platform | File Pattern | Status |
|----------|-------------|--------|
| **GitHub Actions** | `.github/workflows/*.yml` | ✅ Full support |
| **GitLab CI** | `.gitlab-ci.yml` | ✅ Full support |
| **CircleCI** | `.circleci/config.yml` | ✅ Full support |

---

## Installation

### From crates.io
```bash
cargo install pipechecker
```

### From source
```bash
git clone https://github.com/Ayyankhan101/PipeChecker.git
cd PipeChecker
cargo install --path .
```

### Via npm (once published)
```bash
npm install -g pipechecker
```

---

## GitHub Action

PipeChecker can be used directly in your GitHub workflows:

```yaml
name: CI Pipeline Check
on: [push, pull_request]

jobs:
  pipechecker:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: Ayyankhan101/PipeCheck/actions/pipecheck@v0.2.9
```

### Action Inputs

| Input | Required | Default | Description |
|-------|----------|---------|-------------|
| `path` | No | `.github/workflows` | Path to workflow file(s) |
| `strict` | No | `false` | Exit with error on warnings |
| `diff` | No | `false` | Check only changed files |
| `diff-branch` | No | `main` | Base branch for diff mode |

---

## Diff Mode

Check only workflow files that changed since a base branch:

```bash
# Check files changed since main
pipechecker --diff --diff-branch main

# Short form
pipechecker -d
```

This uses `git diff --name-only` to find changed workflow files, then checks only those. Faster in monorepos with many workflows.

---

## Quick Start

### 1. Check a single file
```bash
pipechecker .github/workflows/ci.yml
```

### 2. Auto-detect your workflow
```bash
pipechecker
# ✓ Auto-detected: .github/workflows/ci.yml
# Provider: GitHubActions
# 0 errors, 0 warnings
```

### 3. Audit everything
```bash
pipechecker --all
# Checking 3 workflow file(s)...
#
# 📄 .github/workflows/ci.yml
#    Provider: GitHubActions
#    ✅ No issues found
#
# 📄 .github/workflows/deploy.yml
#    Provider: GitHubActions
#    1 errors, 2 warnings
#    ❌ ERROR: Circular dependency detected (job: deploy)
#       💡 Remove one of the dependencies to break the cycle
#    ⚠️ WARNING: Job 'deploy' has no steps
#
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Total: 1 errors, 2 warnings across 3 files
```

---

## Interactive TUI

PipeChecker includes a **terminal UI** for browsing results across multiple files:

```bash
pipechecker --tui
```

```
┌──────────────────────────────────────────┐
│  🔍 Pipecheck - Interactive Mode         │
├──────────────────────────────────────────┤
│  Workflows                               │
│▶ ❌ deploy.yml │ 2 errors │ 1 warnings   │
│  ✅ ci.yml     │ 0 errors │ 0 warnings   │
│  ⚠️  lint.yml   │ 0 errors │ 3 warnings   │
├──────────────────────────────────────────┤
│  [↑/↓] Navigate  [Enter] Details  [Q] Quit│
└──────────────────────────────────────────┘
```

**Keyboard shortcuts:**

| Key | Action |
|-----|--------|
| `↑` / `k` | Move up |
| `↓` / `j` | Move down |
| `Enter` / `Space` | Toggle detail view |
| `q` / `Esc` | Quit |

---

## All CLI Flags

| Flag | Description |
|------|-------------|
| `FILE` | Path to a specific workflow file |
| `--all`, `-a` | Audit **all** discovered workflow files |
| `--tui` | Launch the interactive terminal UI |
| `--watch`, `-w` | Watch for file changes and re-run audits |
| `--fix` | Auto-fix issues (pin unpinned actions + Docker `:latest` tags) |
| `--install-hook` | Install a git pre-commit hook |
| `--format`, `-f` `<text\|json>` | Output format (default: `text`) |
| `--strict`, `-s` | Treat warnings as errors (exit code 1) |
| `--quiet`, `-q` | Only output errors — suppress warnings and info. Perfect for CI |
| `--verbose` | Show diagnostic info (auditors ran, per-severity counts, discovered files) |
| `--no-pinning` | Skip Docker image and action-pinning checks |
| `--version` | Show version |
| `--help` | Show help |

---

## Output Explained

### Severity Levels

| Symbol | Level | Meaning |
|--------|-------|---------|
| ❌ | **Error** | Must fix — will break your pipeline |
| ⚠️ | **Warning** | Should fix — may cause issues later |
| ℹ️ | **Info** | Informational — nothing to worry about |

### Example output with details

```
Provider: GitHubActions
2 errors, 1 warnings

❌ ERROR: Circular dependency detected (job: deploy) [line 42]
   💡 Remove one of the dependencies to break the cycle

❌ ERROR: Job 'deploy' depends on non-existent job 'build' (job: deploy) [line 45]
   💡 Add a job with id 'build' or remove the dependency

⚠️ WARNING: Job 'lint' has no steps (job: lint) [line 12]
   💡 Add steps to perform work in this job
```

Each issue includes:
- **What** went wrong (clear message)
- **Where** it happened (job name + line number)
- **How** to fix it (actionable suggestion)

---

## JSON Output

Perfect for CI/CD integration or programmatic consumption:

```bash
pipechecker --format json
```

```json
{
  "provider": "GitHubActions",
  "issues": [
    {
      "severity": "Error",
      "message": "Circular dependency detected: job-a -> job-b -> job-a",
      "location": { "line": 42, "column": 3, "job": "deploy" },
      "suggestion": "Remove one of the dependencies to break the cycle"
    }
  ],
  "summary": "1 errors, 0 warnings"
}
```

---

## Modes of Operation

### 🔧 Auto-Fix Mode
Automatically pins unpinned GitHub Actions to known versions:

```bash
pipechecker --fix
```

```
🔧 Auto-fix mode

✨ Fixed 2 issue(s) in .github/workflows/ci.yml:

  actions/checkout → actions/checkout@v4
  actions/setup-node → actions/setup-node@v4

💡 Review the changes and commit them!
```

### 👀 Watch Mode
Monitors workflow files and re-runs on every save — perfect for development:

```bash
pipechecker --watch
```

```
👀 Watching for workflow changes...
   Press Ctrl+C to stop

🔄 File changed: .github/workflows/ci.yml
Provider: GitHubActions
0 errors, 0 warnings
✅ All checks passed
```

### 🤫 Quiet Mode (CI-Friendly)
Only output errors — suppress warnings and info. Perfect for CI pipelines where you want clean output:

```bash
pipechecker --quiet
# or
pipechecker -q
```

```
❌ Circular dependency detected (job: deploy) (in .github/workflows/deploy.yml)
```

Exit code is still `1` if there are errors — works perfectly with `--strict` for failing CI on any issue.

### 📢 Verbose Mode
See exactly what PipeChecker is doing — which files it found, which auditors ran, and per-severity breakdowns:

```bash
pipechecker --verbose
```

```
📄 Auditing: .github/workflows/ci.yml
🔍 Auditors ran: syntax, dag, secrets, pinning
📊 Found: 0 errors, 1 warnings, 0 info
⏱️  Checked in 3.2ms
```

### ⏱️ Timing Metrics
Every audit now shows how long it took — because speed matters:

```bash
pipechecker .github/workflows/ci.yml
```

```
Provider: GitHubActions
0 errors, 0 warnings
✅ All checks passed
⏱️  Checked in 2.1ms
```

### 🔒 Pre-commit Hook
Never commit a broken workflow again:

```bash
pipechecker --install-hook
```

```
✅ Pre-commit hook installed!
   Pipecheck will run before every commit
   Use 'git commit --no-verify' to skip
```

The hook automatically validates any workflow files you stage:

```bash
$ git commit -m "Update CI pipeline"
🔍 Checking workflows with pipechecker...
❌ ERROR: Circular dependency detected (job: deploy) [line 42]
   💡 Remove one of the dependencies to break the cycle

❌ Workflow validation failed!
Fix errors above or use 'git commit --no-verify' to skip
```

---

## Configuration File

Create a `.pipecheckerrc.yml` in your project root to customize behavior:

```yaml
# Files to skip (glob patterns supported)
ignore:
  - .github/workflows/experimental-*.yml
  - .github/workflows/draft-*.yml
  - old-pipeline.yml

# Toggle individual audit rules
rules:
  circular_dependencies: true   # Detect dependency cycles
  missing_secrets: true         # Flag hardcoded secrets
  docker_latest_tag: true       # Warn about :latest tags
```

PipeChecker searches for config in this order:
1. `.pipecheckerrc.yml`
2. `.pipecheckerrc.yaml`
3. `.pipechecker.yml`

---

## How the Auditors Work

### 📋 Syntax Auditor
Validates the structural integrity of your pipeline:

- ✅ Jobs are defined
- ✅ Steps exist within jobs
- ✅ No duplicate job IDs
- ✅ `needs` / `depends_on` targets exist

### 🔗 DAG Auditor (Cycle Detection)
Builds a **dependency graph** of your jobs and runs **Tarjan's Strongly Connected Components** algorithm:

```
  job-a ──depends──▶ job-b
    ▲                   │
    │                   ▼
    └────depends──── job-c
```
→ ❌ **Circular dependency detected:** job-a → job-b → job-c → job-a

### 🔒 Secrets Auditor
Scans for security issues in environment variables and run blocks:

```yaml
env:
  API_KEY: sk_live_abc123         # ⚠️ Hardcoded secret
  TOKEN: ${{ secrets.TOKEN }}     # ✅ Correct way
  RUN: echo ${{ secrets.API_KEY }} # ℹ️ Info — ensure it's configured
  RUN: echo ${{ env.UNDEFINED }}  # ⚠️ Undeclared env var
```

Detects:
- Hardcoded API keys, passwords, tokens
- Secret references in `with:` blocks
- Undeclared `${{ env.X }}` references
- Suspicious values (long alphanumeric strings, base64)

### 🐳 Docker & 📌 Pinning Auditor
Ensures reproducible builds:

```yaml
uses: actions/checkout              # ⚠️ No version pin
uses: actions/checkout@v4           # ✅ Pinned
image: nginx:latest                 # ⚠️ Unpredictable
image: nginx:1.25-alpine            # ✅ Specific
```

---

## Real-World Examples

### Example 1: Valid workflow
```bash
$ pipechecker .github/workflows/ci.yml
Provider: GitHubActions
0 errors, 0 warnings
```

### Example 2: Circular dependency
```yaml
jobs:
  deploy:
    needs: [test]
    steps: [{ run: echo deploy }]
  test:
    needs: [deploy]
    steps: [{ run: echo test }]
```
```bash
$ pipechecker broken.yml
Provider: GitHubActions
1 errors, 0 warnings

❌ ERROR: Circular dependency detected (job: deploy)
   💡 Remove one of the dependencies to break the cycle
```

### Example 3: Hardcoded secrets
```yaml
jobs:
  build:
    env:
      API_SECRET: sk_live_hardcoded_value
    steps: [{ run: echo building }]
```
```bash
$ pipechecker secrets.yml
Provider: GitHubActions
0 errors, 1 warnings

⚠️ WARNING: Job 'build' env 'API_SECRET' may contain a hardcoded secret
   💡 Use secrets.API_SECRET instead of hardcoding
```

---

## Architecture

```
pipechecker/
├── src/
│   ├── main.rs          # CLI entry point (clap)
│   ├── lib.rs           # Public API — audit_file, audit_content, discover_workflows
│   ├── models.rs        # Core types — Pipeline, Job, Step, Issue, Severity
│   ├── error.rs         # Error enum (thiserror)
│   ├── config.rs        # .pipecheckerrc.yml loading
│   ├── fix.rs           # Auto-fix for action pinning
│   ├── tui.rs           # Interactive terminal UI (ratatui + crossterm)
│   ├── parsers/
│   │   ├── mod.rs       # Provider detection + dispatch
│   │   ├── github.rs    # GitHub Actions YAML parser
│   │   ├── gitlab.rs    # GitLab CI YAML parser
│   │   └── circleci.rs  # CircleCI YAML parser
│   └── auditors/
│       ├── mod.rs       # Module gate
│       ├── syntax.rs    # Structural validation
│       ├── dag.rs       # Dependency graph + cycle detection (petgraph)
│       ├── secrets.rs   # Secret/env var scanning (regex)
│       └── pinning.rs   # Action/Docker image pinning
├── tests/
│   ├── parser_test.rs   # Parser integration tests
│   └── auditors_test.rs # Auditor + fixture tests
└── tests/fixtures/      # Sample workflow files for testing
```

---

## CI/CD Integration

Add PipeChecker to your own CI pipeline:

```yaml
- name: Validate workflows
  run: |
    cargo install pipechecker
    pipechecker --all --strict --format json
```

Or use it as a pre-commit hook (recommended):

```bash
pipechecker --install-hook
```

---

## Development

### Run tests
```bash
cargo test
# 103 tests — all passing
```

### Lint & format
```bash
cargo clippy -- -D warnings
cargo fmt -- --check
```

### Coverage
```bash
cargo tarpaulin --fail-under 55
```

---

## License

This project is licensed under either **MIT** or **Apache-2.0** at your option.

```
SPDX: MIT OR Apache-2.0
```

---

<div align="center">

**PipeChecker** — *because waiting 10 minutes for CI to tell you about a typo is nobody's idea of fun.*

[Report a bug](https://github.com/Ayyankhan101/PipeCheck/issues) · [Request a feature](https://github.com/Ayyankhan101/PipeCheck/issues) · [Contributing](docs/CONTRIBUTING.md)

</div>
