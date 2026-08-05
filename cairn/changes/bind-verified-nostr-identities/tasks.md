# Tasks: Bind verified Nostr identities to Basalt subjects

## Policy and subject core

- [ ] [serial] Wait for the reviewed UCAN `VerifiedNostrCredential` contract and pin its exact source. r[nostr_identity_bindings.verified_input]
- [ ] [serial] Add typed Nickel subject and authenticator contracts with generated artifact freshness. r[nostr_identity_bindings.policy_contract]
- [ ] [serial] Add pure canonical-key indexing, subject resolution, and deterministic diagnostics. r[nostr_identity_bindings.subject_resolution]
- [ ] [parallel] Add valid single-key, rotated-key, and canonical-order fixtures. r[nostr_identity_bindings.final_validation]
- [ ] [parallel] Add malformed, duplicate, ambiguous, inactive, revoked, and stale-policy fixtures. r[nostr_identity_bindings.final_validation]

## Authorization composition

- [ ] [serial] Add opaque `AuthenticatedSubject` construction from reviewed verified inputs. r[nostr_identity_bindings.subject_resolution]
- [ ] [serial] Add pure request-bound composition with existing Basalt UCAN decisions. r[nostr_identity_bindings.authority_separation]
- [ ] [parallel] Prove authentication cannot bypass denied, missing, stale, or mismatched authorization. r[nostr_identity_bindings.authority_separation]
- [ ] [parallel] Prove caller-supplied validity and NIP-05 identity claims fail closed. r[nostr_identity_bindings.verified_input]

## Lifecycle and evidence

- [ ] [serial] Add active, retired, and revoked key lifecycle handling. r[nostr_identity_bindings.key_lifecycle]
- [ ] [parallel] Add rotation continuity and revoked-key denial fixtures. r[nostr_identity_bindings.key_lifecycle]
- [ ] [serial] Add redacted BLAKE3 receipts with typed Nostr SHA-256 references. r[nostr_identity_bindings.receipts]
- [ ] [parallel] Add secret-field, digest-substitution, and weakened non-claim rejection fixtures. r[nostr_identity_bindings.receipts]

## Closeout

- [ ] [serial] Update public API, policy, authority, rotation, and non-claim documentation. r[nostr_identity_bindings.final_validation]
- [ ] [serial] Run policy, no-default, CLI, downstream, Cairn, and release verification. r[nostr_identity_bindings.final_validation]

## Blocked

- [ ] **Externally blocked.** This change depends on the reviewed UCAN
  `VerifiedNostrCredential` contract from external change
  `add-nostr-identity-adapter`. The vendored UCAN snapshot has no Nostr support.
  No implementation or fixtures were written against a contract that is not
  present. Re-drain after the external UCAN change is reviewed and vendored.
