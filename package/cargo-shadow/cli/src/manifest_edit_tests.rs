//! The unit tests of the manifest transaction around cargo's breaking update.

use std::cell::RefCell;
use std::fs;

use crate::manifest_edit::{ManifestEdit, record_manifest_output, restore_manifests, snapshot_manifests};
use crate::test_scratch::scratch;

fn edits_for(path: &std::path::Path) -> RefCell<Vec<ManifestEdit>> {
    let edits: RefCell<Vec<ManifestEdit>> = RefCell::new(Vec::new());
    snapshot_manifests(&[path.to_path_buf()], &edits).expect("snapshots");
    return edits;
}

#[test]
fn a_manifest_that_still_holds_the_runs_output_is_put_back() {
    let root = scratch();
    let path = root.path().join("Cargo.toml");
    fs::write(&path, "before").expect("manifest");
    let edits = edits_for(&path);
    fs::write(&path, "cargo's edit").expect("cargo edits it");
    record_manifest_output(&edits);

    let warnings = restore_manifests(&edits).expect("restores");

    assert!(warnings.is_empty());
    assert_eq!(fs::read_to_string(&path).expect("read"), "before");
}

#[test]
fn a_manifest_another_process_changed_after_the_run_is_left_alone_with_a_warning() {
    let root = scratch();
    let path = root.path().join("Cargo.toml");
    fs::write(&path, "before").expect("manifest");
    let edits = edits_for(&path);
    fs::write(&path, "cargo's edit").expect("cargo edits it");
    record_manifest_output(&edits);
    fs::write(&path, "somebody else's edit").expect("another process");

    let warnings = restore_manifests(&edits).expect("restores");

    assert_eq!(warnings.len(), 1);
    assert_eq!(fs::read_to_string(&path).expect("read"), "somebody else's edit");
}

#[test]
fn a_manifest_whose_output_was_never_recorded_is_never_restored() {
    let root = scratch();
    let path = root.path().join("Cargo.toml");
    fs::write(&path, "before").expect("manifest");
    let edits = edits_for(&path);
    fs::write(&path, "unknown author").expect("someone edits it");

    let warnings = restore_manifests(&edits).expect("restores");

    assert_eq!(warnings.len(), 1);
    assert_eq!(fs::read_to_string(&path).expect("read"), "unknown author");
}

#[test]
fn an_unchanged_manifest_needs_nothing_and_a_missing_one_stops_the_snapshot() {
    let root = scratch();
    let path = root.path().join("Cargo.toml");
    fs::write(&path, "before").expect("manifest");
    let edits = edits_for(&path);
    record_manifest_output(&edits);

    assert!(restore_manifests(&edits).expect("restores").is_empty());
    assert!(snapshot_manifests(&[root.path().join("missing.toml")], &RefCell::new(Vec::new())).is_err());
}
