use super::core::operation_profile_digest;
use crate::EnforcementRequest;
use serde::{Deserialize, Serialize};
use ucan::{
    CaveatPolicySet, HolderInvocationLimits, HolderInvocationNonce,
    HolderInvocationReplayAdmission, KeyResolver, OperationProfileId, ProofStore,
    RevocationChecker, SignerSignature, VerificationLimits, VerificationTime,
};

const DOCUMENT_READ_CONTRACT: &str = "document-read";
const DOCUMENT_READ_ABILITY: &str = "document/read";
const DOCUMENT_WRITE_CONTRACT: &str = "document-write";
const DOCUMENT_WRITE_ABILITY: &str = "document/write";

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
    pub(super) enforcement: &'request EnforcementRequest,
    pub(super) compact_token: &'credential str,
    pub(super) action: EnforcementAction,
    pub(super) nonce: HolderInvocationNonce,
    pub(super) holder_signature: &'credential SignerSignature,
}

impl<'request, 'credential> UcanEnforcementRequest<'request, 'credential> {
    /// Construct one borrowed raw UCAN enforcement request.
    #[must_use]
    pub const fn new(
        enforcement: &'request EnforcementRequest,
        compact_token: &'credential str,
        action: EnforcementAction,
        nonce: HolderInvocationNonce,
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
    OversizedCredential,
    MalformedCredential,
    SignatureDenied,
    KeyResolutionDenied,
    ProofDenied,
    TimeDenied,
    CaveatDenied,
    ScopeDenied,
    Revoked,
    HolderDenied,
    ReplayDenied,
    LimitExceeded,
    OperationMismatch,
    UnverifiedAuthorityFields,
    PolicyDenied,
    BackendUnavailable,
    InternalAuthorityDenied,
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
