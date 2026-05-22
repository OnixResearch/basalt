use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::{Duration, Instant};

const TARGET_ENV: &str = "CARGO_TARGET_DIR";
const WRAPPER_ENV: &str = "RUSTC_WRAPPER";
const GIT_FETCH_ENV: &str = "CARGO_NET_GIT_FETCH_WITH_CLI";
const RECEIPT_DIR_ENV: &str = "RELEASE_VERIFY_RECEIPT_DIR";
const POLICY_FRESH_ENV: &str = "RELEASE_VERIFY_POLICY_FRESH";
const CARGO_STEPS_ENV: &str = "RELEASE_VERIFY_CARGO_STEPS";
const DEFAULT_TARGET_DIR: &str = "/tmp/ucan-nickel-contracts-release-target";
const PACKAGE_TARGET_DIR: &str = "/tmp/ucan-nickel-contracts-release-package-target";
const DEFAULT_RECEIPT_DIR: &str = "target/release-verify";
const OUTPUT_TAIL_LIMIT: usize = 4_096;

const REQUIRED_PACKAGE_PATHS: &[&str] = &[
    "Cargo.lock",
    "Cargo.toml",
    "README.md",
    "contracts/generated/policy.json",
    "contracts/policy.ncl",
    "src/lib.rs",
    "src/main.rs",
    "tests/cli_feature.rs",
    "tests/consumer_boundary.rs",
    "tests/policy_artifact.rs",
    "tests/public_api.rs",
];

struct Step {
    name: &'static str,
    program: &'static str,
    args: &'static [&'static str],
    target_dir: &'static str,
    cargo_env: bool,
    kind: StepKind,
}

#[derive(Clone, Copy)]
enum StepKind {
    Normal,
    PolicyFreshness,
    PackageContents,
}

struct StepReceipt {
    name: &'static str,
    command: String,
    status: Option<i32>,
    success: bool,
    elapsed: Duration,
    stdout_tail: String,
    stderr_tail: String,
}

