## Phase 1: Producer and policy contract

- [ ] [serial] Select and record one immutable published Kamacite revision, codec profile, descriptor schema, compatibility schema, and license boundary. r[contract_backends.runtime_effect_semantic_identity.profile]
- [ ] [depends:kamacite-semantic-profile] Extend typed Nickel runtime-effect policy with exact `effect-operation` identities and closed descriptor-cohort facts. r[contract_backends.runtime_effect_semantic_identity.policy]
- [ ] [depends:kamacite-semantic-profile] Add a canonical policy carrier and bind its compatibility JSON to the exact canonical Preserves identity. r[contract_backends.runtime_effect_semantic_identity.carrier]

## Phase 2: Pure admission and decisions

- [ ] [serial] Add Basalt-owned nominal semantic operation and compatibility input types without generic Kamacite values in policy APIs. r[contract_backends.runtime_effect_semantic_identity.profile]
- [ ] [depends:semantic-operation-types] Require exact semantic identity across policy rule, request, decision, and receipt construction. r[contract_backends.runtime_effect_semantic_identity.binding]
- [ ] [depends:semantic-operation-types] Admit only directional compatibility for the exact requested context and reject name-only substitution. r[contract_backends.runtime_effect_semantic_identity.compatibility]
- [ ] [depends:semantic-operation-binding] Preserve UCAN, approval, currentness, and consumer native enforcement as separate required gates. r[contract_backends.runtime_effect_semantic_identity.authority]

## Phase 3: Consumers and evidence

- [ ] [parallel] Add frozen Animus, Lattice, and Molten positive mappings that retain one exact semantic operation identity. r[contract_backends.runtime_effect_semantic_identity.consumers]
- [ ] [parallel] Add negative wrong-domain, stale-descriptor, name-equal mismatch, reverse-compatibility, replay-only compatibility, and lossy-mapping fixtures. r[contract_backends.runtime_effect_semantic_identity.verification]
- [ ] [depends:semantic-operation-binding] Extend metadata-only permission receipts with descriptor, compatibility, policy-carrier, and adapter identities. r[contract_backends.runtime_effect_semantic_identity.evidence]

## Phase 4: Documentation and gates

- [ ] [serial] Document the canonical policy carrier, consumer mapping contract, migration path, authority boundary, and non-claims. r[contract_backends.runtime_effect_semantic_identity.authority] r[contract_backends.runtime_effect_semantic_identity.evidence]
- [ ] [depends:semantic-operation-verification] Run policy freshness, positive and negative fixtures, focused Rust tests, Clippy, release verification, Cairn gates, and relevant Nix checks. r[contract_backends.runtime_effect_semantic_identity.verification]
