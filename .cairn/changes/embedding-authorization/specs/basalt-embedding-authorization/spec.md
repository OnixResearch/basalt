# Embedding Authorization Policy

## ADDED Requirements

### Requirement: Embedding authorization is a reviewable contract

r[basalt.embedding_auth.boundary] Basalt MUST provide embedding authorization as a typed Nickel contract. The contract MUST declare workspace, provider, purpose, grant, and lifecycle facts.

#### Scenario: Operator supplies a policy

- GIVEN an operator authors an embedding-authorization policy with explicit workspace, provider, purpose, grant, and lifecycle facts
- WHEN policy typecheck and review run
- THEN the policy MUST typecheck
- AND every declared fact MUST be explicit.

#### Scenario: Policy omits a required fact

- GIVEN an embedding-authorization policy omits a workspace, provider, purpose, grant, or lifecycle fact
- WHEN policy typecheck runs
- THEN typecheck MUST fail
- AND no default fact MUST be assumed.

### Requirement: Local embedding needs no grant

r[basalt.embedding_auth.local_first] A local embedding operation MUST require no remote grant and MUST remain allowed by default.

#### Scenario: Local embedding runs

- GIVEN an embedding operation stays on the local machine
- WHEN the policy evaluates the operation
- THEN the policy MUST allow it without any grant fact.

### Requirement: Remote embedding requires an explicit current grant

r[basalt.embedding_auth.decision] Remote embedding MUST require an explicit, workspace-scoped, current grant. An absent, stale, or revoked grant MUST deny remote transfer.

#### Scenario: Current grant allows transfer

- GIVEN a request cites one workspace root, one provider identity, one purpose, and a current grant
- WHEN the policy evaluates the request
- THEN the policy MUST allow remote embedding
- AND it MUST produce a redacted BLAKE3 receipt.

#### Scenario: Grant is absent

- GIVEN a remote embedding request has no grant fact
- WHEN the policy evaluates the request
- THEN the policy MUST deny remote transfer.

### Requirement: Grants are workspace- and purpose-scoped

r[basalt.embedding_auth.scope] A grant MUST bind one workspace root, one provider identity, and one purpose. It MUST NOT authorize unrelated egress.

#### Scenario: Scope is explicit

- GIVEN a grant binds one workspace, one provider, and one purpose
- WHEN review runs
- THEN the grant MUST be admitted only for that workspace and purpose.

#### Scenario: Scope is over-broad

- GIVEN a grant binds no workspace or no purpose
- WHEN review runs
- THEN the grant MUST be rejected.

### Requirement: Embedding decisions stay deterministic

r[basalt.embedding_auth.freshness] The decision MUST evaluate supplied request, capability, workspace, provider, purpose, and grant facts. It MUST read no clock, key store, transport, or revocation store. Stale, revoked, or absent grant facts MUST fail closed.

#### Scenario: Revoked grant denies

- GIVEN a request cites a grant marked revoked or stale
- WHEN the policy evaluates the request
- THEN the policy MUST deny remote transfer.

### Requirement: Embedding authorization has positive and negative verification

r[basalt.embedding_auth.verification] The focused rail MUST cover local-allow and current-grant allow, plus negative absent-grant, revoked-grant, stale-grant, wrong-provider, over-broad-scope, malformed-policy, and authority-overclaim cases.

#### Scenario: Embedding-authorization rail passes

- GIVEN all required policy, positive, and negative fixtures are present
- WHEN the focused rail runs
- THEN decisions MUST be deterministic
- AND invalid facts MUST fail without transfers or promoted claims.
