# Proposal: Embedding authorization policy

## Why

Optional vector retrieval in the stack uses embedding models. A local embedding model keeps query text and workspace content on the machine. A remote embedding provider can send that content outside the machine. The stack tenet is explicit permission: no ambient trust and no silent data egress.

The reviewed design reference is zvec-grep at `github.com/zvec-ai/zvec-grep` revision `309a66995809243d3274fa8b5bea63ab11dda1a0` (v0.2.1), Apache-2.0. It separates local MCP bearer authentication from explicit once-only or workspace authorization before remote embedding transfer. Basalt should adapt that separation as a typed policy surface.

## What Changes

- Add a typed embedding-authorization policy contract with workspace, provider, purpose, grant, and lifecycle facts. r[basalt.embedding_auth.boundary]
- Add pure Rust decision logic that evaluates supplied request, capability, workspace, provider, purpose, and grant facts. r[basalt.embedding_auth.decision]
- Add a local-first default: local embedding requires no grant. r[basalt.embedding_auth.local_first]
- Add workspace- and purpose-scoped grant admission with over-broad denial. r[basalt.embedding_auth.scope]
- Add fail-closed stale, revoked, and absent-grant denial with redacted BLAKE3 receipts. r[basalt.embedding_auth.freshness]
- Add positive and negative policy, decision, receipt, and Cairn verification. r[basalt.embedding_auth.verification]

## Lifecycle Prerequisite

None. Basalt keeps accepted specs under `.cairn/specs/`. This change adds an active package under `.cairn/changes/`.

## Impact

- **Contracts**: a new typed embedding-authorization contract and pure decision over supplied facts. The decision reuses the Basalt exact key, request, audience, purpose, policy, lifecycle, digest-role, and non-claim linkage discipline.
- **Consumers**: Animus embedding hosts and other retrieval consumers admit grants before any remote transfer. Local embedding never needs a grant.
- **Boundary**: no provider client, network transport, credential store, revocation event loop, or key distribution authority moves into the policy.

## Out of Scope

- Implementing embedders or provider clients.
- Model selection and local-model packaging.
- Network transport and retry for remote embedding calls.
- Porting the zvec-grep authorization implementation.
- Proving embedding quality, relevance, or provider correctness.

## Affected Specs

- `basalt-embedding-authorization`: boundary, decision, local-first, scope, freshness, and verification.
