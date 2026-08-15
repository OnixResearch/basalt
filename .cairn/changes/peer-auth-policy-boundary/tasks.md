## Policy contract

- [ ] [serial] Pin the reviewed Celld peer-protocol design revision and record the selected constraints and license in `README.md`. r[basalt.peer_auth.reference]
- [ ] [serial] Add the typed peer-auth policy contract (Nickel) with message, version, freshness, replay, and authority facts. r[basalt.peer_auth.policy]
- [ ] [serial] Add Rust enforcement that evaluates the policy over an authenticated-message verdict. r[basalt.peer_auth.enforcement]

## Adapter and boundary

- [ ] [serial] Add the typed peer-auth decision output with redacted public fields. r[basalt.peer_auth.decision]
- [ ] [parallel] Add positive fixture decisions for matching version, admitted freshness, exactly-once replay, and required authority. r[basalt.peer_auth.verification]
- [ ] [parallel] Add negative fixture decisions for wrong version, denied freshness, replay denial, missing authority, and secret fields. r[basalt.peer_auth.verification]
- [ ] [parallel] Add digest-role, capability-promotion, empty-message, and UCAN end-user path rejection cases. r[basalt.peer_auth.verification]
- [ ] [serial] Document consumer-owned transport, key custody, clock, replay, end-user, and authorization responsibilities. r[basalt.peer_auth.boundary]

## Closeout

- [ ] [serial] Run policy freshness, package, workspace, Clippy, Cairn, and Nix checks. r[basalt.peer_auth.verification]
- [ ] [serial] Record the reviewed revision, vectors, dependency revision, and license evidence. r[basalt.peer_auth.reference]
