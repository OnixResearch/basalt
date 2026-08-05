use crate::filesystem::{
    collect_snapshot_entries, compare_text_file, remove_directory_if_present,
};
use crate::model::{
    DESTINATION_DIRECTORY, MANIFEST_FILE, Mode, REVISION_FILE, SNAPSHOT_SCHEMA, SOURCE_REVISION,
    STAGING_DIRECTORY, render_manifest_material, validate_required_entries,
};
use crate::process::{blake3_bytes, extract_source_revision};
use std::fs;
use std::path::Path;

pub(crate) fn run(root: &Path, mode: Mode<'_>) -> Result<(), String> {
    assert!(!SOURCE_REVISION.is_empty());
    match mode {
        Mode::Check => check_snapshot(root),
        Mode::CheckSource(source) => check_source(root, source),
        Mode::Import(source) => import_snapshot(root, source),
    }
}

fn check_snapshot(root: &Path) -> Result<(), String> {
    let destination = root.join(DESTINATION_DIRECTORY);
    let entries = collect_snapshot_entries(&destination)?;
    validate_required_entries(&entries)?;
    let manifest = render_manifest(&entries)?;
    compare_text_file(&root.join(MANIFEST_FILE), &manifest)?;
    compare_text_file(
        &root.join(REVISION_FILE),
        &format!("{SOURCE_REVISION}\n"),
    )?;
    println!(
        "vendored UCAN snapshot passed: revision {SOURCE_REVISION}, {} files",
        entries.len()
    );
    Ok(())
}

fn check_source(root: &Path, source: &Path) -> Result<(), String> {
    let staging = root.join(STAGING_DIRECTORY);
    extract_source_revision(source, &staging)?;
    let result = (|| {
        let entries = collect_snapshot_entries(&staging)?;
        validate_required_entries(&entries)?;
        let manifest = render_manifest(&entries)?;
        compare_text_file(&root.join(MANIFEST_FILE), &manifest)
    })();
    remove_directory_if_present(&staging)?;
    result?;
    println!(
        "vendored UCAN source parity passed: {} at {SOURCE_REVISION}",
        source.display()
    );
    Ok(())
}

fn import_snapshot(root: &Path, source: &Path) -> Result<(), String> {
    let staging = root.join(STAGING_DIRECTORY);
    extract_source_revision(source, &staging)?;
    let entries = collect_snapshot_entries(&staging)?;
    validate_required_entries(&entries)?;
    let manifest = render_manifest(&entries)?;
    let destination = root.join(DESTINATION_DIRECTORY);
    remove_directory_if_present(&destination)?;
    let destination_parent = destination
        .parent()
        .ok_or_else(|| format!("destination has no parent: {}", destination.display()))?;
    fs::create_dir_all(destination_parent)
        .map_err(|error| format!("cannot create {}: {error}", destination_parent.display()))?;
    fs::rename(&staging, &destination).map_err(|error| {
        format!(
            "cannot move {} to {}: {error}",
            staging.display(),
            destination.display()
        )
    })?;
    fs::write(root.join(REVISION_FILE), format!("{SOURCE_REVISION}\n"))
        .map_err(|error| format!("cannot write {REVISION_FILE}: {error}"))?;
    fs::write(root.join(MANIFEST_FILE), manifest)
        .map_err(|error| format!("cannot write {MANIFEST_FILE}: {error}"))?;
    check_snapshot(root)
}

fn render_manifest(entries: &[crate::model::SnapshotEntry]) -> Result<String, String> {
    let material = render_manifest_material(entries);
    let root_digest = blake3_bytes(material.as_bytes())?;
    Ok(format!(
        "schema\t{SNAPSHOT_SCHEMA}\nsource-revision\t{SOURCE_REVISION}\nentry-count\t{}\ncontent-blake3\t{root_digest}\n{material}",
        entries.len()
    ))
}
