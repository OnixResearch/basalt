# Crate Consumability Specification Delta

## Purpose

Extends `crate-consumability` with a true external consumer fixture.

## Requirements

### Requirement: The release rail MUST build a separate consumer fixture against Basalt's public package surface.

r[crate_consumability.external_consumer_fixture.separate_consumer] The release rail MUST build a separate consumer fixture against Basalt's public package surface rather than relying only on in-package integration tests.

#### Scenario: Separate consumer build

r[crate_consumability.external_consumer_fixture.separate_consumer.build]

- GIVEN a downstream-style fixture depends on Basalt
- WHEN the fixture builds with the default library feature set
- THEN it imports `basalt` through public APIs without requiring CLI-only dependencies or private crate internals

### Requirement: The consumer fixture MUST exercise policy consumption and enforcement receipts through public APIs.

r[crate_consumability.external_consumer_fixture.public_api_behavior] The consumer fixture MUST exercise policy consumption and enforcement receipts through public APIs.

#### Scenario: Consumer policy and receipts

r[crate_consumability.external_consumer_fixture.public_api_behavior.policy_and_receipts]

- GIVEN the fixture uses packaged default policy and supplied policy JSON
- WHEN it constructs requests, supplies capabilities, and inspects receipts
- THEN allowed, denied, and malformed/unsupported policy cases behave deterministically through public APIs

### Requirement: The release verifier MUST record consumer-fixture evidence.

r[crate_consumability.external_consumer_fixture.release_evidence] The release verifier MUST record consumer-fixture evidence in its machine-readable and human-readable receipts.

#### Scenario: Fixture receipt

r[crate_consumability.external_consumer_fixture.release_evidence.receipt_step]

- GIVEN release verification runs
- WHEN the external consumer fixture step passes, fails, or is explicitly deferred by sandbox constraints
- THEN the verifier receipt records the fixture command, status, bounded output, and deferral reason if any
