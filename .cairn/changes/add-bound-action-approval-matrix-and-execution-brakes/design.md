# Design: Exact Bound-Action Approval Matrix & Deterministic Execution Brakes

## 1. Exact Binding & Half-Open Expiry (`src/authority_core.rs`)
- Introduce `BoundApprovalEnvelope` binding `task_id`, `generation`, `policy_revision`, `effect_digest` (target + canonical payload digest), and `expires_at_exclusive`.
- Emit typed fail-closed `AuthorityDenyDiagnostic` variants: `TargetSubstitution`, `PayloadSubstitution`, `CrossTaskApproval`, `StaleGeneration`, `StalePolicyRevision`, and `ExpiredAtBoundary`.

## 2. ARTEX Execution Brakes & Post-Effect Observation (`src/runtime_effect.rs`)
- Enforce explicit argv-prefix and target-scope brake rules before effect execution (`RuntimeEffectPolicy`).
- Bind a post-effect observed-state digest onto `EffectPermissionReceipt` (Protocolis verified-operation pattern) so diverged or unobserved outcomes cannot mark `VerifiedComplete`.

## Verification Plan
- `cargo test --all-targets` executing the full 16-case `bound-action-observation-v1` adversarial matrix (6 substitution/staleness/expiry attacks -> 0 forbidden effects; 5 benign cases -> 0 false refusals; post-effect divergence -> fail closed).
- `cargo clippy --all-targets -- -D warnings`.
