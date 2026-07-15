# Ucan Token Flow Specification

## Purpose

Defines the `ucan-token-flow` capability.

## Requirements

### Requirement: UCAN semantics fixture contract
r[ucan_semantics_contract.contract] Basalt MUST define a reviewed fixture contract for the UCAN authorization semantics consumed by Basalt policy enforcement.

#### Scenario: Valid semantics fixture passes
r[ucan_semantics_contract.positive]
- GIVEN a fixture declares policy contract ID, requested resource, requested ability, UCAN capability facts, caveat outcome, proof traversal status, replay status, expected enforcement result, and required non-claims
- WHEN the fixture contract is validated
- THEN validation MUST pass and preserve the expected Basalt reason class.

#### Scenario: Malformed semantics fixture fails closed
r[ucan_semantics_contract.negative]
- GIVEN a fixture omits the requested operation, weakens non-claims, or declares incompatible UCAN and Basalt result fields
- WHEN the fixture contract is validated
- THEN validation MUST fail with deterministic diagnostics.

### Requirement: Runtime adapter fixture replay
r[ucan_semantics_contract.runtime_adapter] Basalt MUST replay exported UCAN semantics fixtures through the Rust enforcement boundary without reimplementing UCAN authorization semantics in Nickel.

#### Scenario: Allow fixture reaches allowed receipt
r[ucan_semantics_contract.runtime_adapter.allow]
- GIVEN a fixture where Basalt policy permits the operation and UCAN capability facts authorize the same concrete operation
- WHEN the Rust fixture replay runs
- THEN Basalt MUST emit an allowed receipt with the expected reason.

#### Scenario: Negative fixture reaches denied receipt
r[ucan_semantics_contract.runtime_adapter.deny]
- GIVEN a fixture with over-delegation, wildcard request misuse, missing caveat policy, stale proof semantics, or replay denial
- WHEN the Rust fixture replay runs
- THEN Basalt MUST emit a denied or error receipt with the expected fail-closed reason.

### Requirement: UCAN dependency boundary
r[ucan_semantics_contract.boundary] Basalt documentation MUST state which UCAN semantic manifest, Git revision, or exported contract identity was validated and which claims remain outside Basalt.

#### Scenario: Dependency identity is visible
r[ucan_semantics_contract.boundary.identity]
- GIVEN a Basalt release or downstream compatibility receipt is produced
- WHEN a reviewer inspects the UCAN dependency section
- THEN the receipt SHOULD name the UCAN semantic manifest digest when available or the pinned UCAN revision when no manifest exists.

#### Scenario: Non-claims are preserved
r[ucan_semantics_contract.boundary.non_claims]
- GIVEN a Basalt enforcement compatibility report passes
- WHEN the supported claim is rendered
- THEN it MUST state that Basalt does not prove key-resolution truth, revocation backend freshness, cryptographic soundness beyond UCAN's boundary, consumer replay-store durability, or application-wide authorization completeness.

### Requirement: Validation evidence
r[ucan_semantics_contract.validation] The change MUST include contract validation, focused Rust fixture replay, and Cairn validation evidence before archive.

#### Scenario: Positive and negative evidence is saved
r[ucan_semantics_contract.validation.fixtures]
- GIVEN the semantics fixture matrix includes positive and negative cases
- WHEN the validation rail runs
- THEN positive fixtures MUST pass, negative fixtures MUST fail closed, and the saved receipt MUST bind fixture and policy identities.

### Requirement: UCAN/Basalt authority core contract
r[basalt.ucan_token_flow.ucan_basalt_authority_core.contract] Basalt MUST define pure authority core contracts for request identity, contract ID, resource, ability, verified UCAN grant refs, UCAN verification receipt refs, Basalt policy refs, caveat/replay dispositions, deny diagnostics, and bounded non-claims.

