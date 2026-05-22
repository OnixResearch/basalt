use std::env;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::{Command, ExitCode};

const TARGET_ENV: &str = "CARGO_TARGET_DIR";
const WRAPPER_ENV: &str = "RUSTC_WRAPPER";
const GIT_FETCH_ENV: &str = "CARGO_NET_GIT_FETCH_WITH_CLI";
const DEFAULT_TARGET_DIR: &str = "/tmp/ucan-nickel-contracts-release-target";
const PACKAGE_TARGET_DIR: &str = "/tmp/ucan-nickel-contracts-release-package-target";

struct Step {
    name: &'static str,
    program: &'static str,
    args: &'static [&'static str],
    target_dir: &'static str,
    cargo_env: bool,
}

fn main() -> ExitCode {
    let root = env::current_dir().unwrap_or_else(|error| {
        eprintln!("failed to read current directory: {error}");
        std::process::exit(1);
    });

    let steps = [
        Step {
            name: "policy freshness",
            program: "nix",
            args: &["build", "--no-link"],
            target_dir: DEFAULT_TARGET_DIR,
            cargo_env: false,
        },
        Step {
            name: "format",
            program: "cargo",
            args: &["fmt", "--check"],
            target_dir: DEFAULT_TARGET_DIR,
            cargo_env: true,
        },
        Step {
            name: "library tests",
            program: "cargo",
            args: &["test", "--no-default-features"],
            target_dir: DEFAULT_TARGET_DIR,
            cargo_env: true,
        },
        Step {
            name: "CLI feature tests",
            program: "cargo",
            args: &["test", "--features", "cli"],
            target_dir: DEFAULT_TARGET_DIR,
            cargo_env: true,
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
        },
        Step {
            name: "package validation",
            program: "cargo",
            args: &["package", "--allow-dirty", "--no-verify"],
            target_dir: PACKAGE_TARGET_DIR,
            cargo_env: true,
        },
        Step {
            name: "package contents",
            program: "cargo",
            args: &["package", "--allow-dirty", "--no-verify", "--list"],
            target_dir: PACKAGE_TARGET_DIR,
            cargo_env: true,
        },
        Step {
            name: "library docs",
            program: "cargo",
            args: &["doc", "--no-deps", "--no-default-features"],
            target_dir: DEFAULT_TARGET_DIR,
            cargo_env: true,
        },
    ];

    for step in steps {
        if let Err(error) = run_step(&root, &step) {
            eprintln!("release verification failed at {}: {error}", step.name);
            return ExitCode::from(1);
        }
    }

    println!("release verification passed");
    ExitCode::SUCCESS
}

fn run_step(root: &PathBuf, step: &Step) -> Result<(), String> {
    let mut args: Vec<OsString> = step.args.iter().map(OsString::from).collect();
    if step.name == "policy freshness" {
        args.push(policy_check_attr()?);
    }

    println!("==> {}", step.name);
    let mut command = Command::new(step.program);
    command.current_dir(root).args(args);
    if step.cargo_env {
        command
            .env(TARGET_ENV, step.target_dir)
            .env(WRAPPER_ENV, "")
            .env(GIT_FETCH_ENV, "true");
    }

    let status = command
        .status()
        .map_err(|error| format!("could not start {}: {error}", step.program))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{} exited with {status}", step.program))
    }
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
