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

## Internal Changes

- Added `templates/` directory with YAML files
- New CLI args: `--init`, `--template`, `--force`
- New function: `init_from_template()`
- Bumped version to 0.2.10