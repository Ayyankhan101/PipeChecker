# Features v0.2.10

## Template Library

Initialize new CI/CD workflows from pre-built templates.

### Usage

```bash
# Create a new workflow
pipechecker --init --template <name>

# Available templates
pipechecker --init --template node      # Node.js CI
pipechecker --init --template rust      # Rust CI
pipechecker --init --template docker    # Docker build & push
pipechecker --init --template gitlab-node # GitLab CI
```

### Options

| Flag | Description |
|------|-------------|
| `--init` | Initialize a new workflow |
| `--template` | Template name (required) |
| `--force` | Overwrite existing files |

### Examples

```bash
# Create Node.js workflow
pipechecker --init --template node

# Create Rust workflow  
pipechecker --init --template rust
# Output: .github/workflows/rust.yml

# Override existing file
pipechecker --init --template node --force
```

---

## Templates

### node.yml
GitHub Actions workflow for Node.js projects with:
- Checkout
- Node.js setup
- Dependency installation
- Linting
- Testing
- Building

### rust.yml
GitHub Actions workflow for Rust projects with:
- Checkout
- Rust toolchain
- Cargo cache
- Build
- Test
- Format check
- Clippy linting

### docker.yml
GitHub Actions workflow for Docker with:
- Docker Buildx setup
- Login to Docker Hub
- Build and push
- Build cache

### gitlab-node.yml
GitLab CI template for Node.js:
- Build stage
- Test stage
- Cache support

---

## GitLab CI include: Block Detection

Parses GitLab CI `include:` blocks to detect:
- Local file includes
- Remote URL includes  
- Project-based includes (`project::path`)

### Example

```yaml
include:
  - local: .gitlab-ci/base.yml
  - remote: https://gitlab.com/example/ci/-/raw/main/template.yml
  - project: mygroup/mysubgroup
    file: .gitlab-ci/pipeline.yml
```

### Warnings

| Type | Message |
|------|---------|
| Local file not found | "Local include not found: path/to/file.yml" |
| Remote include | "Remote includes require network access - add allow_failure: true" |
| Project include | "Project includes depend on external repository" |

---

## JSON Schema Validation

Structural validation for all CI/CD providers:
- Valid YAML syntax
- Required top-level keys per provider
- Job structure validation
- Unknown key detection

### GitHub Actions Checks

- Missing `on` trigger (warning)
- Missing `jobs` key (error)
- Job missing `runs-on` or `container` (error)
- Job missing `steps` (warning)
- Invalid `needs` format (warning)

### GitLab CI Checks

- Missing `stages` (info)
- No jobs defined (error)
- Unknown top-level keys (info)

### CircleCI Checks

- Missing `version` (error)
- Missing `jobs` or `workflows` (warning)
- Job missing executor (info)

---

## HTML Report Output

Generate HTML reports for CI/CD audit results.

### Usage

```bash
# Generate HTML report
pipechecker .github/workflows/ci.yml --format html > report.html

# Generate HTML report for all files
pipechecker --all --format html > report.html
```

### Features

- Styled HTML output
- Summary statistics
- Errors, warnings, and info items
- Color-coded badges
- File-by-file breakdown

---

## Config Rules in --all Mode

Fixed: Config rules from `.pipecheckerrc.yml` now properly applied in `--all` mode.

### Example Config

```yaml
rules:
  circular_dependencies: true   # Detect dependency cycles
  missing_secrets: true         # Flag hardcoded secrets  
  docker_latest_tag: true       # Warn about :latest tags
  timeout_validation: true      # Warn about missing timeouts
```

---

## Internal Changes

- Added `templates/` directory with YAML files
- New CLI args: `--init`, `--template`, `--force`
- New function: `init_from_template()`
- GitLab `include:` parsing in `parsers/gitlab.rs`
- New auditor: `auditors/include.rs`
- New auditor: `auditors/schema.rs`
- HTML report generation in `main.rs`
- Fixed config rules integration in `--all` mode
- Bumped version to 0.2.10