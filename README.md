# 🔍 Pipechecker

[![CI](https://github.com/Ayyankhan101/PipeCheck/workflows/CI/badge.svg)](https://github.com/Ayyankhan101/PipeCheck/actions)
[![Crates.io](https://img.shields.io/crates/v/pipechecker.svg)](https://crates.io/crates/pipechecker)
[![npm](https://img.shields.io/npm/v/pipechecker.svg)](https://www.npmjs.com/package/pipechecker)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

**A blazingly fast CI/CD pipeline auditor that catches errors before you push.**

Stop wasting time debugging CI failures. Pipechecker validates your GitHub Actions, GitLab CI, and CircleCI configurations locally, catching syntax errors, circular dependencies, and security issues instantly.

## 🚀 Quick Start

### Install via npm (recommended)
```bash
npm install -g pipechecker
```

### Install via Cargo
```bash
cargo install pipechecker
```

### Run
```bash
pipechecker .github/workflows/ci.yml
```

## ✨ Features

- ✅ **Syntax Validation** - Parse and validate GitHub Actions, GitLab CI, and CircleCI configs
- 🔄 **Dependency Analysis** - Detect circular dependencies in job workflows
- 🔐 **Secrets Auditing** - Identify hardcoded secrets and environment variable issues
- 🐳 **Docker Validation** - Check Docker image references and tags
- 📊 **Multiple Output Formats** - Text and JSON output for CI integration
- ⚡ **Fast** - Written in Rust for maximum performance
- 🎯 **Zero Config** - Works out of the box

## 💡 Why Pipechecker?

**Before Pipechecker:**
```
git push
→ Wait 5 minutes
→ CI fails: "Circular dependency detected"
→ Fix locally
→ git push again
→ Wait 5 minutes...
```

**With Pipechecker:**
```
pipechecker .github/workflows/ci.yml
→ ❌ ERROR: Circular dependency detected: job-a -> job-c -> job-b
→ Fix immediately
→ git push with confidence ✅
```

## 📖 Usage

### Quick Start

```bash
# Auto-detect and check workflow
pipechecker

# Check specific file
pipechecker .github/workflows/ci.yml

# Check all workflows
pipechecker --all

# Interactive TUI mode
pipechecker --tui
```

### All Options

```
CI/CD Pipeline Auditor - Catch errors before you push

Usage: pipechecker [OPTIONS] [FILE]

Arguments:
  [FILE]  Path to pipeline configuration file (auto-detects if not provided)

Options:
  -a, --all              Check all workflow files in directory
      --install-hook     Install pre-commit hook
  -w, --watch            Watch for file changes and re-check
      --fix              Automatically fix issues where possible
      --tui              Interactive terminal UI mode
  -f, --format <FORMAT>  Output format (text, json) [default: text]
      --no-pinning         Skip action pinning and Docker image checks
  -s, --strict           Enable strict mode (warnings as errors)
  -h, --help             Print help
  -V, --version          Print version
```

### Interactive Features

```bash
# Install pre-commit hook
pipechecker --install-hook

# Watch mode - auto-recheck on file changes
pipechecker --watch

# Interactive TUI mode
pipechecker --tui

# Auto-fix issues (Coming soon!)
pipechecker --fix
```

### Configuration File

Create `.pipecheckrc.yml` in your project root:

```yaml
# Files/patterns to ignore
ignore:
  - .github/workflows/old-*.yml
  - .github/workflows/experimental/
  - .github/workflows/draft-*.yml

# Rule configuration
rules:
  circular_dependencies: true  # Check for circular job dependencies
  missing_secrets: true         # Warn about secrets usage
  docker_latest_tag: true       # Warn about :latest Docker tags
```

### Pre-commit Hook

Pipechecker can install a Git pre-commit hook that automatically validates
workflow files before every commit:

```bash
# Install the hook
pipechecker --install-hook
```

The hook will:
1. Detect changed workflow files (`*.yml` in `.github/workflows/`, `.gitlab-ci.yml`, etc.)
2. Run `pipecheck --all --strict` on them
3. Block the commit if any errors are found

To skip the check (not recommended):
```bash
git commit --no-verify
```

You can also set up the hook manually by copying `templates/pre-commit-hook.sh`
to `.git/hooks/pre-commit` and making it executable.

### Output Formats

```bash
# Text output (default)
pipechecker .github/workflows/ci.yml

# JSON output for CI integration
pipechecker .github/workflows/ci.yml --format json

# Strict mode (warnings as errors)
pipechecker .github/workflows/ci.yml --strict

# Skip action/Docker checks
pipechecker .github/workflows/ci.yml --no-pinning
```

## 📋 Example Output

```
Provider: GitHubActions

1 errors, 0 warnings

❌ ERROR: Circular dependency detected: job-a -> job-c -> job-b
   💡 Remove one of the dependencies to break the cycle

ℹ️  INFO: Job 'build' uses secret: API_KEY
   💡 Ensure this secret is configured in repository settings
```

## 🔧 Supported Platforms

| Platform | Status | File Pattern |
|----------|--------|--------------|
| **GitHub Actions** | ✅ Full Support | `.github/workflows/*.yml` |
| **GitLab CI** | 🔜 Coming Soon | `.gitlab-ci.yml` |
| **CircleCI** | 🔜 Coming Soon | `.circleci/config.yml` |

## 🏗️ Use in CI/CD

### GitHub Actions
```yaml
- name: Validate workflows
  run: |
    npm install -g pipechecker
    pipechecker .github/workflows/*.yml --strict
```

### GitLab CI
```yaml
validate:
  script:
    - cargo install pipechecker
    - pipechecker .gitlab-ci.yml --strict
```

### Pre-commit Hook
```bash
#!/bin/bash
pipechecker .github/workflows/*.yml --strict || exit 1
```

## 🤝 Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## 📝 License

Licensed under either of:
- MIT License ([LICENSE-MIT](LICENSE-MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

at your option.

## 🌟 Show Your Support

If Pipechecker saves you time, give it a ⭐ on GitHub!
