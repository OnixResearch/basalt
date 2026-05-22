# ucan-nickel-contracts

Standalone Rust/Nix repo for enforcing Nickel-authored contract policies with UCAN capability checks.

Flow:

1. Author declarative contract policy in `contracts/policy.ncl`.
2. Export/check it with Nickel: `nickel export contracts/policy.ncl > contracts/generated/policy.json`.
3. Rust loads the generated JSON and enforces both:
   - the selected contract permits the requested resource/ability; and
   - the supplied UCAN capabilities authorize the same resource/ability via `ucan`.

This is intentionally a small functional-core / imperative-shell scaffold. Nickel owns policy shape and reviewable defaults; Rust owns runtime enforcement and receipts.

## Quickstart

```bash
mkdir -p contracts/generated
nickel export contracts/policy.ncl > contracts/generated/policy.json
cargo test
cargo run -- enforce \
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

## Verification

```bash
nix develop -c nickel export contracts/policy.ncl >/tmp/ucan-nickel-policy.json
nix develop -c cargo fmt --check
nix develop -c cargo test
nix develop -c cargo clippy --all-targets -- -D warnings
```
