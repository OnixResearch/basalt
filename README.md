# ucan-nickel-contracts

Standalone Rust/Nix repo for enforcing Nickel-authored contract policies with UCAN capability checks.

Flow:

1. Author declarative contract policy in `contracts/policy.ncl`.
2. Export/check it with Nickel: `nickel export contracts/policy.ncl > contracts/generated/policy.json`.
3. Rust loads the generated JSON and enforces both:
   - the selected contract permits the requested resource/ability; and
   - the supplied UCAN capabilities authorize the same resource/ability via `ucan`.

This is intentionally a small functional-core / imperative-shell scaffold. Nickel owns policy shape and reviewable defaults; Rust owns runtime enforcement and receipts.

Package metadata is kept in `Cargo.toml` for downstream discovery: the crate stays AGPL-3.0-or-later, points registry/source tooling at the OnixResearch repository and this README, and uses UCAN/Nickel/policy keywords that match the crate purpose above.

## Quickstart

```bash
mkdir -p contracts/generated
nickel export contracts/policy.ncl > contracts/generated/policy.json
cargo test
cargo run --features cli -- enforce \
  --policy contracts/generated/policy.json \
  --contract document-write \
  --resource did:key:z6Mkresource/docs/report \
  --ability document/write \
  --capability did:key:z6Mkresource/docs/report=document/write
```

Expected CLI output for the sample request:

```json
{"allowed":true,"reason":"allowed"}
```

## Policy artifact consumption

The crate ships `contracts/policy.ncl` as the reviewed Nickel source and `contracts/generated/policy.json` as the packaged example policy artifact. The Rust library exposes that generated artifact as `DEFAULT_POLICY_JSON` plus `default_policy()` so downstream applications can smoke-test without a Nickel toolchain.

Production applications should either:

- supply their own reviewed Nickel-exported policy JSON to `parse_policy_json`; or
- intentionally use the packaged example policy via `default_policy()` when its contracts match their application.

The generated artifact is package-included in `Cargo.toml`. Release verification must run the Nickel freshness check so drift between `contracts/policy.ncl` and `contracts/generated/policy.json` fails before packaging:

```bash
nix build .#checks.$(nix eval --raw --impure --expr builtins.currentSystem).policy-export --no-link
```

## Library API

Applications can depend on the crate with default features and call the public enforcement API directly:

```rust
use ucan_nickel_contracts::{CapabilityGrant, EnforcementRequest, default_policy, enforce};

let policy = default_policy()?;
let request = EnforcementRequest::new(
    "document-write",
    "did:key:z6Mkresource/docs/report",
    "document/write",
)
.with_capability(CapabilityGrant::new(
    "did:key:z6Mkresource/docs/report",
    "document/write",
));

let receipt = enforce(&policy, &request)?;
assert!(receipt.is_allowed());
assert_eq!(receipt.reason(), "allowed");
# Ok::<(), Box<dyn std::error::Error>>(())
```

## CLI feature

Library consumers get enforcement APIs without enabling CLI parser dependencies by default. Operators who want the bundled `ucan-nickel-contracts` command should enable the `cli` feature:

```bash
cargo run --features cli -- enforce --help
```

## UCAN dependency source

The crate consumes `ucan` from the authoritative remote repository so downstream
applications use the same SSH-authenticated remote that this private repository
uses, instead of requiring this repo to be checked out next to a sibling
`../ucan` tree. For local UCAN development, keep the public manifest
remote-based and use a consumer-local Cargo patch in the consuming application's
`.cargo/config.toml` instead:

```toml
[patch."ssh://git@github.com/OnixResearch/ucan.git"]
ucan = { path = "../ucan" }
```

## Verification

Use a fresh target directory and no inherited Rust compiler wrapper to avoid
shared stale Cargo artifact failures. This repo also checks in `.cargo/config.toml`
so Cargo uses the Git CLI for the private SSH dependency and keeps package
preparation pointed at the reviewed remote UCAN revision instead of the stale
crates.io `ucan` package.

Run the single release rail before tagging or handing the crate to downstream applications:

```bash
nix develop -c rustc tools/release-verify.rs -o /tmp/ucan-nickel-release-verify
nix develop -c /tmp/ucan-nickel-release-verify
```

The rail uses fresh `/tmp/ucan-nickel-contracts-release-*` Cargo target directories, clears `RUSTC_WRAPPER`, and enables Git CLI fetching for the private UCAN dependency. It covers policy freshness, `cargo fmt --check`, library-only tests, CLI-feature tests, clippy for both feature sets, package validation, package contents, and no-dependency library docs.

Individual checks are still useful while iterating:

```bash
nix build .#checks.$(nix eval --raw --impure --expr builtins.currentSystem).policy-export --no-link
nix develop -c env CARGO_TARGET_DIR=/tmp/ucan-nickel-contracts-target RUSTC_WRAPPER= CARGO_NET_GIT_FETCH_WITH_CLI=true cargo test --no-default-features
nix develop -c env CARGO_TARGET_DIR=/tmp/ucan-nickel-contracts-target RUSTC_WRAPPER= CARGO_NET_GIT_FETCH_WITH_CLI=true cargo test --features cli
# The feature-enabled test rail includes the external-style CLI smoke in tests/cli_feature.rs.
```
