#!/usr/bin/env -S CARGO_TARGET_DIR=/tmp/basalt-sync-vendored-ucan nix shell "github:nix-community/fenix?rev=4e49ef62eedaa5c149d62b63b3f53844e1cc45d7#complete.toolchain" nixpkgs#gcc nixpkgs#git nixpkgs#gnutar nixpkgs#b3sum -c cargo -q -Zscript

use std::env;
use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

const SOURCE_REVISION: &str = "34449ba1f1a9ef991225e4e6683b5d6aca8c07cf";
const SNAPSHOT_SCHEMA: &str = "basalt.vendored-ucan.snapshot.v1";
const DESTINATION_DIRECTORY: &str = "vendor/ucan";
const REVISION_FILE: &str = "vendor/ucan-revision.txt";
const MANIFEST_FILE: &str = "vendor/ucan.snapshot";
const STAGING_DIRECTORY: &str = "target/vendored-ucan-import";
const MODE_CHECK: &str = "--check";
const MODE_CHECK_SOURCE: &str = "--check-source";
const MODE_IMPORT: &str = "--import";
const CHECK_ARGUMENT_COUNT: usize = 2;
const SOURCE_ARGUMENT_COUNT: usize = 3;
const BLAKE3_HEX_LENGTH: usize = 64;
const MAXIMUM_SNAPSHOT_ENTRIES: usize = 2_048;
const BYTES_PER_MEBIBYTE: u64 = 1_048_576;
const MAXIMUM_SNAPSHOT_MEBIBYTES: u64 = 64;
const MAXIMUM_SNAPSHOT_BYTES: u64 = MAXIMUM_SNAPSHOT_MEBIBYTES * BYTES_PER_MEBIBYTE;
const EXECUTABLE_MODE_MASK: u32 = 0o111;
const REGULAR_FILE_MODE: &str = "100644";
const EXECUTABLE_FILE_MODE: &str = "100755";
const GIT_COMMIT_SUFFIX: &str = "^{commit}";

const SOURCE_SELECTION: &[&str] = &[
    "Cargo.toml",
    "Cargo.lock",
    "README.md",
    "LICENSES",
    "license-policy.tsv",
    "docs/licensing.md",
    "docs/production-readiness.json",
    "docs/production-readiness.md",
    "docs/verified-boundaries.md",
    "src",
    "crates/ucan-core",
    "vendor/verified-logic",
    "vendor/verified-logic-revision.txt",
    "tools/check-production-readiness",
    "tools/check-aeneas-time-bounds-proof",
];

const REQUIRED_SNAPSHOT_PATHS: &[&str] = &[
    "Cargo.toml",
    "src/lib.rs",
    "src/token.rs",
    "src/token/holder_invocation.rs",
    "crates/ucan-core/Cargo.toml",
    "crates/ucan-core/LICENSE",
    "crates/ucan-core/src/lib.rs",
    "vendor/verified-logic/Cargo.toml",
    "vendor/verified-logic/LICENSE",
    "vendor/verified-logic/src/lib.rs",
    "LICENSES/AGPL-3.0-or-later.txt",
    "LICENSES/MPL-2.0.txt",
    "docs/licensing.md",
    "docs/production-readiness.json",
    "docs/verified-boundaries.md",
    "license-policy.tsv",
];

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct SnapshotEntry {
    mode: &'static str,
    digest: String,
    path: String,
    byte_len: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Mode<'a> {
    Check,
    CheckSource(&'a Path),
    Import(&'a Path),
}

fn parse_mode<'a>(arguments: &'a [String]) -> Result<Mode<'a>, String> {
    if arguments.len() == CHECK_ARGUMENT_COUNT && arguments[1] == MODE_CHECK {
        return Ok(Mode::Check);
    }
    if arguments.len() == SOURCE_ARGUMENT_COUNT && arguments[1] == MODE_CHECK_SOURCE {
        return Ok(Mode::CheckSource(Path::new(&arguments[2])));
    }
    if arguments.len() == SOURCE_ARGUMENT_COUNT && arguments[1] == MODE_IMPORT {
        return Ok(Mode::Import(Path::new(&arguments[2])));
    }
    Err(format!(
        "usage: sync-vendored-ucan.rs {MODE_CHECK} | {MODE_CHECK_SOURCE} SOURCE_REPOSITORY | {MODE_IMPORT} SOURCE_REPOSITORY"
    ))
}

