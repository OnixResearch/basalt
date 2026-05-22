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

```bash
nix develop -c nickel export contracts/policy.ncl >/tmp/ucan-nickel-policy.json
nix develop -c env CARGO_TARGET_DIR=/tmp/ucan-nickel-contracts-target RUSTC_WRAPPER= CARGO_NET_GIT_FETCH_WITH_CLI=true cargo fmt --check
nix develop -c env CARGO_TARGET_DIR=/tmp/ucan-nickel-contracts-target RUSTC_WRAPPER= CARGO_NET_GIT_FETCH_WITH_CLI=true cargo test --no-default-features
nix develop -c env CARGO_TARGET_DIR=/tmp/ucan-nickel-contracts-target RUSTC_WRAPPER= CARGO_NET_GIT_FETCH_WITH_CLI=true cargo test --features cli
nix develop -c env CARGO_TARGET_DIR=/tmp/ucan-nickel-contracts-target RUSTC_WRAPPER= CARGO_NET_GIT_FETCH_WITH_CLI=true cargo clippy --all-targets --features cli -- -D warnings
nix develop -c env CARGO_TARGET_DIR=/tmp/ucan-nickel-contracts-package-target RUSTC_WRAPPER= CARGO_NET_GIT_FETCH_WITH_CLI=true cargo package --allow-dirty --no-verify
```
