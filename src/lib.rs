//! Public API for enforcing Nickel-authored contract policy with UCAN capability checks.
//!
//! Library consumers either parse their own Nickel-exported JSON policy or start
//! from the packaged [`DEFAULT_POLICY_JSON`] example policy, build an
//! [`EnforcementRequest`], and inspect the returned [`EnforcementReceipt`]:
//!
//! ```
//! use ucan_nickel_contracts::{CapabilityGrant, EnforcementRequest, enforce, parse_policy_json};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let policy_json = r#"{
//!   "schema_version": "ucan-nickel-contracts.policy.v1",
//!   "contracts": {
//!     "document-write": {
//!       "id": "document-write",
//!       "description": "Write bounded document resources.",
//!       "resource_prefixes": ["did:key:z6Mkresource/docs/"],
//!       "abilities": ["document/write"]
//!     }
//!   }
//! }"#;
//!
//! let policy = parse_policy_json(policy_json)?;
//! let request = EnforcementRequest::new(
//!     "document-write",
//!     "did:key:z6Mkresource/docs/report",
//!     "document/write",
//! )
//! .with_capability(CapabilityGrant::new(
//!     "did:key:z6Mkresource/docs/report",
//!     "document/write",
//! ));
//!
//! let receipt = enforce(&policy, &request)?;
//! assert!(receipt.is_allowed());
//! assert_eq!(receipt.reason(), "allowed");
//! # Ok(())
//! # }
//! ```

#![allow(clippy::multiple_crate_versions)]

use serde::Deserialize;
use serde::Serialize;
use snafu::Snafu;
use std::collections::BTreeMap;

/// Policy schema version supported by this crate.
pub const SUPPORTED_SCHEMA_VERSION: &str = "ucan-nickel-contracts.policy.v1";

/// Packaged Nickel-exported example policy JSON.
///
/// This is generated from `contracts/policy.ncl` and is included in the crate
/// package so downstream applications can smoke-test integration without
/// carrying a Nickel toolchain. Production applications may provide their own
/// reviewed policy JSON to [`parse_policy_json`].
pub const DEFAULT_POLICY_JSON: &str = include_str!("../contracts/generated/policy.json");

/// Nickel-exported policy document consumed by [`enforce`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Policy {
    /// Schema version marker checked by [`parse_policy_json`].
    pub schema_version: String,
    /// Policies keyed by contract id.
    pub contracts: BTreeMap<String, ContractPolicy>,
}

impl Policy {
    /// Return a contract policy by id.
    #[must_use]
    pub fn contract(&self, id: &str) -> Option<&ContractPolicy> {
        self.contracts.get(id)
    }

    /// Return the schema version string carried by this policy.
    #[must_use]
    pub const fn schema_version(&self) -> &str {
        self.schema_version.as_str()
    }
}

/// Per-contract resource and ability allowlist loaded from policy JSON.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ContractPolicy {
    /// Contract id; must match the key in [`Policy::contracts`].
    pub id: String,
    /// Human-readable policy description.
    #[serde(default)]
    pub description: String,
    /// Resource prefixes this contract permits.
    pub resource_prefixes: Vec<String>,
    /// Abilities this contract permits.
    pub abilities: Vec<String>,
}

impl ContractPolicy {
    /// Return true when `resource` is under one of the contract's permitted prefixes.
    #[must_use]
    pub fn permits_resource(&self, resource: &str) -> bool {
        self.resource_prefixes
            .iter()
            .any(|prefix| resource.starts_with(prefix))
    }

    /// Return true when `ability` is listed by this contract.
    #[must_use]
    pub fn permits_ability(&self, ability: &str) -> bool {
        self.abilities
            .iter()
            .any(|contract_ability| contract_ability == ability)
    }
}

/// Caller-supplied UCAN-like capability grant used by [`enforce`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct CapabilityGrant {
    /// Authorized resource scope.
    pub resource: String,
    /// Authorized ability.
    pub ability: String,
}

impl CapabilityGrant {
    /// Construct a capability grant from resource and ability strings.
    #[must_use]
    pub fn new(resource: impl Into<String>, ability: impl Into<String>) -> Self {
        Self {
            resource: resource.into(),
            ability: ability.into(),
        }
    }

    /// Return the authorized resource scope.
    #[must_use]
    pub const fn resource(&self) -> &str {
        self.resource.as_str()
    }

