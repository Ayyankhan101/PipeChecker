# PipeChecker GitHub Action

Validate CI/CD pipelines before you push — catch errors locally.

## Usage

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

## Inputs

| Input | Required | Default | Description |
|-------|----------|---------|-------------|
| `path` | No | `.github/workflows` | Path to workflow file(s) |
| `strict` | No | `false` | Exit with error on warnings |
| `fail-on-warning` | No | `false` | Treat warnings as errors |
| `diff` | No | `false` | Check only changed files |
| `diff-branch` | No | `main` | Base branch for diff mode |
| `args` | No | `''` | Additional arguments |

## Examples

### Check all workflows strictly

```yaml
- uses: Ayyankhan101/PipeCheck/actions/pipecheck@v0.2.9
  with:
    strict: true
```

### Check only changed files

```yaml
- uses: Ayyankhan101/PipeCheck/actions/pipecheck@v0.2.9
  with:
    diff: true
    diff-branch: main
```

### Custom path

```yaml
- uses: Ayyankhan101/PipeCheck/actions/pipecheck@v0.2.9
  with:
    path: .gitlab-ci.yml
```

### Additional arguments

```yaml
- uses: Ayyankhan101/PipeCheck/actions/pipecheck@v0.2.9
  with:
    args: '--quiet --format json'
```

## License

MIT OR Apache-2.0