## Context

`authority_core` is pure and accepts references to already-verified facts. The pinned UCAN API provides explicit key, proof, revocation, caveat, replay, time, and limit boundaries.

The pinned `verify_invocation_with_replay` function uses `VerificationContext`. That context owns concrete key and proof collections and does not accept a caller-supplied revocation checker.

## Decisions

### Decision: Compose the strongest pinned UCAN shell path

**Choice:** Use `verify_compact_token_with_resolvers_and_revocations`, caveat-aware authorization, the verified invocation bridge, and caller replay admission.

**Rationale:** This composition preserves every required injected boundary. Direct `verify_invocation_with_replay` use would omit caller-supplied revocation and generic resolver or proof-store support.

### Decision: Keep authority live and borrowed

**Choice:** A successful result contains a private live capability that borrows the exact `EnforcementRequest`.

**Rationale:** The capability has no public constructor and no `Serialize` or `Deserialize` implementation. A receipt or copied preverified fields cannot recreate it.

### Decision: Separate unverifiable input from evaluated denial

**Choice:** Verification failures return a typed error with no `AuthorityDecision`. Verified caveat, scope, replay, or Basalt policy failures return typed denials.

**Rationale:** Basalt must not claim an authority decision when UCAN input or a required backend result is unavailable.

### Decision: Build authority evidence internally

**Choice:** The adapter rejects caller capability fields. It creates exact capability and receipt references only after the UCAN shell admits the invocation.

**Rationale:** Caller booleans, token possession, content references, and unverified grants never become authority.

### Decision: Redact durable evidence

**Choice:** Receipts contain stable reason codes, BLAKE3 references, limits, verification time, and explicit non-claims. They contain no compact token, proof token, key, caveat payload, issuer, holder, resource, ability, or action text.

**Rationale:** Persisted receipts remain untrusted evidence. Their deterministic identity supports comparison without carrying authority or credential bytes.

### Decision: Preserve functional core and thin shell

**Choice:** Pure helpers validate exact bindings, classify outcomes, derive references, build authority evidence, and hash receipts. The shell only calls UCAN traits and the existing Basalt core.

**Rationale:** The decision logic remains deterministic and can be tested without ambient services.

## Risks / Trade-offs

- The adapter is synchronous because the pinned UCAN shell traits are synchronous.
- Backend freshness and durability remain caller-owned facts.
- A redacted receipt cannot diagnose raw credential content.
- The live capability ends when its borrowed request lifetime ends.
