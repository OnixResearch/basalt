#![allow(clippy::cargo_common_metadata)]
#![allow(clippy::multiple_crate_versions)]

use serde::Deserialize;
use serde::Serialize;
use snafu::Snafu;
use std::collections::BTreeMap;

const SUPPORTED_SCHEMA_VERSION: &str = "ucan-nickel-contracts.policy.v1";

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Policy {
    pub schema_version: String,
    pub contracts: BTreeMap<String, ContractPolicy>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ContractPolicy {
    pub id: String,
    #[serde(default)]
    pub description: String,
    pub resource_prefixes: Vec<String>,
    pub abilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityGrant {
    pub resource: String,
    pub ability: String,
}

impl CapabilityGrant {
    #[must_use]
    pub fn new(resource: impl Into<String>, ability: impl Into<String>) -> Self {
        Self {
            resource: resource.into(),
            ability: ability.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnforcementRequest {
    pub contract: String,
    pub resource: String,
    pub ability: String,
    pub capabilities: Vec<CapabilityGrant>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct EnforcementReceipt {
    pub allowed: bool,
    pub reason: String,
}

impl EnforcementReceipt {
    #[must_use]
    pub fn allowed() -> Self {
        Self {
            allowed: true,
            reason: "allowed".to_owned(),
        }
    }

    #[must_use]
    pub fn denied(reason: impl Into<String>) -> Self {
        Self {
            allowed: false,
            reason: reason.into(),
        }
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
