# Proposal: external consumer fixture

## Summary

Add a checked consumer fixture that builds a separate application against Basalt's public package surface so crate-consumability regressions are caught before release.

## Problem

The current integration tests import Basalt like a downstream app, but they still live inside the same Cargo package. A separate fixture catches package metadata, feature, artifact, and public API issues that in-package tests can miss.

## Scope

- Add a small fixture application or generated temp consumer crate.
- Depend on Basalt through the public package path and default library feature set.
- Exercise packaged default policy, supplied policy JSON, enforcement receipts, and optional CLI feature smoke where appropriate.
- Keep the fixture deterministic and runnable through the release verifier.

## Non-goals

- No publishing to a registry.
- No remote deployment or network dependency beyond the existing pinned UCAN source requirements.
