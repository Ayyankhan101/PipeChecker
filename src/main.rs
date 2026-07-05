use clap::Parser;
use clap_complete::{generate, Shell};
use pipechecker::{
    audit_file, discover_workflows, load_config, load_from, rule_codes, AuditOptions,
    DiscoveryOptions,
};
use std::{fs, io, path::Path, process, time::Instant};

fn init_from_template(template: Option<String>, force: bool) {
    let tmpl = template.unwrap_or_else(|| {
        eprintln!("Please specify a template: --init --template <node|rust|docker|gitlab-node|python|go|java|circleci>");
        process::exit(1);
    });

    // Templates embedded at compile time — no relative path resolution needed
    let templates: &[(&str, &str)] = &[
        ("node", include_str!("../templates/node.yml")),
        ("rust", include_str!("../templates/rust.yml")),
        ("docker", include_str!("../templates/docker.yml")),
        ("gitlab-node", include_str!("../templates/gitlab-node.yml")),
        ("python", include_str!("../templates/python.yml")),
        ("go", include_str!("../templates/go.yml")),
        ("java", include_str!("../templates/java.yml")),
        ("circleci", include_str!("../templates/circleci.yml")),
    ];

    let (name, content) = templates
        .iter()
        .find(|(n, _)| *n == tmpl)
        .map(|(n, c)| (*n, *c))
        .unwrap_or_else(|| {
            eprintln!("Unknown template: {}", tmpl);
            eprintln!("Available: node, rust, docker, gitlab-node, python, go, java, circleci");
            process::exit(1);
        });

    let dest = if name == "gitlab-node" {
        Path::new(".gitlab-ci.yml").to_path_buf()
    } else if name == "circleci" {
        Path::new(".circleci/config.yml").to_path_buf()
    } else {
        Path::new(".github/workflows").join(format!("{}.yml", name))
    };

    if dest.exists() && !force {
        eprintln!(
            "File {} already exists. Use --force to overwrite.",
            dest.display()
        );
        process::exit(1);
    }

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).ok();
    }
    fs::write(&dest, content).expect("Failed to write template file");

    println!("✅ Created {} from template '{}'", dest.display(), name);
    println!("   Run 'pipechecker {}' to validate", dest.display());
}

/// Auto-detect a single workflow file from common patterns.
/// Uses `discover_workflows` under the hood, then prefers known filenames.
fn auto_detect_workflow() -> String {
    let files = discover_workflows(Path::new("."), &DiscoveryOptions::default());

    // Try common naming patterns first
    let common_patterns = [
        ".github/workflows/ci.yml",
        ".github/workflows/main.yml",
        ".github/workflows/build.yml",
        ".gitlab-ci.yml",
        ".circleci/config.yml",
    ];

    for pattern in &common_patterns {
        if files.iter().any(|f| f == pattern) {
            eprintln!("✓ Auto-detected: {}", pattern);
            return pattern.to_string();
        }
    }

    // Return first discovered file
    if let Some(first) = files.first() {
        eprintln!("✓ Auto-detected: {}", first);
        return first.clone();
    }

    eprintln!("❌ No workflow files found. Please specify a file:");
    eprintln!("   pipechecker <FILE>");
    eprintln!("\nSearched for:");
    eprintln!("  - .github/workflows/*.yml");
    eprintln!("  - .gitlab-ci.yml");
    eprintln!("  - .circleci/config.yml");
    process::exit(1)
}

/// Get workflow files changed since the given base branch
fn get_changed_workflows(base_branch: &str) -> Vec<String> {
    let output = std::process::Command::new("git")
        .args(["diff", "--name-only", &format!("{}...", base_branch)])
        .output();

    match output {
        Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|f| {
                f.contains(".github/workflows")
                    || f.contains(".gitlab-ci")
                    || f.contains(".circleci")
            })
            .filter(|f| f.ends_with(".yml") || f.ends_with(".yaml"))
            .map(String::from)
            .collect(),
        _ => Vec::new(),
    }
}

