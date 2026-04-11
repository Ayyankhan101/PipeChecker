# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
