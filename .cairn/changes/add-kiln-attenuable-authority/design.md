## Context

`basalt::kiln` frames every implementation request into `kiln://implementation/v1/` with
the holder first, and `check_roots` only checks that each proof path reaches a trusted
root. UCAN attenuation is byte-prefix resource coverage plus ability subsumption, where
`*` and `a/*` globs subsume. So any holder of a covering grant can re-delegate, and no
cross-holder ceiling is expressible.

## Decisions

### Decision: Versioned framing selected per request

**Choice:** `Request.framing` selects `v1` (default; frozen bytes) or `v2`. The pure
framing lives in `src/kiln/framing.rs`. The implementation contracts admit both prefixes,
and `EnforcementAction::resource_prefix` for implementation actions becomes the shared
family prefix `kiln://implementation/`.

**Rationale:** Both resources are injective and distinct by prefix, and the holder
signature covers whichever one is signed. A per-request selector keeps v1 verifiable for
the compatibility window with no second verifier type. The operation-profile digest does
not include the prefix, so existing holder bindings are unchanged.

### Decision: Every segment ends on `/` and markers are letters

**Choice:**

- Text is `<len>/<hex>/`, numbers are canonical decimal plus `/`, and digests are 64 hex
  characters plus `/`.
- The source context is `n/` or `a/<attempt>/<fence>/`.
- Path components are text segments, followed by the terminal `e/` (entry) or `s/`
  (subtree node).

**Rationale:** A prefix ending on a boundary names exactly one subtree of the grammar.
Decimal lengths cannot collide with letter markers, so `a/b` cannot alias `a/bc`, and an
entry grant cannot reach descendants.

### Decision: Chain rule applies only to attenuable families

**Choice:** For v2 and source requests, `check_roots` rejects a chain in any of these
cases:

- a covering grant, anywhere in the chain, ends inside a segment;
- a delegator issued a covering grant without holding, from a parent proof, an uncaveated
  capability with exactly the family delegate ability whose resource is a byte prefix of
  that grant;
- the number of delegation hops exceeds `Verifier::max_delegation_depth`.

The implementation family uses `kiln/delegate`, which is chain-only and has no contract.
The source family uses `kiln/source/delegate`, which is also the invocable `delegate`
operation.

**Rationale:** Exact ability equality makes delegation explicit, so a wildcard cannot
grant it. Checking *covering* grants rather than only family-prefixed ones stops a wide
grant such as `kiln://` from bypassing the rule. v1 keeps the reviewed compatibility
behaviour.

### Decision: Source scope order

**Choice:** `audience/generation/repository/context/path…/terminal/snapshot/view/digest`.

**Rationale:** Path ceilings (Kiln C8 `ViewCeiling`) and attempt-bound worker scopes must
be byte prefixes. A snapshot or view pin can still extend an entry-terminal grant.

## Risks / Trade-offs

- Adding public struct fields breaks struct-literal construction in Kiln's adapter until
  re-import. The serialized v1 request form stays compatible.
- The generation is framed early, so a generation bump invalidates every v2 and source
  grant. This is intended currentness, not an accident.
- Source resources are bounded at 4096 bytes, which allows roughly 1.8 KiB of hex-framed
  path. Longer paths deny.
- The chain rule proves only explicit delegation along the proof chain. Budget lineage,
  issuance intent and ceiling-versus-view checks remain Kiln's.
