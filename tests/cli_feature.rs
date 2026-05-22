use std::process::Command;

#[test]
fn cli_feature_exposes_enforce_smoke() {
    let Some(binary) = option_env!("CARGO_BIN_EXE_ucan-nickel-contracts") else {
        // Cargo does not build feature-gated binaries for `cargo test --no-default-features`.
        // The CLI smoke is asserted by the `--features cli` verification rail.
        return;
    };
    if !std::path::Path::new(binary).exists() {
        // Cargo can carry the binary path without materializing the feature-gated binary
        // on the no-default-features rail.
        return;
    }

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