// ---------------------------------------------------------------------------
// Shared output helpers
// ---------------------------------------------------------------------------

/// Load config from --config path or default, then apply CLI overrides
fn load_config_rules(cli: &Cli) -> pipechecker::Rules {
    let mut rules = if let Some(ref config_path) = cli.config {
        load_from(config_path).rules
    } else {
        load_config().rules
    };
    if cli.no_permissions {
        rules.permissions_check = false;
    }
    if cli.no_schema {
        rules.schema_validation = false;
    }
    rules
}

/// Format an AuditResult as SARIF 2.1.0 JSON
fn format_sarif(result: &pipechecker::AuditResult, file_path: &str) -> String {
    let sarif_results: Vec<serde_json::Value> = result
        .issues
        .iter()
        .map(|issue| {
            let rule_id = issue.rule_code.unwrap_or("PC000");
            let level = match issue.severity {
                pipechecker::Severity::Error => "error",
                pipechecker::Severity::Warning => "warning",
                pipechecker::Severity::Info => "note",
            };

            let mut location_obj = serde_json::json!({
                "physicalLocation": {
                    "artifactLocation": {
                        "uri": file_path,
                        "uriBaseId": "%SRCROOT%"
                    }
                }
            });

            if let Some(loc) = &issue.location {
                if loc.line > 0 {
                    location_obj["physicalLocation"]["region"] = serde_json::json!({
                        "startLine": loc.line,
                        "startColumn": loc.column,
                    });
                }
            }

            let mut result_obj = serde_json::json!({
                "ruleId": rule_id,
                "level": level,
                "message": {
                    "text": issue.message
                },
                "locations": [location_obj],
            });

            if let Some(suggestion) = &issue.suggestion {
                result_obj["fixes"] = serde_json::json!([{
                    "description": { "text": suggestion },
                }]);
            }

            result_obj
        })
        .collect();

    let sarif = serde_json::json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "pipechecker",
                    "version": env!("CARGO_PKG_VERSION"),
                    "rules": []
                }
            },
            "results": sarif_results
        }]
    });

    serde_json::to_string_pretty(&sarif).unwrap()
}

/// Print a single issue in text format.
fn print_issue(issue: &pipechecker::Issue, quiet: bool) {
    if quiet && issue.severity != pipechecker::Severity::Error {
        return;
    }
    let prefix = match issue.severity {
        pipechecker::Severity::Error => "\u{274c} ERROR",
        pipechecker::Severity::Warning => "\u{26a0}\u{fe0f}  WARNING",
        pipechecker::Severity::Info => "\u{2139}\u{fe0f}  INFO",
    };
    print!("{}: {}", prefix, issue.message);
    if let Some(loc) = &issue.location {
        if let Some(job) = &loc.job {
            print!(" (job: {})", job);
        }
        if loc.line > 0 {
            print!(" [line {}]", loc.line);
        }
    }
    if let Some(code) = issue.rule_code {
        print!(" [{}]", code);
    }
    println!();
    if let Some(suggestion) = &issue.suggestion {
        println!("   \u{1f4a1} {}", suggestion);
    }
    println!();
}

