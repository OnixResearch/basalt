//! Live UCAN holder verification composed with Basalt policy authority.
//!
//! Basalt completes every fallible policy operation before UCAN performs its
//! final replay commit. After replay succeeds, the shell only moves prebuilt
//! values into [`LiveUcanEnforcement`].

use crate::authority_core::{
    AuthorityAdapterError, AuthorityCaveatDisposition, AuthorityClaim, AuthorityDecision,
    AuthorityEvidence, AuthorityReplayDisposition, BasaltPolicyRef, REQUIRED_AUTHORITY_NON_CLAIMS,
    UcanVerificationReceiptRef, VerificationRefDisposition, VerifiedGrantBinding,
    VerifiedGrantKind, adapt_enforcement_request, evaluate_authority, is_blake3_reference,
};
use crate::ucan_semantics_identity::UCAN_RUNTIME_REVISION;
use crate::{CapabilityGrant, EnforcementRequest, Policy};
use serde::{Deserialize, Serialize};
use std::fmt;
use ucan::{
    AuthorizationDecision, AuthorizationDenialReason, CaveatPolicySet, HolderInvocationAuthorized,
    HolderInvocationError, HolderInvocationLimits, HolderInvocationPolicyContext,
    HolderInvocationReplayAdmission, HolderInvocationReplayError, HolderInvocationRequest,
    HolderTokenVerificationContext, KeyResolver, KeyResolverError, OperationBinding,
    OperationProfileAdmission, OperationProfileAdmissionError, OperationProfileId, ProofStore,
    ProofStoreError, RequestBindingDigest, RevocationChecker, SignerSignature, TokenError,
    VerificationLimitError, VerificationLimits, VerificationTime, VerifiedToken,
    verify_compact_token_str_with_resolvers_and_revocations, verify_holder_signed_invocation,
};

const RECEIPT_SCHEMA: &str = "basalt.ucan-enforcement-receipt.v1";
const RECEIPT_HASH_DOMAIN: &[u8] = b"basalt/ucan-enforcement-receipt/v1";
const OPERATION_PROFILE_DOMAIN: &[u8] = b"basalt/operation-profile/v1";
const REQUEST_BINDING_DOMAIN: &[u8] = b"basalt/ucan-enforcement-request/v1";
const VERIFIED_GRANT_REFERENCE_DOMAIN: &[u8] = b"basalt/ucan-enforcement-grant/v1";
const VERIFICATION_RECEIPT_REFERENCE_DOMAIN: &[u8] = b"basalt/ucan-enforcement-verification/v1";
const BLAKE3_REFERENCE_PREFIX: &str = "blake3:";
const BLAKE3_DIGEST_BYTES: usize = 32;
const HEX_CHARACTERS_PER_BYTE: usize = 2;
const HEX_RADIX: u8 = 16;
const HEX_ALPHA_OFFSET: u8 = 10;
const BLAKE3_HEX_LENGTH: usize = BLAKE3_DIGEST_BYTES * HEX_CHARACTERS_PER_BYTE;
const MAXIMUM_REQUEST_FIELD_BYTES: usize = 4_096;
const RECEIPT_NON_CLAIM_COUNT: usize = 8;
const DOCUMENT_READ_CONTRACT: &str = "document-read";
const DOCUMENT_READ_ABILITY: &str = "document/read";
const DOCUMENT_WRITE_CONTRACT: &str = "document-write";
const DOCUMENT_WRITE_ABILITY: &str = "document/write";

/// Claim boundaries carried by each redacted adapter receipt.
pub const UCAN_ENFORCEMENT_NON_CLAIMS: [&str; RECEIPT_NON_CLAIM_COUNT] = [
    "key-discovery",
    "global-revocation-freshness",
    "replay-durability",
    "transport-security",
    "policy-truth",
    "request-canonicalizer-truth",
    "effect-correctness",
    "whole-system-safety",
];

/// Closed downstream actions with reviewed Basalt contract and UCAN ability mappings.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum EnforcementAction {
    /// Read one document resource.
    DocumentRead,
    /// Write one document resource.
    DocumentWrite,
}

impl EnforcementAction {
    /// Return the stable action identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DocumentRead => "document-read",
            Self::DocumentWrite => "document-write",
        }
    }

    /// Return the reviewed Basalt contract identifier.
    #[must_use]
    pub const fn contract_id(self) -> &'static str {
        match self {
            Self::DocumentRead => DOCUMENT_READ_CONTRACT,
            Self::DocumentWrite => DOCUMENT_WRITE_CONTRACT,
        }
    }

    /// Return the reviewed UCAN ability.
    #[must_use]
    pub const fn ability(self) -> &'static str {
        match self {
            Self::DocumentRead => DOCUMENT_READ_ABILITY,
            Self::DocumentWrite => DOCUMENT_WRITE_ABILITY,
        }
    }

    /// Return the fixed BLAKE3 operation-profile identity for this mapping.
    #[must_use]
    pub fn operation_profile_id(self) -> OperationProfileId {
        OperationProfileId::from_bytes(operation_profile_digest(self))
    }
}

