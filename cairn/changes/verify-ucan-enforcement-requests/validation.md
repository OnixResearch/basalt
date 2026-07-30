# Validation Status

## Result

**BLOCKED.** Keep this change active. Do not sync or archive it against pinned UCAN revision `2aad993027d48ff148028c537cdaf91f6e5285ca`.

## Baseline Evidence

- The focused authority-core baseline passed: 8 tests passed, 0 failed.
- Default Cairn validation first failed because the repository policy artifact predates `gate_policy.substance`.
- Canonical-policy validation then found an empty accepted `test-hardening` specification.
- Commit `fd0d59f` restored the accepted requirements from the archived source.
- Canonical-policy Cairn validation passed after that repair.
- Proposal, design, and tasks gates passed before implementation review.

## Exact Pinned-UCAN Blockers

### Bearer-copy and erasure blocker

At revision `2aad993027d48ff148028c537cdaf91f6e5285ca`:

- `src/token.rs` defines `CompactToken(String)`.
- `TryFrom<&str>` calls `value.to_owned()`.
- `verify_compact_token_str` calls `token.to_owned()`.
- `Cargo.toml` has no `zeroize` dependency.
- The compact token has no `Zeroize`, `ZeroizeOnDrop`, or erasing `Drop` implementation.

Basalt cannot erase the private owned string. A borrowed input would still create an unwiped bearer copy inside UCAN.

### Holder proof-of-possession blocker

At the same revision:

- `InvocationRequest` contains token, context, resource, ability, and caveat policies only.
- It contains no holder signature, nonce, canonical request envelope, or holder verification key input.
- `verify_invocation` uses the verified token audience as both invocation audience and invoker.
- The function constructs `invoker` directly from the audience DID.

Audience equality proves audience binding. It does not prove that the request sender controls the holder key.

### Action mapping blocker

The pinned readiness boundary states that consumer resource and action mapping remains downstream-owned. The invocation API accepts resource and ability strings only.

String equality between a caller action and a UCAN ability cannot prove a general action-to-ability mapping.

## Required Dependency

Basalt depends on UCAN change `harden-compact-token-secret-ownership`.

That change must provide safe compact-token secret ownership, holder proof-of-possession, and a typed operation mapping boundary. Basalt must then pin the reviewed UCAN commit.

## Discarded Draft

An uncommitted adapter draft copied the compact bearer into `CompactToken`, used audience equality as holder proof, and used action string equality. The draft was removed before commit.

## Blocked-State Validation

After the blocker record and explicit task markers were added:

- The focused authority-core test passed: 8 tests passed, 0 failed.
- Canonical-policy Cairn validation passed.
- The proposal gate passed.
- The design gate passed.
- The tasks gate passed with 11 explicit `[serial]` markers and no malformed dependency markers.

## Stop Condition

No implementation, focused adapter test matrix, sync, archive, workspace validation, Clippy rail, or release rail can complete safely against the current pin.
