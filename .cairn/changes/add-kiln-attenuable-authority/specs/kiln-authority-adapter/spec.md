# Kiln Authority Adapter

## MODIFIED Requirements

### Requirement: Exact canonical request binding

r[basalt.kiln_authority.binding] The adapter MUST frame the deployment audience,
project, work, work version, attempt, fence, authority generation, payload digest, target
and applicable candidate identity into one canonical resource that both the UCAN grant and
the holder signature cover. The frozen `kiln://implementation/v1/` framing MUST also frame
the holder first and MUST keep its bytes unchanged. Attenuable framings MUST bind the
holder through UCAN audience equality and the holder signature instead of a resource
segment. Every string field MUST be bounded, ASCII and free of control bytes, and
delimiter or Unicode normalization MUST NOT alias two different requests.

#### Scenario: Substituted binding invalidates the invocation

r[basalt.kiln_authority.binding.scenario.substitution]
- GIVEN a signed invocation for one exact Kiln request
- WHEN the holder, audience, project, work, work version, attempt, fence, generation, digest, target or candidate changes
- THEN verification MUST deny
- AND no replay boundary MUST be consumed

#### Scenario: Candidate identity is required exactly where a mutation binds one

r[basalt.kiln_authority.binding.scenario.candidate]
- GIVEN submission, verification, publication, integration and lifecycle operations bind a candidate identity
- WHEN a candidate identity is zero for one of them, or non-zero for claim, dispatch or cancellation
- THEN admission MUST deny before replay

### Requirement: Reviewed policy contracts gate every operation

r[basalt.kiln_authority.policy] Every Kiln implementation and source operation MUST
require its own reviewed contract with its distinct ability. Each implementation contract
MUST admit the `kiln://implementation/v1/` and `kiln://implementation/v2/` prefixes, and
each source contract MUST admit `kiln://source/v1/`. If a policy lacks the selected
contract, or the contract does not admit the selected framing prefix and ability,
admission MUST deny. An unadmitted policy MUST NOT create live authority.

#### Scenario: Missing contract denies

r[basalt.kiln_authority.policy.scenario.missing]
- GIVEN a policy without the selected Kiln contract
- WHEN a request for that operation is verified
- THEN the adapter MUST deny before replay

#### Scenario: Packaged policy carries every reviewed contract

r[basalt.kiln_authority.policy.scenario.packaged]
- GIVEN the packaged policy artifact
- WHEN it is parsed
- THEN it MUST contain each implementation contract with both implementation prefixes and its ability
- AND it MUST contain each source contract with the source prefix and its ability

## ADDED Requirements

### Requirement: Attenuable implementation framing

r[basalt.kiln_authority.attenuable] Basalt MUST offer a `kiln://implementation/v2/`
framing without a holder segment, ordered audience, generation, project, work, work
version, attempt, fence, target, payload digest and candidate. Every segment MUST end on
`/`, so that a byte-prefix grant stopping after a segment expresses a deployment, project,
work or attempt ceiling for any holder. Basalt MUST expose a typed prefix builder that
denies a later segment without every earlier one. The frozen v1 framing MUST remain
verifiable, and a v1 grant MUST NOT cover a v2 request.

#### Scenario: One ceiling serves different holders

r[basalt.kiln_authority.attenuable.scenario.cross_holder]
- GIVEN a delegator holding a project ceiling and the explicit delegate ability
- WHEN it delegates an exact request to one holder and an attempt ceiling to another
- THEN both holders' invocations MUST be admitted and revalidated
- AND the attempt ceiling MUST NOT admit a sibling attempt

#### Scenario: Frozen v1 bytes and chains remain verifiable

r[basalt.kiln_authority.attenuable.scenario.v1_stable]
- GIVEN a v1 request and a v1 chain without any delegate capability
- WHEN it is framed and verified with a delegation depth of zero
- THEN its resource MUST equal the recorded pre-v2 bytes
- AND the chain MUST be admitted while the same request framed as v2 denies

### Requirement: Explicit delegation chain rule

