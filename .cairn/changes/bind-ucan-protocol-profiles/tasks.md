## Phase 1: Current compact cohort

- [ ] [serial] Pin the archived UCAN protocol-profile catalog and reviewed vendored source cohort. r[basalt.ucan_token_flow.protocol_profiles.cohort]
- [ ] [depends:basalt.ucan_token_flow.protocol_profiles.cohort] Add a closed protocol-profile field to Nickel policy, Rust identity DTOs, raw enforcement inputs, and semantics fixtures. r[basalt.ucan_token_flow.protocol_profiles.selection]
- [ ] [depends:basalt.ucan_token_flow.protocol_profiles.selection] Bind the current raw adapter only to `onix-compact-ed25519-v1` before token parsing or replay. r[basalt.ucan_token_flow.protocol_profiles.compact]
- [ ] [parallel] Add positive compact-profile cases and negative missing, unknown, stale, unsupported, and profile/source mismatch cases. r[basalt.ucan_token_flow.protocol_profiles.tests]

## Phase 2: Profile-bound evidence

- [ ] [serial] Version UCAN enforcement and semantics fixture schemas and bind profile, source revision, support catalog, and Basalt policy into receipt BLAKE3 framing. r[basalt.ucan_token_flow.protocol_profiles.receipt]
- [ ] [parallel] Add receipt determinism, redaction, old-schema rejection, profile substitution, and persisted-receipt non-authority tests. r[basalt.ucan_token_flow.protocol_profiles.receipt]
- [ ] [serial] Update operator and vendored-source documentation with exact profile and upgrade boundaries. r[basalt.ucan_token_flow.protocol_profiles.docs]

## Phase 3: Adapter boundary

- [ ] [serial] Add the Basalt-owned verified UCAN authority input contract and project the current compact adapter into it. r[basalt.ucan_token_flow.protocol_profiles.adapter_contract]
- [ ] [parallel] Add negative caller-assertion, wrong-cohort, and wire-type leakage tests for the adapter contract. r[basalt.ucan_token_flow.protocol_profiles.tests]

## Phase 4: Validation

- [ ] [serial] Run policy freshness, vendored snapshot checks, focused positive and negative tests, formatting, Clippy, release verification, Cairn gates, and `nix flake check -L`. r[basalt.ucan_token_flow.protocol_profiles.validation]