fn run(root: &Path, mode: Mode<'_>) -> Result<(), String> {
    assert!(!SOURCE_REVISION.is_empty());
    assert!(!SOURCE_SELECTION.is_empty());
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
    let manifest = render_manifest(root, &entries)?;
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
        let manifest = render_manifest(root, &entries)?;
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
    let manifest = render_manifest(root, &entries)?;
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

fn extract_source_revision(source: &Path, staging: &Path) -> Result<(), String> {
    verify_source_revision(source)?;
    remove_directory_if_present(staging)?;
    fs::create_dir_all(staging)
        .map_err(|error| format!("cannot create {}: {error}", staging.display()))?;

    let mut archive_command = Command::new("git");
    archive_command
        .arg("-C")
        .arg(source)
        .arg("archive")
        .arg("--format=tar")
        .arg(SOURCE_REVISION)
        .args(SOURCE_SELECTION)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut archive = archive_command
        .spawn()
        .map_err(|error| format!("cannot start git archive: {error}"))?;
    let archive_stdout = archive
        .stdout
        .take()
        .ok_or_else(|| String::from("git archive stdout was not piped"))?;
    let extraction = Command::new("tar")
        .arg("--extract")
        .arg("--file=-")
        .arg("--directory")
        .arg(staging)
        .arg("--no-same-owner")
        .stdin(Stdio::from(archive_stdout))
        .output()
        .map_err(|error| format!("cannot start tar extraction: {error}"))?;
    let archive_output = archive
        .wait_with_output()
        .map_err(|error| format!("cannot wait for git archive: {error}"))?;
    if !archive_output.status.success() {
        return Err(format!(
            "git archive failed: {}",
            String::from_utf8_lossy(&archive_output.stderr)
        ));
    }
    if !extraction.status.success() {
        return Err(format!(
            "tar extraction failed: {}",
            String::from_utf8_lossy(&extraction.stderr)
        ));
    }
    Ok(())
}

fn verify_source_revision(source: &Path) -> Result<(), String> {
    let commit = format!("{SOURCE_REVISION}{GIT_COMMIT_SUFFIX}");
    let output = Command::new("git")
        .arg("-C")
        .arg(source)
        .arg("cat-file")
        .arg("-e")
        .arg(&commit)
        .output()
        .map_err(|error| format!("cannot inspect source revision: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "source repository does not contain commit {SOURCE_REVISION}: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

fn collect_snapshot_entries(snapshot_root: &Path) -> Result<Vec<SnapshotEntry>, String> {
    if !snapshot_root.is_dir() {
        return Err(format!(
            "snapshot directory does not exist: {}",
            snapshot_root.display()
        ));
    }
    let mut directories = vec![snapshot_root.to_path_buf()];
    let mut files = Vec::new();
    while let Some(directory) = directories.pop() {
        let mut children = fs::read_dir(&directory)
            .map_err(|error| format!("cannot read {}: {error}", directory.display()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("cannot enumerate {}: {error}", directory.display()))?;
        children.sort_by_key(std::fs::DirEntry::file_name);
        for child in children {
            let file_type = child
                .file_type()
                .map_err(|error| format!("cannot inspect {}: {error}", child.path().display()))?;
            if file_type.is_symlink() {
                return Err(format!("snapshot symlink is not admitted: {}", child.path().display()));
            }
            if file_type.is_dir() {
                directories.push(child.path());
            } else if file_type.is_file() {
                files.push(child.path());
            } else {
                return Err(format!("snapshot entry has unsupported type: {}", child.path().display()));
            }
            if files.len() + directories.len() > MAXIMUM_SNAPSHOT_ENTRIES {
                return Err(format!(
                    "snapshot exceeds entry limit {MAXIMUM_SNAPSHOT_ENTRIES}"
                ));
            }
        }
    }
    files.sort();
    build_snapshot_entries(snapshot_root, &files)
}

fn build_snapshot_entries(
    snapshot_root: &Path,
    files: &[PathBuf],
) -> Result<Vec<SnapshotEntry>, String> {
    let mut total_bytes = 0_u64;
    let mut entries = Vec::with_capacity(files.len());
    for file in files {
        let metadata = fs::metadata(file)
            .map_err(|error| format!("cannot read metadata for {}: {error}", file.display()))?;
        total_bytes = total_bytes
            .checked_add(metadata.len())
            .ok_or_else(|| String::from("snapshot byte total overflowed"))?;
        if total_bytes > MAXIMUM_SNAPSHOT_BYTES {
            return Err(format!(
                "snapshot exceeds byte limit {MAXIMUM_SNAPSHOT_BYTES}"
            ));
        }
        let relative = safe_relative_path(snapshot_root, file)?;
        let mode = if metadata.permissions().mode() & EXECUTABLE_MODE_MASK == 0 {
            REGULAR_FILE_MODE
        } else {
            EXECUTABLE_FILE_MODE
        };
        entries.push(SnapshotEntry {
            mode,
            digest: blake3_file(file)?,
            path: relative,
            byte_len: metadata.len(),
        });
    }
    assert_eq!(entries.len(), files.len());
    Ok(entries)
}

fn safe_relative_path(root: &Path, path: &Path) -> Result<String, String> {
    let relative = path
        .strip_prefix(root)
        .map_err(|error| format!("{} escaped {}: {error}", path.display(), root.display()))?;
    let mut components = Vec::new();
    for component in relative.components() {
        match component {
            Component::Normal(value) => {
                let text = value
                    .to_str()
                    .ok_or_else(|| format!("snapshot path is not UTF-8: {}", path.display()))?;
                if text.chars().any(char::is_control) || text.contains('\t') {
                    return Err(format!("snapshot path contains a control character: {}", path.display()));
                }
                components.push(text);
            }
            _ => return Err(format!("snapshot path is not normalized: {}", path.display())),
        }
    }
    if components.is_empty() {
        return Err(format!("snapshot file has an empty relative path: {}", path.display()));
    }
    Ok(components.join("/"))
}

fn blake3_file(path: &Path) -> Result<String, String> {
    let output = Command::new("b3sum")
        .arg("--no-names")
        .arg(path)
        .output()
        .map_err(|error| format!("cannot start b3sum for {}: {error}", path.display()))?;
    parse_blake3_output(&output, path.to_string_lossy().as_ref())
}

fn blake3_bytes(bytes: &[u8]) -> Result<String, String> {
    let mut child = Command::new("b3sum")
        .arg("--no-names")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("cannot start b3sum for manifest material: {error}"))?;
    let mut input = child
        .stdin
        .take()
        .ok_or_else(|| String::from("b3sum stdin was not piped"))?;
    input
        .write_all(bytes)
        .map_err(|error| format!("cannot write manifest material to b3sum: {error}"))?;
    drop(input);
    let output = child
        .wait_with_output()
        .map_err(|error| format!("cannot wait for manifest b3sum: {error}"))?;
    parse_blake3_output(&output, "manifest material")
}

fn parse_blake3_output(output: &std::process::Output, source: &str) -> Result<String, String> {
    if !output.status.success() {
        return Err(format!(
            "b3sum failed for {source}: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let digest = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("b3sum output was not UTF-8 for {source}: {error}"))?
        .trim()
        .to_owned();
    if digest.len() != BLAKE3_HEX_LENGTH
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(format!("b3sum returned a non-canonical digest for {source}"));
    }
    Ok(digest)
}

fn validate_required_entries(entries: &[SnapshotEntry]) -> Result<(), String> {
    if entries.is_empty() {
        return Err(String::from("vendored UCAN snapshot is empty"));
    }
    for required in REQUIRED_SNAPSHOT_PATHS {
        if !entries.iter().any(|entry| entry.path == *required) {
            return Err(format!("vendored UCAN snapshot is missing required path {required}"));
        }
    }
    Ok(())
}

fn render_manifest(_root: &Path, entries: &[SnapshotEntry]) -> Result<String, String> {
    let material = render_manifest_material(entries);
    let root_digest = blake3_bytes(material.as_bytes())?;
    Ok(format!(
        "schema\t{SNAPSHOT_SCHEMA}\nsource-revision\t{SOURCE_REVISION}\nentry-count\t{}\ncontent-blake3\t{root_digest}\n{material}",
        entries.len()
    ))
}

fn render_manifest_material(entries: &[SnapshotEntry]) -> String {
    let mut material = String::new();
    for entry in entries {
        material.push_str(entry.mode);
        material.push('\t');
        material.push_str(&entry.digest);
        material.push('\t');
        material.push_str(&entry.byte_len.to_string());
        material.push('\t');
        material.push_str(&entry.path);
        material.push('\n');
    }
    material
}

fn compare_text_file(path: &Path, expected: &str) -> Result<(), String> {
    let actual = fs::read_to_string(path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    if actual != expected {
        return Err(format!(
            "{} is stale; run `tools/sync-vendored-ucan.rs --import SOURCE_REPOSITORY`",
            path.display()
        ));
    }
    Ok(())
}

fn remove_directory_if_present(path: &Path) -> Result<(), String> {
    if path.exists() {
        fs::remove_dir_all(path)
            .map_err(|error| format!("cannot remove {}: {error}", path.display()))?;
    }
    Ok(())
}

fn main() -> ExitCode {
    let arguments = env::args().collect::<Vec<_>>();
    let root = match env::current_dir() {
        Ok(root) => root,
        Err(error) => {
            eprintln!("error: cannot read current directory: {error}");
            return ExitCode::FAILURE;
        }
    };
    let result = parse_mode(&arguments).and_then(|mode| run(&root, mode));
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EXECUTABLE_FILE_MODE;
    use super::REGULAR_FILE_MODE;
    use super::REQUIRED_SNAPSHOT_PATHS;
    use super::SnapshotEntry;
    use super::render_manifest_material;
    use super::validate_required_entries;

    const TEST_DIGEST: &str =
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
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

        let error = validate_required_entries(&entries)
            .expect_err("missing AGPL license text must fail");
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
}
