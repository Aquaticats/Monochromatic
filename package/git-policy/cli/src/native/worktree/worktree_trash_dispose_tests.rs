//! Controls of the emptying that are decided below the handler:
//! which object the manifest's name must still hold when it is unlinked.

use super::remove_the_manifest;
use crate::fs_beneath::{identity_at, open_directory_at, open_root, plain_components};
use crate::test_support::{fixture, remove};
use crate::worktree_trash::TRASH_PARENT;
use crate::worktree_trash_own_manifest::MANIFEST_FILE_NAME;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// The trash `name` below the trash parent of `root`,
/// opened the way the emptying opens one.
fn opened(root: &Path, name: &str) -> std::fs::File {
    let held: std::fs::File = open_root(root).expect("the root");
    let names = plain_components(Path::new(TRASH_PARENT)).expect("a plain parent");
    let mut parent: std::fs::File = held;
    for part in names {
        parent = open_directory_at(&parent, part.as_os_str()).expect("the trash parent");
    }
    return open_directory_at(&parent, OsStr::new(name)).expect("the trash");
}

/// A trash directory `name` below the trash parent of `root` with a file of the manifest's name in it,
/// and that file's identity.
fn trash_with_a_manifest(root: &Path, name: &str) -> crate::fs_beneath::ObjectId {
    let trash: PathBuf = root.join(TRASH_PARENT).join(name);
    std::fs::create_dir_all(trash.as_path()).expect("a trash");
    std::fs::write(trash.join(MANIFEST_FILE_NAME), b"0:1:0\n").expect("a manifest");
    let held: std::fs::File = opened(root, name);
    return read_identity(&held);
}

/// The identity of the file at the manifest's name in `trash`.
fn read_identity(trash: &std::fs::File) -> crate::fs_beneath::ObjectId {
    return match identity_at(trash, OsStr::new(MANIFEST_FILE_NAME)) {
        Ok(found) => found,
        Err(error) => panic!("the manifest's name cannot be examined: {error}"),
    };
}

/// What:
///  The manifest's name is unlinked only while it still holds the object that was read,
///  so a file another process moved onto it is kept and reported.
/// Why:
///   The look that proves the file is the trash's manifest and the unlink are two steps,
/// and nothing in the
///  kernel makes them one;
/// a file renamed onto the name between them is a file the emptying never read,
///  so it is left where it is and the trash directory with it,
/// which is what the code promises for anything it cannot prove it made.
#[test]
fn a_manifest_replaced_before_the_removal_is_kept_and_reported() {
    let root: PathBuf = fixture("trash-manifest-replaced");
    let name: String = "c".repeat(40);
    let read: crate::fs_beneath::ObjectId = trash_with_a_manifest(root.as_path(), name.as_str());
    let trash: std::fs::File = opened(root.as_path(), name.as_str());
    let inside: PathBuf = root.join(TRASH_PARENT).join(name.as_str());
    // The interleaving: another process renames its own file onto the name after the read.
    let replacement: PathBuf = root.join("replacement");
    std::fs::write(replacement.as_path(), b"somebody else's bytes\n").expect("a replacement");
    std::fs::rename(replacement.as_path(), inside.join(MANIFEST_FILE_NAME)).expect("the rename onto the name");
    let outcome = remove_the_manifest(&trash, inside.as_path(), read);
    assert!(outcome.is_err(), "a file that was never read is not removed, and the call answered {outcome:?}");
    assert!(inside.join(MANIFEST_FILE_NAME).is_file(), "the replacement stays");
    remove(root.as_path());
}

/// What:
///  The file the read examined goes,
///  which is the ordinary emptying.
/// Why:
///   A control that only ever refuses would pass while the emptying never removed a manifest at all.
#[test]
fn the_manifest_the_read_examined_is_removed() {
    let root: PathBuf = fixture("trash-manifest-removed");
    let name: String = "b".repeat(40);
    let read: crate::fs_beneath::ObjectId = trash_with_a_manifest(root.as_path(), name.as_str());
    let trash: std::fs::File = opened(root.as_path(), name.as_str());
    let inside: PathBuf = root.join(TRASH_PARENT).join(name.as_str());
    remove_the_manifest(&trash, inside.as_path(), read).expect("the file the read examined is removed");
    assert!(std::fs::symlink_metadata(inside.join(MANIFEST_FILE_NAME)).is_err(), "and the name is free");
    remove(root.as_path());
}
