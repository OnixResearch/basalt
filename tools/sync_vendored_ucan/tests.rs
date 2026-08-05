use crate::model::{
    EXECUTABLE_FILE_MODE, REGULAR_FILE_MODE, REQUIRED_SNAPSHOT_PATHS, SnapshotEntry,
    render_manifest_material, validate_required_entries,
};

const TEST_DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const TEST_BYTE_LEN: u64 = 1;

fn entry(path: &str) -> SnapshotEntry {
    SnapshotEntry {
        mode: REGULAR_FILE_MODE,
        digest: TEST_DIGEST.to_owned(),
        path: path.to_owned(),
        byte_len: TEST_BYTE_LEN,
    }
}

#[test]
fn required_license_and_source_inventory_is_admitted() {
    let entries = REQUIRED_SNAPSHOT_PATHS
        .iter()
        .map(|path| entry(path))
        .collect::<Vec<_>>();

    assert!(validate_required_entries(&entries).is_ok());
}

#[test]
fn missing_license_is_rejected() {
    let entries = REQUIRED_SNAPSHOT_PATHS
        .iter()
        .filter(|path| !path.ends_with("AGPL-3.0-or-later.txt"))
        .map(|path| entry(path))
        .collect::<Vec<_>>();

    let error =
        validate_required_entries(&entries).expect_err("missing AGPL license text must fail");
    assert!(error.contains("AGPL-3.0-or-later.txt"));
}

#[test]
fn manifest_material_is_deterministic_and_mode_sensitive() {
    let regular = entry("src/lib.rs");
    let mut executable = regular.clone();
    executable.mode = EXECUTABLE_FILE_MODE;

    let first = render_manifest_material(&[regular.clone(), executable.clone()]);
    let second = render_manifest_material(&[regular, executable]);

    assert_eq!(first, second);
    assert!(first.contains(REGULAR_FILE_MODE));
    assert!(first.contains(EXECUTABLE_FILE_MODE));
}
