# 🎉 PipeChecker v0.2.6 - Documentation & Testing Improvements

## ✨ What's New

### 1. Documentation Consolidation

All documentation has been reorganized into the `docs/` folder for better organization:

```
docs/
├── index.md              ← New! Main entry point
├── START_HERE.md         ← Quick start guide
├── COMPLETE_GUIDE.md      ← Full documentation
├── QUICK_REFERENCE.md    ← CLI cheat sheet
├── TUI_GUIDE.md          ← Interactive UI guide
├── PROJECT_SUMMARY.md     ← Architecture overview
├── CONTRIBUTING.md       ← Contribution guidelines
├── NPM_PUBLISHING.md     ← NPM release guide
├── PUBLISHING_CHECKLIST.md ← Release checklist
└── FEATURES_*.md         ← Version feature logs
```

---

### 2. Test Fixtures Added

Added test fixtures for GitLab CI and CircleCI to ensure parser correctness:

- `tests/fixtures/gitlab/valid.yml` - Sample GitLab CI pipeline
- `tests/fixtures/circleci/valid.yml` - Sample CircleCI config

---

### 3. Configuration Improvements

- Added `timeout_validation` toggle to configuration rules
- Config file loading test added to ensure reliability

---

### 4. Parser Improvements

- **find_job_line** now has better scoping - more accurate line reporting
- **Circular dependency errors** now show full cycle paths for easier debugging
- **Auto-fix mode** now preserves trailing newlines correctly

---

## 📊 Feature Matrix

| Feature | Status |
|---------|--------|
| Syntax validation | ✅ All providers |
| DAG / cycle detection | ✅ All providers |
| Secrets auditing | ✅ All providers |
| Docker :latest checks | ✅ All providers |
| Timeout validation | ✅ All providers |
| Configuration file | ✅ Full support |

---

## 🚀 Quick Start

### Install
```bash
cargo install pipechecker
```

### Run
```bash
# Check GitHub Actions
pipechecker .github/workflows/ci.yml

# Check GitLab CI
pipechecker .gitlab-ci.yml

# Check CircleCI
pipechecker .circleci/config.yml

# Check all
pipechecker --all
```

---

## 📝 Upgrading

**No breaking changes!** Just update:

```bash
cargo install pipechecker --force
```

---

## 🔄 Coming in v0.2.7

- CircleCI env vars parsing (secrets auditor)
- CircleCI service images parsing (Docker checks)

---

**PipeChecker v0.2.6 — Better docs, more tests, improved reliability! 🚀**