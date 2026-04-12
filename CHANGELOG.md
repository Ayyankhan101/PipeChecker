# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.4] - 2026-04-12

### Added
- **Timing metrics** — every audit now shows `⏱️ Checked in Xms` so you can see how fast PipeChecker is
- **`--quiet` / `-q` flag** — only output errors, suppress warnings and info. Perfect for CI pipelines
- **`--verbose` flag** — show diagnostic info including which auditors ran and per-severity counts
- **Timeout auditor** — warns when jobs lack `timeout-minutes` (GitHub), `timeout` (GitLab), or `max_time` (CircleCI). Prevents runaway CI jobs that waste money
- **`--fix` now pins Docker `:latest` tags** — auto-replaces `node:latest` → `node:20-alpine`, `postgres:latest` → `postgres:16-alpine`, and 11 other common images
- **Config file `rules:` toggles are now wired up** — `.pipecheckerrc.yml` can disable `circular_dependencies`, `missing_secrets`, or `docker_latest_tag` checks individually

### Changed
- `AuditOptions` now carries an optional `Rules` struct to control which auditors run
- All parsers (GitHub Actions, GitLab CI, CircleCI) now extract job timeout fields

[0.2.4]: https://github.com/Ayyankhan101/PipeCheck/compare/v0.2.3...v0.2.4

## [0.2.3] - 2026-04-12

### Fixed
- Eliminated clippy warnings (unused imports, needless borrows)
- Fixed false positive in secrets auditor: `${{ secrets.* }}` references no longer flagged as hardcoded secrets
- Added `#[cfg(test)]` to DAG test module (tests were compiling in release builds)
- Downgraded cargo-deny-action from v2 to v1 for CI compatibility
- Added `--all-features` flag to cargo-deny CI step
- Updated deny.toml with missing fields and additional allowed licenses (CC0-1.0, MPL-2.0, Unicode-3.0)
- Fixed job name matching in `find_job_line` to avoid partial prefix matches

## [0.2.2] - 2024-05-21

### Added
- GitHub Actions parser and validator (Full Support)
- Circular dependency detection for GitHub Actions
- Secrets auditing for environment variables
- Docker image validation for GitHub Actions
- Text and JSON output formats
- CLI with `--strict` and `--no-docker` flags
- Cross-platform support (Linux, macOS, Windows)
- Auto-detection of workflow files
- `--all` flag to check all workflows at once
- Pre-commit hook installer (`--install-hook`)
- Watch mode (`--watch`)
- Interactive TUI mode (`--tui`)
- Configuration file support (`.pipecheckrc.yml`)
- Better error messages with line numbers
- Ignore patterns

### Fixed
- Updated package name to `pipechecker` for consistency
- Improved provider detection using YAML structure inspection
- Corrected `--fix` flag behavior to exit gracefully
- Added proper error handling for unimplemented providers

### Changed
- GitLab CI and CircleCI support marked as **Coming Soon**
- Defaulted `check_docker_images` to `true` in `AuditOptions`

## [0.0.1] - 2024-04-07

### Added
- Initial project structure and concept
- Basic GitHub Actions parsing logic

[0.2.2]: https://github.com/Ayyankhan101/PipeCheck/compare/v0.0.1...v0.2.2
[0.0.1]: https://github.com/Ayyankhan101/PipeCheck/releases/tag/v0.0.1
