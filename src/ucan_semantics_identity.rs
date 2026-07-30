//! Immutable identities shared by UCAN semantics replay and release receipts.

/// Schema accepted by the UCAN semantics fixture adapter.
pub const UCAN_SEMANTICS_FIXTURE_SCHEMA: &str = "basalt.ucan-semantics-fixture.v1";
/// Reviewed UCAN revision in Basalt's validated vendored source boundary.
pub const UCAN_RUNTIME_REVISION: &str = "34449ba1f1a9ef991225e4e6683b5d6aca8c07cf";
/// Reviewed UCAN revision that produced the bound support catalog.
pub const UCAN_SUPPORT_CATALOG_REVISION: &str = "34449ba1f1a9ef991225e4e6683b5d6aca8c07cf";
/// BLAKE3 identity of the reviewed UCAN integration support catalog.
pub const UCAN_SUPPORT_CATALOG_BLAKE3: &str =
    "3266e7b1f2ce66f2ab5c8cbd7028f8bcf3502c7e64fe8d058c7b7e1f762dedb6";
/// BLAKE3 identity of Basalt's checked-in generated policy artifact.
pub const BASALT_POLICY_BLAKE3: &str =
    "7d223f48abcf31c9ba932d7434babfa675122f393100547ff09a595ee1fe8adb";
