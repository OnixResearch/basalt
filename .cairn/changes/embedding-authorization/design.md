# Design: Embedding authorization policy

## Context

Basalt is the hardened UCAN/Nickel/Steel policy boundary. It owns exact capability evaluation, policy shape, and release receipts. Remote embedding is the one search path where query text or workspace content can leave the machine. That transfer needs an explicit, workspace-scoped, revocable grant.

zvec-grep at `github.com/zvec-ai/zvec-grep` revision `309a66995809243d3274fa8b5bea63ab11dda1a0` separates endpoint bearer authentication from embedding-provider authorization and requires explicit once-only or workspace authorization before remote transfer. Basalt adapts this separation into Nickel-policy and Rust-decision surfaces.

## Success Contract

Given a request, workspace, provider, purpose, and current grant facts, the policy returns a deterministic allow or deny decision and a redacted BLAKE3 receipt. Remote embedding transfer happens only with an explicit current grant.

## Decisions

### 1. Local-first default

Local embedding needs no grant. Remote embedding requires an explicit workspace-scoped grant. An absent grant denies remote transfer.

### 2. Separate endpoint auth from data egress

The local server bearer token protects the loopback endpoint. It does not authorize remote embedding. Data egress requires the embedding grant. This mirrors the zvec separation.

### 3. Keep grants workspace- and purpose-scoped

A grant binds one workspace root, one provider identity, one stated purpose, and one reviewable policy reference. An over-broad grant fails review.

### 4. Evaluate with the exact Basalt discipline

The decision reuses exact key, request, audience, purpose, policy, lifecycle, digest-role, and non-claim linkage validation. It reads no clock, key store, or transport.

### 5. Fail closed on drift and revocation

Revoked or stale grants deny. Consumers re-probe current authority before transfer. Absence of a current grant denies by default.

## Alternatives Considered

### Ambient allow for remote embedding

Rejected. It violates the explicit-permission tenet and the zvec reference discipline.

### Fold authorization into the MCP bearer token

Rejected. The bearer protects the local endpoint. It does not authorize data egress. The two boundaries are separate.

### Port the zvec authorization implementation

Rejected. Policy stays authored in Nickel and enforced by Basalt Rust, per repo authority.

## Risks and Controls

- Grant scope can grow. Purpose and workspace fields keep review explicit.
- Consumers can skip the gate. Transfer hosts must admit decisions before effects.
- Convenience pressure. The default remains fail-closed denial.

## Non-Claims

This design does not prove provider correctness, embedding quality, endpoint security, key distribution, revocation freshness beyond supplied facts, or release eligibility.
