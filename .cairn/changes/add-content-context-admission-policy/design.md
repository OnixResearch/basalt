# Design: Content-context admission policy

## Boundary

Nickel owns reviewed policy shape and defaults. Rust owns deterministic runtime admission over already-loaded facts. UCAN supplies optional verified capability facts through existing Basalt boundaries.

Callers own catalog queries, content loading, classifier execution, clocks, quarantine storage, key resolution, revocation, replay, and protected effects.

## Policy model

The Nickel contract defines closed values for:

- source classes: `authored`, `collaborative`, `external`, and `unknown`
- trust tiers: `authored`, `collaborative`, `external`, and `unknown`
- screening dispositions: `not-screened`, `clear-observation`, `flagged-observation`, `classifier-unavailable`, and `screening-failed`
- quarantine states: `not-quarantined`, `quarantined`, `released`, and `unknown`
- freshness classes selected by reviewed policy
- abilities: `catalog/search`, `agent/context-read`, and `agent/context-review`

Policy rules map exact combinations to `allow`, `deny`, or `review-required`. `review-required` is not an allow.

## Request model

The pure evaluator receives:

- subject and audience identities
- resource and logical record identities
- exact payload identity
- requested ability
- source and trust facts
- screening profile and observation identities
- screening disposition
- quarantine and freshness facts
- policy identity
- optional verified UCAN grant facts
- explicit non-claims

The evaluator receives no raw content.

## Decision order

1. Validate request roles, schema, lexical bounds, and non-claims.
2. Reject unknown or unsupported values.
3. Reject active quarantine for context-read and catalog-search abilities.
4. Apply freshness and screening requirements.
5. Evaluate the closed Nickel-authored rule projection.
6. Intersect the result with optional UCAN authority facts.
7. Emit one exact decision and redacted receipt.

Local policy and UCAN can narrow authority. Neither can turn `review-required` or `deny` into `allow` without a new exact request and policy path.

## Screening boundary

A clear screening observation can satisfy only a policy precondition. It cannot prove safe content.

A flagged observation can deny or require review. It cannot prove malicious intent or content.

Classifier absence or failure uses explicit policy. No default can silently treat it as clear.

## Integration

Animus validates exact decision binding before context payload loading and provider dispatch.

The resource catalog retains trust and quarantine state. Basalt receives bounded facts and returns policy meaning only.

## Receipts

Receipts include safe role labels, hashes, disposition, reason class, policy identity, grant identities, freshness class, and non-claims.

They reject content bodies, prompts, email, messages, document text, classifier inputs or outputs, raw tokens, credentials, headers, URLs with secrets, and host paths.

## Verification

Nickel fixtures cover every closed value and rule family. Rust fixtures prove parity and fail closed when Nickel is bypassed.

Positive and negative tests cover quarantine, unknown values, screening absence, classifier failure, stale observations, wrong abilities, decision binding, UCAN narrowing, and receipt leaks.

## Claim boundary

A passing decision proves only that supplied facts met the selected policy. It does not prove content safety, classifier correctness, current catalog truth, authorization outside the exact ability, or release readiness.
