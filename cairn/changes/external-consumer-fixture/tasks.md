# Tasks

- [ ] [serial] Add a separate downstream-style consumer fixture or temp-workspace generator that depends on Basalt by public package path. r[crate_consumability.external_consumer_fixture.separate_consumer]
- [ ] [serial] Prove the fixture builds with default library features and without CLI-only dependencies. r[crate_consumability.external_consumer_fixture.separate_consumer.build]
- [ ] [serial] Exercise packaged policy, supplied policy JSON, public request construction, enforcement receipts, and fail-closed consumer mistakes. r[crate_consumability.external_consumer_fixture.public_api_behavior]
- [ ] [serial] Add release-verifier receipt coverage for the consumer fixture step including bounded output and any explicit sandbox deferral. r[crate_consumability.external_consumer_fixture.release_evidence]
- [ ] [serial] Run Cargo tests, consumer fixture, clippy, Nix release checks, Cairn validate/gates, and `git diff --check` before archive. r[crate_consumability.external_consumer_fixture.release_evidence.receipt_step]
