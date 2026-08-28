# Tasks: Add content-context admission policy

## Nickel policy

- [ ] [serial] Add closed source, trust-tier, screening, quarantine, freshness, ability, and disposition contracts. r[basalt.content_context.policy]
- [ ] [parallel] Add valid and invalid Nickel fixtures for every closed value and rule family. r[basalt.content_context.fixtures]
- [ ] [serial] Generate and check the runtime policy projection without runtime Nickel evaluation. r[basalt.content_context.parity]

## Pure Rust enforcement

- [ ] [serial] Add nominal request, observation, decision, reason, and receipt types. r[basalt.content_context.request]
- [ ] [serial] Add pure fail-closed evaluation for `catalog/search`, `agent/context-read`, and `agent/context-review`. r[basalt.content_context.decision]
- [ ] [serial] Keep `review-required` distinct from allow and deny. r[basalt.content_context.decision]
- [ ] [serial] Intersect optional verified UCAN facts without expanding policy. r[basalt.content_context.authority]
- [ ] [serial] Add an exact decision-binding helper for Animus consumers. r[basalt.content_context.binding]

## Safety and verification

- [ ] [serial] Add redacted BLAKE3-bound receipts and raw-content leak rejection. r[basalt.content_context.receipts]
- [ ] [parallel] Add positive tests for authored, collaborative, admitted external, and review-required cases. r[basalt.content_context.fixtures]
- [ ] [parallel] Add negative tests for quarantine, unknown values, missing screening, classifier failure, stale facts, wrong ability, mismatched binding, and UCAN denial. r[basalt.content_context.fixtures]
- [ ] [parallel] Add external-JSON bypass tests that prove Rust rejects malformed policy without Nickel. r[basalt.content_context.parity]
- [ ] [serial] Document classifier, catalog, authority, content, and effect non-claims plus Spacedrive reference provenance. r[basalt.content_context.evidence]
- [ ] [serial] Run policy export, focused tests, Clippy, release verifier, Cairn validation, and proposal, design, and tasks gates. r[basalt.content_context.evidence]
