## Phase 1: Authority and shared primitives

- [ ] [serial] Inventory the exact Nickel source fields, generated JSON fields, Rust DTOs, supported backend enum, parser preconditions, and active iddqd/UCAN/authority-core ownership boundaries. r[basalt.policy_authoring_contracts.runtime_boundary]
- [ ] [serial] Add shared exact-schema, normalized-id, supported-backend, unique-non-empty collection, map-entry alignment, and deterministic diagnostic contracts. r[basalt.policy_authoring_contracts.shared_primitives]
- [ ] [parallel] Add valid-minimal and primitive rejection fixtures for the shared prelude. r[basalt.policy_authoring_contracts.fixtures]

## Phase 2: Whole-policy source validation

- [ ] [serial] Harden the policy contract for exact schema, non-empty contracts, contract/backend key-id alignment, unique identities, supported backend metadata, and non-empty unique allowlists. r[basalt.policy_authoring_contracts.whole_policy]
- [ ] [serial] Add lexical resource-prefix and ability contracts matching Rust parser preconditions without implementing UCAN containment or authorization semantics. r[basalt.policy_authoring_contracts.lexical_values] r[basalt.policy_authoring_contracts.runtime_boundary]
- [ ] [serial] Coordinate source-level identity diagnostics with `iddqd-policy-identity-maps` while preserving the runtime identity-map check for external JSON. r[basalt.policy_authoring_contracts.layered_identity]

## Phase 3: Projection and external-input parity

- [ ] [serial] Extend policy freshness with a pure Rust normalized DTO/map parity check over the generated artifact, leaving file reads and Nickel invocation in a thin shell. r[basalt.policy_authoring_contracts.projection_parity]
- [ ] [parallel] Add positive source/projection fixtures for current Nickel and Steel backend metadata plus document read/write contracts. r[basalt.policy_authoring_contracts.fixtures]
- [ ] [parallel] Add negative Nickel and external-JSON fixtures for wrong schema, unknown backend, key-id mismatch, empty contracts/allowlists, duplicates, malformed lexical values, stale projection, and contract/DTO drift. r[basalt.policy_authoring_contracts.fixtures]

## Phase 4: Verification

- [ ] [serial] Run Nickel policy export/freshness, focused policy and identity-map tests, no-default and CLI-feature tests, release verification, Cairn validation, and proposal/design/tasks gates. r[basalt.policy_authoring_contracts.projection_parity] r[basalt.policy_authoring_contracts.fixtures]
- [ ] [serial] Document that Nickel policy conformance does not authorize requests, validate UCAN tokens, establish backend truth, or replace Rust receipt authority. r[basalt.policy_authoring_contracts.runtime_boundary]
