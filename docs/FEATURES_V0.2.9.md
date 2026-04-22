# Features v0.2.9

## GitHub Action

PipeChecker is now available as a GitHub Action for direct CI integration.

### Usage

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

### Inputs

| Input | Required | Default | Description |
|-------|----------|---------|-------------|
| `path` | No | `.github/workflows` | Path to workflow file(s) |
| `strict` | No | `false` | Exit with error on warnings |
| `fail-on-warning` | No | `false` | Treat warnings as errors |
| `diff` | No | `false` | Check only changed files |
| `diff-branch` | No | `main` | Base branch for diff mode |
| `args` | No | `''` | Additional arguments |

---

## Diff Mode

Check only workflow files that changed since a base branch.

### CLI Usage

```bash
# Check files changed since main
pipechecker --diff

# Short form
pipechecker -d

# Check against different branch
pipechecker --diff --diff-branch develop
```

### How It Works

1. Runs `git diff --name-only <branch>...` to get changed files
2. Filters for workflow files (`.github/workflows/*.yml`, `.gitlab-ci.yml`, `.circleci/config.yml`)
3. Audits only those files

### Benefits

- Faster in monorepos with many workflows
- Only checks what changed, not everything
- Great for PR validation

---

## Internal Changes

- Added `Copy` derive to `AuditOptions` and `Rules` structs
- CLI args: `--diff` and `--diff-branch`
- New function: `get_changed_workflows()`