/// Borrowed raw holder invocation bound to one existing Basalt request.
pub struct UcanEnforcementRequest<'request, 'credential> {
    enforcement: &'request EnforcementRequest,
    compact_token: &'credential str,
    action: EnforcementAction,
    nonce: ucan::HolderInvocationNonce,
    holder_signature: &'credential SignerSignature,
}

impl<'request, 'credential> UcanEnforcementRequest<'request, 'credential> {
    /// Construct one borrowed raw UCAN enforcement request.
    #[must_use]
    pub const fn new(
        enforcement: &'request EnforcementRequest,
        compact_token: &'credential str,
        action: EnforcementAction,
        nonce: ucan::HolderInvocationNonce,
        holder_signature: &'credential SignerSignature,
    ) -> Self {
        Self {
            enforcement,
            compact_token,
            action,
            nonce,
            holder_signature,
        }
    }

    /// Return the exact borrowed Basalt request.
    #[must_use]
    pub const fn enforcement(&self) -> &EnforcementRequest {
        self.enforcement
    }

    /// Return the selected typed downstream action.
    #[must_use]
    pub const fn action(&self) -> EnforcementAction {
        self.action
    }
}

/// Explicit caller-owned services and limits for UCAN verification.
pub struct UcanEnforcementBoundaries<'a> {
    /// Explicit verification time.
    pub verification_time: VerificationTime,
    /// Caller-owned issuer and holder key resolver.
    pub key_resolver: &'a dyn KeyResolver,
    /// Caller-owned proof store.
    pub proof_store: &'a dyn ProofStore,
    /// Caller-owned revocation checker.
    pub revocations: &'a dyn RevocationChecker,
    /// Non-consuming caveat policy observations.
    pub caveat_policies: &'a dyn CaveatPolicySet,
    /// Final state-consuming holder replay admission.
    pub replay: &'a dyn HolderInvocationReplayAdmission,
    /// Compact-token and proof limits.
    pub verification_limits: VerificationLimits,
    /// Holder canonical request limits.
    pub holder_limits: HolderInvocationLimits,
}

/// Stable typed adapter denial reason.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum UcanEnforcementReason {
    /// The complete compact token exceeds the configured byte limit.
    OversizedCredential,
    /// Compact-token syntax or claims are malformed.
    MalformedCredential,
    /// A token signature did not verify.
    SignatureDenied,
    /// Issuer key resolution failed or was unavailable.
    KeyResolutionDenied,
    /// Proof resolution, traversal, or attenuation failed.
    ProofDenied,
    /// Explicit verification time denied the token.
    TimeDenied,
    /// A caveat was denied or lacked a policy.
    CaveatDenied,
    /// The effective UCAN capability did not authorize the exact scope.
    ScopeDenied,
    /// A token or proof was revoked.
    Revoked,
    /// The live holder did not prove possession of the audience key.
    HolderDenied,
    /// Final replay admission denied or failed.
    ReplayDenied,
    /// An explicit verification or canonicalization limit failed.
    LimitExceeded,
    /// The typed action did not match the exact contract and ability.
    OperationMismatch,
    /// Caller-supplied capability fields were present.
    UnverifiedAuthorityFields,
    /// Basalt policy or authority-core evaluation denied before replay.
    PolicyDenied,
    /// An injected verification backend could not provide an answer.
    BackendUnavailable,
    /// The internally generated authority evidence could not be adapted.
    InternalAuthorityDenied,
    /// All checks and final replay admission succeeded.
    Allowed,
}

impl UcanEnforcementReason {
    /// Return the stable serialized reason identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OversizedCredential => "oversized-credential",
            Self::MalformedCredential => "malformed-credential",
            Self::SignatureDenied => "signature-denied",
            Self::KeyResolutionDenied => "key-resolution-denied",
            Self::ProofDenied => "proof-denied",
            Self::TimeDenied => "time-denied",
            Self::CaveatDenied => "caveat-denied",
            Self::ScopeDenied => "scope-denied",
            Self::Revoked => "revoked",
            Self::HolderDenied => "holder-denied",
            Self::ReplayDenied => "replay-denied",
            Self::LimitExceeded => "limit-exceeded",
            Self::OperationMismatch => "operation-mismatch",
            Self::UnverifiedAuthorityFields => "unverified-authority-fields",
            Self::PolicyDenied => "policy-denied",
            Self::BackendUnavailable => "backend-unavailable",
            Self::InternalAuthorityDenied => "internal-authority-denied",
            Self::Allowed => "allowed",
        }
    }
}

