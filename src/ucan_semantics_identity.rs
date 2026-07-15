//! Immutable identities shared by UCAN semantics replay and release receipts.

/// Schema accepted by the UCAN semantics fixture adapter.
pub const UCAN_SEMANTICS_FIXTURE_SCHEMA: &str = "basalt.ucan-semantics-fixture.v1";
/// Fetchable UCAN runtime revision linked by this Basalt release.
pub const UCAN_RUNTIME_REVISION: &str = "2aad993027d48ff148028c537cdaf91f6e5285ca";
/// Reviewed UCAN revision that produced the bound support catalog.
pub const UCAN_SUPPORT_CATALOG_REVISION: &str = "e362c3fa2236d5b9d61423921a820cd12dd64e51";
/// BLAKE3 identity of the reviewed UCAN integration support catalog.
pub const UCAN_SUPPORT_CATALOG_BLAKE3: &str =
    "02fb9fec563b5df9f40a58bbbddfd0bcf5abe74da373ae0b506712647e0ee6ad";
/// BLAKE3 identity of Basalt's checked-in generated policy artifact.
pub const BASALT_POLICY_BLAKE3: &str =
    "7d223f48abcf31c9ba932d7434babfa675122f393100547ff09a595ee1fe8adb";
