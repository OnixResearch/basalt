## Phase 1: Core decisions

- [ ] [serial] Implement the canonical key projection over policy-source identity, normalized inputs, and capability-context, backed by BLAKE3. r[contract_backends.policy_cache.projection]
- [ ] [serial] Implement the pure retention decision core: eviction target, promotion disposition, watermark trim plans, and deferred-release plans from supplied values. r[contract_backends.policy_cache.decision] r[contract_backends.policy_cache.deferral]
- [ ] [serial] Require an explicit capacity with no implicit default bound. r[contract_backends.policy_cache.bound]

## Phase 2: Standard store

- [ ] [depends:contract_backends.policy_cache.bound] Implement the bounded store that relies on core decisions. r[contract_backends.policy_cache.bound]
- [ ] [depends:contract_backends.policy_cache.deferral] Execute deferred releases strictly after the guard releases, with no user destructor inside the synchronization boundary. r[contract_backends.policy_cache.deferral]
- [ ] [serial] Keep revocation and policy evaluation freshness consumer-owned. Route cache misses to the existing evaluators unchanged. r[contract_backends.policy_cache.boundary]

## Phase 3: Fixtures and evidence

- [ ] [parallel] Add positive fixtures for projection stability, bounded retention, eviction order, promotion at and below threshold, and deferred-release ordering. r[contract_backends.policy_cache.verification]
- [ ] [parallel] Add negative and boundary fixtures for missing bounds, reversed watermarks, zero capacity, single slot, threshold 0 and 100, and decision invariance on hit. r[contract_backends.policy_cache.verification]
- [ ] [serial] Document the claim boundary and the recorded cross-repo retention contract in repo docs. r[contract_backends.policy_cache.boundary]
- [ ] [serial] Run policy freshness, workspace, Clippy, formatting, Cairn, and Nix checks, then record evidence. r[contract_backends.policy_cache.verification]