/// Deterministic redacted evidence for one adapter outcome.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct UcanEnforcementReceipt {
    /// Receipt schema identifier.
    pub schema: String,
    /// Whether live exact-request authority was returned.
    pub allowed: bool,
    /// Stable typed outcome reason.
    pub reason: UcanEnforcementReason,
    /// Observed compact-token length without credential bytes.
    pub credential_bytes: u64,
    /// Complete bounded compact-token identity, absent for oversized input.
    pub credential_blake3: Option<String>,
    /// Exact request-binding identity without request text.
    pub request_binding_blake3: Option<String>,
    /// Reviewed operation-profile identity without ability text.
    pub operation_profile_blake3: String,
    /// Trusted Basalt policy identity when it is a canonical BLAKE3 reference.
    pub policy_blake3: Option<String>,
    /// Exact reviewed UCAN source revision.
    pub ucan_revision: String,
    /// Explicit unsupported claims.
    pub non_claims: Vec<String>,
    /// BLAKE3 identity of this redacted receipt material.
    pub receipt_blake3: String,
}

impl UcanEnforcementReceipt {
    /// Return true when this receipt accompanies live authority.
    #[must_use]
    pub const fn is_allowed(&self) -> bool {
        self.allowed
    }
}

/// Typed fail-closed adapter error with redacted evidence only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UcanEnforcementFailure {
    reason: UcanEnforcementReason,
    receipt: Box<UcanEnforcementReceipt>,
}

impl UcanEnforcementFailure {
    /// Return the stable denial reason.
    #[must_use]
    pub const fn reason(&self) -> UcanEnforcementReason {
        self.reason
    }

    /// Return the redacted denial receipt.
    #[must_use]
    pub const fn receipt(&self) -> &UcanEnforcementReceipt {
        &self.receipt
    }
}

impl fmt::Display for UcanEnforcementFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "UCAN enforcement denied: {}",
            self.reason.as_str()
        )
    }
}

impl std::error::Error for UcanEnforcementFailure {}

/// Live exact-request authority after final UCAN replay admission.
///
/// This type has private construction and no clone or serde surface.
pub struct LiveUcanEnforcement<'request> {
    request: &'request EnforcementRequest,
    _authority_decision: AuthorityDecision,
    receipt: UcanEnforcementReceipt,
    holder_authorization: HolderInvocationAuthorized,
}

impl fmt::Debug for LiveUcanEnforcement<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LiveUcanEnforcement")
            .field("allowed", &true)
            .field("receipt_blake3", &self.receipt.receipt_blake3)
            .finish_non_exhaustive()
    }
}

impl<'request> LiveUcanEnforcement<'request> {
    /// Return the exact borrowed request admitted by this live capability.
    #[must_use]
    pub const fn request(&self) -> &'request EnforcementRequest {
        self.request
    }

    /// Return the redacted evidence receipt.
    #[must_use]
    pub const fn receipt(&self) -> &UcanEnforcementReceipt {
        &self.receipt
    }

    /// Return the final live UCAN holder authorization.
    #[must_use]
    pub const fn holder_authorization(&self) -> &HolderInvocationAuthorized {
        &self.holder_authorization
    }
}

#[derive(Clone, Copy)]
struct ReceiptContext {
    credential_bytes: u64,
    credential: Option<[u8; BLAKE3_DIGEST_BYTES]>,
    request_binding: Option<[u8; BLAKE3_DIGEST_BYTES]>,
    operation_profile: [u8; BLAKE3_DIGEST_BYTES],
    policy: Option<[u8; BLAKE3_DIGEST_BYTES]>,
}

struct ProspectiveAuthority {
    decision: AuthorityDecision,
    receipt: UcanEnforcementReceipt,
    operation: OperationBinding,
    receipt_context: ReceiptContext,
}

#[derive(Clone, Copy)]
struct ExactOperationProfile {
    expected: OperationProfileId,
}

impl OperationProfileAdmission for ExactOperationProfile {
    fn admit_operation_profile(
        &self,
        profile: &OperationProfileId,
    ) -> Result<(), OperationProfileAdmissionError> {
        if profile == &self.expected {
            Ok(())
        } else {
            Err(OperationProfileAdmissionError::UnknownProfile)
        }
    }
}

