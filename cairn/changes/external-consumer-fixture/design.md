# Design: external consumer fixture

## Approach

Basalt will add an external-style fixture under a clear fixture directory or release-verifier temp workspace. The fixture must compile outside the crate's own integration-test namespace and use only public package APIs.

## Fixture expectations

- Library-only build does not enable CLI-only dependencies.
- Packaged policy source/generated JSON are present where consumer docs say they are.
- Public APIs cover policy parsing, request construction, enforcement, receipt inspection, and DTO serialization.
- Negative cases fail closed for malformed policy, unsupported schema, wrong resource, wrong ability, and missing capability.

## Verification

The release verifier should run the fixture with a fresh target dir and `RUSTC_WRAPPER=` hygiene, then record a receipt step so fixture regressions are visible in CI artifacts.