/// Print detailed explanation for a rule code.
fn explain_rule(code: &str) {
    let explanations: &[(&str, &str, &str)] = &[
        (
            rule_codes::EMPTY_PIPELINE,
            "Pipeline has no jobs defined",
            "Every CI/CD pipeline must define at least one job. An empty pipeline \
             will fail immediately when triggered. Add a 'jobs:' block with at \
             least one job entry.",
        ),
        (
            rule_codes::DUPLICATE_JOB_ID,
            "Duplicate job ID",
            "Each job in a pipeline must have a unique identifier. Duplicate IDs \
             cause ambiguous dependency references and will be rejected by most CI \
             providers. Rename one of the duplicate jobs.",
        ),
        (
            rule_codes::EMPTY_JOB_STEPS,
            "Job has no steps",
            "A job with no steps does no work. This is usually a mistake — either \
             you forgot to add steps, or you should remove the job entirely.",
        ),
        (
            rule_codes::MISSING_DEPENDENCY,
            "Job depends on non-existent job",
            "A job listed in 'needs:' does not exist. This will cause the pipeline \
             to fail at startup. Either add the missing job or remove the dependency.",
        ),
        (
            rule_codes::CIRCULAR_DEPENDENCY,
            "Circular dependency detected",
            "Two or more jobs form a dependency cycle (A needs B, B needs A). \
             The pipeline can never start because each job is waiting for another. \
             Remove one of the edges in the cycle to break it.",
        ),
        (
            rule_codes::HARDCODED_SECRET,
            "Hardcoded secret in env var",
            "An environment variable name or value looks like it contains a secret \
             (API key, token, password). Hardcoded secrets are committed to version \
             control and are a serious security risk. Use '${{ secrets.YOUR_SECRET }}' \
             instead and store the value in your repository's secret settings.",
        ),
        (
            rule_codes::SECRET_REFERENCE,
            "Secret reference in run block",
            "Informational: a step references a secret via '${{ secrets.X }}'. \
             Ensure the secret is configured in your repository or organisation settings.",
        ),
        (
            rule_codes::UNDECLARED_ENV_VAR,
            "Undeclared environment variable referenced",
            "A step references '${{ env.X }}' but X is not declared in any 'env:' \
             block at the pipeline, job, or step level. This will expand to an empty \
             string at runtime, which is almost never intentional. Declare the variable \
             in an appropriate 'env:' block.",
        ),
        (
            rule_codes::UNPINNED_ACTION,
            "Unpinned GitHub Action",
            "An action is referenced without a version tag (e.g. 'uses: actions/checkout'). \
             Without a pin, the action can change silently on the next run, breaking \
             your pipeline or introducing security issues. Add '@v4' (or a specific SHA) \
             to pin the version.",
        ),
        (
            rule_codes::DOCKER_LATEST_TAG,
            "Docker image uses :latest tag",
            "Using ':latest' makes builds non-reproducible — the image can change on \
             any push. Pin to a specific version tag (e.g. 'node:20-alpine') to ensure \
             consistent behaviour. Run 'pipechecker --fix' to apply known pinned versions \
             automatically.",
        ),
        (
            rule_codes::MISSING_TIMEOUT,
            "Job has no timeout",
            "A job without a timeout can run indefinitely if it hangs, wasting CI \
             minutes and blocking other runs. Add 'timeout-minutes: 30' (or an \
             appropriate value) to the job definition.",
        ),
        (
            rule_codes::MISSING_TRIGGER,
            "Workflow missing 'on' trigger",
            "A GitHub Actions workflow without an 'on:' block will never be triggered \
             automatically. Add at least one trigger such as 'on: push' or \
             'on: [push, pull_request]'.",
        ),
        (
            rule_codes::MISSING_RUNS_ON,
            "Job missing 'runs-on'",
            "GitHub Actions requires every job to specify where it runs via 'runs-on:' \
             (e.g. 'runs-on: ubuntu-latest'). Without this field the workflow is invalid.",
        ),
        (
            rule_codes::MISSING_PERMISSIONS,
            "Missing permissions declaration",
            "A GitHub Actions job without an explicit 'permissions:' block inherits \
             the repository-level GITHUB_TOKEN permissions, which may be 'write-all' \
             by default. This violates the principle of least privilege and can allow \
             a compromised action to write to your repository. Add a 'permissions:' \
             block to each job (or at the top level) to restrict access. \
             See: https://docs.github.com/en/actions/security-guides/automatic-token-authentication",
        ),
        (
            rule_codes::INVALID_YAML,
            "Invalid YAML syntax",
            "The file is not valid YAML. Fix syntax errors (unclosed brackets, bad \
             indentation, reserved characters) before running other checks. \
             Tools like 'yamllint' can help locate the exact problem.",
        ),
        (
            rule_codes::CONCURRENCY_CANCEL_MISSING,
            "Concurrency group missing 'cancel-in-progress'",
            "When using 'concurrency:' to limit concurrent workflow runs, it is highly \
             recommended to set 'cancel-in-progress: true'. Without it, new runs will \
             queue up rather than cancelling outdated runs, wasting CI minutes.",
        ),
        (
            rule_codes::CACHE_STATIC_KEY,
            "Cache action uses static key without 'hashFiles'",
            "Using actions/cache with a static string key means the cache is never \
             automatically invalidated when your dependencies change. Include \
             '${{ hashFiles(...) }}' in the key to ensure fresh caches.",
        ),
        (
            rule_codes::ARTIFACT_NO_RETENTION,
            "Upload-artifact missing 'retention-days'",
            "The actions/upload-artifact action retains artifacts for 90 days by default. \
             This quickly consumes repository storage quotas. Add 'retention-days: 7' \
             (or another low number) to save space.",
        ),
        (
            rule_codes::DEPRECATED_ACTION,
            "Deprecated GitHub Actions feature",
            "This workflow uses a deprecated GitHub Actions feature. 'set-output' and \
             'save-state' commands are deprecated — use '$GITHUB_OUTPUT' and \
             '$GITHUB_STATE' environment files instead. Node.js 12 and 16 are \
             end-of-life — upgrade to node:20 or later. Old action versions (@v1, @v2) \
             may have security vulnerabilities.",
        ),
        (
            rule_codes::EXCESSIVE_TIMEOUT,
            "Excessive job timeout",
            "This job has a timeout exceeding 60 minutes. Excessive timeouts waste CI \
             minutes and may indicate a configuration problem. Consider reducing the \
             timeout or splitting the job into smaller units.",
        ),
        (
            rule_codes::MISSING_CONCURRENCY,
            "Missing concurrency group",
            "This workflow has no 'concurrency:' group. Without concurrency control, \
             multiple runs of the same workflow can execute simultaneously, wasting CI \
             minutes. Add a concurrency group with 'cancel-in-progress: true' to \
             automatically cancel outdated runs.",
        ),
        (
            rule_codes::LARGE_MATRIX_NO_FAILFAST,
            "Large matrix without fail-fast",
            "This matrix strategy has more than 9 combinations without 'fail-fast: false'. \
             A failing job in a large matrix will cancel all other jobs, potentially hiding \
             useful test results. Consider adding 'fail-fast: false' for large matrices.",
        ),
        (
            rule_codes::PR_TARGET_INSECURE,
            "Insecure pull_request_target usage",
            "This workflow uses 'pull_request_target' with 'actions/checkout'. This is a \
             known security vulnerability pattern — pull_request_target runs with write \
             permissions and can leak secrets to attacker-controlled PRs. If you need to \
             check out PR code, use 'pull_request' instead, or carefully restrict the \
             ref: to a fixed branch.",
        ),
        (
            rule_codes::REUSABLE_WORKFLOW_MISSING_INPUTS,
            "Reusable workflow missing inputs",
            "A reusable workflow call is missing required inputs. Ensure all required \
             inputs are passed when calling a reusable workflow via 'uses:'.",
        ),
    ];

    let upper = code.to_uppercase();
    match explanations.iter().find(|(c, _, _)| *c == upper.as_str()) {
        Some((c, title, detail)) => {
            println!("\n\u{1f4d6} Rule {} — {}\n", c, title);
            println!("{}", detail);
            println!();
        }
        None => {
            eprintln!("Unknown rule code: '{}'", code);
            eprintln!("Known codes: PC001-PC024");
            process::exit(1);
        }
    }
}

