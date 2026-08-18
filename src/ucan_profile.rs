//! Basalt-owned UCAN protocol cohorts and verified application input.
//!
//! This module carries nominal application facts only. UCAN retains wire,
//! verification, proof, policy, caveat, revocation, and replay semantics.
// r[impl basalt.ucan_token_flow.protocol_profiles.cohort]
// r[impl basalt.ucan_token_flow.protocol_profiles.selection]
// r[impl basalt.ucan_token_flow.protocol_profiles.adapter_contract]

use serde::{Deserialize, Serialize};

use crate::ucan_semantics_identity::{
    BASALT_POLICY_BLAKE3, UCAN_RUNTIME_REVISION, UCAN_SUPPORT_CATALOG_BLAKE3,
};

const VERIFIED_INPUT_NON_CLAIM_COUNT: usize = 6;

/// Claim boundaries preserved by the Basalt-owned verified input.
pub const VERIFIED_UCAN_INPUT_NON_CLAIMS: [&str; VERIFIED_INPUT_NON_CLAIM_COUNT] = [
    "key-trust",
    "revocation-freshness",
    "replay-durability",
    "policy-completeness",
    "transport-security",
    "effect-correctness",
];

/// Closed UCAN protocol profile known to Basalt.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub enum BasaltUcanProtocolProfile {
    /// Existing local compact Ed25519 profile.
    #[serde(rename = "onix-compact-ed25519-v1")]
    OnixCompactEd25519V1,
    /// Final UCAN Working Group `1.0.0` profile.
    #[serde(rename = "ucan-wg-1.0.0")]
    UcanWg100,
}

impl BasaltUcanProtocolProfile {
    /// Return the stable profile identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OnixCompactEd25519V1 => "onix-compact-ed25519-v1",
            Self::UcanWg100 => "ucan-wg-1.0.0",
        }
    }
}

/// Exact reviewed UCAN dependency cohort.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UcanProtocolCohort {
    profile: BasaltUcanProtocolProfile,
    runtime_revision: &'static str,
    support_catalog_blake3: &'static str,
    basalt_policy_blake3: &'static str,
}

impl UcanProtocolCohort {
    /// Return the selected protocol profile.
    #[must_use]
    pub const fn profile(self) -> BasaltUcanProtocolProfile {
        self.profile
    }

    /// Return the exact vendored UCAN revision.
    #[must_use]
    pub const fn runtime_revision(self) -> &'static str {
        self.runtime_revision
    }

    /// Return the reviewed UCAN support-catalog identity.
    #[must_use]
    pub const fn support_catalog_blake3(self) -> &'static str {
        self.support_catalog_blake3
    }

    /// Return the generated Basalt policy identity.
    #[must_use]
    pub const fn basalt_policy_blake3(self) -> &'static str {
        self.basalt_policy_blake3
    }
}

/// Return Basalt's current compact UCAN cohort.
#[must_use]
pub const fn compact_ucan_cohort() -> UcanProtocolCohort {
    UcanProtocolCohort {
        profile: BasaltUcanProtocolProfile::OnixCompactEd25519V1,
        runtime_revision: UCAN_RUNTIME_REVISION,
        support_catalog_blake3: UCAN_SUPPORT_CATALOG_BLAKE3,
        basalt_policy_blake3: BASALT_POLICY_BLAKE3,
    }
}

/// Stable protocol-profile admission error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UcanProtocolProfileError {
    /// The compact adapter received a different nominal profile.
    CompactAdapterMismatch,
}

impl std::fmt::Display for UcanProtocolProfileError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CompactAdapterMismatch => {
                formatter.write_str("selected UCAN profile does not match the compact adapter")
            }
        }
    }
}

impl std::error::Error for UcanProtocolProfileError {}

/// Admit only the current compact profile and exact reviewed cohort.
///
/// # Errors
/// Returns [`UcanProtocolProfileError`] for every cross-profile selection.
pub fn admit_compact_profile(
    profile: BasaltUcanProtocolProfile,
) -> Result<UcanProtocolCohort, UcanProtocolProfileError> {
    let cohort = compact_ucan_cohort();
    if profile == cohort.profile() {
        Ok(cohort)
    } else {
        Err(UcanProtocolProfileError::CompactAdapterMismatch)
    }
}

