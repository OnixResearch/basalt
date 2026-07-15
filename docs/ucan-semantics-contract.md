# UCAN semantics compatibility contract

Basalt's UCAN semantics fixtures test one bounded integration claim: decision facts supplied by a UCAN integration are rejected when any required fact is non-admitted, and only admitted effective grants are replayed through Basalt's existing Nickel-authored enforcement core.

## Identity boundary

Every fixture binds four immutable identities:

- Basalt's fetchable UCAN runtime dependency revision: `2aad993027d48ff148028c537cdaf91f6e5285ca`;
- the reviewed UCAN support-catalog revision: `e362c3fa2236d5b9d61423921a820cd12dd64e51`;
- that catalog's BLAKE3 identity: `02fb9fec563b5df9f40a58bbbddfd0bcf5abe74da373ae0b506712647e0ee6ad`;
- the generated Basalt policy artifact's BLAKE3 identity: `7d223f48abcf31c9ba932d7434babfa675122f393100547ff09a595ee1fe8adb`.

The two UCAN revisions are intentionally distinct. The runtime revision is the exact remotely fetchable library source currently linked by Basalt. The support-catalog revision identifies the accepted catalog vocabulary used to review this compatibility profile. This change does not claim that the unpublished catalog revision is Basalt's runtime dependency.

## Fixture and adapter flow

1. Human-reviewed fixtures are authored in Nickel under `contracts/ucan-semantics/fixtures/` and checked against `contracts/ucan-semantics/contracts.ncl`.
2. Nickel exports deterministic JSON into `fixtures/generated/`; the Nix freshness check rejects source/artifact drift and verifies that malformed contract fixtures cannot export.
3. `basalt::ucan_semantics` validates schema, dependency identity, policy identity, bounded capability counts, UCAN capability syntax, expectations, and non-claims.
4. The adapter rejects wildcard requests and any non-admitted delegation, caveat, proof-traversal, or replay fact.
5. Only admitted direct and effective delegated grants become an `EnforcementRequest` passed to the existing `basalt::enforce` function.

The adapter consumes UCAN decisions as data. It does not parse tokens, validate signatures, derive effective delegation, load proofs, resolve keys, evaluate caveats, inspect clocks, or own replay storage.

## Covered fixtures

| Fixture | Expected result |
|---|---|
| `positive/allow.ncl` | Matching policy and capability allow |
| `positive/capability-deny.ncl` | Matching policy with non-authorizing capability denies |
| `negative/over-delegation.ncl` | UCAN over-delegation fact denies before enforcement |
| `negative/wildcard-request.ncl` | Non-concrete operation request denies |
| `negative/missing-caveat-policy.ncl` | Missing caveat policy denies |
| `negative/stale-proof.ncl` | Stale proof traversal denies |
| `negative/replay-denied.ncl` | Replay backend failure denies |
| `malformed/missing-operation.ncl` | Nickel contract rejects empty operation |
| `malformed/weakened-non-claims.ncl` | Nickel contract rejects removed claim boundary |
| `malformed/incompatible-expected-result.ncl` | Nickel contract rejects an allow expectation for denied facts |
| `malformed/stale-catalog.ncl` | Nickel contract rejects catalog identity drift |

## Explicit non-claims

These fixtures and replay reports do **not** prove:

- revocation-backend freshness;
- key-resolution truth;
- UCAN cryptographic correctness;
- consumer replay-store durability;
- application-level authorization completeness.

UCAN or its caller remains authoritative for token cryptography, proof and caveat facts, explicit verification time, revocation inputs, replay state, and key resolution. Applications remain authoritative for the completeness of their policy and for acting on Basalt's receipt.

## Validation

Run the fixture freshness and replay checks from the repository root:

```bash
nix build .#checks.$(nix eval --raw --impure --expr builtins.currentSystem).ucan-semantics-fixtures --no-link
nix develop -c env CARGO_TARGET_DIR=/tmp/basalt-semantics-target RUSTC_WRAPPER= CARGO_NET_GIT_FETCH_WITH_CLI=true cargo test --locked --test ucan_semantics_contract --no-default-features
```

The direct release verifier and `nix flake check -L` include these rails. Package-content validation also requires the Nickel contract, generated fixtures, adapter module, and integration test.