fn main() -> ExitCode {
    let root = env::current_dir().unwrap_or_else(|error| {
        eprintln!("failed to read current directory: {error}");
        std::process::exit(1);
    });
    let receipt_dir = env::var_os(RECEIPT_DIR_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_RECEIPT_DIR));
    if let Err(error) = fs::create_dir_all(&receipt_dir) {
        eprintln!(
            "failed to create receipt directory {}: {error}",
            receipt_dir.display()
        );
        return ExitCode::from(1);
    }

    let steps = [
        Step {
            name: "policy freshness",
            program: "nix",
            args: &["build", "--no-link"],
            target_dir: DEFAULT_TARGET_DIR,
            cargo_env: false,
            kind: StepKind::PolicyFreshness,
        },
        Step {
            name: "format",
            program: "cargo",
            args: &["fmt", "--check"],
            target_dir: DEFAULT_TARGET_DIR,
            cargo_env: true,
            kind: StepKind::Normal,
        },
        Step {
            name: "library tests",
            program: "cargo",
            args: &["nextest", "run", "--no-default-features"],
            target_dir: DEFAULT_TARGET_DIR,
            cargo_env: true,
            kind: StepKind::Normal,
        },
        Step {
            name: "CLI feature tests",
            program: "cargo",
            args: &["nextest", "run", "--features", "cli"],
            target_dir: DEFAULT_TARGET_DIR,
            cargo_env: true,
            kind: StepKind::Normal,
        },
        Step {
            name: "doctests",
            program: "cargo",
            args: &["test", "--doc", "--no-default-features"],
            target_dir: DEFAULT_TARGET_DIR,
            cargo_env: true,
            kind: StepKind::Normal,
        },
        Step {
            name: "library clippy",
            program: "cargo",
            args: &[
                "clippy",
                "--all-targets",
                "--no-default-features",
                "--",
                "-D",
                "warnings",
            ],
            target_dir: DEFAULT_TARGET_DIR,
            cargo_env: true,
            kind: StepKind::Normal,
        },
        Step {
            name: "CLI clippy",
            program: "cargo",
            args: &[
                "clippy",
                "--all-targets",
                "--features",
                "cli",
                "--",
                "-D",
                "warnings",
            ],
            target_dir: DEFAULT_TARGET_DIR,
            cargo_env: true,
            kind: StepKind::Normal,
        },
        Step {
            name: "package validation",
            program: "cargo",
            args: &["package", "--allow-dirty", "--no-verify"],
            target_dir: PACKAGE_TARGET_DIR,
            cargo_env: true,
            kind: StepKind::Normal,
        },
        Step {
            name: "package contents",
            program: "cargo",
            args: &["package", "--allow-dirty", "--no-verify", "--list"],
            target_dir: PACKAGE_TARGET_DIR,
            cargo_env: true,
            kind: StepKind::PackageContents,
        },
        Step {
            name: "library docs",
            program: "cargo",
            args: &["doc", "--no-deps", "--no-default-features"],
            target_dir: DEFAULT_TARGET_DIR,
            cargo_env: true,
            kind: StepKind::Normal,
        },
    ];

    let mut receipts = Vec::new();
    let mut failed = false;
    for step in steps {
        let receipt = run_step(&root, &step);
        if !receipt.success {
            eprintln!("release verification failed at {}", step.name);
            failed = true;
        }
        receipts.push(receipt);
        if failed {
            break;
        }
    }

    let all_passed = !failed;
    if let Err(error) = write_receipts(&receipt_dir, all_passed, &receipts) {
        eprintln!("failed to write release verification receipts: {error}");
        return ExitCode::from(1);
    }

    if all_passed {
        println!("release verification passed");
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn run_step(root: &Path, step: &Step) -> StepReceipt {
    if matches!(step.kind, StepKind::PolicyFreshness) && env_flag(POLICY_FRESH_ENV) {
        println!("==> {}", step.name);
        return StepReceipt {
            name: step.name,
            command: format!("{}=1 prevalidated by flake check input", POLICY_FRESH_ENV),
            status: Some(0),
            success: true,
            elapsed: Duration::ZERO,
            stdout_tail: String::new(),
            stderr_tail: String::new(),
        };
    }
    if step.cargo_env && env::var_os(CARGO_STEPS_ENV).is_some_and(|value| value == "0") {
        println!("==> {}", step.name);
        return StepReceipt {
            name: step.name,
            command: format!("{}=0 skipped in sandboxed flake check", CARGO_STEPS_ENV),
            status: Some(0),
            success: true,
            elapsed: Duration::ZERO,
            stdout_tail: "skipped: run the direct release verifier for Cargo-dependent rails".to_owned(),
            stderr_tail: String::new(),
        };
    }

    let mut args: Vec<OsString> = step.args.iter().map(OsString::from).collect();
    if matches!(step.kind, StepKind::PolicyFreshness) {
        match policy_check_attr() {
            Ok(attr) => args.push(attr),
            Err(error) => {
                return StepReceipt {
                    name: step.name,
                    command: format_command(step.program, &args),
                    status: None,
                    success: false,
                    elapsed: Duration::ZERO,
                    stdout_tail: String::new(),
                    stderr_tail: error,
                };
            }
        }
    }

    println!("==> {}", step.name);
    let start = Instant::now();
    let mut command = Command::new(step.program);
    command.current_dir(root).args(&args);
    if step.cargo_env {
        command
            .env(TARGET_ENV, step.target_dir)
            .env(WRAPPER_ENV, "")
            .env(GIT_FETCH_ENV, "true");
    }

    match command.output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
            let package_ok = if matches!(step.kind, StepKind::PackageContents) && output.status.success()
            {
                validate_package_contents(&stdout)
            } else {
                Ok(())
            };
            if output.status.success() {
                print!("{stdout}");
                eprint!("{stderr}");
            }
            let mut success = output.status.success();
            let mut stderr_tail = tail(&stderr);
            if let Err(error) = package_ok {
                success = false;
                if !stderr_tail.is_empty() {
                    stderr_tail.push('\n');
                }
                stderr_tail.push_str(&error);
                eprintln!("{error}");
            }
            StepReceipt {
                name: step.name,
                command: format_command(step.program, &args),
                status: output.status.code(),
                success,
                elapsed: start.elapsed(),
                stdout_tail: tail(&stdout),
                stderr_tail,
            }
        }
        Err(error) => StepReceipt {
            name: step.name,
            command: format_command(step.program, &args),
            status: None,
            success: false,
            elapsed: start.elapsed(),
            stdout_tail: String::new(),
            stderr_tail: format!("could not start {}: {error}", step.program),
        },
    }
}

