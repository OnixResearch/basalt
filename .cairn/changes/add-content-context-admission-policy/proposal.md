# Proposal: Add content-context admission policy

## Why

Basalt owns hardened Nickel policy and deterministic Rust enforcement. It does not yet define a closed policy for catalog records that can enter agent context.

External files, email, messages, and indexed records need different policy from authored notes. Screening and quarantine facts also need exact policy interpretation without becoming safety or authority claims.

## What Changes

- Add typed Nickel contracts for content source, trust tier, screening disposition, quarantine state, freshness, and context-use rules.
- Add a pure Rust evaluator for context-read, context-review, and catalog-search decisions.
- Bind each decision to exact subject, resource, payload, observation, policy, and action identities.
- Keep screening observations advisory.
- Compose optional UCAN grants as authority ceilings without expanding local policy.
- Emit redacted BLAKE3-bound receipts.

## Impact

Animus and other consumers can obtain one narrow policy decision before they expose indexed content to an agent.

Basalt does not store content, run classifiers, query catalogs, load keys, or execute agent actions.

## Dependencies

The first implementation uses fake caller-supplied facts. Live adoption requires a published resource-catalog contract.

The downstream Animus change consumes the exact Basalt policy and decision identity.

## Non-goals

- Do not classify content.
- Do not claim prompt-injection prevention.
- Do not store quarantine or revocation state.
- Do not grant model, tool, filesystem, process, network, memory, or workflow authority.
- Do not include raw content, classifier bodies, credentials, or paths in policy artifacts or receipts.
- Do not add a Spacedrive source dependency.