/// Verify one raw holder invocation and return live exact-request Basalt authority.
///
/// Basalt policy evaluation and receipt construction finish before the final
/// holder verification calls replay. Repeated resolver, proof, revocation, and
/// caveat calls must remain non-consuming. The final UCAN verification is authoritative.
///
/// # Errors
///
/// Returns [`UcanEnforcementFailure`] without an authority decision when any
/// boundary denies or cannot answer.
pub fn verify_ucan_enforcement_request<'request>(
    policy: &Policy,
    trusted_policy_ref: &BasaltPolicyRef,
    request: &UcanEnforcementRequest<'request, '_>,
    boundaries: &UcanEnforcementBoundaries<'_>,
) -> Result<LiveUcanEnforcement<'request>, UcanEnforcementFailure> {
    let operation_profile = operation_profile_digest(request.action);
    let policy_digest = decode_blake3_reference(trusted_policy_ref.artifact_ref.as_str());
    let credential_bytes = u64::try_from(request.compact_token.len()).unwrap_or(u64::MAX);
    let initial_context = ReceiptContext {
        credential_bytes,
        credential: None,
        request_binding: None,
        operation_profile,
        policy: policy_digest,
    };
    if request.compact_token.len() > boundaries.verification_limits.max_compact_token_bytes() {
        return Err(failure(
            UcanEnforcementReason::OversizedCredential,
            initial_context,
        ));
    }
    let credential = *blake3::hash(request.compact_token.as_bytes()).as_bytes();
    let bounded_context = ReceiptContext {
        credential: Some(credential),
        ..initial_context
    };
    let operation = bind_enforcement_action(request.action, request.enforcement)
        .map_err(|reason| failure(reason, bounded_context))?;
    let request_binding = *operation.request_binding().as_bytes();
    let ready_context = ReceiptContext {
        request_binding: Some(request_binding),
        ..bounded_context
    };
    let exact_profile = ExactOperationProfile {
        expected: operation.profile(),
    };
    let prospective = prepare_prospective_authority(
        policy,
        trusted_policy_ref,
        request,
        boundaries,
        operation,
        &exact_profile,
        ready_context,
    )?;

    let holder_request = HolderInvocationRequest::new(
        request.compact_token,
        request.enforcement.resource(),
        request.enforcement.ability(),
        prospective.operation,
        request.nonce,
        request.holder_signature,
    );
    let token_context = HolderTokenVerificationContext::new(
        boundaries.verification_time,
        boundaries.key_resolver,
        boundaries.proof_store,
        boundaries.revocations,
        boundaries.verification_limits,
    );
    let policy_context = HolderInvocationPolicyContext::new(
        boundaries.caveat_policies,
        &exact_profile,
        boundaries.replay,
        boundaries.holder_limits,
    );

    match verify_holder_signed_invocation(&holder_request, &token_context, &policy_context) {
        Ok(holder_authorization) => Ok(LiveUcanEnforcement {
            request: request.enforcement,
            _authority_decision: prospective.decision,
            receipt: prospective.receipt,
            holder_authorization,
        }),
        Err(error) => Err(failure(
            holder_error_reason(&error),
            prospective.receipt_context,
        )),
    }
}

fn prepare_prospective_authority(
    policy: &Policy,
    trusted_policy_ref: &BasaltPolicyRef,
    request: &UcanEnforcementRequest<'_, '_>,
    boundaries: &UcanEnforcementBoundaries<'_>,
    operation: OperationBinding,
    profile: &ExactOperationProfile,
    receipt_context: ReceiptContext,
) -> Result<ProspectiveAuthority, UcanEnforcementFailure> {
    let verified = verify_compact_token_str_with_resolvers_and_revocations(
        request.compact_token,
        boundaries.verification_time,
        boundaries.key_resolver,
        boundaries.proof_store,
        boundaries.revocations,
        boundaries.verification_limits,
    )
    .map_err(|error| failure(token_error_reason(&error), receipt_context))?;
    profile
        .admit_operation_profile(&operation.profile())
        .map_err(|error| failure(profile_error_reason(error), receipt_context))?;
    let authorization = verified.authorize_with_policies(
        request.enforcement.resource(),
        request.enforcement.ability(),
        boundaries.caveat_policies,
    );
    let AuthorizationDecision::Allowed { .. } = authorization else {
        let reason = authorization.denial_reason().map_or(
            UcanEnforcementReason::ScopeDenied,
            authorization_error_reason,
        );
        return Err(failure(reason, receipt_context));
    };
    let decision = build_authority_decision(
        policy,
        trusted_policy_ref,
        request.enforcement,
        &verified,
        boundaries.verification_time,
        &receipt_context,
    )
    .map_err(|reason| failure(reason, receipt_context))?;
    if !decision.is_allowed() {
        return Err(failure(
            UcanEnforcementReason::PolicyDenied,
            receipt_context,
        ));
    }
    let receipt = build_receipt(UcanEnforcementReason::Allowed, true, receipt_context);
    Ok(ProspectiveAuthority {
        decision,
        receipt,
        operation,
        receipt_context,
    })
}

