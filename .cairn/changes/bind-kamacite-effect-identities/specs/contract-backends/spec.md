# Contract Backends Specification Delta

## Purpose

Bind portable Basalt runtime-effect policy to exact Kamacite semantic operation identities without transferring authorization or execution authority.

## ADDED Requirements

### Requirement: Runtime-effect profiles bind an exact Kamacite cohort

r[contract_backends.runtime_effect_semantic_identity.profile] Basalt MUST pin one immutable Kamacite revision and exact semantic descriptor, compatibility, Preserves codec, and identity-domain contracts for portable runtime-effect policy.

#### Scenario: Supported cohort is loaded

- GIVEN the configured Kamacite revision, schemas, codec, and identity domains match the reviewed cohort
- WHEN semantic operation profile admission runs
- THEN Basalt MUST admit the cohort for portable effect-policy validation

#### Scenario: Cohort identity drifts

- GIVEN any required revision, schema, codec, or identity-domain value differs from the reviewed cohort
- WHEN profile admission runs
- THEN Basalt MUST reject the cohort before policy or request evaluation

### Requirement: Portable policy rules use exact semantic identities

r[contract_backends.runtime_effect_semantic_identity.policy] Every portable runtime-effect rule MUST bind one exact Kamacite `effect-operation` identity. Native scope and operation labels MUST remain mapping metadata.

#### Scenario: Exact descriptor identity is present

- GIVEN a rule contains a supported scope, native operation label, and admitted semantic operation identity
- WHEN policy validation runs
- THEN Basalt MUST retain all fields in their distinct roles

#### Scenario: Local label replaces semantic identity

- GIVEN a rule contains a known local operation label without an admitted semantic operation identity
- WHEN portable policy validation runs
- THEN Basalt MUST reject the rule instead of deriving identity from the label

### Requirement: Policy, request, decision, and receipt identities match

r[contract_backends.runtime_effect_semantic_identity.binding] Basalt MUST require one exact semantic operation identity across the selected policy rule, normalized request, authority decision, and permission receipt.

#### Scenario: Complete identity chain matches

- GIVEN policy, request, decision inputs, and receipt material reference the same admitted semantic operation identity
- WHEN runtime-effect evaluation runs
- THEN Basalt MAY continue subject to all existing policy and authority gates

#### Scenario: Name matches but identity differs

- GIVEN the local scope and operation labels match but the request identity differs from the selected rule identity
- WHEN runtime-effect evaluation runs
- THEN Basalt MUST deny before returning an allow decision

### Requirement: Compatibility is exact and directional

r[contract_backends.runtime_effect_semantic_identity.compatibility] Basalt MAY admit a different operation identity only through an exact directional Kamacite compatibility artifact that permits the requested execution context.

#### Scenario: Live execution compatibility is admitted

- GIVEN compatibility binds the exact source and target identities in the requested direction and permits `live-host-execution`
- WHEN compatibility admission runs
- THEN Basalt MAY evaluate the target under the selected compatibility policy

#### Scenario: Replay compatibility attempts live execution

- GIVEN compatibility permits replay only, reverses direction, or references another identity
- WHEN live effect authorization runs
- THEN Basalt MUST reject the substitution

### Requirement: Portable policy has one canonical carrier

r[contract_backends.runtime_effect_semantic_identity.carrier] Basalt MUST produce canonical Preserves policy bytes that bind the generated policy identity, descriptor cohort, adapter identity, Kamacite revision, and non-claims. JSON MUST remain a compatibility projection.

#### Scenario: Canonical carrier and JSON agree

- GIVEN the generated policy and admitted descriptor cohort are current
- WHEN the portable carrier is produced
- THEN canonical Preserves identity MUST bind the exact JSON projection and adapter receipt

#### Scenario: JSON is edited after carrier generation

- GIVEN the compatibility JSON bytes differ from the projection bound by the canonical carrier
- WHEN policy artifact validation runs
- THEN Basalt MUST reject the artifact as stale or modified

### Requirement: Semantic identity does not grant authority

r[contract_backends.runtime_effect_semantic_identity.authority] A matching semantic operation identity MUST NOT replace Basalt policy, UCAN facts, approval, currentness, consumer native validation, or host enforcement.

#### Scenario: Identity matches without authority

- GIVEN the descriptor identity matches but required policy or UCAN facts are missing
- WHEN Basalt evaluates the request
- THEN Basalt MUST deny and no consumer effect can be admitted from that identity alone

### Requirement: Permission evidence preserves identity roles

r[contract_backends.runtime_effect_semantic_identity.evidence] Permission receipts MUST bind semantic descriptor, compatibility, policy-carrier, adapter, request, decision, and non-claim identities without effect payloads or secrets.

#### Scenario: Safe receipt is produced

- GIVEN a runtime-effect decision completes with valid safe metadata
- WHEN receipt projection runs
- THEN each identity MUST retain its declared role and the receipt MUST exclude payloads, credentials, tokens, and raw host paths

### Requirement: Consumer mappings retain one semantic identity

r[contract_backends.runtime_effect_semantic_identity.consumers] Frozen Animus, Lattice, and Molten mappings MUST preserve the exact semantic operation identity while retaining consumer-owned parsing, currentness, adapters, execution, and observations.

#### Scenario: Consumer mapping is lossless

- GIVEN a frozen consumer request can represent every behavior-relevant operation field and exact semantic identity
- WHEN mapping validation runs
- THEN the mapping MUST pass without granting consumer authority to Basalt or Kamacite

#### Scenario: Consumer mapping loses identity

- GIVEN a mapping keeps only the local scope or operation label
- WHEN mapping validation runs
- THEN Basalt MUST classify the mapping as unsupported for portable authorization

### Requirement: Verification covers identity drift and overclaims

r[contract_backends.runtime_effect_semantic_identity.verification] The focused rail MUST cover positive exact-identity cases and negative wrong-domain, stale, name-equal, reverse, replay-only, lossy, missing-authority, secret, and overclaim cases.

#### Scenario: Focused semantic identity rail runs

- GIVEN policy artifacts, descriptor fixtures, compatibility fixtures, consumer mappings, and receipts are current
- WHEN the focused rail runs
- THEN it MUST report the supported identity boundary and authorization non-claims
