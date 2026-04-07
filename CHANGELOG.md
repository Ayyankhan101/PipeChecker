# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.1] - 2026-04-07

### Added
- Initial release
- GitHub Actions parser and validator
- GitLab CI parser and validator
- CircleCI parser and validator
- Circular dependency detection
- Secrets auditing
- Docker image validation
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
- `--version` flag

[Unreleased]: https://github.com/Ayyankhan101/PipeCheck/compare/v0.0.1...HEAD
[0.0.1]: https://github.com/Ayyankhan101/PipeCheck/releases/tag/v0.0.1
