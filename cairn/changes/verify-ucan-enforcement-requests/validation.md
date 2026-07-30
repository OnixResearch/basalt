# Validation Status

## Result

**IN PROGRESS.** The previous UCAN dependency blockers are closed by the reviewed vendored source boundary.
The adapter, full rails, specification sync, and archive remain incomplete.

## Baseline Evidence

- The focused authority-core baseline passed with 8 tests.
- Canonical-policy Cairn validation passed.
- The proposal gate passed.
- The design gate passed.
- The tasks gate passed.

## Reviewed UCAN Boundary

Basalt records UCAN commit `34449ba1f1a9ef991225e4e6683b5d6aca8c07cf`.
The source worktree was clean at that revision.
The UCAN compact-token and holder-signed invocation changes are archived with date `2026-07-30`.

Basalt does not claim that this unpushed revision is available from a remote source.
The durable boundary is the checked-in `vendor/ucan/` snapshot.

`tools/sync-vendored-ucan.rs` imports only the pinned commit.
Its offline check covers the revision, file set, file modes, byte lengths, BLAKE3 identities, and required license files.
Its source-parity mode compares the snapshot with the exact Git commit.
The Nix check runs the checker tests and offline snapshot check.

## Closed Dependency Criteria

The reviewed UCAN snapshot provides these facts:

- borrowed compact-token verification with bounded secret ownership
- explicit key, proof, revocation, caveat, time, and limit inputs
- holder-signed exact request verification
- fixed operation-profile and request-binding types
- one final holder replay commit
- private, non-serde, non-clone live holder authorization

## Composition Invariant

Basalt policy work must finish before holder replay.
A Basalt policy denial must make zero replay calls.

The adapter will build the prospective authority decision and redacted receipt before final holder verification.
Fresh holder verification remains authoritative and performs the one replay commit.
After replay succeeds, Basalt can only move prebuilt values into the live capability.

## Remaining Evidence

The change remains active until these items pass:

- focused positive and negative adapter tests
- receipt redaction and non-authority tests
- formatting, workspace tests, and strict Clippy
- direct release verification and `nix flake check -L`
- canonical-policy Cairn validation and gates
- accepted specification sync
- archive execution with `CAIRN_ARCHIVE_DATE=2026-07-30`
