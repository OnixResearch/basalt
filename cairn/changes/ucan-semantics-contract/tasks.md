## Tasks

- [ ] [serial] Define the Basalt-facing UCAN semantics fixture contract. r[ucan_semantics_contract.contract]
- [ ] [serial] Add positive allow/deny fixtures for matching Basalt policy and UCAN capability behavior. r[ucan_semantics_contract.positive]
- [ ] [serial] Add negative fixtures for over-delegation, wildcard request misuse, missing caveat policy, stale proof semantics, and replay denial. r[ucan_semantics_contract.negative]
- [ ] [serial] Add focused Rust tests that replay exported semantics fixtures through Basalt enforcement. r[ucan_semantics_contract.runtime_adapter]
- [ ] [serial] Document the UCAN dependency boundary, manifest digest path, and non-claims for downstream consumers. r[ucan_semantics_contract.boundary]
- [ ] [serial] Run policy export freshness, focused tests, and Cairn validation/gates for this change. r[ucan_semantics_contract.validation]
