# Crate Consumability Specification

## Purpose

Defines the `crate-consumability` capability.

## Requirements

### Requirement: The crate MUST express its UCAN dependency in a way that downstream applications can resolve without a sibling `../ucan` checkout.

r[crate_consumability.use_remote_ucan_dependency.dependency_source] The crate MUST express its UCAN dependency in a way that downstream applications can resolve without a sibling `../ucan` checkout.

#### Scenario: Remote resolution

r[crate_consumability.use_remote_ucan_dependency.dependency_source.remote_resolution]

- GIVEN an application adds this crate as a dependency

- WHEN Cargo resolves the dependency graph

- THEN UCAN is fetched from the configured remote source or an explicit consumer override, not from this repo's parent directory

#### Scenario: Local override

r[crate_consumability.use_remote_ucan_dependency.dependency_source.local_override]

- GIVEN a maintainer wants to test against a local UCAN checkout

- WHEN they use the documented local override path

- THEN the public manifest remains valid for remote consumers

### Requirement: The crate MUST package successfully with the remote dependency contract.

r[crate_consumability.use_remote_ucan_dependency.package_verification] The crate MUST package successfully with the remote dependency contract.

#### Scenario: Cargo package

r[crate_consumability.use_remote_ucan_dependency.package_verification.cargo_package]

- GIVEN release verification runs

- WHEN `cargo package` executes

- THEN the package manifest passes dependency-source validation and does not fail because UCAN is path-only

### Requirement: The crate manifest MUST include enough package metadata for applications and registry tooling to identify source, docs, README, license, and purpose.

r[crate_consumability.package_metadata.metadata_complete] The crate manifest MUST include enough package metadata for applications and registry tooling to identify source, docs, README, license, and purpose.

#### Scenario: Cargo metadata

r[crate_consumability.package_metadata.metadata_complete.cargo_metadata]

- GIVEN a consumer or registry reads package metadata

- WHEN Cargo emits metadata for the crate

- THEN repository/readme/license/description/discoverability fields are present and accurate

### Requirement: The release rail MUST detect missing or invalid public package metadata before publication.

r[crate_consumability.package_metadata.metadata_verification] The release rail MUST detect missing or invalid public package metadata before publication.

#### Scenario: Package warning regression

r[crate_consumability.package_metadata.metadata_verification.package_warning_regression]

- GIVEN a maintainer prepares a package

- WHEN package metadata checks run

- THEN missing repository/documentation/homepage-style warnings are fixed or explicitly justified

### Requirement: The library crate MUST be consumable without enabling CLI-only dependencies.

r[crate_consumability.cli_feature_split.library_without_cli_deps] The library crate MUST be consumable without enabling CLI-only dependencies.

#### Scenario: Consumer library build

r[crate_consumability.cli_feature_split.library_without_cli_deps.consumer_library_build]

- GIVEN a downstream app depends on the crate for enforcement APIs only

- WHEN Cargo builds the dependency graph without the CLI feature

- THEN CLI parser dependencies are not required for the library path

### Requirement: The CLI binary MUST remain buildable and runnable when the CLI feature is enabled.

r[crate_consumability.cli_feature_split.cli_feature_preserves_binary] The CLI binary MUST remain buildable and runnable when the CLI feature is enabled.

#### Scenario: Cli enabled build

r[crate_consumability.cli_feature_split.cli_feature_preserves_binary.cli_enabled_build]

- GIVEN an operator enables the CLI feature

- WHEN the binary target builds and the enforce command is smoke-tested

- THEN existing CLI behavior and JSON receipt shape are preserved

### Requirement: The crate MUST document the supported public API that downstream applications should call.

r[crate_consumability.stabilize_public_api.documented_api] The crate MUST document the supported public API that downstream applications should call.

#### Scenario: Docs example

r[crate_consumability.stabilize_public_api.documented_api.docs_example]

- GIVEN a downstream developer opens the crate docs or README

- WHEN they follow the library example

- THEN they can parse policy JSON, construct a request, enforce it, and inspect the receipt without using private knowledge

### Requirement: The crate MUST have tests that lock intended public allow/deny/error behavior.

r[crate_consumability.stabilize_public_api.api_regression_tests] The crate MUST have tests that lock intended public allow/deny/error behavior.

#### Scenario: Public behavior tests

r[crate_consumability.stabilize_public_api.api_regression_tests.public_behavior_tests]

- GIVEN public API tests run

- WHEN positive and negative enforcement scenarios execute

- THEN expected receipts and errors remain stable

### Requirement: The crate MUST define how applications obtain the policy JSON consumed by the Rust API.

r[crate_consumability.policy_artifact_consumption.artifact_contract] The crate MUST define how applications obtain the policy JSON consumed by the Rust API.

#### Scenario: Consumer policy setup

r[crate_consumability.policy_artifact_consumption.artifact_contract.consumer_policy_setup]

- GIVEN a downstream app integrates the crate

- WHEN it needs a policy input

- THEN the docs state whether to include, generate, or supply policy JSON and how to verify freshness

### Requirement: The release rail MUST catch drift between Nickel policy source and generated policy artifacts when generated artifacts are used.

r[crate_consumability.policy_artifact_consumption.freshness_gate] The release rail MUST catch drift between Nickel policy source and generated policy artifacts when generated artifacts are used.

#### Scenario: Policy export check

r[crate_consumability.policy_artifact_consumption.freshness_gate.policy_export_check]

- GIVEN policy source changes

- WHEN the freshness check runs

- THEN stale generated JSON or package artifacts cause verification failure

### Requirement: The crate MUST include integration tests that use only the public consumer API.

r[crate_consumability.consumer_integration_tests.public_surface_tests] The crate MUST include integration tests that use only the public consumer API.

