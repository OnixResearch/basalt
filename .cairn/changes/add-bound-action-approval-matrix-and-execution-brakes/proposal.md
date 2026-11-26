# Why

Cross-vault reconciliation connected `onix-labs-mneme`'s active `security-lab/bound-action-observation-v1` blocker directly to `OnixResearch/basalt`:

- `arm:presence-only` admitted **6 forbidden adversarial effects** (`target-substitution`, `payload-substitution`, `cross-task-approval`, `stale-generation`, `stale-policy`, `expiry-boundary`), whereas `arm:exact-binding` achieved **0/16 forbidden effects** and **0 false refusals**.
- `onix-insights-mneme/12-artex-dual-graph-and-brakes.adoc` and `x-bookmarks-mneme/papers/protocolis-solid-state-web.adoc` further motivate deterministic argv-prefix/effect execution brakes and post-effect observed-state digest verification (`13-mneme-post-merge-audit-invariants.adoc`, `16-ecosystem-swarm-invariants-and-lessons-2026-11.adoc` `kiln#118`).

# What Changes

- Extend `src/authority_core.rs` (`AuthorityInput`, `VerifiedGrantBinding`, `AuthorityDenyDiagnostic`) with `BoundApprovalEnvelope { task_id, generation, policy_revision, effect_digest, expires_at_exclusive }` and explicit fail-closed diagnostics for all 6 adversarial failure classes plus half-open expiry (`now >= expires_at_exclusive`).
- Extend `src/runtime_effect.rs` (`RuntimeEffectPolicy`, `EffectRequest`, `EffectPermissionReceipt`) with deterministic ARTEX argv-prefix/target brake rules and Protocolis-style post-effect observation digest binding before marking an effect `VerifiedComplete`.
- Add a 16-case adversarial regression matrix mirroring `security-lab/bound-action-observation-v1`.
