# 🔍 Pipecheck

[![CI](https://github.com/Ayyankhan101/PipeCheck/workflows/CI/badge.svg)](https://github.com/Ayyankhan101/PipeCheck/actions)
[![Crates.io](https://img.shields.io/crates/v/pipecheck.svg)](https://crates.io/crates/pipecheck)
[![npm](https://img.shields.io/npm/v/pipecheck.svg)](https://www.npmjs.com/package/pipecheck)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

**A blazingly fast CI/CD pipeline auditor that catches errors before you push.**

Stop wasting time debugging CI failures. Pipecheck validates your GitHub Actions, GitLab CI, and CircleCI configurations locally, catching syntax errors, circular dependencies, and security issues instantly.

## 🚀 Quick Start

### Install via npm (recommended)
```bash
npm install -g pipecheck
```

### Install via Cargo
```bash
cargo install pipecheck
```

### Run
```bash
pipecheck .github/workflows/ci.yml
```

## ✨ Features

- ✅ **Syntax Validation** - Parse and validate GitHub Actions, GitLab CI, and CircleCI configs
- 🔄 **Dependency Analysis** - Detect circular dependencies in job workflows
- 🔐 **Secrets Auditing** - Identify hardcoded secrets and environment variable issues
- 🐳 **Docker Validation** - Check Docker image references and tags
- 📊 **Multiple Output Formats** - Text and JSON output for CI integration
- ⚡ **Fast** - Written in Rust for maximum performance
- 🎯 **Zero Config** - Works out of the box

## 💡 Why Pipecheck?

**Before Pipecheck:**
```
git push
→ Wait 5 minutes
→ CI fails: "Circular dependency detected"
→ Fix locally
→ git push again
→ Wait 5 minutes...
```

**With Pipecheck:**
```
pipecheck .github/workflows/ci.yml
→ ❌ ERROR: Circular dependency detected: job-a -> job-c -> job-b
→ Fix immediately
→ git push with confidence ✅
```

## 📖 Usage

### Quick Start

```bash
# Auto-detect and check workflow
pipecheck

# Check specific file
pipecheck .github/workflows/ci.yml

# Check all workflows
pipecheck --all

# Interactive TUI mode
pipecheck --tui
```

### All Options

```
CI/CD Pipeline Auditor - Catch errors before you push

Usage: pipecheck [OPTIONS] [FILE]

Arguments:
  [FILE]  Path to pipeline configuration file (auto-detects if not provided)

Options:
  -a, --all              Check all workflow files in directory
      --install-hook     Install pre-commit hook
  -w, --watch            Watch for file changes and re-check
      --fix              Automatically fix issues where possible
      --tui              Interactive terminal UI mode
  -f, --format <FORMAT>  Output format (text, json) [default: text]
      --no-docker        Skip Docker image checks
  -s, --strict           Enable strict mode (warnings as errors)
  -h, --help             Print help
  -V, --version          Print version
```

### Interactive Features

```bash
# Install pre-commit hook
pipecheck --install-hook

# Watch mode - auto-recheck on file changes
pipecheck --watch

# Interactive TUI mode
pipecheck --tui

# Auto-fix issues (Coming soon!)
pipecheck --fix
```

### Configuration File

Create `.pipecheckrc.yml` in your project root:

```yaml
ignore:
  - .github/workflows/old-*.yml
  
rules:
  circular_dependencies: true
  missing_secrets: true
  docker_latest_tag: true
```

### Output Formats

```bash
# Text output (default)
pipecheck .github/workflows/ci.yml

# JSON output for CI integration
pipecheck .github/workflows/ci.yml --format json

# Strict mode (warnings as errors)
pipecheck .github/workflows/ci.yml --strict

# Skip Docker checks
pipecheck .github/workflows/ci.yml --no-docker
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
| **GitLab CI** | ✅ Full Support | `.gitlab-ci.yml` |
| **CircleCI** | ✅ Full Support | `.circleci/config.yml` |

## 🏗️ Use in CI/CD

### GitHub Actions
```yaml
- name: Validate workflows
  run: |
    npm install -g pipecheck
    pipecheck .github/workflows/*.yml --strict
```

### GitLab CI
```yaml
validate:
  script:
    - cargo install pipecheck
    - pipecheck .gitlab-ci.yml --strict
```

### Pre-commit Hook
```bash
#!/bin/bash
pipecheck .github/workflows/*.yml --strict || exit 1
```

## 🤝 Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## 📝 License

Licensed under either of:
- MIT License ([LICENSE-MIT](LICENSE-MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

at your option.

## 🌟 Show Your Support

If Pipecheck saves you time, give it a ⭐ on GitHub!
