use clap::Parser;
use pipechecker::{audit_file, discover_workflows, load_config, AuditOptions, DiscoveryOptions};
use std::{
    fs,
    path::Path,
    process, thread,
    time::{Duration, Instant},
};

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

    /// Interactive terminal UI mode
    #[arg(long)]
    tui: bool,

    /// Output format (text, json)
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
}

fn main() {
    let cli = Cli::parse();

    if cli.install_hook {
        install_git_hook();
        return;
    }

    if cli.watch {
        watch_mode(&cli);
        return;
    }

    if cli.tui {
        let options = AuditOptions {
            check_docker_images: !cli.no_pinning,
            strict_mode: cli.strict,
            rules: Some(load_config().rules),
        };
        if let Err(e) = pipechecker::tui::run_tui(options) {
            eprintln!("TUI error: {}", e);
            process::exit(1);
        }
        return;
    }

    if cli.fix {
        println!("🔧 Auto-fix mode\n");

        let file = cli.file.unwrap_or_else(auto_detect_workflow);

        match pipechecker::fix::fix_file(&file) {
            Ok(result) => {
                if result.fixed == 0 {
                    println!("✅ No fixable issues found in {}", file);
                    println!("   All actions are already pinned or use local references");
                } else {
                    println!("✨ Fixed {} issue(s) in {}:\n", result.fixed, file);
                    for change in &result.changes {
                        if change.starts_with("  ") {
                            println!("{}", change);
                        }
                    }
                    println!("\n💡 Review the changes and commit them!");
                }
            }
            Err(e) => {
                eprintln!("❌ Error fixing {}: {}", file, e);
                process::exit(1);
            }
        }
        process::exit(0);
    }

    let options = AuditOptions {
        check_docker_images: !cli.no_pinning,
        strict_mode: cli.strict,
        rules: Some(load_config().rules),
    };

    if cli.all {
        audit_all_workflows(options, &cli.format, cli.strict, cli.quiet, cli.verbose);
        return;
    }

    let file = cli.file.unwrap_or_else(auto_detect_workflow);

    if cli.verbose {
        eprintln!("📄 Auditing: {}", file);
    }

    match audit_file(&file, options) {
        Ok(result) => {
            if cli.verbose {
                eprintln!("🔍 Auditors ran: syntax, dag, secrets, pinning");
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

            if cli.format == "json" {
                println!("{}", serde_json::to_string_pretty(&result).unwrap());
            } else {
                println!("Provider: {:?}", result.provider);
                println!("\n{}", result.summary);
                println!();

                for issue in &result.issues {
                    // In quiet mode, only show errors
                    if cli.quiet && issue.severity != pipechecker::Severity::Error {
                        continue;
                    }

                    let prefix = match issue.severity {
                        pipechecker::Severity::Error => "❌ ERROR",
                        pipechecker::Severity::Warning => "⚠️  WARNING",
                        pipechecker::Severity::Info => "ℹ️  INFO",
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
                    println!();

                    if let Some(suggestion) = &issue.suggestion {
                        println!("   💡 {}", suggestion);
                    }
                    println!();
                }

                // Only show timing in non-quiet mode
                if !cli.quiet {
                    println!("⏱️  Checked in {:.1}ms", result.elapsed.as_millis());
                }
            }

            let has_errors = result
                .issues
                .iter()
                .any(|i| i.severity == pipechecker::Severity::Error);

            if has_errors || (cli.strict && !result.issues.is_empty()) {
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
    use std::collections::HashMap;
    use std::time::SystemTime;

    eprintln!("👀 Watching for workflow changes...");
    eprintln!("   Press Ctrl+C to stop\n");

    let mut last_modified: HashMap<String, SystemTime> = HashMap::new();

    // Initial check
    let options = AuditOptions {
        check_docker_images: !cli.no_pinning,
        strict_mode: cli.strict,
        rules: Some(load_config().rules),
    };

    if cli.all {
        audit_all_workflows(options, &cli.format, cli.strict, cli.quiet, cli.verbose);
    } else if let Some(file) = &cli.file {
        let _ = audit_file(file, options);
    }

    loop {
        thread::sleep(Duration::from_secs(2));

        let files = if cli.all {
            discover_workflows(Path::new("."), &DiscoveryOptions::default())
        } else if let Some(file) = &cli.file {
            vec![file.clone()]
        } else {
            continue;
        };

        for file in &files {
            if let Ok(metadata) = fs::metadata(file) {
                if let Ok(modified) = metadata.modified() {
                    let changed = last_modified
                        .get(file)
                        .map(|&last| modified > last)
                        .unwrap_or(false);

                    if changed {
                        eprintln!("\n🔄 File changed: {}", file);
                        let opts = AuditOptions {
                            check_docker_images: !cli.no_pinning,
                            strict_mode: cli.strict,
                            rules: Some(load_config().rules),
                        };
                        let _ = audit_file(file, opts);
                    }

                    last_modified.insert(file.clone(), modified);
                }
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
    let config = load_config();
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

    for file in &all_files {
        if config.should_ignore(file) {
            continue;
        }

        let opts = AuditOptions {
            check_docker_images: options.check_docker_images,
            strict_mode: options.strict_mode,
            rules: options.rules.clone(),
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
