# Publishing Pipecheck to npm

## Prerequisites

1. **Rust toolchain** installed (for building)
2. **Node.js and npm** installed
3. **npm account** - Sign up at https://www.npmjs.com

## Setup

1. Login to npm:
```bash
npm login
```

2. Update package name in `package.json` if needed (must be unique on npm)

## Build for Multiple Platforms

### Option A: GitHub Actions (Recommended)
Create `.github/workflows/release.yml` to automatically build for all platforms.

### Option B: Manual Cross-Compilation

**For Linux:**
```bash
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu
```

**For macOS:**
```bash
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
```

**For Windows:**
```bash
cargo build --release --target x86_64-pc-windows-gnu
```

Copy binaries to `npm/` directory with correct names:
- `pipecheck-linux-x64`
- `pipecheck-linux-arm64`
- `pipecheck-darwin-x64`
- `pipecheck-darwin-arm64`
- `pipecheck-x64.exe` (Windows)

## Publish

```bash
# Test locally first
npm pack

# Publish to npm
npm publish
```

## Alternative: Platform-Specific Packages

Create separate packages for each platform:
- `@pipecheck/linux-x64`
- `@pipecheck/darwin-x64`
- `@pipecheck/darwin-arm64`
- `@pipecheck/win32-x64`

Main package uses `optionalDependencies` to install correct binary.

## Better Alternative: Use cargo-dist

```bash
cargo install cargo-dist
cargo dist init
cargo dist build
```

This handles cross-compilation and npm publishing automatically!

## Recommendation

Use **cargo-dist** - it's specifically designed for this use case and handles:
- Cross-platform builds
- GitHub releases
- npm publishing
- Installers for multiple platforms

Install it and run:
```bash
cargo install cargo-dist
cargo dist init --installer=npm
```