#### Scenario: External style test

r[crate_consumability.consumer_integration_tests.public_surface_tests.external_style_test]

- GIVEN integration tests run

- WHEN they import the crate like a downstream app

- THEN policy parsing, request construction, enforcement, and receipt inspection work through public APIs

### Requirement: The integration suite MUST prove fail-closed behavior for common consumer mistakes.

r[crate_consumability.consumer_integration_tests.negative_coverage] The integration suite MUST prove fail-closed behavior for common consumer mistakes.

#### Scenario: Consumer misconfigurations

r[crate_consumability.consumer_integration_tests.negative_coverage.consumer_misconfigurations]

- GIVEN tests exercise malformed policy, unsupported schema, wrong resource, wrong ability, and missing capability

- WHEN enforcement runs

- THEN the crate returns deterministic errors or denied receipts without panics

### Requirement: The repo MUST expose one documented release-readiness rail for crate consumers.

r[crate_consumability.release_verification_rail.single_gate] The repo MUST expose one documented release-readiness rail for crate consumers.

#### Scenario: Maintainer release check

r[crate_consumability.release_verification_rail.single_gate.maintainer_release_check]

- GIVEN a maintainer prepares the crate for downstream use

- WHEN they run the documented rail

- THEN fmt, tests, clippy, policy freshness, feature builds, and package validation are covered

### Requirement: The verification rail MUST avoid stale shared target/sccache artifacts causing false failures.

r[crate_consumability.release_verification_rail.target_hygiene] The verification rail MUST avoid stale shared target/sccache artifacts causing false failures.

#### Scenario: Clean environment

r[crate_consumability.release_verification_rail.target_hygiene.clean_environment]

- GIVEN verification runs in a fresh or repo-local target directory

- WHEN Cargo builds dependencies

- THEN missing `.rlib`/`.rmeta` artifacts from prior stale state do not affect the result

### Requirement: The repo MUST expose a flake release check that delegates to the canonical Rust release verifier.

r[crate_consumability.harden_test_harness.flake_release_check] The repo MUST expose a flake release check that delegates to the canonical Rust release verifier and records any sandbox-only deferrals in the verifier receipt.

#### Scenario: Flake release check

r[crate_consumability.harden_test_harness.flake_release_check.default_flake_gate]

- GIVEN a maintainer runs the default flake checks
- WHEN the release check executes in a sandbox that cannot fetch the private UCAN Git dependency
- THEN it compiles and runs the Rust release verifier with policy freshness prevalidated as a Nix input and records Cargo-dependent rails as deferred in the receipt

### Requirement: The release verifier MUST emit machine-readable and human-readable receipts.

r[crate_consumability.harden_test_harness.release_receipts] The release verifier MUST emit machine-readable and human-readable receipts.

#### Scenario: Receipt materialization

r[crate_consumability.harden_test_harness.release_receipts.receipt_materialization]

- GIVEN the release verifier runs
- WHEN each step completes or a step fails
- THEN JSON and Markdown summaries record step names, commands, statuses, elapsed time, and bounded output tails

### Requirement: The release rail MUST run integration tests through nextest while retaining doctest coverage.

r[crate_consumability.harden_test_harness.nextest_rail] The release rail MUST run integration tests through nextest while retaining doctest coverage.

#### Scenario: Nextest and doctests

r[crate_consumability.harden_test_harness.nextest_rail.nextest_and_doctests]

- GIVEN the release rail executes no-default and CLI-feature tests
- WHEN integration tests and doctests are selected
- THEN nextest runs the integration/unit tests and Cargo runs doctests explicitly

### Requirement: Enforcement tests MUST cover boundary, fail-closed, and multi-capability invariants.

r[crate_consumability.harden_test_harness.enforcement_invariants] Enforcement tests MUST cover boundary, fail-closed, and multi-capability invariants.

#### Scenario: Enforcement invariant coverage

r[crate_consumability.harden_test_harness.enforcement_invariants.boundary_and_capabilities]

- GIVEN the enforcement regression suite runs
- WHEN resource prefix siblings, unknown contracts, id mismatches, malformed contract shape, and multiple capabilities are tested
- THEN the crate returns deterministic allowed, denied, or error outcomes without panics

### Requirement: CLI tests MUST cover common negative operator paths.

r[crate_consumability.harden_test_harness.cli_negative_smokes] CLI tests MUST cover common negative operator paths.

#### Scenario: CLI negative coverage

r[crate_consumability.harden_test_harness.cli_negative_smokes.operator_failures]

- GIVEN the CLI feature test rail runs
- WHEN denied requests, malformed policy files, and bad capability syntax are exercised
- THEN the CLI emits the expected JSON receipt or nonzero parse/error result

### Requirement: Public API and package-content guardrails MUST protect consumer-facing surfaces.

r[crate_consumability.harden_test_harness.api_and_package_guards] Public API and package-content guardrails MUST protect consumer-facing surfaces.

#### Scenario: API and package guards

r[crate_consumability.harden_test_harness.api_and_package_guards.consumer_surface]

- GIVEN public API tests and the package-content verifier run
- WHEN DTO accessors/serialization and package file lists are checked
- THEN required public API behavior and packaged source/policy artifacts remain present

### Requirement: Release verification MUST use a pinned Rust toolchain selection.

r[crate_consumability.harden_test_harness.toolchain_pin] Release verification MUST use a pinned Rust toolchain selection.

#### Scenario: Toolchain reproducibility

r[crate_consumability.harden_test_harness.toolchain_pin.explicit_toolchain]

- GIVEN a maintainer enters or runs the repo toolchain
- WHEN Rust tooling is selected
- THEN the toolchain file names an explicit Rust version rather than a moving stable channel
