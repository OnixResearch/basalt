//! Reviewed Kiln holder-invocation boundary. Admission facts cannot mint sessions.
//!
//! Every request field is length-framed into one canonical Kiln resource. The
//! resource and a distinct ability are covered by both the UCAN grant and the holder
//! signature. Three framings exist:
//!
//! - `kiln://implementation/v1/`: frozen, holder first, admitted through any verified
//!   chain that reaches a trusted root (compatibility window);
//! - `kiln://implementation/v2/`: attenuable, no holder segment, so byte-prefix
//!   grants express deployment, project, work and attempt ceilings across holders;
//! - `kiln://source/v1/`: attenuable source-view scope framed per path component.
//!
//! An attenuable chain is admitted only when every intermediate delegator holds the
//! family's explicit delegate ability over the prefix it delegated, every grant that
//! covers the request stops on a segment boundary, and the chain is no deeper than
//! `Verifier::max_delegation_depth`. Callers own keys, proof loading, revocation
//! currentness, monotonic time and durable replay. Kiln owns current work/fence/policy
//! facts, claims, budgets, verification, publication and effects.
mod framing;
mod source;

pub use source::{
    PathScope, SourceAttempt, SourceOperation, SourceRequest, SourceScope, SourceSession,
};

use crate::authority_core::BasaltPolicyRef;
use crate::ucan_enforcement::{
    EnforcementAction, UcanEnforcementBoundaries, UcanEnforcementRequest,
    verify_ucan_enforcement_request,
};
use crate::ucan_profile::BasaltUcanProtocolProfile;
use crate::{EnforcementRequest, Policy};
use framing::{Ceiling, Family};
use ucan::{
    HolderInvocationNonce, SignerSignature, verify_compact_token_str_with_resolvers_and_revocations,
};

const DIGEST_BYTES: usize = framing::DIGEST_BYTES;

/// Closed Kiln implementation operations. No operation implies another grant.
// r[impl basalt.kiln_authority.actions]
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Operation {
    ClaimAcquire,
    ClaimRenew,
    Dispatch,
    CandidateSubmit,
    VerificationAdmit,
    AttemptCancel,
    PublicationPublish,
    IntegrationLand,
    LifecycleComplete,
}
impl Operation {
    /// Return the distinct reviewed action; no action implies another grant.
    #[must_use]
    pub const fn action(self) -> EnforcementAction {
        match self {
            Self::ClaimAcquire => EnforcementAction::KilnClaimAcquire,
            Self::ClaimRenew => EnforcementAction::KilnClaimRenew,
            Self::Dispatch => EnforcementAction::KilnDispatch,
            Self::CandidateSubmit => EnforcementAction::KilnCandidateSubmit,
            Self::VerificationAdmit => EnforcementAction::KilnVerificationAdmit,
            Self::AttemptCancel => EnforcementAction::KilnAttemptCancel,
            Self::PublicationPublish => EnforcementAction::KilnPublicationPublish,
            Self::IntegrationLand => EnforcementAction::KilnIntegrationLand,
            Self::LifecycleComplete => EnforcementAction::KilnLifecycleComplete,
        }
    }

    /// Return whether this operation binds a candidate identity.
    ///
    /// Claim, dispatch and cancellation act before a candidate exists; every later
    /// operation MUST name the exact candidate it admits or mutates.
    // r[impl basalt.kiln_authority.binding.scenario.candidate]
    #[must_use]
    pub const fn requires_candidate(self) -> bool {
        match self {
            Self::ClaimAcquire | Self::ClaimRenew | Self::Dispatch | Self::AttemptCancel => false,
            Self::CandidateSubmit
            | Self::VerificationAdmit
            | Self::PublicationPublish
            | Self::IntegrationLand
            | Self::LifecycleComplete => true,
        }
    }
}

/// Canonical implementation framing selected by a request.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ImplementationFraming {
    /// Frozen holder-first `kiln://implementation/v1/` framing.
    #[default]
    V1,
    /// Attenuable `kiln://implementation/v2/` framing without the holder segment.
    V2,
}

/// Untrusted request, not a grant. `subject` is the signing UCAN holder DID;
/// deployment `audience` is independent and also bound into the signed resource.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub operation: Operation,
    pub subject: String,
    pub audience: String,
    pub project: String,
    pub work: String,
    pub work_version: u64,
    pub attempt: String,
    pub fence: u64,
    pub generation: u64,
    pub request_digest: [u8; DIGEST_BYTES],
    pub target: String,
    pub candidate: [u8; DIGEST_BYTES],
    /// Framing version; serialized requests without it are v1.
    #[serde(default)]
    pub framing: ImplementationFraming,
}

