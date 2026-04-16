# 🎉 PipeChecker v0.2.7 - CircleCI Support Complete

## ✨ What's New

### CircleCI Support Now Complete

We've fixed two critical gaps in CircleCI support that made multi-provider auditing incomplete.

---

## 🐛 Fixes

### 1. CircleCI Global Environment Variables Now Parsed

Previously, the CircleCI parser ignored global environment variables defined at the top level. This meant the secrets auditor couldn't detect hardcoded secrets in CircleCI configs.

**Before:** No secrets detection for CircleCI `environment:` block
**After:** Secrets auditor now works for CircleCI

```yaml
version: 2.1

environment:
  API_KEY: "sk_live_abc123"  # Now detected!
  DATABASE_URL: "postgres://..."

jobs:
  build:
    steps:
      - run: echo "Building"
```

**Detection:**
```
⚠️  WARNING: Pipeline env 'API_KEY' may contain a hardcoded secret
   💡 Use secrets.secret-key instead of hardcoding
```

---

### 2. CircleCI Service Images Now Parsed

Previously, only the first docker image in a job was captured as `container_image`. Additional docker images (services) were ignored, meaning Docker `:latest` tag checks didn't work for CircleCI services.

**Before:** Service images ignored, no `:latest` warnings for services
**After:** All docker images parsed, proper :latest detection

```yaml
jobs:
  test:
    docker:
      - image: node:18          # Primary container
      - image: postgres:latest   # Service - now checked!
      - image: redis:alpine      # Service - now checked!
    steps:
      - run: npm test
```

**Detection:**
```
⚠️  WARNING: Job 'test' uses :latest Docker image in services: postgres:latest
   💡 Pin to a specific image tag for reproducible builds
```

---

## 📊 What's Now Works for All Providers

| Feature | GitHub Actions | GitLab CI | CircleCI |
|---------|---------------|-----------|----------|
| Syntax validation | ✅ | ✅ | ✅ |
| DAG / cycle detection | ✅ | ✅ | ✅ |
| Secrets auditing | ✅ | ✅ | ✅ (now fixed) |
| Docker :latest checks | ✅ | ✅ | ✅ (now fixed) |
| Timeout validation | ✅ | ✅ | ✅ |

---

## 🚀 Quick Start

### Upgrade
```bash
cargo install pipechecker --force
```

### Test CircleCI
```bash
pipechecker .circleci/config.yml
```

### Check for secrets
```bash
pipechecker .circleci/config.yml --verbose
```

---

## 📝 Changelog

### v0.2.7 Fixes
- ✅ CircleCI global env vars now parsed
- ✅ CircleCI service images now parsed
- ✅ Secrets auditor works for CircleCI
- ✅ Docker :latest checks work for CircleCI

### v0.2.6 Features
- ✅ Documentation consolidation into docs/ folder
- ✅ GitLab CI and CircleCI test fixtures added
- ✅ timeout_validation toggle in config

---

**PipeChecker v0.2.7 — Complete multi-provider CI/CD auditing! 🚀**