/// Bind one typed action to an exact Basalt request for holder signing.
///
/// # Errors
///
/// Returns a typed denial when caller capability fields are present, the action
/// mapping differs, or a request field exceeds the explicit bound.
pub fn bind_enforcement_action(
    action: EnforcementAction,
    request: &EnforcementRequest,
) -> Result<OperationBinding, UcanEnforcementReason> {
    if !request.capabilities().is_empty() {
        return Err(UcanEnforcementReason::UnverifiedAuthorityFields);
    }
    if request.contract() != action.contract_id() {
        return Err(UcanEnforcementReason::OperationMismatch);
    }
    if request.ability() != action.ability() {
        return Err(UcanEnforcementReason::OperationMismatch);
    }
    for field in [request.contract(), request.resource(), request.ability()] {
        if !bounded_request_field(field) {
            return Err(UcanEnforcementReason::LimitExceeded);
        }
    }
    let request_binding =
        derive_request_binding(action, request).ok_or(UcanEnforcementReason::LimitExceeded)?;
    Ok(OperationBinding::new(
        action.operation_profile_id(),
        RequestBindingDigest::from_bytes(request_binding),
    ))
}

fn build_authority_decision(
    policy: &Policy,
    trusted_policy_ref: &BasaltPolicyRef,
    request: &EnforcementRequest,
    verified: &VerifiedToken,
    verification_time: VerificationTime,
    receipt_context: &ReceiptContext,
) -> Result<AuthorityDecision, UcanEnforcementReason> {
    let Some(credential) = receipt_context.credential else {
        return Err(UcanEnforcementReason::InternalAuthorityDenied);
    };
    let Some(request_binding) = receipt_context.request_binding else {
        return Err(UcanEnforcementReason::InternalAuthorityDenied);
    };
    let kind = if verified.proofs().as_slice().is_empty() {
        VerifiedGrantKind::Direct
    } else {
        VerifiedGrantKind::Delegated
    };
    let grant_digest = derive_reference(
        VERIFIED_GRANT_REFERENCE_DOMAIN,
        &[credential.as_slice(), request_binding.as_slice()],
    )
    .ok_or(UcanEnforcementReason::LimitExceeded)?;
    let verification_seconds = verification_time.as_unix_seconds().to_be_bytes();
    let receipt_digest = derive_reference(
        VERIFICATION_RECEIPT_REFERENCE_DOMAIN,
        &[
            grant_digest.as_slice(),
            request_binding.as_slice(),
            verification_seconds.as_slice(),
        ],
    )
    .ok_or(UcanEnforcementReason::LimitExceeded)?;
    let request_ref = format_digest(request_binding);
    let grant_ref = format_digest(grant_digest);
    let verification_receipt_ref = format_digest(receipt_digest);
    let internal_request = EnforcementRequest::new(
        request.contract().to_owned(),
        request.resource().to_owned(),
        request.ability().to_owned(),
    )
    .with_capability(CapabilityGrant::new(
        request.resource().to_owned(),
        request.ability().to_owned(),
    ));
    let evidence = AuthorityEvidence {
        request_ref: request_ref.clone(),
        grant_bindings: vec![VerifiedGrantBinding {
            grant_ref: grant_ref.clone(),
            verification_receipt_ref: verification_receipt_ref.clone(),
            kind,
        }],
        verification_receipts: vec![UcanVerificationReceiptRef {
            receipt_ref: verification_receipt_ref,
            request_ref,
            verified_grant_refs: vec![grant_ref],
            disposition: VerificationRefDisposition::Current,
        }],
        policy_refs: vec![trusted_policy_ref.clone()],
        caveat: AuthorityCaveatDisposition::Satisfied,
        replay: AuthorityReplayDisposition::Admitted,
        claims: vec![AuthorityClaim::ExactRequest],
        non_claims: REQUIRED_AUTHORITY_NON_CLAIMS.map(str::to_owned).to_vec(),
    };
    let input = adapt_enforcement_request(&internal_request, evidence)
        .map_err(authority_adapter_error_reason)?;
    Ok(evaluate_authority(policy, trusted_policy_ref, &input))
}

const fn authority_adapter_error_reason(_error: AuthorityAdapterError) -> UcanEnforcementReason {
    UcanEnforcementReason::InternalAuthorityDenied
}

fn failure(reason: UcanEnforcementReason, context: ReceiptContext) -> UcanEnforcementFailure {
    UcanEnforcementFailure {
        reason,
        receipt: Box::new(build_receipt(reason, false, context)),
    }
}

fn build_receipt(
    reason: UcanEnforcementReason,
    allowed: bool,
    context: ReceiptContext,
) -> UcanEnforcementReceipt {
    let receipt_digest = receipt_digest(reason, allowed, &context);
    UcanEnforcementReceipt {
        schema: RECEIPT_SCHEMA.to_owned(),
        allowed,
        reason,
        credential_bytes: context.credential_bytes,
        credential_blake3: context.credential.map(format_digest),
        request_binding_blake3: context.request_binding.map(format_digest),
        operation_profile_blake3: format_digest(context.operation_profile),
        policy_blake3: context.policy.map(format_digest),
        ucan_revision: UCAN_RUNTIME_REVISION.to_owned(),
        non_claims: UCAN_ENFORCEMENT_NON_CLAIMS.map(str::to_owned).to_vec(),
        receipt_blake3: format_digest(receipt_digest),
    }
}

