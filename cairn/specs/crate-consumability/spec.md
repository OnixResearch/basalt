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
