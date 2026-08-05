//! Live UCAN holder verification composed with Basalt policy authority.
//!
//! The shell prebuilds policy authority and evidence before UCAN performs its
//! final replay commit. The success branch only moves prebuilt values.

mod core;
mod error;
mod receipt;
mod shell;
mod types;

pub use core::bind_enforcement_action;
pub use error::UcanEnforcementFailure;
pub use receipt::{UCAN_ENFORCEMENT_NON_CLAIMS, UcanEnforcementReceipt};
pub use shell::{LiveUcanEnforcement, verify_ucan_enforcement_request};
pub use types::{
    EnforcementAction, UcanEnforcementBoundaries, UcanEnforcementReason, UcanEnforcementRequest,
};

#[cfg(test)]
mod tests;