fn receipt_digest(
    reason: UcanEnforcementReason,
    allowed: bool,
    context: &ReceiptContext,
) -> [u8; BLAKE3_DIGEST_BYTES] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(RECEIPT_HASH_DOMAIN);
    hasher.update(&[u8::from(allowed)]);
    hasher.update(reason.as_str().as_bytes());
    hasher.update(&context.credential_bytes.to_be_bytes());
    hash_optional_digest(&mut hasher, context.credential.as_ref());
    hash_optional_digest(&mut hasher, context.request_binding.as_ref());
    hasher.update(&context.operation_profile);
    hash_optional_digest(&mut hasher, context.policy.as_ref());
    hasher.update(UCAN_RUNTIME_REVISION.as_bytes());
    for non_claim in UCAN_ENFORCEMENT_NON_CLAIMS {
        hasher.update(non_claim.as_bytes());
        hasher.update(&[0]);
    }
    *hasher.finalize().as_bytes()
}

fn hash_optional_digest(hasher: &mut blake3::Hasher, digest: Option<&[u8; BLAKE3_DIGEST_BYTES]>) {
    match digest {
        Some(digest) => {
            hasher.update(&[1]);
            hasher.update(digest);
        }
        None => {
            hasher.update(&[0]);
        }
    }
}

fn operation_profile_digest(action: EnforcementAction) -> [u8; BLAKE3_DIGEST_BYTES] {
    derive_reference(
        OPERATION_PROFILE_DOMAIN,
        &[
            action.as_str().as_bytes(),
            action.contract_id().as_bytes(),
            action.ability().as_bytes(),
        ],
    )
    .expect("closed operation-profile fields must fit fixed u64 length prefixes")
}

fn derive_request_binding(
    action: EnforcementAction,
    request: &EnforcementRequest,
) -> Option<[u8; BLAKE3_DIGEST_BYTES]> {
    derive_reference(
        REQUEST_BINDING_DOMAIN,
        &[
            action.as_str().as_bytes(),
            request.contract().as_bytes(),
            request.resource().as_bytes(),
            request.ability().as_bytes(),
        ],
    )
}

fn derive_reference(domain: &[u8], fields: &[&[u8]]) -> Option<[u8; BLAKE3_DIGEST_BYTES]> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    for field in fields {
        let length = u64::try_from(field.len()).ok()?;
        hasher.update(&length.to_be_bytes());
        hasher.update(field);
    }
    Some(*hasher.finalize().as_bytes())
}

fn bounded_request_field(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAXIMUM_REQUEST_FIELD_BYTES
        && !value.chars().any(char::is_control)
}

fn format_digest(digest: [u8; BLAKE3_DIGEST_BYTES]) -> String {
    format!(
        "{BLAKE3_REFERENCE_PREFIX}{}",
        blake3::Hash::from_bytes(digest).to_hex()
    )
}

fn decode_blake3_reference(reference: &str) -> Option<[u8; BLAKE3_DIGEST_BYTES]> {
    if !is_blake3_reference(reference) {
        return None;
    }
    let hex = reference.strip_prefix(BLAKE3_REFERENCE_PREFIX)?;
    if hex.len() != BLAKE3_HEX_LENGTH {
        return None;
    }
    let bytes = hex.as_bytes();
    let mut digest = [0_u8; BLAKE3_DIGEST_BYTES];
    for (index, output) in digest.iter_mut().enumerate() {
        let high_index = index.checked_mul(HEX_CHARACTERS_PER_BYTE)?;
        let low_index = high_index.checked_add(1)?;
        *output = hex_nibble(bytes[high_index])?
            .checked_mul(HEX_RADIX)?
            .checked_add(hex_nibble(bytes[low_index])?)?;
    }
    Some(digest)
}

const fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + HEX_ALPHA_OFFSET),
        _ => None,
    }
}