r[basalt.kiln_authority.delegation.chain] For attenuable Kiln families the adapter MUST
admit a delegated chain only when three rules hold. First, every intermediate delegator
holds, from a parent proof, an uncaveated capability whose ability is exactly the family's
delegate ability (`kiln/delegate` or `kiln/source/delegate`). That capability's resource
must be a byte prefix of each grant the delegator issued that covers the request. Second,
every grant in the chain that covers the request either is wider than the family prefix
or stops on a segment boundary. Third, the chain contains no more delegation hops than the
operator-configured maximum depth. Wildcard abilities and the other family's delegate
ability MUST NOT satisfy the rule. Only an operator-trusted root MUST issue a token without
proofs. This is the compact-profile counterpart of UCAN's rule that the root Delegation
issuer is the chain subject: a delegator that holds the delegate ability never becomes a
root. Revalidation MUST apply the same rule.

#### Scenario: Scoped delegation is admitted

r[basalt.kiln_authority.delegation.chain.scenario.scoped]
- GIVEN a trusted root granting a scope plus the family delegate ability to a delegator
- WHEN the delegator issues a narrower scope to a holder
- THEN the holder's invocation MUST be admitted and revalidated

#### Scenario: Missing delegate ability denies

r[basalt.kiln_authority.delegation.chain.scenario.missing_delegate]
- GIVEN a delegator without the explicit family delegate ability over the delegated prefix
- WHEN it issues a grant that UCAN attenuation alone would accept
- THEN verification MUST deny before replay

#### Scenario: Excess depth denies

r[basalt.kiln_authority.delegation.chain.scenario.depth]
- GIVEN a chain with more delegation hops than the configured maximum
- WHEN verification runs
- THEN it MUST deny before replay
- AND the same chain MUST be admitted when the maximum allows its depth

#### Scenario: Grants cut inside a segment deny

r[basalt.kiln_authority.delegation.chain.scenario.aligned]
- GIVEN a covering grant whose resource ends inside a framed segment
- WHEN it appears anywhere in the chain
- THEN verification MUST deny before replay

#### Scenario: Root substitution denies

r[basalt.kiln_authority.delegation.chain.scenario.root]
- GIVEN an attenuable chain whose proof-free root token is issued by an untrusted issuer, or by a delegator that holds the delegate ability
- WHEN verification runs
- THEN it MUST deny before replay

### Requirement: Closed Kiln source operations

r[basalt.kiln_authority.source_actions] Basalt MUST expose the closed `kiln://source/v1/`
operations discover, read, history, prepare, modify, create, delegate and result. Each
MUST map to exactly one reviewed contract and one `kiln/source/*` ability. Read, modify
and create MUST name exactly one entry. Unknown operations MUST NOT deserialize and MUST
NOT authorize anything.

#### Scenario: Source operations are distinct and exact

r[basalt.kiln_authority.source_actions.scenario.distinct]
- GIVEN a signed source session for one operation, path, scope and digest
- WHEN another operation, path, scope or digest is checked or substituted
- THEN the check MUST deny
- AND no two source operations MUST share a contract or ability

#### Scenario: Unknown source operations deny

r[basalt.kiln_authority.source_actions.scenario.unknown]
- GIVEN a source request naming an operation outside the closed vocabulary
- WHEN it is deserialized or a policy lacks the selected source contract
- THEN admission MUST deny

### Requirement: Component-framed source path scope

r[basalt.kiln_authority.source_path] Source path scope MUST be framed per normalized
component, each length-prefixed and hex-encoded. It MUST be followed by a terminal marker
that distinguishes an exact entry (`e/`) from a subtree (`s/`). A component MUST be
non-empty, bounded, free of separators and control characters, and never `.` or `..`. A
grant for one path MUST NOT cover a sibling whose name shares a byte prefix, and an entry
grant MUST NOT cover descendants or the subtree.

#### Scenario: Paths never alias

r[basalt.kiln_authority.source_path.scenario.alias]
- GIVEN subtree, entry and subtree-node grants for `a/b`
- WHEN requests for `a/bc`, `ab`, `a/b/c` and `a/b` are framed
- THEN the subtree grant MUST cover only `a/b` and its descendants
- AND the entry grant MUST cover only the `a/b` entry
- AND an operator-context grant MUST NOT cover an attempt-context request
