# Content Context Policy Delta

## ADDED Requirements

### Requirement: Nickel defines a closed content-context policy

r[basalt.content_context.policy] Basalt MUST define typed Nickel contracts for source class, trust tier, screening disposition, quarantine state, freshness class, requested ability, and policy disposition.

#### Scenario: Valid policy exports

- GIVEN a policy uses supported values, exact map identities, non-empty rule sets, deterministic precedence, and explicit non-claims
- WHEN Nickel evaluates the policy
- THEN export MUST produce the supported runtime projection
- AND export MUST NOT authorize a specific request.

#### Scenario: Policy contains an unknown or ambiguous value

- GIVEN a rule uses an unknown tier, screening result, quarantine state, ability, disposition, duplicate identity, or ambiguous precedence
- WHEN Nickel evaluates the policy
- THEN export MUST fail before generated JSON changes.

### Requirement: Requests contain bounded metadata only

r[basalt.content_context.request] Basalt MUST evaluate content-context requests over explicit bounded identities and trust facts without accepting raw content, prompts, classifier bodies, credentials, or host paths.

#### Scenario: Valid request is admitted for evaluation

- GIVEN a request supplies subject, audience, resource, record, payload, ability, source, trust, screening, quarantine, freshness, policy, grant, and non-claim facts
- WHEN request validation runs
- THEN Basalt MUST preserve each fact in its nominal role.

#### Scenario: Request contains raw content

- GIVEN a request embeds document, email, message, prompt, classifier, credential, header, token, or host-path material
- WHEN request validation runs
- THEN Basalt MUST reject it before policy evaluation.

### Requirement: Runtime decisions are pure and fail-closed

r[basalt.content_context.decision] Basalt MUST evaluate `catalog/search`, `agent/context-read`, and `agent/context-review` through pure deterministic logic that returns allow, deny, or review-required.

#### Scenario: External context meets policy

- GIVEN current trust facts and optional authority satisfy one exact rule for the requested ability
- WHEN Rust evaluation runs
- THEN Basalt MUST return the rule disposition and a deterministic reason class
- AND evaluation MUST perform no file, database, clock, classifier, policy-loading, credential, network, persistence, logging, or UI effect.

#### Scenario: Screening is required but unavailable

- GIVEN policy requires a screening observation and the request reports not-screened, classifier-unavailable, screening-failed, stale, or malformed facts
- WHEN evaluation runs
- THEN Basalt MUST deny or require review according to explicit policy
- AND it MUST NOT treat the content as clear.

### Requirement: Screening observations do not become safety claims

r[basalt.content_context.screening_boundary] Basalt MUST treat clear, flagged, unavailable, and failed screening results as policy inputs rather than proof of safety, truth, maliciousness, or authority.

#### Scenario: Clear observation satisfies a precondition

- GIVEN a current clear observation matches the selected screening profile
- WHEN a rule uses that observation
- THEN the observation MAY satisfy only the declared rule precondition
- AND the receipt MUST retain the classifier and safety non-claims.

#### Scenario: Flagged observation is denied

- GIVEN a flagged observation causes a deny result
- WHEN the decision is displayed
- THEN Basalt MUST describe a policy denial over supplied facts
- AND it MUST NOT claim that the content is malicious.

### Requirement: UCAN authority can only narrow admission

r[basalt.content_context.authority] Optional verified UCAN facts MUST intersect with content-context policy and MUST NOT expand a deny or review-required result into allow.

#### Scenario: Policy and grant both allow context-read

- GIVEN policy allows the exact context-read request and a verified grant authorizes the same resource and ability
- WHEN authority composition runs
- THEN Basalt MAY return allow for that exact request.

#### Scenario: Grant is broader than policy

- GIVEN a verified grant covers a broader resource or ability while local policy denies or requires review
- WHEN composition runs
- THEN the final result MUST remain deny or review-required.

### Requirement: Consumer binding is exact

r[basalt.content_context.binding] Basalt MUST expose a pure binding check for subject, action, resource, record, payload, trust, screening, quarantine, freshness, policy, and decision identities.

#### Scenario: Animus validates a matching decision

- GIVEN an Animus context request matches every bound decision field
- WHEN binding validation runs
- THEN the decision MAY satisfy the Basalt policy requirement for that request
- AND Animus MUST retain its own payload, provider, tool, and workflow authority.

#### Scenario: One binding differs

- GIVEN any required field differs or is absent
- WHEN binding validation runs
- THEN Basalt MUST reject the decision as stale or mismatched.

### Requirement: Receipts contain safe metadata only

r[basalt.content_context.receipts] Content-context receipts MUST contain bounded labels, hashes, disposition, reason class, freshness, policy, grant identities, and non-claims without raw sensitive material.

#### Scenario: Safe decision receipt validates

- GIVEN one request reaches allow, deny, or review-required
- WHEN receipt projection runs
- THEN the receipt MUST bind the exact request, policy, observation, authority, and decision identities.

#### Scenario: Receipt contains raw material

- GIVEN a receipt includes content, prompt, classifier, token, credential, header, provider, URL-secret, or host-path material
- WHEN receipt validation runs
- THEN Basalt MUST reject the receipt with a typed leak diagnostic.

### Requirement: Nickel and Rust meaning stay aligned

r[basalt.content_context.parity] Basalt MUST check Nickel export freshness and Rust DTO meaning for every supported policy rule and MUST reject malformed external JSON when Nickel is bypassed.

#### Scenario: Fresh export matches Rust

- GIVEN the reviewed Nickel source exports successfully
- WHEN parity validation loads the generated artifact
- THEN Rust MUST admit the same closed values, identities, and rule dispositions.

#### Scenario: External JSON bypasses source checks

- GIVEN external JSON contains unknown values, duplicate rules, invalid precedence, missing non-claims, or unsupported dispositions
- WHEN Rust parses the artifact
- THEN Rust MUST reject it before enforcement.

### Requirement: Fixtures cover acceptance and rejection

r[basalt.content_context.fixtures] Basalt MUST maintain positive and negative Nickel, Rust, binding, authority, parity, and receipt fixtures for content-context policy.

#### Scenario: Closeout evidence is complete

- GIVEN maintainers intend to archive the change
- WHEN closeout review runs
- THEN evidence MUST name policy export, focused tests, Clippy, release verification, Cairn, positive, and negative checks
- AND each skipped check MUST include an exact blocker.

### Requirement: Evidence keeps bounded claims

r[basalt.content_context.evidence] Content-context evidence MUST state that it does not prove content safety, semantic truth, classifier correctness, catalog truth, authorization beyond the exact ability, protected-effect completion, or release readiness.

#### Scenario: Decision allows context-read

- GIVEN an exact request receives allow
- WHEN the result is recorded
- THEN it MUST describe a bounded policy decision over supplied facts
- AND it MUST NOT describe the content as safe, true, or trusted globally.