const fn token_error_reason(error: &TokenError) -> UcanEnforcementReason {
    match error {
        TokenError::LimitExceeded {
            source: VerificationLimitError::CompactTokenTooLarge { .. },
        } => UcanEnforcementReason::OversizedCredential,
        TokenError::LimitExceeded { .. } => UcanEnforcementReason::LimitExceeded,
        TokenError::Signature { .. } => UcanEnforcementReason::SignatureDenied,
        TokenError::MissingVerificationKey { .. } => UcanEnforcementReason::KeyResolutionDenied,
        TokenError::KeyResolver { source } => key_resolver_error_reason(source),
        TokenError::ProofStore { source } => proof_store_error_reason(source),
        TokenError::ProofChain { .. }
        | TokenError::Attenuation { .. }
        | TokenError::SelfDelegation { .. } => UcanEnforcementReason::ProofDenied,
        TokenError::Revoked { .. } => UcanEnforcementReason::Revoked,
        TokenError::Revocation { .. }
        | TokenError::BackendTranscript { .. }
        | TokenError::VerifiedPrimitive { .. }
        | TokenError::JsonEncode { .. }
        | TokenError::InvalidSecretKeyLength { .. }
        | TokenError::Signer { .. } => UcanEnforcementReason::BackendUnavailable,
        TokenError::Temporal { .. } => UcanEnforcementReason::TimeDenied,
        TokenError::EmptyCompactToken
        | TokenError::CompactTokenMustBeTrimmed
        | TokenError::InvalidCompactTokenStructure { .. }
        | TokenError::CompactTokenFramingMismatch
        | TokenError::Base64Decode { .. }
        | TokenError::JsonDecode { .. }
        | TokenError::UnsupportedHeader
        | TokenError::Capability { .. }
        | TokenError::MissingCapabilityField { .. }
        | TokenError::CoreClaims { .. }
        | TokenError::InvalidPublicKeyLength { .. }
        | TokenError::DidEncodingUtf8
        | TokenError::SecretCapacity { .. }
        | TokenError::JsonEscapesNotSupported { .. } => UcanEnforcementReason::MalformedCredential,
    }
}

const fn authorization_error_reason(reason: &AuthorizationDenialReason) -> UcanEnforcementReason {
    match reason {
        AuthorizationDenialReason::CaveatRejected { .. }
        | AuthorizationDenialReason::CaveatPolicyMissing { .. } => {
            UcanEnforcementReason::CaveatDenied
        }
        AuthorizationDenialReason::MalformedRequest { .. }
        | AuthorizationDenialReason::NoMatchingCapability => UcanEnforcementReason::ScopeDenied,
    }
}

const fn profile_error_reason(error: OperationProfileAdmissionError) -> UcanEnforcementReason {
    match error {
        OperationProfileAdmissionError::UnknownProfile => UcanEnforcementReason::OperationMismatch,
        OperationProfileAdmissionError::Backend => UcanEnforcementReason::BackendUnavailable,
    }
}

const fn holder_error_reason(error: &HolderInvocationError) -> UcanEnforcementReason {
    match error {
        HolderInvocationError::Token { source } => token_error_reason(source),
        HolderInvocationError::Authorization { reason } => authorization_error_reason(reason),
        HolderInvocationError::Input { .. }
        | HolderInvocationError::Canonical {
            source:
                ucan::core::holder_invocation::HolderInvocationCanonicalError::EmptyField { .. }
                | ucan::core::holder_invocation::HolderInvocationCanonicalError::OutputLengthMismatch {
                    ..
                },
        } => UcanEnforcementReason::MalformedCredential,
        HolderInvocationError::Canonical { .. } | HolderInvocationError::Limit { .. } => {
            UcanEnforcementReason::LimitExceeded
        }
        HolderInvocationError::KeyResolution { source } => key_resolver_error_reason(source),
        HolderInvocationError::ResolvedAudienceKeyMismatch
        | HolderInvocationError::Signer { .. }
        | HolderInvocationError::SignerAudienceMismatch
        | HolderInvocationError::SignaturePrimitive { .. }
        | HolderInvocationError::InvalidHolderSignature => UcanEnforcementReason::HolderDenied,
        HolderInvocationError::OperationProfile { source } => profile_error_reason(*source),
        HolderInvocationError::Replay { source } => replay_error_reason(*source),
    }
}

const fn proof_store_error_reason(error: &ProofStoreError) -> UcanEnforcementReason {
    match error {
        ProofStoreError::MissingProof { .. } | ProofStoreError::Malformed { .. } => {
            UcanEnforcementReason::ProofDenied
        }
        ProofStoreError::Stale { .. } | ProofStoreError::Backend { .. } => {
            UcanEnforcementReason::BackendUnavailable
        }
    }
}

const fn key_resolver_error_reason(error: &KeyResolverError) -> UcanEnforcementReason {
    match error {
        KeyResolverError::MissingKey { .. }
        | KeyResolverError::UnsupportedMethod { .. }
        | KeyResolverError::Malformed { .. } => UcanEnforcementReason::KeyResolutionDenied,
        KeyResolverError::Stale { .. } | KeyResolverError::Backend { .. } => {
            UcanEnforcementReason::BackendUnavailable
        }
    }
}

const fn replay_error_reason(_error: HolderInvocationReplayError) -> UcanEnforcementReason {
    UcanEnforcementReason::ReplayDenied
}

