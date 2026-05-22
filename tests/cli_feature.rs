use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn cli_binary() -> Option<&'static str> {
    let binary = option_env!("CARGO_BIN_EXE_basalt")?;
    if Path::new(binary).exists() {
        Some(binary)
    } else {
        // Cargo can carry the binary path without materializing the feature-gated binary
        // on the no-default-features rail.
        None
    }
}

fn unique_policy_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "basalt-{name}-{}-{unique}.json",
        std::process::id()
    ))
}

#[test]
fn cli_feature_exposes_enforce_smoke() {
    let Some(binary) = cli_binary() else {
        // The CLI smoke is asserted by the `--features cli` verification rail.
        return;
    };

    let output = Command::new(binary)
        .arg("enforce")
        .arg("--policy")
        .arg("contracts/generated/policy.json")
        .arg("--contract")
        .arg("document-write")
        .arg("--resource")
        .arg("did:key:z6Mkresource/docs/report")
        .arg("--ability")
        .arg("document/write")
        .arg("--capability")
        .arg("did:key:z6Mkresource/docs/report=document/write")
        .output()
        .expect("CLI process should run");

    assert!(output.status.success(), "CLI failed: {output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        r#"{"allowed":true,"reason":"allowed"}"#
    );
    assert!(output.stderr.is_empty(), "CLI stderr should be empty");
}

#[test]
fn cli_feature_reports_denied_receipt_for_policy_denial() {
    let Some(binary) = cli_binary() else {
        return;
    };

    let output = Command::new(binary)
        .arg("enforce")
        .arg("--policy")
        .arg("contracts/generated/policy.json")
        .arg("--contract")
        .arg("document-write")
        .arg("--resource")
        .arg("did:key:z6Mkresource/private/report")
        .arg("--ability")
        .arg("document/write")
        .arg("--capability")
        .arg("did:key:z6Mkresource/private/report=document/write")
        .output()
        .expect("CLI process should run");

    assert!(output.status.success(), "policy denial should be a receipt");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(r#""allowed":false"#));
    assert!(stdout.contains("outside contract"));
}

#[test]
fn cli_feature_rejects_malformed_policy_file() {
    let Some(binary) = cli_binary() else {
        return;
    };
    let policy_path = unique_policy_path("malformed-policy");
    fs::write(&policy_path, "not-json").expect("policy fixture should write");

    let output = Command::new(binary)
        .arg("enforce")
        .arg("--policy")
        .arg(&policy_path)
        .arg("--contract")
        .arg("document-write")
        .arg("--resource")
        .arg("did:key:z6Mkresource/docs/report")
        .arg("--ability")
        .arg("document/write")
        .arg("--capability")
        .arg("did:key:z6Mkresource/docs/report=document/write")
        .output()
        .expect("CLI process should run");

    fs::remove_file(&policy_path).expect("policy fixture should clean up");
    assert!(!output.status.success(), "malformed policy should fail");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("PolicyJson"),
        "unexpected stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn cli_feature_rejects_bad_capability_syntax() {
    let Some(binary) = cli_binary() else {
        return;
    };

    let output = Command::new(binary)
        .arg("enforce")
        .arg("--policy")
        .arg("contracts/generated/policy.json")
        .arg("--contract")
        .arg("document-write")
        .arg("--resource")
        .arg("did:key:z6Mkresource/docs/report")
        .arg("--ability")
        .arg("document/write")
        .arg("--capability")
        .arg("did:key:z6Mkresource/docs/report")
        .output()
        .expect("CLI process should run");

    assert!(
        !output.status.success(),
        "bad capability syntax should fail"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("expected RESOURCE=ABILITY"),
        "unexpected stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
