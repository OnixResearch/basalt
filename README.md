# Basalt

Basalt is a standalone Rust/Nix repo for hardened UCAN contract boundaries across Nickel policy artifacts and Steel DTO evaluation backends.

Flow:

1. Author declarative contract policy in `contracts/policy.ncl`.
2. Export/check it with Nickel: `nickel export contracts/policy.ncl > contracts/generated/policy.json`.
3. Rust loads the generated JSON and enforces both:
   - the selected contract permits the requested resource/ability; and
   - the supplied UCAN capabilities authorize the same resource/ability via `ucan`.

This is intentionally a small functional-core / imperative-shell scaffold. Nickel owns policy shape and reviewable defaults; Rust owns runtime enforcement and receipts.

## Publication status

Basalt is under active development. Its current Cargo dependency on UCAN is
pinned to an authenticated SSH source, so anonymous source builds are not yet a
supported path. This access limitation does not strengthen any policy,
security, or verification claim. Before public consumers adopt Basalt, the
reviewed UCAN revision must be available through an anonymous HTTPS source or
an equivalently reviewed vendored boundary.

Package metadata is kept in `Cargo.toml` for downstream discovery: the crate stays AGPL-3.0-or-later, points registry/source tooling at the OnixResearch Basalt repository and this README, and uses UCAN/Nickel/Steel/policy keywords that match the crate purpose above. Basalt was formerly named `ucan-nickel-contracts`; that former name may remain in historical Cairn archives and stable policy schema identifiers.

## License boundary

This crate is licensed as `AGPL-3.0-or-later`, as declared in `Cargo.toml`; the complete text is in [`LICENSE`](LICENSE). Downstream applications should review that license before adopting the crate. Third-party dependencies retain their own terms. Earlier grants remain valid. This README does not grant an alternate license, compatibility exception, or legal advice for a specific deployment model.

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
use basalt::{CapabilityGrant, EnforcementRequest, default_policy, enforce};

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

Library consumers get enforcement APIs without enabling CLI parser dependencies by default. Operators who want the bundled `basalt` command should enable the `cli` feature:

```bash
cargo run --features cli -- enforce --help
```

## UCAN dependency source

The crate currently consumes `ucan` from an exact revision of its authoritative
repository through an authenticated SSH URL. That keeps the reviewed source
identity explicit, but it prevents anonymous builds and is not the intended
public-consumer boundary. For local UCAN development, keep the committed
manifest remote-based and use a consumer-local Cargo patch in the consuming
application's `.cargo/config.toml` instead:

```toml
[patch."ssh://git@github.com/OnixResearch/ucan.git"]
ucan = { path = "../ucan" }
```

## Verification

Use a fresh target directory and no inherited Rust compiler wrapper to avoid
shared stale Cargo artifact failures. This repo also checks in `.cargo/config.toml`
so Cargo uses the Git CLI for the authenticated SSH dependency and keeps
package preparation pointed at the reviewed remote UCAN revision instead of
the stale crates.io `ucan` package.

Run the single release rail before tagging or handing the crate to downstream applications:

```bash
nix develop -c rustc tools/release-verify.rs -o /tmp/basalt-release-verify
nix develop -c /tmp/basalt-release-verify
```

The same verifier is exposed as the default release flake check:

```bash
nix flake check -L
```

Because the crate currently has an authentication-required Git dependency, the sandboxed flake check prevalidates the Nickel policy check as a Nix input and emits a verifier receipt that marks Cargo-dependent rails as deferred. Run the direct verifier above for the full release rail before tagging or handoff. The checked-in GitHub Actions workflow runs this sandbox-safe `nix flake check -L` path without repository secrets; maintainers with UCAN dependency access should run the direct verifier in their trusted environment for full Cargo coverage.

The rail uses fresh `/tmp/basalt-release-*` Cargo target directories, clears `RUSTC_WRAPPER`, and enables Git CLI fetching for the authenticated UCAN dependency. It covers policy freshness, `cargo fmt --check`, AGPL license metadata preservation, no-default and CLI-feature test rails through `cargo nextest`, deterministic property/Steel-adapter/UCAN-token fixture tests, doctests, clippy for both feature sets, package validation, asserted package contents, the `../clankers/` downstream consumer fixture when present, and no-dependency library docs. Direct runs write `target/release-verify/summary.json` and `target/release-verify/summary.md`; Nix checks redirect those receipts into the check output, and CI uploads the summary files when the build result exposes them.

Individual checks are still useful while iterating:

```bash
nix build .#checks.$(nix eval --raw --impure --expr builtins.currentSystem).policy-export --no-link
nix develop -c env CARGO_TARGET_DIR=/tmp/basalt-target RUSTC_WRAPPER= CARGO_NET_GIT_FETCH_WITH_CLI=true cargo nextest run --no-default-features
nix develop -c env CARGO_TARGET_DIR=/tmp/basalt-target RUSTC_WRAPPER= CARGO_NET_GIT_FETCH_WITH_CLI=true cargo nextest run --features cli
nix develop -c env CARGO_TARGET_DIR=/tmp/basalt-target RUSTC_WRAPPER= CARGO_NET_GIT_FETCH_WITH_CLI=true cargo test --doc --no-default-features
# The feature-enabled test rail includes the external-style CLI smoke and negative CLI tests in tests/cli_feature.rs.
```

## Downstream compatibility

Basalt keeps a focused downstream compatibility runner for consumers that exercise the public Basalt boundary:

```bash
nix develop -c rustc --edition=2024 tools/downstream-compat.rs -o /tmp/basalt-downstream-compat
/tmp/basalt-downstream-compat --workspace-root .. --consumer clankers --consumer aspen --consumer onixos --require-present
```

The runner creates temporary worktrees under `target/downstream-compat/`, links the current Basalt checkout into each consumer, and runs narrow checks for Clankers, Aspen, and OnixOS. The legacy `--consumer onix-modules` spelling remains an alias for `onixos`. It is intentionally local-only: keep sibling checkouts under the same workspace root and do not require GitHub Actions tokens for this compatibility matrix.
