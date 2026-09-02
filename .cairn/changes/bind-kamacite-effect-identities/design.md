## Context

Basalt already owns portable runtime-effect policy over closed scopes and operations. Molten already carries Kamacite semantic operation identities across selected runtime surfaces.

The missing link is policy admission. Basalt currently cannot distinguish two behaviorally different operation descriptors that retain the same local label.

## Success Contract

A portable effect request passes only when policy, request, decision, receipt, and consumer mapping bind one exact admitted Kamacite semantic operation identity.

A changed descriptor produces a new identity. The old policy cannot authorize it without an explicit reviewed update or directional compatibility artifact.

## Decisions

### 1. Keep Kamacite decoding outside the Basalt policy core

A narrow adapter validates canonical Preserves bytes, identity domain, descriptor shape, Kamacite revision, and declared compatibility.

The adapter returns a Basalt-owned `SemanticOperationRef`. Generic Kamacite values and codec types do not enter policy evaluation APIs.

### 2. Match exact semantic identity before local policy rules

Each portable rule binds one `effect-operation` identity. Native scope, operation, and subject fields remain required for consumer enforcement.

Name equality, schema-shape equality, and matching local operation labels cannot replace identity equality.

### 3. Keep compatibility directional and contextual

A compatibility artifact must bind exact source and target identities. It must permit the `live-host-execution` context in the requested direction.

Replay-only, reverse, stale, or unsupported compatibility fails before Basalt returns an allow decision.

### 4. Publish one canonical policy carrier

Canonical Preserves bytes bind the generated Basalt policy identity, admitted descriptor cohort, adapter identity, Kamacite revision, and non-claims.

JSON remains the Rust runtime compatibility projection. Existing same-repository freshness checks remain necessary.

### 5. Keep authorization and execution owners unchanged

A semantic identity states which operation contract is referenced. Basalt still decides policy and UCAN facts.

Consumers still recheck current capabilities, native subjects, adapters, resources, and host state before an effect.

### 6. Require cross-stack fixtures without ambient sibling dependencies

Frozen fixtures cover Animus, Lattice, and Molten mapping shapes. Each fixture records immutable producer revisions and exact expected identities.

Release behavior must not depend on sibling working trees.

## Risks and Controls

- Descriptor drift can block valid effects. Explicit compatibility and migration fixtures make the transition reviewable.
- A consumer can map the wrong native operation. Consumer fixtures and current native enforcement remain required.
- Canonical identity can be mistaken for authorization. Receipts retain explicit non-claims and current authority requirements.
- A broad dependency can leak generic values inward. Dependency and API checks keep codec types in the adapter boundary.

## Non-Claims

This design does not prove handler correctness, policy correctness, semantic equivalence, runtime execution, effect success, sandboxing, or release eligibility.