    /// Return the authorized ability.
    #[must_use]
    pub const fn ability(&self) -> &str {
        self.ability.as_str()
    }
}

/// Public enforcement request consumed by [`enforce`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct EnforcementRequest {
    /// Contract id to enforce.
    pub contract: String,
    /// Requested resource.
    pub resource: String,
    /// Requested ability.
    pub ability: String,
    /// Caller-supplied capability grants.
    pub capabilities: Vec<CapabilityGrant>,
}

impl EnforcementRequest {
    /// Construct a request without any capability grants.
    #[must_use]
    pub fn new(
        contract: impl Into<String>,
        resource: impl Into<String>,
        ability: impl Into<String>,
    ) -> Self {
        Self {
            contract: contract.into(),
            resource: resource.into(),
            ability: ability.into(),
            capabilities: Vec::new(),
        }
    }

    /// Append one capability grant and return the updated request.
    #[must_use]
    pub fn with_capability(mut self, capability: CapabilityGrant) -> Self {
        self.capabilities.push(capability);
        self
    }

    /// Replace the request's capability grants.
    #[must_use]
    pub fn with_capabilities(mut self, capabilities: impl Into<Vec<CapabilityGrant>>) -> Self {
        self.capabilities = capabilities.into();
        self
    }

    /// Return the requested contract id.
    #[must_use]
    pub const fn contract(&self) -> &str {
        self.contract.as_str()
    }

    /// Return the requested resource.
    #[must_use]
    pub const fn resource(&self) -> &str {
        self.resource.as_str()
    }

    /// Return the requested ability.
    #[must_use]
    pub const fn ability(&self) -> &str {
        self.ability.as_str()
    }

    /// Return caller-supplied capability grants.
    #[must_use]
    pub const fn capabilities(&self) -> &[CapabilityGrant] {
        self.capabilities.as_slice()
    }
}

/// Stable JSON-serializable enforcement result.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct EnforcementReceipt {
    /// Whether the request was allowed.
    pub allowed: bool,
    /// Stable human-readable allow/deny reason.
    pub reason: String,
}

impl EnforcementReceipt {
    /// Construct an allowed receipt.
    #[must_use]
    pub fn allowed() -> Self {
        Self {
            allowed: true,
            reason: "allowed".to_owned(),
        }
    }

    /// Construct a denied receipt with a stable reason.
    #[must_use]
    pub fn denied(reason: impl Into<String>) -> Self {
        Self {
            allowed: false,
            reason: reason.into(),
        }
    }

    /// Return true when the request was allowed.
    #[must_use]
    pub const fn is_allowed(&self) -> bool {
        self.allowed
    }

    /// Return true when the request was denied.
    #[must_use]
    pub const fn is_denied(&self) -> bool {
        !self.allowed
    }

    /// Return the receipt reason.
    #[must_use]
    pub const fn reason(&self) -> &str {
        self.reason.as_str()
    }
}

#[derive(Debug, Snafu)]
pub enum EnforcementError {
    #[snafu(display("failed to parse policy JSON: {source}"))]
    PolicyJson { source: serde_json::Error },
    #[snafu(display("unsupported policy schema version {version:?}"))]
    UnsupportedSchema { version: String },
    #[snafu(display("invalid UCAN capability: {source}"))]
    InvalidCapability { source: ucan::Error },
    #[snafu(display("UCAN authorization check failed: {source}"))]
    UcanAuthorization {
        source: ucan::VerifiedPrimitiveError,
    },
}

pub type EnforcementResult<T> = Result<T, EnforcementError>;

/// Load the packaged example policy JSON.
///
/// # Errors
///
/// Returns [`EnforcementError`] if the checked-in generated policy artifact is malformed
/// or carries an unsupported schema version.
pub fn default_policy() -> EnforcementResult<Policy> {
    parse_policy_json(DEFAULT_POLICY_JSON)
}

/// Load a Nickel-exported JSON policy and validate the schema version expected by this crate.
///
/// # Errors
///
/// Returns [`EnforcementError`] when JSON parsing fails or the policy schema version is unsupported.
pub fn parse_policy_json(input: &str) -> EnforcementResult<Policy> {
    let policy: Policy =
        serde_json::from_str(input).map_err(|source| EnforcementError::PolicyJson { source })?;
    if policy.schema_version != SUPPORTED_SCHEMA_VERSION {
        return Err(EnforcementError::UnsupportedSchema {
            version: policy.schema_version,
        });
    }
    Ok(policy)
}