/// Run audits on a list of files and print results. Returns true if any errors found.
fn run_audits_on_files(files: &[String], options: AuditOptions, quiet: bool, strict: bool) -> bool {
    let mut has_error = false;
    for file in files {
        match audit_file(file, options) {
            Ok(result) => {
                let file_has_errors = result
                    .issues
                    .iter()
                    .any(|i| i.severity == pipechecker::Severity::Error);
                has_error = has_error || file_has_errors;

                if file_has_errors || (strict && !result.issues.is_empty()) {
                    for issue in &result.issues {
                        print_issue(issue, quiet);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error auditing {}: {}", file, e);
                has_error = true;
            }
        }
    }
    has_error
}

#[derive(Parser)]
#[command(name = "pipechecker")]
#[command(version)]
#[command(about = "CI/CD Pipeline Auditor - Catch errors before you push", long_about = None)]
struct Cli {
    /// Path to pipeline configuration file (auto-detects if not provided)
    #[arg(value_name = "FILE")]
    file: Option<String>,

    /// Check all workflow files in directory
    #[arg(short, long)]
    all: bool,

    /// Install pre-commit hook
    #[arg(long)]
    install_hook: bool,

    /// Watch for file changes and re-check
    #[arg(short, long)]
    watch: bool,

    /// Automatically fix issues where possible
    #[arg(long)]
    fix: bool,

    /// Preview fixes without writing changes (requires --fix)
    #[arg(long, requires = "fix")]
    fix_dry_run: bool,

    /// Interactive terminal UI mode
    #[arg(long)]
    tui: bool,

    /// Output format (text, json, sarif)
    #[arg(short, long, default_value = "text")]
    format: String,

    /// Skip action pinning and Docker image checks
    #[arg(long)]
    no_pinning: bool,

    /// Enable strict mode (warnings as errors)
    #[arg(short, long)]
    strict: bool,

    /// Quiet mode — only show errors
    #[arg(short, long)]
    quiet: bool,

    /// Verbose mode — show detailed diagnostic information
    #[arg(long)]
    verbose: bool,

    /// CI mode — implies --quiet --strict --format json (ideal for automation)
    #[arg(long)]
    ci: bool,

    /// Check only files changed since base branch
    #[arg(short, long)]
    diff: bool,

    /// Base branch for diff mode
    #[arg(long, default_value = "main")]
    diff_branch: String,

    /// Initialize a new workflow from template
    #[arg(long)]
    init: bool,

    /// Template name (node, rust, docker, gitlab-node)
    #[arg(long, requires = "init")]
    template: Option<String>,

    /// Force overwrite existing files
    #[arg(long, requires = "init")]
    force: bool,

    /// Skip permissions checks (GitHub Actions only)
    #[arg(long)]
    no_permissions: bool,

    /// Skip schema validation
    #[arg(long)]
    no_schema: bool,

    /// Show detailed explanation for a rule code (e.g. --explain PC005)
    #[arg(long, value_name = "CODE")]
    explain: Option<String>,

    /// Path to a custom config file (overrides auto-detection)
    #[arg(long, value_name = "PATH")]
    config: Option<String>,

    /// Subcommand
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Generate shell completions for the specified shell
    Completions {
        /// Shell to generate completions for
        #[arg(short, long)]
        shell: Shell,
    },
}

impl Cli {
    /// Effective quiet setting — true when --quiet or --ci is set
    fn effective_quiet(&self) -> bool {
        self.quiet || self.ci
    }

    /// Effective strict setting — true when --strict or --ci is set
    fn effective_strict(&self) -> bool {
        self.strict || self.ci
    }

    /// Effective output format — "json" when --ci is set
    fn effective_format(&self) -> &str {
        if self.ci {
            "json"
        } else {
            &self.format
        }
    }
}

fn main() {
    let cli = Cli::parse();

    // Handle subcommands first
    if let Some(Commands::Completions { shell }) = cli.command {
        let mut cmd = <Cli as clap::CommandFactory>::command();
        generate(shell, &mut cmd, "pipechecker", &mut io::stdout());
        return;
    }

    // Handle --explain before anything else — no file needed
    if let Some(ref code) = cli.explain {
        explain_rule(code);
        return;
    }

    if cli.init {
        init_from_template(cli.template, cli.force);
        return;
    }

    if cli.install_hook {
        install_git_hook();
        return;
    }

    if cli.watch {
        watch_mode(&cli);
        return;
    }

    if cli.tui {
        let rules = load_config_rules(&cli);
        let options = AuditOptions {
            check_docker_images: !cli.no_pinning,
            strict_mode: cli.effective_strict(),
            rules: Some(rules),
        };
        if let Err(e) = pipechecker::tui::run_tui(options) {
            eprintln!("TUI error: {}", e);
            process::exit(1);
        }
        return;
    }

    if cli.fix {
        println!("🔧 Auto-fix mode\n");

        let file = cli.file.clone().unwrap_or_else(auto_detect_workflow);

        match pipechecker::fix::fix_file(&file, cli.fix_dry_run) {
            Ok(result) => {
                if result.fixed == 0 {
                    println!("✅ No fixable issues found in {}", file);
                    println!("   All actions are already pinned or use local references");
                } else {
                    if cli.fix_dry_run {
                        println!(
                            "📋 Dry run — {} issue(s) would be fixed in {}:\n",
                            result.fixed, file
                        );
                    } else {
                        println!("✨ Fixed {} issue(s) in {}:\n", result.fixed, file);
                    }
                    for change in &result.changes {
                        if change.starts_with("  ") {
                            println!("{}", change);
                        }
                    }
                    if cli.fix_dry_run {
                        println!("\n💡 Run without --fix-dry-run to apply these changes.");
                    } else {
                        println!("\n💡 Review the changes and commit them!");
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Error fixing {}: {}", file, e);
                process::exit(1);
            }
        }
        process::exit(0);
    }

    let rules = load_config_rules(&cli);

    let options = AuditOptions {
        check_docker_images: !cli.no_pinning,
        strict_mode: cli.effective_strict(),
        rules: Some(rules),
    };

    if cli.diff {
        let changed_files = get_changed_workflows(&cli.diff_branch);
        if changed_files.is_empty() {
            println!("No workflow files changed since {}", cli.diff_branch);
            return;
        }
        println!(
            "📁 Checking {} file(s) changed since {}...\n",
            changed_files.len(),
            cli.diff_branch
        );
        let has_error = run_audits_on_files(
            &changed_files,
            options,
            cli.effective_quiet(),
            cli.effective_strict(),
        );
        if has_error {
            process::exit(1);
        }
        println!("✅ All changed workflows valid!");
        return;
    }

    if cli.all {
        audit_all_workflows(
            options,
            cli.effective_format(),
            cli.effective_strict(),
            cli.effective_quiet(),
            cli.verbose,
        );
        return;
    }

    let file = cli.file.clone().unwrap_or_else(auto_detect_workflow);

    if cli.verbose {
        eprintln!("📄 Auditing: {}", file);
    }

    match audit_file(&file, options) {
        Ok(result) => {
            if cli.verbose {
                eprintln!("🔍 Auditors ran: syntax, dag, secrets, timeout, permissions, schema, concurrency, artifacts, pinning");
                eprintln!(
                    "📊 Found: {} errors, {} warnings, {} info",
                    result
                        .issues
                        .iter()
                        .filter(|i| i.severity == pipechecker::Severity::Error)
                        .count(),
                    result
                        .issues
                        .iter()
                        .filter(|i| i.severity == pipechecker::Severity::Warning)
                        .count(),
                    result
                        .issues
                        .iter()
                        .filter(|i| i.severity == pipechecker::Severity::Info)
                        .count(),
                );
            }

            if cli.effective_format() == "json" {
                println!("{}", serde_json::to_string_pretty(&result).unwrap());
            } else if cli.effective_format() == "sarif" {
                println!("{}", format_sarif(&result, &file));
            } else {
                println!("Provider: {:?}", result.provider);
                println!("\n{}", result.summary);
                println!();

                for issue in &result.issues {
                    print_issue(issue, cli.effective_quiet());
                }

                // Only show timing in non-quiet mode
                if !cli.effective_quiet() {
                    println!("⏱️  Checked in {:.1}ms", result.elapsed.as_millis());
                }
            }

            let has_errors = result
                .issues
                .iter()
                .any(|i| i.severity == pipechecker::Severity::Error);

            if has_errors || (cli.effective_strict() && !result.issues.is_empty()) {
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

fn install_git_hook() {
    let hook_path = Path::new(".git/hooks/pre-commit");

    if !Path::new(".git").exists() {
        eprintln!("❌ Not a git repository");
        process::exit(1);
    }

    let hook_content = r#"#!/bin/bash
# Pipecheck pre-commit hook

echo "🔍 Checking workflows with pipechecker..."

WORKFLOW_FILES=$(git diff --cached --name-only | grep -E '(\.github/workflows|\.gitlab-ci|\.circleci).*\.ya?ml$')

if [ -n "$WORKFLOW_FILES" ]; then
    if command -v pipechecker &> /dev/null; then
        pipechecker --all --strict
        if [ $? -ne 0 ]; then
            echo ""
            echo "❌ Workflow validation failed!"
            echo "Fix errors above or use 'git commit --no-verify' to skip"
            exit 1
        fi
        echo "✅ All workflows valid!"
    else
        echo "⚠️  pipechecker not installed, skipping"
    fi
fi
"#;

    if hook_path.exists() {
        eprint!("⚠️  Pre-commit hook already exists. Overwrite? (y/N): ");
        use std::io::{self, BufRead};
        let stdin = io::stdin();
        let mut line = String::new();
        stdin.lock().read_line(&mut line).unwrap();
        if !line.trim().eq_ignore_ascii_case("y") {
            eprintln!("Cancelled");
            return;
        }
    }

    fs::write(hook_path, hook_content).expect("Failed to write hook");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(hook_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(hook_path, perms).unwrap();
    }

    eprintln!("✅ Pre-commit hook installed!");
    eprintln!("   Pipecheck will run before every commit");
    eprintln!("   Use 'git commit --no-verify' to skip");
}

fn watch_mode(cli: &Cli) {
    eprintln!("👀 Watching for workflow changes...");
    eprintln!("   Press Ctrl+C to stop\n");

    let rules = load_config_rules(cli);
    let options = AuditOptions {
        check_docker_images: !cli.no_pinning,
        strict_mode: cli.effective_strict(),
        rules: Some(rules),
    };

    // Initial audit pass
    if cli.diff {
        let changed_files = get_changed_workflows(&cli.diff_branch);
        if changed_files.is_empty() {
            println!("No workflow files changed since {}", cli.diff_branch);
            return;
        }
        println!(
            "📁 Checking {} file(s) changed since {}...\n",
            changed_files.len(),
            cli.diff_branch
        );
        run_audits_on_files(
            &changed_files,
            options,
            cli.effective_quiet(),
            cli.effective_strict(),
        );
        return;
    }

    if cli.all {
        audit_all_workflows(
            options,
            cli.effective_format(),
            cli.effective_strict(),
            cli.effective_quiet(),
            cli.verbose,
        );
    } else if let Some(file) = &cli.file {
        let _ = audit_file(file, options);
    }

    use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
    use std::sync::mpsc::channel;

    let (tx, rx) = channel();
    let mut watcher = match RecommendedWatcher::new(tx, Config::default()) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Failed to initialize file watcher: {}", e);
            process::exit(1);
        }
    };

    let watch_path = if let Some(file) = &cli.file {
        Path::new(file)
    } else {
        Path::new(".")
    };

    if let Err(e) = watcher.watch(watch_path, RecursiveMode::Recursive) {
        eprintln!("Failed to watch {}: {}", watch_path.display(), e);
        process::exit(1);
    }

    loop {
        match rx.recv() {
            Ok(Ok(event)) => {
                if matches!(
                    event.kind,
                    notify::EventKind::Modify(_) | notify::EventKind::Create(_)
                ) {
                    for path in event.paths {
                        let path_str = path.to_string_lossy();
                        // Only trigger if it's a workflow file or the explicitly passed file
                        if (cli.file.as_deref().is_some_and(|f| path_str.ends_with(f))
                            || path_str.contains(".github/workflows")
                            || path_str.contains(".gitlab-ci")
                            || path_str.contains(".circleci"))
                            && (path_str.ends_with(".yml") || path_str.ends_with(".yaml"))
                        {
                            eprintln!("\n🔄 File changed: {}", path.display());
                            let opts = AuditOptions {
                                check_docker_images: !cli.no_pinning,
                                strict_mode: cli.effective_strict(),
                                rules: Some(load_config_rules(cli)),
                            };
                            let _ = audit_file(&path_str, opts);
                        }
                    }
                }
            }
            Ok(Err(e)) => eprintln!("Watch error: {:?}", e),
            Err(_) => {
                eprintln!("Watch channel closed");
                break;
            }
        }
    }
}

fn audit_all_workflows(
    options: AuditOptions,
    format: &str,
    strict: bool,
    quiet: bool,
    verbose: bool,
) {
    let all_files = discover_workflows(Path::new("."), &DiscoveryOptions::default());

    if all_files.is_empty() {
        eprintln!("❌ No workflow files found");
        process::exit(1);
    }

    if verbose {
        eprintln!("📄 Discovered {} workflow file(s)", all_files.len());
        for f in &all_files {
            eprintln!("   - {}", f);
        }
        eprintln!();
    }

    eprintln!("Checking {} workflow file(s)...\n", all_files.len());

    let total_start = Instant::now();
    let mut total_errors = 0;
    let mut total_warnings = 0;

    let config = load_config();

    for file in &all_files {
        if config.should_ignore(file) {
            continue;
        }

        let opts = AuditOptions {
            check_docker_images: options.check_docker_images,
            strict_mode: options.strict_mode,
            rules: options.rules,
        };
        match audit_file(file, opts) {
            Ok(result) => {
                if format == "json" {
                    println!("{}", serde_json::to_string_pretty(&result).unwrap());
                } else {
                    let errors = result
                        .issues
                        .iter()
                        .filter(|i| i.severity == pipechecker::Severity::Error)
                        .count();
                    let warnings = result
                        .issues
                        .iter()
                        .filter(|i| i.severity == pipechecker::Severity::Warning)
                        .count();

                    total_errors += errors;
                    total_warnings += warnings;

                    if quiet {
                        // Only print errors in quiet mode
                        for issue in &result.issues {
                            if issue.severity == pipechecker::Severity::Error {
                                println!("❌ {} (in {})", issue.message, file);
                            }
                        }
                    } else {
                        println!("📄 {}", file);
                        println!("   Provider: {:?}", result.provider);

                        if errors > 0 || warnings > 0 {
                            println!("   {} errors, {} warnings", errors, warnings);
                            for issue in &result.issues {
                                if issue.severity != pipechecker::Severity::Info {
                                    let prefix = match issue.severity {
                                        pipechecker::Severity::Error => "❌",
                                        pipechecker::Severity::Warning => "⚠️",
                                        _ => "ℹ️",
                                    };
                                    println!("   {} {}", prefix, issue.message);
                                }
                            }
                        } else {
                            println!("   ✅ No issues found");
                        }
                        println!();
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Error checking {}: {}", file, e);
                total_errors += 1;
            }
        }
    }

    if format != "json" {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!(
            "Total: {} errors, {} warnings across {} files",
            total_errors,
            total_warnings,
            all_files.len()
        );
        println!("⏱️  Checked in {:.1}ms", total_start.elapsed().as_millis());
    }

    if total_errors > 0 || (strict && total_warnings > 0) {
        process::exit(1);
    }
}
