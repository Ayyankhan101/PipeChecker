use clap::Parser;
use pipecheck::{audit_file, AuditOptions, Config};
use std::{fs, path::Path, process, thread, time::Duration};

#[derive(Parser)]
#[command(name = "pipecheck")]
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

    /// Skip Docker image checks
    #[arg(long)]
    no_docker: bool,

    /// Enable strict mode (warnings as errors)
    #[arg(short, long)]
    strict: bool,
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
            check_docker_images: !cli.no_docker,
            strict_mode: cli.strict,
        };
        if let Err(e) = pipecheck::tui::run_tui(options) {
            eprintln!("TUI error: {}", e);
            process::exit(1);
        }
        return;
    }

    if cli.fix {
        eprintln!("🔧 Auto-fix mode");
        eprintln!("⚠️  This feature is experimental");
        eprintln!("   Currently supports:");
        eprintln!("   - Fixing indentation");
        eprintln!("   - Adding missing fields");
        eprintln!();
        eprintln!("❌ Auto-fix not yet implemented");
        eprintln!("   Coming in next version!");
        process::exit(1);
    }

    let options = AuditOptions {
        check_docker_images: !cli.no_docker,
        strict_mode: cli.strict,
    };

    if cli.all {
        audit_all_workflows(options, &cli.format, cli.strict);
        return;
    }

    let file = cli.file.unwrap_or_else(auto_detect_workflow);

    match audit_file(&file, options) {
        Ok(result) => {
            if cli.format == "json" {
                println!("{}", serde_json::to_string_pretty(&result).unwrap());
            } else {
                println!("Provider: {:?}", result.provider);
                println!("\n{}", result.summary);
                println!();

                for issue in &result.issues {
                    let prefix = match issue.severity {
                        pipecheck::Severity::Error => "❌ ERROR",
                        pipecheck::Severity::Warning => "⚠️  WARNING",
                        pipecheck::Severity::Info => "ℹ️  INFO",
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
            }

            let has_errors = result
                .issues
                .iter()
                .any(|i| i.severity == pipecheck::Severity::Error);

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

fn auto_detect_workflow() -> String {
    let patterns = vec![
        ".github/workflows/ci.yml",
        ".github/workflows/main.yml",
        ".github/workflows/build.yml",
        ".gitlab-ci.yml",
        ".circleci/config.yml",
    ];

    for pattern in patterns {
        if Path::new(pattern).exists() {
            eprintln!("✓ Auto-detected: {}", pattern);
            return pattern.to_string();
        }
    }

    // Check all files in .github/workflows/
    if let Ok(entries) = fs::read_dir(".github/workflows") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("yml")
                || path.extension().and_then(|s| s.to_str()) == Some("yaml")
            {
                let path_str = path.to_string_lossy().to_string();
                eprintln!("✓ Auto-detected: {}", path_str);
                return path_str;
            }
        }
    }

    eprintln!("❌ No workflow files found. Please specify a file:");
    eprintln!("   pipecheck <FILE>");
    eprintln!("\nSearched for:");
    eprintln!("  - .github/workflows/*.yml");
    eprintln!("  - .gitlab-ci.yml");
    eprintln!("  - .circleci/config.yml");
    process::exit(1);
}

fn install_git_hook() {
    let hook_path = Path::new(".git/hooks/pre-commit");

    if !Path::new(".git").exists() {
        eprintln!("❌ Not a git repository");
        process::exit(1);
    }

    let hook_content = r#"#!/bin/bash
# Pipecheck pre-commit hook

echo "🔍 Checking workflows with pipecheck..."

WORKFLOW_FILES=$(git diff --cached --name-only | grep -E '(\.github/workflows|\.gitlab-ci|\.circleci).*\.ya?ml$')

if [ -n "$WORKFLOW_FILES" ]; then
    if command -v pipecheck &> /dev/null; then
        pipecheck --all --strict
        if [ $? -ne 0 ]; then
            echo ""
            echo "❌ Workflow validation failed!"
            echo "Fix errors above or use 'git commit --no-verify' to skip"
            exit 1
        fi
        echo "✅ All workflows valid!"
    else
        echo "⚠️  pipecheck not installed, skipping"
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
        check_docker_images: !cli.no_docker,
        strict_mode: cli.strict,
    };

    if cli.all {
        audit_all_workflows(options, &cli.format, cli.strict);
    } else if let Some(file) = &cli.file {
        let _ = audit_file(file, options);
    }

    loop {
        thread::sleep(Duration::from_secs(2));

        let mut files = Vec::new();

        if cli.all {
            if let Ok(entries) = fs::read_dir(".github/workflows") {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("yml")
                        || path.extension().and_then(|s| s.to_str()) == Some("yaml")
                    {
                        files.push(path.to_string_lossy().to_string());
                    }
                }
            }
            if Path::new(".gitlab-ci.yml").exists() {
                files.push(".gitlab-ci.yml".to_string());
            }
            if Path::new(".circleci/config.yml").exists() {
                files.push(".circleci/config.yml".to_string());
            }
        } else if let Some(file) = &cli.file {
            files.push(file.clone());
        }

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
                            check_docker_images: !cli.no_docker,
                            strict_mode: cli.strict,
                        };
                        let _ = audit_file(file, opts);
                    }

                    last_modified.insert(file.clone(), modified);
                }
            }
        }
    }
}

fn audit_all_workflows(options: AuditOptions, format: &str, strict: bool) {
    let config = Config::load();
    let mut all_files = Vec::new();

    if let Ok(entries) = fs::read_dir(".github/workflows") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("yml")
                || path.extension().and_then(|s| s.to_str()) == Some("yaml")
            {
                all_files.push(path.to_string_lossy().to_string());
            }
        }
    }

    if Path::new(".gitlab-ci.yml").exists() {
        all_files.push(".gitlab-ci.yml".to_string());
    }

    if Path::new(".circleci/config.yml").exists() {
        all_files.push(".circleci/config.yml".to_string());
    }

    if all_files.is_empty() {
        eprintln!("❌ No workflow files found");
        process::exit(1);
    }

    eprintln!("Checking {} workflow file(s)...\n", all_files.len());

    let mut total_errors = 0;
    let mut total_warnings = 0;

    for file in &all_files {
        if config.should_ignore(file) {
            continue;
        }

        let opts = AuditOptions {
            check_docker_images: options.check_docker_images,
            strict_mode: options.strict_mode,
        };
        match audit_file(file, opts) {
            Ok(result) => {
                if format == "json" {
                    println!("{}", serde_json::to_string_pretty(&result).unwrap());
                } else {
                    println!("📄 {}", file);
                    println!("   Provider: {:?}", result.provider);

                    let errors = result
                        .issues
                        .iter()
                        .filter(|i| i.severity == pipecheck::Severity::Error)
                        .count();
                    let warnings = result
                        .issues
                        .iter()
                        .filter(|i| i.severity == pipecheck::Severity::Warning)
                        .count();

                    total_errors += errors;
                    total_warnings += warnings;

                    if errors > 0 || warnings > 0 {
                        println!("   {} errors, {} warnings", errors, warnings);
                        for issue in &result.issues {
                            if issue.severity != pipecheck::Severity::Info {
                                let prefix = match issue.severity {
                                    pipecheck::Severity::Error => "❌",
                                    pipecheck::Severity::Warning => "⚠️",
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
    }

    if total_errors > 0 || (strict && total_warnings > 0) {
        process::exit(1);
    }
}