#### Scenario: Matching authority input admits request
r[basalt.ucan_token_flow.ucan_basalt_authority_core.fixtures.positive]
- GIVEN an authority input names a Basalt contract, resource, ability, request ref, verified UCAN grant refs, matching UCAN verification receipt refs, and matching Basalt policy refs
- WHEN the authority core evaluates the input
- THEN the decision MUST allow only the exact bounded request described by the input.

#### Scenario: Invalid authority input denies request
r[basalt.ucan_token_flow.ucan_basalt_authority_core.fixtures.negative]
- GIVEN an authority input has missing verified grants, wrong resource, wrong ability, stale verification refs, caveat denial, replay denial, missing policy refs, malformed request refs, or authority overclaims
- WHEN the authority core evaluates the input
- THEN the decision MUST deny with deterministic diagnostics naming the failed binding.

### Requirement: UCAN/Basalt authority validation is pure
r[basalt.ucan_token_flow.ucan_basalt_authority_core.validation] Basalt MUST implement authority decision validation as pure deterministic logic over in-memory request facts, verified UCAN fact refs, policy refs, caveat dispositions, and replay dispositions.

#### Scenario: Shell owns verification inputs
r[basalt.ucan_token_flow.ucan_basalt_authority_core.validation.shell]
- GIVEN a caller parses tokens, resolves keys, loads policy JSON, reads clocks, or consults revocation/replay backends
- WHEN the authority core evaluates a request
- THEN those side effects MUST remain outside the authority core.

### Requirement: UCAN verification remains external input
r[basalt.ucan_token_flow.ucan_basalt_authority_core.ucan_boundary] The authority core MUST consume verified UCAN facts and MUST NOT parse compact tokens, verify signatures, resolve DIDs, read clocks, fetch proofs, consult revocation stores, or evaluate replay stores.

#### Scenario: Raw token is insufficient
r[basalt.ucan_token_flow.ucan_basalt_authority_core.ucan_boundary.raw_token]
- GIVEN a request carries only a raw token or content reference without verified grant refs and UCAN verification receipt refs
- WHEN the authority core evaluates the request
- THEN the decision MUST deny before treating token possession as current authority.

### Requirement: Nickel remains policy review boundary
r[basalt.ucan_token_flow.ucan_basalt_authority_core.policy_boundary] Basalt SHOULD keep human-authored contract policy in Nickel while the authority core consumes deterministic Rust policy values.

#### Scenario: Core does not evaluate Nickel
r[basalt.ucan_token_flow.ucan_basalt_authority_core.policy_boundary.no_runtime_nickel]
- GIVEN a caller loads Basalt policy from a Nickel-exported runtime artifact
- WHEN it invokes the authority core
- THEN Nickel evaluation and filesystem reads MUST occur outside the authority core.

### Requirement: Compatibility adapters preserve existing Basalt requests
r[basalt.ucan_token_flow.ucan_basalt_authority_core.adapters] Basalt SHOULD provide adapters for existing Basalt enforcement requests and UCAN verification receipt references without changing public request fields.

#### Scenario: Existing enforcement request adapts
r[basalt.ucan_token_flow.ucan_basalt_authority_core.adapters.request]
- GIVEN an existing Basalt enforcement request uses current public contract, resource, ability, and capability fields
- WHEN an adapter validates the request
- THEN the adapter SHOULD preserve those fields while routing reusable authority checks through the shared core.

### Requirement: Authority core remains bounded
r[basalt.ucan_token_flow.ucan_basalt_authority_core.docs] Passing authority-core validation MUST NOT grant transport authority, storage authority, persistence authority, replay-store truth, key-resolution trust, downstream business authority, release eligibility, or whole-system safety.

#### Scenario: Boundary is visible
r[basalt.ucan_token_flow.ucan_basalt_authority_core.final_validation]
- GIVEN an authority-core decision allows a request
- WHEN an operator reads the supported claim
- THEN the report MUST state that the decision applies only to the exact contract/resource/ability/request binding supplied as verified input.
