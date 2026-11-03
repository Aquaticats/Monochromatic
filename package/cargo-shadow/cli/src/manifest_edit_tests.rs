//! The unit tests of the manifest transaction around cargo's breaking update.

use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;

use crate::journal::KeepAs;
use crate::manifest_edit::{ManifestEdit, restore_manifests, run_recording_manifests, snapshot_manifests};
use crate::test_scratch::scratch;

const ROOT_A: &str = "package/a";
const ROOT_B: &str = "package/b";

fn keep() -> KeepAs {
    return KeepAs::next_to_target(".displaced");
}

fn edits_for(crate_dir: &str, path: &std::path::Path) -> RefCell<Vec<ManifestEdit>> {
    let edits: RefCell<Vec<ManifestEdit>> = RefCell::new(Vec::new());
    snapshot_manifests(crate_dir, &[path.to_path_buf()], &edits).expect("snapshots");
    return edits;
}

#[test]
fn a_manifest_the_call_rewrote_is_recorded_and_put_back() {
    let root = scratch();
    let path = root.path().join("Cargo.toml");
    fs::write(&path, "before").expect("manifest");
    let edits = edits_for(ROOT_A, &path);

    run_recording_manifests(ROOT_A, &edits, || { fs::write(&path, "cargo's edit").expect("cargo edits it"); }).expect("records");
    let warnings = restore_manifests(&edits, &keep()).expect("restores");

    assert!(warnings.is_empty(), "{warnings:?}");
    assert_eq!(fs::read_to_string(&path).expect("read"), "before");
}

#[test]
fn a_manifest_another_process_changed_after_the_call_is_left_alone_with_a_warning() {
    let root = scratch();
    let path = root.path().join("Cargo.toml");
    fs::write(&path, "before").expect("manifest");
    let edits = edits_for(ROOT_A, &path);
    run_recording_manifests(ROOT_A, &edits, || { fs::write(&path, "cargo's edit").expect("cargo edits it"); }).expect("records");
    fs::write(&path, "somebody else's edit").expect("another process");

    let warnings = restore_manifests(&edits, &keep()).expect("restores");

    assert_eq!(warnings.len(), 1);
    assert_eq!(fs::read_to_string(&path).expect("read"), "somebody else's edit");
}

/// Review finding (2b 1, 2c 4): an editor changes a member the call never touched, during the call of another root; that change must
/// not be recorded as the call's output and must survive the restore.
#[test]
fn an_edit_to_a_manifest_the_call_did_not_write_is_not_recorded_and_survives() {
    let root = scratch();
    let first = root.path().join("a.toml");
    let second = root.path().join("b.toml");
    fs::write(&first, "a before").expect("manifest");
    fs::write(&second, "b before").expect("manifest");
    let edits: RefCell<Vec<ManifestEdit>> = RefCell::new(Vec::new());
    snapshot_manifests(ROOT_A, &[first.clone()], &edits).expect("snapshots");
    snapshot_manifests(ROOT_B, &[second.clone()], &edits).expect("snapshots");

    run_recording_manifests(ROOT_A, &edits, || {
        fs::write(&first, "a edited by cargo").expect("cargo edits its own root");
        fs::write(&second, "b edited by a person meanwhile").expect("an editor");
    })
    .expect("records");
    let warnings = restore_manifests(&edits, &keep()).expect("restores");

    assert_eq!(fs::read_to_string(&first).expect("read"), "a before");
    assert_eq!(fs::read_to_string(&second).expect("read"), "b edited by a person meanwhile");
    assert_eq!(warnings.len(), 1, "{warnings:?}");
}

/// A change that lands after the call returned is outside its window and is not the call's output.
#[test]
fn a_change_published_after_the_call_returned_is_not_taken_for_its_output() {
    let root = scratch();
    let path = root.path().join("Cargo.toml");
    fs::write(&path, "before").expect("manifest");
    let edits = edits_for(ROOT_A, &path);
    let late: PathBuf = path.clone();

    run_recording_manifests(ROOT_A, &edits, || {}).expect("records");
    std::thread::sleep(std::time::Duration::from_millis(50));
    fs::write(&late, "published after the call").expect("a late writer");
    let warnings = restore_manifests(&edits, &keep()).expect("restores");

    assert_eq!(fs::read_to_string(&path).expect("read"), "published after the call");
    assert_eq!(warnings.len(), 1, "{warnings:?}");
}

#[test]
fn a_manifest_whose_output_was_never_recorded_is_never_restored() {
    let root = scratch();
    let path = root.path().join("Cargo.toml");
    fs::write(&path, "before").expect("manifest");
    let edits = edits_for(ROOT_A, &path);
    fs::write(&path, "unknown author").expect("someone edits it");

    let warnings = restore_manifests(&edits, &keep()).expect("restores");

    assert_eq!(warnings.len(), 1);
    assert_eq!(fs::read_to_string(&path).expect("read"), "unknown author");
}

#[test]
fn an_unchanged_manifest_needs_nothing_and_a_missing_or_unreadable_one_stops_the_snapshot() {
    let root = scratch();
    let path = root.path().join("Cargo.toml");
    fs::write(&path, "before").expect("manifest");
    let edits = edits_for(ROOT_A, &path);

    assert!(restore_manifests(&edits, &keep()).expect("restores").is_empty());
    assert!(snapshot_manifests(ROOT_A, &[root.path().join("missing.toml")], &RefCell::new(Vec::new())).is_err());
    fs::write(root.path().join("binary.toml"), [0xff, 0xfe, 0x00]).expect("not utf-8");
    assert!(snapshot_manifests(ROOT_A, &[root.path().join("binary.toml")], &RefCell::new(Vec::new())).is_err(), "invalid UTF-8 is an error, not an empty manifest");
}
