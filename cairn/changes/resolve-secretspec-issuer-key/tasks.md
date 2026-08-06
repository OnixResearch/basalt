# Tasks: Resolve the OIDC issuer key with SecretSpec

## Dependency and declaration

- [ ] [serial] Pin SecretSpec `v0.17.0` and its reviewed source identity through Cargo and Nix. r[ucan_token_flow.oidc.secretspec_dependency]
- [ ] [serial] Declare the UCAN issuer signing key as metadata under an explicit `basalt-oidc-exchange` scope and profile. r[ucan_token_flow.oidc.secretspec_issuer_key]

## Bounded loader

- [ ] [serial] Resolve the key only in the imperative shell and bound any provider process through an owned bounded worker. r[ucan_token_flow.oidc.secretspec_issuer_key]
- [ ] [serial] Disable SecretSpec write operations and plaintext caches. r[ucan_token_flow.oidc.secretspec_issuer_key]
- [ ] [serial] Support explicit key identifiers and a rotation overlap window without exposing private material. r[ucan_token_flow.oidc.secretspec_issuer_key]

## Fixtures and verification

- [ ] [parallel] Add positive key-load and rotation fixtures. r[ucan_token_flow.oidc.secretspec_fixtures]
- [ ] [parallel] Add negative missing, wrong-scope, timeout, oversized-output, and provider-failure fixtures. r[ucan_token_flow.oidc.secretspec_fixtures]

## Closeout

- [ ] [serial] Run focused tests, formatting, Clippy, release verification, and canonical Cairn gates. r[ucan_token_flow.oidc.secretspec_issuer_key] r[ucan_token_flow.oidc.secretspec_dependency]
- [ ] [serial] Sync accepted requirements and archive only after validation passes. r[ucan_token_flow.oidc.secretspec_fixtures]