/// Typed ceiling for issuing a `kiln://implementation/v2/` capability.
///
/// Each later field requires every earlier one. The first `None` ends the prefix, so
/// the grant covers every request below it for any holder.
pub struct ImplementationScope<'a> {
    pub audience: &'a str,
    pub generation: u64,
    pub project: Option<&'a str>,
    pub work: Option<&'a str>,
    pub work_version: Option<u64>,
    pub attempt: Option<&'a str>,
    pub fence: Option<u64>,
}

impl ImplementationScope<'_> {
    /// Frame the exact resource prefix a grant or ceiling must carry.
    ///
    /// # Errors
    /// Returns `Denied` for a zero generation, an inadmissible field, a later field
    /// without an earlier one, or an oversized resource.
    // r[impl basalt.kiln_authority.attenuable]
    pub fn prefix(&self) -> Result<String, Denied> {
        framing::implementation_v2_prefix(
            self.audience,
            self.generation,
            &[
                Ceiling::Text(self.project),
                Ceiling::Text(self.work),
                Ceiling::Number(self.work_version),
                Ceiling::Text(self.attempt),
                Ceiling::Number(self.fence),
            ],
        )
    }
}

/// Credential-bearing input deliberately has no Debug/serde surface.
pub struct Invocation<'a> {
    pub compact_token: &'a str,
    pub nonce: HolderInvocationNonce,
    pub holder_signature: &'a SignerSignature,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Denied;
impl std::fmt::Display for Denied {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("kiln-authority-denied")
    }
}
impl std::error::Error for Denied {}

/// Live grant state shared by implementation and source sessions.
struct Grant {
    token: ucan::CompactToken,
    policy_ref: String,
    expires_unix_s: u64,
    effective: ucan::CapabilitySet,
    decision_id: String,
}

/// Non-deserializable, non-cloneable authority minted only after final replay.
/// Do not persist this value or treat its redacted projections as live authority.
// r[impl basalt.kiln_authority.ingress.scenario.opaque]
pub struct Session {
    request: Request,
    grant: Grant,
}
impl Session {
    /// Return the exact signed request observation, not a reusable grant DTO.
    #[must_use]
    pub const fn request(&self) -> &Request {
        &self.request
    }
    /// Leaf capabilities that each pass the full chain rule of their own family when
    /// evaluated as a request resource (implementation v2, source v1, or v1 while it is
    /// accepted). Grants outside those families, wider than one family, or failing the
    /// rule are omitted. These observations are not transferable authority; do not log
    /// payloads.
    #[must_use]
    pub const fn effective_capabilities(&self) -> &ucan::CapabilitySet {
        &self.grant.effective
    }
    /// Return the effective signed deadline.
    #[must_use]
    pub const fn expires_unix_s(&self) -> u64 {
        self.grant.expires_unix_s
    }
    /// Return the reviewed policy artifact reference.
    #[must_use]
    pub fn policy_identity(&self) -> &str {
        &self.grant.policy_ref
    }
    /// Return the real successful Basalt admission receipt reference.
    #[must_use]
    pub fn decision_id(&self) -> &str {
        &self.grant.decision_id
    }
    /// Check the exact operation, target and normalized digest before any effect.
    ///
    /// # Errors
    /// Returns `Denied` for any operation, target or digest mismatch.
    pub fn require(
        &self,
        operation: Operation,
        target: &str,
        digest: [u8; DIGEST_BYTES],
    ) -> Result<(), Denied> {
        if self.request.operation == operation
            && self.request.target == target
            && self.request.request_digest == digest
        {
            Ok(())
        } else {
            Err(Denied)
        }
    }
    /// Check the exact admitted candidate before any candidate-bound effect.
    ///
    /// # Errors
    /// Returns `Denied` for a different candidate, or for an operation that binds none.
    pub fn require_candidate(&self, candidate: [u8; DIGEST_BYTES]) -> Result<(), Denied> {
        if self.request.operation.requires_candidate() && self.request.candidate == candidate {
            Ok(())
        } else {
            Err(Denied)
        }
    }
}
impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KilnSession").finish_non_exhaustive()
    }
}