/// Application-owned authority facts projected from one verified UCAN value.
///
/// Construction remains crate-private. Caller assertions cannot construct this
/// value or turn persisted evidence into live authority.
pub struct VerifiedUcanAuthorityInput {
    profile: BasaltUcanProtocolProfile,
    source_revision: &'static str,
    support_catalog_blake3: &'static str,
    subject: String,
    operation_profile: [u8; blake3::OUT_LEN],
    request_binding: [u8; blake3::OUT_LEN],
    token_identity: [u8; blake3::OUT_LEN],
    delegated: bool,
    non_claims: [&'static str; VERIFIED_INPUT_NON_CLAIM_COUNT],
}

impl VerifiedUcanAuthorityInput {
    pub(crate) fn from_compact_verified(
        profile: BasaltUcanProtocolProfile,
        subject: &str,
        operation_profile: [u8; blake3::OUT_LEN],
        request_binding: [u8; blake3::OUT_LEN],
        token_identity: [u8; blake3::OUT_LEN],
        delegated: bool,
    ) -> Result<Self, UcanProtocolProfileError> {
        let cohort = admit_compact_profile(profile)?;
        Ok(Self {
            profile,
            source_revision: cohort.runtime_revision(),
            support_catalog_blake3: cohort.support_catalog_blake3(),
            subject: subject.to_owned(),
            operation_profile,
            request_binding,
            token_identity,
            delegated,
            non_claims: VERIFIED_UCAN_INPUT_NON_CLAIMS,
        })
    }

    /// Return the admitted nominal profile.
    #[must_use]
    pub const fn profile(&self) -> BasaltUcanProtocolProfile {
        self.profile
    }

    /// Return the exact source revision.
    #[must_use]
    pub const fn source_revision(&self) -> &'static str {
        self.source_revision
    }

    /// Return the support-catalog BLAKE3 identity.
    #[must_use]
    pub const fn support_catalog_blake3(&self) -> &'static str {
        self.support_catalog_blake3
    }

    /// Return the verified UCAN audience subject.
    #[must_use]
    pub const fn subject(&self) -> &str {
        self.subject.as_str()
    }

    /// Return the Basalt operation-profile identity.
    #[must_use]
    pub const fn operation_profile(&self) -> &[u8; blake3::OUT_LEN] {
        &self.operation_profile
    }

    /// Return the exact Basalt request binding.
    #[must_use]
    pub const fn request_binding(&self) -> &[u8; blake3::OUT_LEN] {
        &self.request_binding
    }

    /// Return the bounded compact-token identity.
    #[must_use]
    pub const fn token_identity(&self) -> &[u8; blake3::OUT_LEN] {
        &self.token_identity
    }

    /// Return whether UCAN verified a delegated rather than direct grant.
    #[must_use]
    pub const fn is_delegated(&self) -> bool {
        self.delegated
    }

    /// Return bounded claims that this input does not make.
    #[must_use]
    pub const fn non_claims(&self) -> &[&'static str; VERIFIED_INPUT_NON_CLAIM_COUNT] {
        &self.non_claims
    }
}

impl std::fmt::Debug for VerifiedUcanAuthorityInput {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("VerifiedUcanAuthorityInput")
            .field("profile", &self.profile)
            .field("source_revision", &self.source_revision)
            .field("subject", &self.subject)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_cohort_is_exact_and_final_profile_is_rejected() {
        let cohort = admit_compact_profile(BasaltUcanProtocolProfile::OnixCompactEd25519V1)
            .expect("compact profile must be admitted");
        assert_eq!(cohort.runtime_revision(), UCAN_RUNTIME_REVISION);
        assert_eq!(cohort.support_catalog_blake3(), UCAN_SUPPORT_CATALOG_BLAKE3);
        assert_eq!(cohort.basalt_policy_blake3(), BASALT_POLICY_BLAKE3);
        assert_eq!(
            admit_compact_profile(BasaltUcanProtocolProfile::UcanWg100),
            Err(UcanProtocolProfileError::CompactAdapterMismatch)
        );
    }
}
