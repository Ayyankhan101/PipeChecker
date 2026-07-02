# Pipechecker Improvements Roadmap

## Recently Completed (v0.2.10 - v0.3.0)

- Rule codes (PC001-PC018) on every issue
- `--explain <CODE>` for detailed rule guidance
- Permissions auditor (PC014) — missing permissions block
- Concurrency auditor (PC016) — missing cancel-in-progress
- Artifacts auditor (PC017) — missing artifact retention
- Schema auditor (PC012-PC013, PC015) — structural validation
- `--ci` flag (--quiet --strict --format json)
- `--no-permissions`, `--no-schema` flags
- Embedded templates (compile-time include_str!)
- Kernel-level file watching via `notify` crate
- `--diff` mode (check only changed files)
- `--install-hook` pre-commit hook
- `--fix` auto-fix for unpinned actions + Docker images
- Configuration file (.pipecheckerrc.yml) with 8 rule toggles
- Template library (rust, node, docker, gitlab-node)

## Planned (v0.3.1)

- [ ] **Integrate test for --fix, --diff, --install-hook**
- [ ] **HTML report output** (`--report html`)
- [ ] **Progress indicator** for `--all` mode
- [ ] **Updated templates** with latest action versions

## Future Ideas

### Medium Priority
- **GitHub Action** — official `pipechecker/action` for PR checks
- **Workflow visualization** — Mermaid/Graphviz dependency graph
- **Performance metrics** — estimated CI time/cost savings
- **Benchmark mode** — compare workflow run times
- **Policy engine** — custom YAML policy enforcement

### Advanced
- **AI-powered suggestions** — LLM-driven optimization tips (opt-in)
- **VS Code extension** — LSP-based inline diagnostics
- **Team dashboard** — web UI for team workflow health
- **More providers** — Jenkins, Travis CI, Azure Pipelines
- **IDE integrations** — JetBrains, Vim/Neovim

## Contribution Ideas

- Add more auto-fix mappings to `src/fix.rs`
- Improve CircleCI parser (service image edge cases)
- Add `--self-update` command
- Support for monorepo multi-workflow analysis
- Rule code regression tests

---

**Prioritize based on user feedback.** Open an issue or PR on GitHub!