/// Whether the frozen `kiln://implementation/v1/` framing is still admitted.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LegacyFraming {
    /// Admit v1 requests during the compatibility window.
    #[default]
    Accept,
    /// Close the window: v1 requests deny and v1 grants leave the effective set.
    Refuse,
}

/// Operator-loaded policy/configuration, never caller-supplied admission facts.
/// The artifact reference must name these exact reviewed policy bytes.
pub struct Verifier<'a> {
    pub policy: &'a Policy,
    pub policy_ref: &'a BasaltPolicyRef,
    pub generation: u64,
    pub audience: &'a str,
    /// Operator-admitted root issuers; holder keys are NOT issuer authority.
    pub trusted_roots: &'a [String],
    /// Maximum delegation hops below a trusted root for attenuable families: `0` admits
    /// only root-issued tokens. The frozen v1 framing does not consult it.
    pub max_delegation_depth: u8,
    /// Whether v1 requests are still admitted.
    pub implementation_v1: LegacyFraming,
}

/// Delegation rule required by the framing family of one bound request.
#[derive(Clone, Copy)]
enum ChainRule {
    /// Frozen v1: any verified chain that reaches a trusted root. If any covering grant
    /// in the chain is wider than the v1 family, every delegator also needs explicit
    /// `kiln/delegate`.
    Legacy,
    /// Attenuable family: explicit delegate ability, aligned generation-pinned grants
    /// and bounded depth.
    Delegated(Family),
}

/// Accumulated facts from one chain walk.
struct Walk {
    remaining: usize,
    /// A v1 chain carries a covering grant wider than `kiln://implementation/v1/`.
    wide: bool,
    /// A v1 delegator issued a covering grant without explicit `kiln/delegate`.
    undelegated: bool,
}

/// One admitted canonical binding and the chain rule its framing requires.
struct Bound {
    enforcement: EnforcementRequest,
    action: EnforcementAction,
    rule: ChainRule,
}