/// Enforce a Nickel-authored policy contract and a caller-supplied UCAN capability set.
///
/// # Errors
///
/// Returns [`EnforcementError`] when capability construction or UCAN authorization validation fails.
pub fn enforce(
    policy: &Policy,
    request: &EnforcementRequest,
) -> EnforcementResult<EnforcementReceipt> {
    let Some(contract) = policy.contracts.get(request.contract.as_str()) else {
        return Ok(EnforcementReceipt::denied(format!(
            "unknown contract {:?}",
            request.contract
        )));
    };

    if contract.id != request.contract {
        return Ok(EnforcementReceipt::denied(format!(
            "contract key {:?} does not match policy id {:?}",
            request.contract, contract.id
        )));
    }

    if !contract
        .abilities
        .iter()
        .any(|ability| ability == &request.ability)
    {
        return Ok(EnforcementReceipt::denied(format!(
            "ability {:?} is outside contract {:?}",
            request.ability, request.contract
        )));
    }

    if !contract
        .resource_prefixes
        .iter()
        .any(|prefix| request.resource.starts_with(prefix))
    {
        return Ok(EnforcementReceipt::denied(format!(
            "resource {:?} is outside contract {:?}",
            request.resource, request.contract
        )));
    }

    let capabilities = request
        .capabilities
        .iter()
        .map(|capability| {
            ucan::CapabilityDocument::new(capability.resource.clone(), capability.ability.clone())
                .map_err(|source| EnforcementError::InvalidCapability { source })
        })
        .collect::<EnforcementResult<Vec<_>>>()?;

    let authorized = ucan::capability_set_authorizes(
        capabilities.as_slice(),
        &request.resource,
        &request.ability,
    )
    .map_err(|source| EnforcementError::UcanAuthorization { source })?;

    if authorized {
        Ok(EnforcementReceipt::allowed())
    } else {
        Ok(EnforcementReceipt::denied(
            "ucan capability set does not authorize request",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const POLICY_JSON: &str = r#"{
      "schema_version": "ucan-nickel-contracts.policy.v1",
      "contracts": {
        "document-write": {
          "id": "document-write",
          "description": "Write bounded document resources.",
          "resource_prefixes": ["did:key:z6Mkresource/docs/"],
          "abilities": ["document/write"]
        }
      }
    }"#;

    fn request(resource: &str, ability: &str) -> EnforcementRequest {
        EnforcementRequest {
            contract: "document-write".to_owned(),
            resource: resource.to_owned(),
            ability: ability.to_owned(),
            capabilities: vec![CapabilityGrant::new(resource, ability)],
        }
    }

    #[test]
    fn allows_policy_and_ucan_authorized_request() {
        let policy = parse_policy_json(POLICY_JSON).expect("policy should parse");
        let receipt = enforce(
            &policy,
            &request("did:key:z6Mkresource/docs/report", "document/write"),
        )
        .expect("enforcement should not error");

        assert_eq!(receipt, EnforcementReceipt::allowed());
    }

    #[test]
    fn denies_resource_outside_nickel_contract() {
        let policy = parse_policy_json(POLICY_JSON).expect("policy should parse");
        let receipt = enforce(
            &policy,
            &request("did:key:z6Mkresource/private/report", "document/write"),
        )
        .expect("enforcement should not error");

        assert!(!receipt.allowed);
        assert!(receipt.reason.contains("outside contract"));
    }

    #[test]
    fn denies_ability_outside_nickel_contract() {
        let policy = parse_policy_json(POLICY_JSON).expect("policy should parse");
        let receipt = enforce(
            &policy,
            &request("did:key:z6Mkresource/docs/report", "document/delete"),
        )
        .expect("enforcement should not error");

        assert!(!receipt.allowed);
        assert!(receipt.reason.contains("outside contract"));
    }

    #[test]
    fn denies_missing_ucan_capability_even_when_contract_allows() {
        let policy = parse_policy_json(POLICY_JSON).expect("policy should parse");
        let mut request = request("did:key:z6Mkresource/docs/report", "document/write");
        request.capabilities = vec![CapabilityGrant::new(
            "did:key:z6Mkresource/docs/other",
            "document/write",
        )];

        let receipt = enforce(&policy, &request).expect("enforcement should not error");

        assert!(!receipt.allowed);
        assert!(receipt.reason.contains("ucan capability set"));
    }
}
