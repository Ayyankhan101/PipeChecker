# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.2] - 2026-07-05

### Added
- **GitLab CI `extends:` support** — Parser now resolves `extends:` chains (single string or list) before processing jobs, enabling most real-world GitLab configs
- **GitLab CI hidden jobs filtering** — Jobs prefixed with `.` (e.g., `.base_template`) are now excluded from validation
- **GitLab CI `rules:` parsing** — Modern `rules:` keyword is now parsed into `RuleCondition` structs on each job
- **GitLab CI `workflow:rules`** — Top-level `workflow:rules:` block is now parsed
- **GitHub Actions reusable workflows** — `on: workflow_call` trigger is detected and `inputs:`/`secrets:` definitions are parsed
- **Matrix strategy auditor** (PC022) — Warns when matrix has >9 combinations without `fail-fast: false`
- **Deprecated features auditor** (PC019) — Detects `set-output`, `save-state`, EOL Node.js versions, and old action versions
- **Cost/efficiency auditor** (PC020-PC021) — Warns on excessive timeouts (>60 min) and missing concurrency groups
- **`pull_request_target` security check** (PC023) — Warns about insecure `pull_request_target` + `actions/checkout` pattern
- **`--fix-dry-run` flag** — Preview auto-fixes without modifying files
- **Shell completions** — `pipechecker completions --shell bash|zsh|fish` for tab completion
- **SARIF output format** — `--format sarif` for GitHub Code Scanning integration
- **`--config` flag** — Specify custom config file path instead of auto-discovery
- **4 new templates** — `python`, `go`, `java`, `circleci` added to `--init`
- **Expanded auto-fix mappings** — KNOWN_ACTIONS expanded from 16 to 42 entries

### Changed
- Version bumped to 0.3.2
- Coverage threshold increased from 54% to 65%
- 6 new rule codes: PC019-PC024
- 2 new config toggles: `deprecated_feature_check`, `cost_efficiency_check`
- Test count increased from 134 to 257

[0.3.2]: https://github.com/Ayyankhan101/PipeCheck/compare/v0.3.1...v0.3.2

## [0.3.1] - 2026-07-02

### Changed
- Updated templates (rust, node, docker, gitlab-node) with real multi-job workflows
- Rewrote START_HERE.md, COMPLETE_GUIDE.md, QUICK_REFERENCE.md for v0.3.x accuracy
- Updated `.pipecheckrc.example.yml` with all 8 rule fields
- Cleaned up IMPROVEMENTS.md — removed stale "new!" markers, accurate status
- Fixed binary name references (`pipecheck` → `pipechecker`) in docs
- Added missing flags to documentation (--ci, --diff, --fix, --explain, --init, --no-permissions, --no-schema, --verbose, --quiet)

### Fixed
- npm publish: requires Automation token with 2FA bypass

[0.3.1]: https://github.com/Ayyankhan101/PipeCheck/compare/v0.3.0...v0.3.1

## [0.3.0] - 2026-06-27

### Added
- **Rule codes** — every issue now carries a machine-readable code (`PC001`–`PC018`) shown in output and JSON.
- **`--explain <CODE>`** — print a detailed explanation, rationale, and fix guidance for any rule code (e.g. `pipechecker --explain PC005`).
- **Permissions auditor** (`PC014`) — warns when GitHub Actions jobs have no explicit `permissions:` block, preventing silent inheritance of write-all token permissions.
- **`--ci` flag** — convenience flag that implies `--quiet --strict --format json`. Ideal for CI automation without having to remember three separate flags.
- **`--no-permissions` flag** — skip the new permissions auditor independently of other checks.
- **`permissions_check` config rule** — disable the permissions auditor in `.pipecheckerrc.yml`.
- **Embedded templates** — `--init` templates are now embedded at compile time via `include_str!()`, fixing path resolution when the binary is run from any directory.
- **Improved watch mode** — Replaced CPU-heavy 2-second polling with kernel-level file watching using the `notify` crate, making `--watch` instant and battery-friendly.

### Changed
- Refactored duplicated audit-loop in `main.rs` into a single `run_audits_on_files()` helper (eliminated ~80 lines of copy-paste).
- `print_issue()` helper now also displays the rule code in brackets (e.g. `[PC005]`) when present.
- Verbose mode now lists all 9 auditors instead of 6.
- Wired up all 18 rule codes — every auditor now assigns its designated code to issues.
- Verbose mode auditor list updated to reflect all active auditors.

### Fixed
- `--init --template <name>` no longer fails when run from a directory other than the project root.
- Fixed compile errors in `artifacts.rs` (`serde_yaml::Value` type mismatches).
- `--explain` error message now correctly shows `PC001-PC018` instead of `PC001-PC015`.

[0.3.0]: https://github.com/Ayyankhan101/PipeCheck/compare/v0.2.10...v0.3.0

## [0.2.10] - Unreleased

### Added
- **Template Library** — New `--init --template` flag to initialize workflows from pre-built templates.
- Templates: `node`, `rust`, `docker`, `gitlab-node`
- Usage: `pipechecker --init --template rust`
- **Self-loop detection** — DAG auditor now correctly detects and reports jobs that depend on themselves.
- **Enhanced Secret Detection** — Secrets auditor now analyzes environment variable keys (e.g., `API_KEY`, `PASSWORD`) in addition to values.

### Changed
- Bumped version to 0.2.10
- Improved cycle detection path reporting for complex dependency graphs.

[0.2.10]: https://github.com/Ayyankhan101/PipeCheck/compare/v0.2.9...v0.2.10

## [0.2.9] - Unreleased

### Fixed
- **NPM wrapper binary path fix** — Fixed npm package to correctly locate the binary inside platform-specific folders. Users can now run `npm install pipechecker` and use it immediately without errors.

### Changed
- Updated `bin/pipechecker.js` to append `/pipechecker` suffix to binary path

[0.2.8]: https://github.com/Ayyankhan101/PipeCheck/compare/v0.2.7...v0.2.8

## [0.2.7] - 2026-04-16

### Fixed
- **CircleCI global env vars now parsed** — secrets auditor now detects hardcoded secrets in CircleCI `environment:` top-level config
- **CircleCI service images now parsed** — Docker `:latest` tag checks now work for CircleCI services defined in job `docker:` entries

### Changed
- CircleCI parser now correctly extracts service images from multiple docker entries (2nd, 3rd, etc. images treated as services)

[0.2.7]: https://github.com/Ayyankhan101/PipeCheck/compare/v0.2.6...v0.2.7

## [0.2.6] - 2026-04-15

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
