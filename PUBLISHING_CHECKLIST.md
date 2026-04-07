# Publishing Checklist

## Before First Release

### 1. Setup Repository
- [ ] Create GitHub repository
- [ ] Update `repository` URL in `Cargo.toml`
- [ ] Update `repository` URL in `package.json`
- [ ] Add repository description and topics on GitHub

### 2. Create Accounts
- [ ] Create npm account at https://www.npmjs.com/signup
- [ ] Create crates.io account (login with GitHub at https://crates.io)

### 3. Setup Secrets
Add these to GitHub repository secrets (Settings → Secrets → Actions):
- [ ] `NPM_TOKEN` - Get from https://www.npmjs.com/settings/tokens
- [ ] `CARGO_TOKEN` - Get from https://crates.io/me

### 4. Test Locally
```bash
# Build and test
cargo build --release
cargo test

# Test CLI
./target/release/pipecheck tests/fixtures/github/valid.yml
./target/release/pipecheck tests/fixtures/github/circular.yml

# Test npm package locally
npm pack
npm install -g pipecheck-0.1.0.tgz
pipecheck tests/fixtures/github/valid.yml
```

### 5. Verify Files
- [ ] README.md is complete
- [ ] LICENSE-MIT exists
- [ ] CONTRIBUTING.md exists
- [ ] .github/workflows/ci.yml exists
- [ ] .github/workflows/release.yml exists

## Publishing Steps

### First Release (v0.1.0)

1. **Commit all changes**
```bash
git add .
git commit -m "chore: prepare for v0.1.0 release"
git push origin main
```

2. **Create and push tag**
```bash
git tag v0.1.0
git push origin v0.1.0
```

3. **GitHub Actions will automatically:**
   - Build binaries for all platforms
   - Create GitHub release
   - Publish to npm
   - Publish to crates.io

4. **Verify publication**
   - Check https://github.com/Ayyankhan101/PipeCheck/releases
   - Check https://www.npmjs.com/package/pipecheck
   - Check https://crates.io/crates/pipecheck

### Future Releases

1. Update version in:
   - `Cargo.toml`
   - `package.json`

2. Update CHANGELOG.md

3. Commit, tag, and push:
```bash
git commit -am "chore: bump version to v0.2.0"
git tag v0.2.0
git push origin main --tags
```

## Post-Release

- [ ] Announce on Twitter/LinkedIn
- [ ] Post on Reddit (r/rust, r/devops)
- [ ] Share in relevant Discord/Slack communities
- [ ] Update documentation site (if any)
- [ ] Write blog post about the tool

## Marketing Ideas

1. **Create demo GIF** showing pipecheck catching errors
2. **Write blog post**: "How Pipecheck Saves Hours of CI Debugging"
3. **Submit to**:
   - https://www.producthunt.com
   - https://news.ycombinator.com
   - https://dev.to
4. **Create comparison table** with other tools
5. **Add to awesome lists**: awesome-rust, awesome-devops

## Monitoring

After release, monitor:
- GitHub issues
- npm download stats: https://npm-stat.com/charts.html?package=pipecheck
- crates.io download stats
- GitHub stars/forks