impl Verifier<'_> {
    /// Public canonicalization shared by the issuer and holder signer.
    ///
    /// # Errors
    /// Returns `Denied` for stale deployment bindings, a misused candidate field or
    /// malformed/bounded request fields.
    pub fn enforcement(&self, request: &Request) -> Result<EnforcementRequest, Denied> {
        self.bind(request).map(|bound| bound.enforcement)
    }

    /// Public source canonicalization shared by the issuer and holder signer.
    ///
    /// # Errors
    /// Returns `Denied` for stale deployment bindings, a subtree scope on an entry-only
    /// operation, an inadmissible path or field, or an oversized resource.
    pub fn source_enforcement(
        &self,
        request: &SourceRequest,
    ) -> Result<EnforcementRequest, Denied> {
        self.bind_source(request).map(|bound| bound.enforcement)
    }

    fn admit_deployment(&self, audience: &str, generation: u64) -> Result<(), Denied> {
        if generation != self.generation || self.generation == 0 || audience != self.audience {
            Err(Denied)
        } else {
            Ok(())
        }
    }

    fn admit_contract(&self, action: EnforcementAction, prefix: &str) -> Result<(), Denied> {
        let Some(contract) = self.policy.contract(action.contract_id()) else {
            return Err(Denied);
        };
        if contract.permits_resource(prefix) && contract.permits_ability(action.ability()) {
            Ok(())
        } else {
            Err(Denied)
        }
    }

    // r[impl basalt.kiln_authority.binding]
    // r[impl basalt.kiln_authority.attenuable]
    fn bind(&self, request: &Request) -> Result<Bound, Denied> {
        self.admit_deployment(&request.audience, request.generation)?;
        let candidate = request.candidate != [0; DIGEST_BYTES];
        if candidate != request.operation.requires_candidate() {
            return Err(Denied);
        }
        let action = request.operation.action();
        let (resource, rule) = match request.framing {
            ImplementationFraming::V1 => {
                if self.implementation_v1 == LegacyFraming::Refuse {
                    return Err(Denied);
                }
                self.admit_contract(action, framing::IMPLEMENTATION_V1)?;
                (framing::implementation_v1(request)?, ChainRule::Legacy)
            }
            ImplementationFraming::V2 => {
                self.admit_contract(action, framing::IMPLEMENTATION_V2)?;
                (
                    framing::implementation_v2(request)?,
                    ChainRule::Delegated(framing::IMPLEMENTATION_V2_FAMILY),
                )
            }
        };
        Ok(Bound {
            enforcement: EnforcementRequest::new(action.contract_id(), resource, action.ability()),
            action,
            rule,
        })
    }

    // r[impl basalt.kiln_authority.source_actions]
    // r[impl basalt.kiln_authority.source_path]
    fn bind_source(&self, request: &SourceRequest) -> Result<Bound, Denied> {
        self.admit_deployment(&request.audience, request.generation)?;
        if request.operation.requires_entry() && request.scope != PathScope::Entry {
            return Err(Denied);
        }
        let action = request.operation.action();
        self.admit_contract(action, framing::SOURCE_V1)?;
        let resource = framing::source_v1(request)?;
        Ok(Bound {
            enforcement: EnforcementRequest::new(action.contract_id(), resource, action.ability()),
            action,
            rule: ChainRule::Delegated(framing::SOURCE_V1_FAMILY),
        })
    }

    fn selected_policy(&self, action: EnforcementAction) -> BasaltPolicyRef {
        BasaltPolicyRef::new(
            &self.policy_ref.artifact_ref,
            &self.policy_ref.schema_version,
            action.contract_id(),
        )
    }

    /// Verify a signed capability and holder request before consuming replay.
    ///
    /// # Errors
    /// Returns only `Denied` when any cryptographic, policy or live boundary denies.
    // r[impl basalt.kiln_authority.ingress]
    pub fn verify(
        &self,
        request: Request,
        invocation: &Invocation<'_>,
        boundaries: &UcanEnforcementBoundaries<'_>,
    ) -> Result<Session, Denied> {
        let bound = self.bind(&request)?;
        let grant = self.admit(&bound, &request.subject, invocation, boundaries)?;
        Ok(Session { request, grant })
    }

    /// Verify a signed source capability and holder request before consuming replay.
    ///
    /// # Errors
    /// Returns only `Denied` when any cryptographic, policy or live boundary denies.
    // r[impl basalt.kiln_authority.ingress]
    pub fn verify_source(
        &self,
        request: SourceRequest,
        invocation: &Invocation<'_>,
        boundaries: &UcanEnforcementBoundaries<'_>,
    ) -> Result<SourceSession, Denied> {
        let bound = self.bind_source(&request)?;
        let grant = self.admit(&bound, &request.subject, invocation, boundaries)?;
        Ok(SourceSession { request, grant })
    }

    /// Revalidate signatures, full proof attenuation, time, current revocations,
    /// caveats and loaded policy on EVERY dispatch. This does not re-spend replay:
    /// possession of Session proves its invocation already passed final admission.
    ///
    /// # Errors
    /// Returns `Denied` when any current authority binding no longer holds.
    // r[impl basalt.kiln_authority.revalidation]
    pub fn revalidate(
        &self,
        session: &Session,
        boundaries: &UcanEnforcementBoundaries<'_>,
    ) -> Result<(), Denied> {
        let bound = self.bind(&session.request)?;
        self.recheck(&bound, &session.request.subject, &session.grant, boundaries)
    }

    /// Revalidate a source session exactly as [`Verifier::revalidate`] does.
    ///
    /// # Errors
    /// Returns `Denied` when any current authority binding no longer holds.
    // r[impl basalt.kiln_authority.revalidation]
    pub fn revalidate_source(
        &self,
        session: &SourceSession,
        boundaries: &UcanEnforcementBoundaries<'_>,
    ) -> Result<(), Denied> {
        let bound = self.bind_source(&session.request)?;
        self.recheck(&bound, &session.request.subject, &session.grant, boundaries)
    }

    fn admit(
        &self,
        bound: &Bound,
        subject: &str,
        invocation: &Invocation<'_>,
        boundaries: &UcanEnforcementBoundaries<'_>,
    ) -> Result<Grant, Denied> {
        let token = ucan::CompactToken::new(invocation.compact_token).map_err(|_| Denied)?;
        // Bind holder identity BEFORE the consuming replay boundary. A credential
        // valid for a different holder must never burn a nonce for this request.
        let verified = verify_compact_token_str_with_resolvers_and_revocations(
            invocation.compact_token,
            boundaries.verification_time,
            boundaries.key_resolver,
            boundaries.proof_store,
            boundaries.revocations,
            boundaries.verification_limits,
        )
        .map_err(|_| Denied)?;
        if verified.audience().as_str() != subject {
            return Err(Denied);
        }
        self.check_chain(
            &verified,
            bound.enforcement.resource(),
            bound.rule,
            boundaries,
        )?;
        // Computed before the consuming replay so a failure burns no nonce.
        let effective = self.checked_effective(&verified, boundaries)?;
        let selected = self.selected_policy(bound.action);
        let raw = UcanEnforcementRequest::new(
            &bound.enforcement,
            invocation.compact_token,
            BasaltUcanProtocolProfile::OnixCompactEd25519V1,
            bound.action,
            invocation.nonce,
            invocation.holder_signature,
        );
        let live = verify_ucan_enforcement_request(self.policy, &selected, &raw, boundaries)
            .map_err(|_| Denied)?;
        let expires_unix_s = live
            .holder_authorization()
            .effective_time_bounds()
            .expires_at_seconds();
        let decision_id = live.receipt().receipt_blake3.clone();
        Ok(Grant {
            token,
            policy_ref: selected.artifact_ref,
            expires_unix_s,
            effective,
            decision_id,
        })
    }

    fn recheck(
        &self,
        bound: &Bound,
        subject: &str,
        grant: &Grant,
        boundaries: &UcanEnforcementBoundaries<'_>,
    ) -> Result<(), Denied> {
        if grant.policy_ref != self.policy_ref.artifact_ref {
            return Err(Denied);
        }
        let enforcement = &bound.enforcement;
        let verified = verify_compact_token_str_with_resolvers_and_revocations(
            grant.token.as_str(),
            boundaries.verification_time,
            boundaries.key_resolver,
            boundaries.proof_store,
            boundaries.revocations,
            boundaries.verification_limits,
        )
        .map_err(|_| Denied)?;
        self.check_chain(&verified, enforcement.resource(), bound.rule, boundaries)?;
        if verified.audience().as_str() != subject
            || !verified
                .authorize_with_policies(
                    enforcement.resource(),
                    enforcement.ability(),
                    boundaries.caveat_policies,
                )
                .is_allowed()
        {
            return Err(Denied);
        }
        // No capability DTO supplied by the caller reaches policy enforcement.
        let local = EnforcementRequest::new(
            enforcement.contract(),
            enforcement.resource(),
            enforcement.ability(),
        )
        .with_capability(crate::CapabilityGrant::new(
            enforcement.resource(),
            enforcement.ability(),
        ));
        if !crate::enforce(self.policy, &local)
            .map_err(|_| Denied)?
            .is_allowed()
        {
            return Err(Denied);
        }
        Ok(())
    }

    /// The chain rule for a capability resource, by its family.
    fn rule_for(&self, resource: &str) -> Option<ChainRule> {
        if resource.starts_with(framing::IMPLEMENTATION_V2) {
            Some(ChainRule::Delegated(framing::IMPLEMENTATION_V2_FAMILY))
        } else if resource.starts_with(framing::SOURCE_V1) {
            Some(ChainRule::Delegated(framing::SOURCE_V1_FAMILY))
        } else if resource.starts_with(framing::IMPLEMENTATION_V1)
            && self.implementation_v1 == LegacyFraming::Accept
        {
            Some(ChainRule::Legacy)
        } else {
            None
        }
    }

    /// Keep only leaf capabilities whose own resource passes its family's chain rule.
    // r[impl basalt.kiln_authority.effective_scope]
    fn checked_effective(
        &self,
        verified: &ucan::VerifiedToken,
        boundaries: &UcanEnforcementBoundaries<'_>,
    ) -> Result<ucan::CapabilitySet, Denied> {
        let checked = verified
            .effective_delegation()
            .capabilities()
            .as_slice()
            .iter()
            .filter(|grant| {
                self.rule_for(&grant.resource).is_some_and(|rule| {
                    self.check_chain(verified, &grant.resource, rule, boundaries)
                        .is_ok()
                })
            })
            .cloned()
            .collect();
        ucan::CapabilitySet::new(checked).map_err(|_| Denied)
    }

    fn check_chain(
        &self,
        leaf: &ucan::VerifiedToken,
        resource: &str,
        rule: ChainRule,
        boundaries: &UcanEnforcementBoundaries<'_>,
    ) -> Result<(), Denied> {
        let mut walk = Walk {
            remaining: boundaries.verification_limits.max_proof_count(),
            wide: false,
            undelegated: false,
        };
        self.check_roots(leaf, resource, rule, boundaries, &mut walk, 0)?;
        if walk.wide && walk.undelegated {
            Err(Denied)
        } else {
            Ok(())
        }
    }

    /// Walk every proof path to a trusted root. Attenuable families also require
    /// aligned, generation-pinned covering grants, an explicit delegate capability held
    /// by every intermediate delegator over each covering grant it issued, and bounded
    /// depth. v1 records whether a wide grant or an undelegated hop occurs.
    // r[impl basalt.kiln_authority.delegation]
    // r[impl basalt.kiln_authority.delegation.chain]
    fn check_roots(
        &self,
        token: &ucan::VerifiedToken,
        resource: &str,
        rule: ChainRule,
        boundaries: &UcanEnforcementBoundaries<'_>,
        walk: &mut Walk,
        depth: usize,
    ) -> Result<(), Denied> {
        match rule {
            ChainRule::Delegated(family) => {
                if !aligned_grants(token, resource, family) {
                    return Err(Denied);
                }
            }
            ChainRule::Legacy => {
                if covering(token, resource)
                    .any(|grant| !grant.resource.starts_with(framing::IMPLEMENTATION_V1))
                {
                    walk.wide = true;
                }
            }
        }
        if token.proofs().as_slice().is_empty() {
            return if self
                .trusted_roots
                .iter()
                .any(|root| root == token.issuer().as_str())
            {
                Ok(())
            } else {
                Err(Denied)
            };
        }
        let depth = depth + 1;
        if matches!(rule, ChainRule::Delegated(_)) && depth > usize::from(self.max_delegation_depth)
        {
            return Err(Denied);
        }
        let mut parents = Vec::with_capacity(token.proofs().as_slice().len());
        for reference in token.proofs().as_slice() {
            walk.remaining = walk.remaining.checked_sub(1).ok_or(Denied)?;
            let proof = boundaries
                .proof_store
                .resolve_proof(reference)
                .map_err(|_| Denied)?;
            let proof_token = match &proof {
                ucan::ProofToken::Borrowed(token) => *token,
                ucan::ProofToken::Owned(token) => token,
            };
            if ucan::proof_reference(proof_token) != *reference {
                return Err(Denied);
            }
            let raw = match &proof {
                ucan::ProofToken::Borrowed(token) => token.as_str(),
                ucan::ProofToken::Owned(token) => token.as_str(),
            };
            let parent = verify_compact_token_str_with_resolvers_and_revocations(
                raw,
                boundaries.verification_time,
                boundaries.key_resolver,
                boundaries.proof_store,
                boundaries.revocations,
                boundaries.verification_limits,
            )
            .map_err(|_| Denied)?;
            parents.push(parent);
        }
        let delegate = match rule {
            ChainRule::Legacy => framing::IMPLEMENTATION_DELEGATE_ABILITY,
            ChainRule::Delegated(family) => family.delegate,
        };
        if !delegation_permitted(token, &parents, resource, delegate) {
            match rule {
                ChainRule::Delegated(_) => return Err(Denied),
                ChainRule::Legacy => walk.undelegated = true,
            }
        }
        for parent in &parents {
            self.check_roots(parent, resource, rule, boundaries, walk, depth)?;
        }
        Ok(())
    }
}

