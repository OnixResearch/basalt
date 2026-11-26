# Tasks

## Phase 1 — Exact Binding Envelope & Diagnostics
- [ ] Add `BoundApprovalEnvelope` and the 6 fail-closed `AuthorityDenyDiagnostic` variants (with half-open `now >= expires_at_exclusive`) in `src/authority_core.rs`.
- [ ] Add deterministic ARTEX argv-prefix/target brake rules and post-effect observation digest verification in `src/runtime_effect.rs`.

## Phase 2 — Adversarial Matrix & Verification
- [ ] Add the 16-case adversarial regression matrix mirroring `security-lab/bound-action-observation-v1` plus post-effect divergence tests.
- [ ] Verify `cargo test --all-targets` and `cargo clippy --all-targets -- -D warnings` pass cleanly.
