//! Controls for the fill's trash reading:
//!  no trash is none,
//!  one trash with its own record is the fill's,
//!  a trash without one refuses,
//!  and two trashes refuse.

use super::the_trash_of_a_filled_clone;
use crate::fs_beneath::identity_at;
use crate::test_support::{fixture, remove};
use crate::worktree_trash_own_manifest;
use crate::worktree_trash::{TRASH_PARENT, TrashManifest, is_trash_name};
use std::path::PathBuf;

/// A clone root with a trash place.
fn a_clone_root(name: &str) -> PathBuf {
    let root: PathBuf = fixture(name);
    std::fs::create_dir_all(root.join(TRASH_PARENT)).expect("the trash place");
    return root;
}

/// Make one trash below the root and answer its name.
fn make_a_trash(root: &std::path::Path, fill: u8) -> String {
    let name: String = std::iter::repeat_n(format!("{fill:02x}"), 20).collect();
    assert!(is_trash_name(name.as_str()));
    let folder: PathBuf = root.join(TRASH_PARENT).join(name.as_str());
    std::fs::create_dir(folder.as_path()).expect("the trash");
    let parent: std::fs::File = std::fs::File::open(root.join(TRASH_PARENT)).expect("the parent opens");
    let identity = identity_at(&parent, std::ffi::OsStr::new(name.as_str())).expect("the identity");
    let manifest: TrashManifest = TrashManifest { name: name.clone(), trash: identity, entries: Vec::new() };
    worktree_trash_own_manifest::write(root, &manifest).expect("the manifest");
    return name;
}

#[test]
fn a_clone_with_no_trash_answers_none() {
    let root: PathBuf = a_clone_root("fill-no-trash");
    assert_eq!(the_trash_of_a_filled_clone(root.as_path()).expect("no trash is no refusal"), None);
    remove(root.as_path());
}

#[test]
fn a_trash_with_its_own_record_is_the_fill_s() {
    let root: PathBuf = a_clone_root("fill-one-trash");
    let name: String = make_a_trash(root.as_path(), 0xab);
    let found: TrashManifest = the_trash_of_a_filled_clone(root.as_path()).expect("the record reads").expect("one trash stands");
    assert_eq!(found.name, name);
    remove(root.as_path());
}

#[test]
fn a_trash_without_its_record_and_two_trashes_refuse() {
    let root: PathBuf = a_clone_root("fill-bad-trash");
    let name: String = std::iter::repeat_n(String::from("cd"), 20).collect();
    std::fs::create_dir(root.join(TRASH_PARENT).join(name.as_str())).expect("a trash with no record");
    let refusal: String = the_trash_of_a_filled_clone(root.as_path()).expect_err("a trash with no record proves nothing");
    assert!(refusal.contains("holds no record this add wrote"), "{refusal}");
    std::fs::remove_dir_all(root.join(TRASH_PARENT).join(name.as_str())).expect("the recordless trash goes");
    make_a_trash(root.as_path(), 0x11);
    make_a_trash(root.as_path(), 0x22);
    let two: String = the_trash_of_a_filled_clone(root.as_path()).expect_err("two trashes are two add records");
    assert!(two.contains("more than one trash"), "{two}");
    remove(root.as_path());
}