/// Grants in `token` that cover `resource`.
fn covering<'t>(
    token: &'t ucan::VerifiedToken,
    resource: &'t str,
) -> impl Iterator<Item = &'t ucan::CapabilityDocument> {
    token
        .capabilities()
        .as_slice()
        .iter()
        .filter(move |grant| resource.starts_with(grant.resource.as_str()))
}

/// Every grant covering the request pins the family generation and stops on a segment
/// boundary, so it cannot survive a generation bump or cover a sibling whose framed
/// bytes share a partial segment.
fn aligned_grants(token: &ucan::VerifiedToken, resource: &str, family: Family) -> bool {
    covering(token, resource).all(|grant| family.aligned(&grant.resource))
}

/// Every covering grant a delegator issued is itself covered by an explicit,
/// uncaveated delegate capability that the delegator holds from a parent proof.
/// Wildcard abilities never count as the explicit delegate ability.
fn delegation_permitted(
    token: &ucan::VerifiedToken,
    parents: &[ucan::VerifiedToken],
    resource: &str,
    delegate: &str,
) -> bool {
    covering(token, resource).all(|grant| {
        parents.iter().any(|parent| {
            parent.capabilities().as_slice().iter().any(|held| {
                held.ability == delegate
                    && held.caveats.is_empty()
                    && grant.resource.starts_with(held.resource.as_str())
            })
        })
    })
}
