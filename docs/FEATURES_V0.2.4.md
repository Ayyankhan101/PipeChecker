# 🎉 PipeChecker v0.2.4 - Speed, Control & Safety

## ✨ What's New

### 1. ⏱️ Timing Metrics
Every audit now shows how long it took — because you should know how fast your tools are.

```bash
pipechecker .github/workflows/ci.yml
```

```
Provider: GitHubActions
0 errors, 0 warnings
✅ All checks passed
⏱️  Checked in 2.1ms
```

**Why it matters:** PipeChecker is blazing fast. Now you can see it. Sub-5ms audits mean zero friction in your workflow.

---

### 2. 🤫 Quiet Mode (`--quiet` / `-q`)
Only output errors — suppress warnings and info messages. Perfect for CI pipelines.

```bash
pipechecker --quiet
```

```
❌ Circular dependency detected (job: deploy) (in .github/workflows/deploy.yml)
```

**Use case:** Clean CI output where only failures matter. Pair with `--strict` to fail on any issue.

---

### 3. 📢 Verbose Mode (`--verbose`)
See exactly what PipeChecker is doing — which files it found, which auditors ran, and per-severity breakdowns.

```bash
pipechecker --verbose
```

```
📄 Auditing: .github/workflows/ci.yml
🔍 Auditors ran: syntax, dag, secrets, pinning
📊 Found: 0 errors, 1 warnings, 0 info
⏱️  Checked in 3.2ms
```

**Use case:** Debugging, understanding what's being checked, or satisfying curiosity.

---

### 4. ⏰ Timeout Auditor
Warns when jobs lack timeout configuration:
- **GitHub Actions:** `timeout-minutes`
- **GitLab CI:** `timeout`
- **CircleCI:** `max_time`

```
⚠️ WARNING: Job 'deploy' has no timeout set (job: deploy) [line 42]
   💡 Add 'timeout-minutes: 30' to prevent runaway jobs
```

**Why it matters:** Runaway CI jobs waste money. A job without a timeout can run for hours, burning through your CI minutes budget. This auditor catches every job that's missing this safety net.

**Real cost:** A stuck deployment running for 6 hours = 36 CI minutes wasted. At $0.08/min (GitHub Actions Linux runner), that's $2.88 per incident. Multiply by your team size and frequency.

---

### 5. 🐳 `--fix` Now Pins Docker `:latest` Tags
Auto-replaces unstable `:latest` Docker tags with pinned, reproducible versions.

**Before `--fix`:**
```yaml
container:
  image: node:latest
services:
  db:
    image: postgres:latest
```

**After `--fix`:**
```yaml
container:
  image: node:20-alpine
services:
  db:
    image: postgres:16-alpine
```

**Supported images:**
| `:latest` Tag | Pinned To |
|---------------|-----------|
| `node:latest` | `node:20-alpine` |
| `python:latest` | `python:3.12-slim` |
| `ruby:latest` | `ruby:3.3-slim` |
| `nginx:latest` | `nginx:1.25-alpine` |
| `postgres:latest` | `postgres:16-alpine` |
| `redis:latest` | `redis:7-alpine` |
| `mysql:latest` | `mysql:8.0` |
| `ubuntu:latest` | `ubuntu:22.04` |
| `alpine:latest` | `alpine:3.19` |
| `golang:latest` | `golang:1.22-alpine` |
| `rust:latest` | `rust:1.75-slim` |
| `maven:latest` | `maven:3.9-eclipse-temurin-21` |
| `gradle:latest` | `gradle:8.6-jdk21` |

**Why it matters:** `:latest` tags change without notice. Your build works today, breaks tomorrow. Pinning ensures reproducible, reliable builds.

---

### 6. ⚙️ Config File Rules Are Now Wired Up
`.pipecheckerrc.yml` can now disable individual auditors:

```yaml
rules:
  circular_dependencies: false  # Skip cycle detection
  missing_secrets: false        # Skip secrets auditing
  docker_latest_tag: false      # Skip Docker :latest warnings
```

**Use case:** Large legacy codebases where some checks aren't practical yet. Disable what you can't fix now, enable it later.

---

## 📊 Complete Feature List

### v0.2.4 Features
- ✅ Timing metrics on every audit
- ✅ `--quiet` / `-q` flag for CI pipelines
- ✅ `--verbose` flag for diagnostics
- ✅ Timeout auditor (catches missing job timeouts)
- ✅ `--fix` pins Docker `:latest` tags
- ✅ Config file `rules:` toggles wired up
- ✅ All v0.2.0–v0.2.3 features

### v0.2.0–v0.2.3 Features (Still Available)
- ✅ Pre-commit hook installer
- ✅ Watch mode
- ✅ Interactive TUI mode
- ✅ Configuration file support
- ✅ Better error messages with line numbers
- ✅ Ignore patterns
- ✅ `--version` flag
- ✅ GitHub Actions, GitLab CI, CircleCI support
- ✅ Auto-detection of workflow files
- ✅ Circular dependency detection
- ✅ Secrets auditing
- ✅ Docker validation
- ✅ JSON output format
- ✅ Strict mode

---

## 🚀 Quick Start

### Upgrade
```bash
cargo install pipechecker --force
```

### Try the new features
```bash
# See how fast it is
pipechecker .github/workflows/ci.yml

# CI-friendly: only errors
pipechecker --quiet

# See what's happening
pipechecker --verbose

# Fix Docker tags automatically
pipechecker --fix
```

### Config example
```yaml
# .pipecheckerrc.yml
rules:
  circular_dependencies: true
  missing_secrets: true
  docker_latest_tag: true
```

---

## 📈 Impact

### Speed
- **Typical audit:** 2–5ms
- **Multi-file audit (10 files):** < 50ms
- **Zero perceptible delay**

### Cost Prevention
- **Timeout auditor:** Prevents runaway jobs that waste CI minutes
- **Docker pinning:** Prevents broken builds from upstream image changes
- **Quiet mode:** Cleaner CI logs, faster debugging

### Developer Experience
- ⏱️ Know exactly how long audits take
- 🤫 Keep CI output clean with `--quiet`
- 📢 Debug with `--verbose` when something's off
- ⏰ Never have a job run forever again
- 🐳 One command to pin all Docker tags

---

## 📝 Upgrade Guide

### From v0.2.3 to v0.2.4

**No breaking changes!** All existing flags and behavior still work.

**What's different:**
- Every audit now shows timing output at the bottom
- New `--quiet` and `--verbose` flags available
- `--fix` now also pins Docker `:latest` tags (in addition to GitHub Actions)
- Config file `rules:` now actually control which auditors run

**Recommended:**
```bash
# Update
cargo install pipechecker --force

# Try verbose mode to see what's happening
pipechecker --verbose

# Run --fix to pin any Docker :latest tags
pipechecker --fix
```

---

**PipeChecker v0.2.4 — Fast, controlled, and keeping your CI bills in check! 🚀**
