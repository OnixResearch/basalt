## Context

`authority_core` is pure and accepts references to already-verified facts. The pinned UCAN revision is `2aad993027d48ff148028c537cdaf91f6e5285ca`.

The pinned API provides key, proof, revocation, caveat, replay, time, and limit boundaries. It does not provide safe compact-token secret ownership or holder proof-of-possession.

Basalt change `verify-ucan-enforcement-requests` depends on UCAN change `harden-compact-token-secret-ownership`.

## Decisions

### Decision: Block implementation on the pinned UCAN boundary

**Choice:** Do not implement, sync, or archive the Basalt adapter against the current pinned UCAN API.

**Rationale:** `CompactToken` owns a normal `String`. Borrowed conversion and `verify_compact_token_str` copy the bearer with `to_owned()`.

The pinned crate has no `zeroize` dependency, `ZeroizeOnDrop` implementation, borrowed token view, or equivalent secret-erasure contract. Basalt cannot erase the private `CompactToken` field.

### Decision: Require holder proof-of-possession

**Choice:** Require a UCAN invocation API that verifies a holder-signed canonical invocation binding.

**Rationale:** The current `InvocationRequest` contains only token, context, resource, ability, and caveat policies. `verify_invocation` derives the invoker from the token audience.

Audience equality proves token audience binding only. It does not prove that the request sender controls the holder key.

### Decision: Require typed operation mapping

**Choice:** Bind a typed normalized operation identity to the Basalt request and UCAN ability through a reviewed mapping.

**Rationale:** String equality between an action and an ability is not a general action-to-ability binding. The current UCAN readiness boundary leaves consumer action mapping downstream-owned.

### Decision: Omit identity for oversized credentials

**Choice:** Reject oversized credentials before parsing and emit no credential identity for that input.

**Rationale:** A hash over a fixed prefix plus length is not exact content identity. Hashing the complete oversized bearer would defeat bounded rejection.

### Decision: Keep authority live and borrowed

**Choice:** A future successful result will contain a private live capability that borrows the exact `EnforcementRequest`.

**Rationale:** The capability will have no public constructor and no `Serialize` or `Deserialize` implementation. Persisted evidence cannot recreate it.

### Decision: Build authority evidence internally

**Choice:** A future adapter will reject caller capability fields and validity booleans.

**Rationale:** Only successful UCAN shell checks can create finite facts for `adapt_enforcement_request` and `evaluate_authority`.

### Decision: Preserve functional core and thin shell

**Choice:** Pure helpers will validate typed bindings and hash redacted receipts. The shell will only call injected UCAN services and the existing Basalt core.

**Rationale:** This split keeps the future decision logic deterministic and removes ambient effects.

## Dependency Completion Criteria

UCAN change `harden-compact-token-secret-ownership` must provide these observable facts before Basalt implementation resumes:

1. Verification accepts borrowed secret bytes or an owned token that erases its bearer allocation on drop.
2. Generic key, proof, revocation, replay, caveat, time, and limit injection remains available.
3. A holder-signed invocation path proves possession for the audience key over a canonical request binding.
4. The API exposes a stable typed operation input or a reviewed mapping boundary for downstream action binding.

A future Basalt pin must name the reviewed UCAN commit that provides these facts.

## Risks / Trade-offs

- The active Basalt change cannot ship until the UCAN dependency satisfies the completion criteria.
- A later UCAN revision can require Basalt adapter design changes.
- Backend freshness and replay durability will remain caller-owned non-claims.