#[cfg(test)]
mod tests {
    use super::{
        BLAKE3_REFERENCE_PREFIX, EnforcementAction, OPERATION_PROFILE_DOMAIN, RECEIPT_SCHEMA,
        ReceiptContext, UcanEnforcementReason, build_receipt, decode_blake3_reference,
        derive_request_binding, operation_profile_digest, profile_error_reason,
    };
    use crate::EnforcementRequest;
    use ucan::OperationProfileAdmissionError;

    const CREDENTIAL_BYTE: u8 = 3;
    const REQUEST_BYTE: u8 = 5;
    const POLICY_BYTE: u8 = 7;
    const DIGEST_BYTES: usize = 32;

    fn receipt_context() -> ReceiptContext {
        ReceiptContext {
            credential_bytes: u64::try_from(DIGEST_BYTES).expect("digest width must fit u64"),
            credential: Some([CREDENTIAL_BYTE; DIGEST_BYTES]),
            request_binding: Some([REQUEST_BYTE; DIGEST_BYTES]),
            operation_profile: operation_profile_digest(EnforcementAction::DocumentWrite),
            policy: Some([POLICY_BYTE; DIGEST_BYTES]),
        }
    }

    #[test]
    fn typed_actions_have_distinct_reviewed_bindings() {
        let read = operation_profile_digest(EnforcementAction::DocumentRead);
        let write = operation_profile_digest(EnforcementAction::DocumentWrite);
        let request = EnforcementRequest::new(
            "document-write",
            "did:key:z6Mkresource/docs/report",
            "document/write",
        );
        let first = derive_request_binding(EnforcementAction::DocumentWrite, &request)
            .expect("bounded request must bind");
        let second = derive_request_binding(EnforcementAction::DocumentWrite, &request)
            .expect("the same bounded request must bind");

        assert_ne!(read, write);
        assert_eq!(first, second);
        assert_eq!(EnforcementAction::DocumentRead.ability(), "document/read");
        assert_eq!(EnforcementAction::DocumentWrite.ability(), "document/write");
    }

    #[test]
    fn operation_profile_hash_uses_unambiguous_length_prefixes() {
        let action = EnforcementAction::DocumentWrite;
        let fields = [
            action.as_str().as_bytes(),
            action.contract_id().as_bytes(),
            action.ability().as_bytes(),
        ];
        let mut framed = blake3::Hasher::new();
        framed.update(OPERATION_PROFILE_DOMAIN);
        for field in fields {
            let field_length = u64::try_from(field.len()).expect("test field length must fit u64");
            framed.update(&field_length.to_be_bytes());
            framed.update(field);
        }
        let mut ambiguous = blake3::Hasher::new();
        ambiguous.update(OPERATION_PROFILE_DOMAIN);
        for field in fields {
            ambiguous.update(field);
        }

        assert_eq!(
            operation_profile_digest(action),
            *framed.finalize().as_bytes()
        );
        assert_ne!(
            operation_profile_digest(action),
            *ambiguous.finalize().as_bytes()
        );
    }

    #[test]
    fn profile_reason_mapping_is_stable() {
        assert_eq!(
            profile_error_reason(OperationProfileAdmissionError::UnknownProfile),
            UcanEnforcementReason::OperationMismatch
        );
        assert_eq!(
            profile_error_reason(OperationProfileAdmissionError::Backend),
            UcanEnforcementReason::BackendUnavailable
        );
    }

    #[test]
    fn receipt_is_deterministic_and_redacted() {
        let first = build_receipt(UcanEnforcementReason::Allowed, true, receipt_context());
        let second = build_receipt(UcanEnforcementReason::Allowed, true, receipt_context());
        let json = serde_json::to_string(&first).expect("receipt must serialize");

        assert_eq!(first, second);
        assert_eq!(first.schema, RECEIPT_SCHEMA);
        assert!(first.receipt_blake3.starts_with(BLAKE3_REFERENCE_PREFIX));
        assert!(!json.contains("did:key"));
        assert!(!json.contains("document/write"));
        assert!(!json.contains("header.payload.signature"));
        assert!(!json.contains("caveat"));
    }

    #[test]
    fn oversized_receipt_omits_credential_identity() {
        let mut context = receipt_context();
        context.credential = None;
        let receipt = build_receipt(UcanEnforcementReason::OversizedCredential, false, context);

        assert!(receipt.credential_blake3.is_none());
        assert!(!receipt.is_allowed());
    }

    #[test]
    fn blake3_reference_decoder_rejects_malformed_input() {
        let valid = format!("{BLAKE3_REFERENCE_PREFIX}{}", "a".repeat(DIGEST_BYTES * 2));

        assert!(decode_blake3_reference(&valid).is_some());
        assert!(decode_blake3_reference("blake3:short").is_none());
        assert!(
            decode_blake3_reference(
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            )
            .is_none()
        );
    }
}
