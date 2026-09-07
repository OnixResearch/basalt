# Minimal enforcement profile

Gate oidc_exchange and bounded-http under oidc-exchange. Gate durable_authority and
its two external core dependencies under durable-authority. Both features are default;
explicit no-default consumption opts out. No source or dependency revision changes.

Core enforcement keeps the same typed requests, exact policy and operation identities,
signature validation, injected revocation/replay services and private live grants.
Optional concrete service adapters are not the core verification services themselves.
No runtime fallback or weakened verification is introduced.

Feature-specific tests follow their owning module; all defaults still test both
modules. A minimum-profile test and dependency-tree control must prove enforcement
remains present and all four Rust-1.96 HTTP/durable packages are absent. Empty or
forbidden dependency observations fail. A pinned Rust-1.95 compatibility shell
provides actual consumer-toolchain evidence, not --ignore-rust-version.

Release verification retains its existing complete default checks and adds the
minimum profile. Docs distinguish default API preservation from explicit opt-out.
No general compatibility, host isolation or production authority claim.