fn validate_package_contents(package_list: &str) -> Result<(), String> {
    let missing = REQUIRED_PACKAGE_PATHS
        .iter()
        .filter(|path| !package_list.lines().any(|line| line.trim() == **path))
        .copied()
        .collect::<Vec<_>>();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "package contents missing required paths: {}",
            missing.join(", ")
        ))
    }
}

fn write_receipts(dir: &Path, passed: bool, receipts: &[StepReceipt]) -> Result<(), String> {
    let summary_path = dir.join("summary.json");
    let markdown_path = dir.join("summary.md");
    fs::write(summary_path, render_json(passed, receipts)).map_err(|error| error.to_string())?;
    fs::write(markdown_path, render_markdown(passed, receipts)).map_err(|error| error.to_string())?;
    Ok(())
}

fn render_json(passed: bool, receipts: &[StepReceipt]) -> String {
    let mut output = String::new();
    output.push_str("{\n");
    output.push_str(&format!("  \"passed\": {},\n", passed));
    output.push_str("  \"steps\": [\n");
    for (index, receipt) in receipts.iter().enumerate() {
        if index > 0 {
            output.push_str(",\n");
        }
        output.push_str("    {\n");
        output.push_str(&format!("      \"name\": {},\n", json_string(receipt.name)));
        output.push_str(&format!("      \"command\": {},\n", json_string(&receipt.command)));
        match receipt.status {
            Some(status) => output.push_str(&format!("      \"status\": {status},\n")),
            None => output.push_str("      \"status\": null,\n"),
        }
        output.push_str(&format!("      \"success\": {},\n", receipt.success));
        output.push_str(&format!(
            "      \"elapsed_ms\": {},\n",
            receipt.elapsed.as_millis()
        ));
        output.push_str(&format!(
            "      \"stdout_tail\": {},\n",
            json_string(&receipt.stdout_tail)
        ));
        output.push_str(&format!(
            "      \"stderr_tail\": {}\n",
            json_string(&receipt.stderr_tail)
        ));
        output.push_str("    }");
    }
    output.push_str("\n  ]\n}\n");
    output
}

fn render_markdown(passed: bool, receipts: &[StepReceipt]) -> String {
    let mut output = String::new();
    output.push_str("# Release Verification Summary\n\n");
    output.push_str(if passed { "Status: PASS\n\n" } else { "Status: FAIL\n\n" });
    output.push_str("| Step | Result | Status | Elapsed ms |\n");
    output.push_str("| --- | --- | --- | ---: |\n");
    for receipt in receipts {
        let status = receipt
            .status
            .map_or_else(|| "not-started".to_owned(), |status| status.to_string());
        output.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            receipt.name,
            if receipt.success { "PASS" } else { "FAIL" },
            status,
            receipt.elapsed.as_millis()
        ));
    }
    output
}

fn policy_check_attr() -> Result<OsString, String> {
    let output = Command::new("nix")
        .args([
            "eval",
            "--raw",
            "--impure",
            "--expr",
            "builtins.currentSystem",
        ])
        .output()
        .map_err(|error| format!("could not evaluate current Nix system: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "nix eval exited with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let system = String::from_utf8(output.stdout)
        .map_err(|error| format!("Nix system was not UTF-8: {error}"))?;
    Ok(OsString::from(format!(
        ".#checks.{}.policy-export",
        system.trim()
    )))
}

fn env_flag(name: &str) -> bool {
    env::var_os(name).is_some_and(|value| value == "1" || value == "true")
}

fn format_command(program: &str, args: &[OsString]) -> String {
    let mut command = program.to_owned();
    for arg in args {
        command.push(' ');
        command.push_str(&arg.to_string_lossy());
    }
    command
}

fn tail(input: &str) -> String {
    if input.len() <= OUTPUT_TAIL_LIMIT {
        return input.to_owned();
    }
    let mut start = input.len() - OUTPUT_TAIL_LIMIT;
    while !input.is_char_boundary(start) {
        start += 1;
    }
    input[start..].to_owned()
}

fn json_string(input: &str) -> String {
    let mut output = String::from("\"");
    for character in input.chars() {
        match character {
            '\"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            control if control.is_control() => {
                output.push_str(&format!("\\u{:04x}", control as u32));
            }
            other => output.push(other),
        }
    }
    output.push('\"');
    output
}
