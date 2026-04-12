# PipeChecker

A Rust‑native CI/CD pipeline auditor that validates GitHub Actions, GitLab CI, and CircleCI workflows.

## Quick start
```bash
# Build and install (if not already built)
cargo install --path .

# Run the auditor on a repository (auto‑detects workflow files)
pipechecker --all
```

## CLI flags
| Flag | Description |
|------|-------------|
| `--all` | Audit **all** workflow files in the repository |
| `--watch` | Watch files for changes and re‑run the audit |
| `--fix` | Attempt automatic fixes (e.g., pin unpinned actions) |
| `--tui` | Launch the interactive terminal UI |
| `--format json` | Output results as JSON |
| `--strict` | Treat warnings as errors |
| `--no-pinning` | Skip Docker image and action‑pinning checks |

## Symbols used in output
- `✅` – No issues found
- `⚠️` – **Warning** (non‑critical issue)
- `❌` – **Error** (must be addressed)
- `🔧` – Auto‑fix mode

## Testing
Run the full test suite:
```bash
cargo test
```
The repository includes unit tests for the auditors (syntax, DAG, secrets) to ensure future changes don’t re‑introduce bugs.

## CI configuration
The GitHub Actions CI (`.github/workflows/ci.yml`) already runs:
- **Clippy** with `-D warnings`
- **rustfmt** checks
- **cargo audit** and **cargo deny** for security and licensing
- **Coverage** with `cargo tarpaulin`
- **Matrix builds** across Linux, macOS, and Windows, including cross‑compilation for `aarch64`.

The `network` feature (Docker image pinning) is exercised in the CI matrix via the `test` job, which builds the project with all optional features enabled.

## License
This project is licensed under either **MIT** or **Apache‑2.0** at your option. The `deny.toml` also includes **MPL‑2.0** and **Unicode‑3.0** as allowed licenses.
