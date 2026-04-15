# Contributing to Pipecheck

Thank you for your interest in contributing! 🎉

## Quick Start

1. Fork the repository
2. Clone your fork: `git clone https://github.com/Ayyankhan101/PipeCheck.git`
3. Create a branch: `git checkout -b feature/my-feature`
4. Make your changes
5. Run tests: `cargo test`
6. Format code: `cargo fmt`
7. Submit a pull request

## Development Setup

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build
cargo build

# Run tests
cargo test

# Run locally
cargo run -- tests/fixtures/github/valid.yml
```

## Adding New Features

### Adding a New CI Platform

1. Create parser in `src/parsers/yourplatform.rs`
2. Implement `parse()` function returning `Pipeline`
3. Add to `src/parsers/mod.rs`
4. Add test fixtures in `tests/fixtures/yourplatform/`

### Adding a New Auditor

1. Create auditor in `src/auditors/yourauditor.rs`
2. Implement `audit(&Pipeline) -> Result<Vec<Issue>>`
3. Add to `src/lib.rs` audit pipeline
4. Add tests

## Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy` and fix warnings
- Write tests for new features
- Update documentation

## Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

## Commit Messages

Use conventional commits:
- `feat: add GitLab CI support`
- `fix: handle missing job dependencies`
- `docs: update README examples`
- `test: add circular dependency tests`

## Questions?

Open an issue or discussion on GitHub!
