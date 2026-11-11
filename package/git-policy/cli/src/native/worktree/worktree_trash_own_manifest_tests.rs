//! Controls of the manifest a trash carries inside itself:
//! what it proves,
//! and what a directory that cannot show it is.

use super::{MANIFEST_FILE_NAME, is_its_own, read, write};
use crate::fs_beneath::{identity_at, identity_of, open_directory_at, open_root, plain_components};
use crate::test_support::{fixture, remove};
use crate::worktree_trash::TRASH_PARENT;
use crate::worktree_trash_manifest::{TrashManifest, entry_name};
use std::path::{Path, PathBuf};

/// The trash `name` below the trash parent of `root`,
/// opened the way every reader opens one.
fn opened(root: &Path, name: &str) -> std::fs::File {
    let held: std::fs::File = open_root(root).expect("the root");
    let names = plain_components(Path::new(TRASH_PARENT)).expect("a plain parent");
    let mut parent: std::fs::File = held;
    for part in names {
        parent = open_directory_at(&parent, part.as_os_str()).expect("the trash parent");
    }
    return open_directory_at(&parent, std::ffi::OsStr::new(name)).expect("the trash");
}

/// A trash directory `name` below the trash parent of `root` with `count` entries,
/// and the manifest the
///  leave-out would have written for it.
fn planted(root: &Path, name: &str, count: usize) -> TrashManifest {
    for index in 0..count {
        let entry: PathBuf = root.join(TRASH_PARENT).join(name).join(entry_name(index)).join("payload");
        std::fs::create_dir_all(entry.as_path()).expect("an entry");
        std::fs::write(entry.join("credentials"), b"another worktree's token\n").expect("an entry file");
    }
    let trash: std::fs::File = opened(root, name);
    let entries = (0..count).map(|index: usize| return identity_at(&trash, std::ffi::OsStr::new(entry_name(index).as_str())).expect("an entry identity")).collect();
    return TrashManifest { name: String::from(name), trash: identity_of(&trash).expect("the trash identity"), entries };
}

/// What:
///  A manifest written into its trash reads back as that trash's own.
/// Why:
///   The bytes are the record the handler already uses,
/// and the identity of the directory is what makes them this
///  trash's own rather than another one's.
#[test]
fn a_written_manifest_reads_back_as_the_trashs_own() {
    let root: PathBuf = fixture("trash-own-manifest");
    let name: String = "a".repeat(40);
    let manifest: TrashManifest = planted(root.as_path(), name.as_str(), 2);
    write(root.as_path(), &manifest).expect("the manifest is written");
    assert_eq!(read(root.as_path(), name.as_str()), Some(manifest), "the manifest reads back whole");
    assert!(is_its_own(&opened(root.as_path(), name.as_str()), name.as_str()), "and the emptying takes the file for its own");
    remove(root.as_path());
}

/// What:
///  A trash that carries no manifest proves nothing,
/// and neither does a damaged one,
/// or one whose identities name another directory.
/// Why:
///   Only what the leave-out itself wrote may be deleted,
/// and a file that cannot be read back as this trash's own
///  manifest is no proof of who made it.
#[test]
fn a_manifest_that_is_not_the_trashs_own_proves_nothing() {
    let root: PathBuf = fixture("trash-own-manifest-none");
    let name: String = "b".repeat(40);
    let manifest: TrashManifest = planted(root.as_path(), name.as_str(), 1);
    assert_eq!(read(root.as_path(), name.as_str()), None, "no manifest, no proof");
    let other: String = "c".repeat(40);
    let other_manifest: TrashManifest = planted(root.as_path(), other.as_str(), 1);
    // A manifest that names the other trash's identity is not this one's.
    let forged: TrashManifest = TrashManifest { name: name.clone(), trash: other_manifest.trash, entries: manifest.entries.clone() };
    write(root.as_path(), &forged).expect("the file is written");
    assert_eq!(read(root.as_path(), name.as_str()), None, "identities that are not the directory's prove nothing");
    std::fs::write(root.join(TRASH_PARENT).join(&name).join(MANIFEST_FILE_NAME), b"not a manifest").expect("damage");
    assert_eq!(read(root.as_path(), name.as_str()), None, "a damaged manifest proves nothing");
    assert!(!is_its_own(&opened(root.as_path(), name.as_str()), name.as_str()), "and is no file of the emptying's");
    remove(root.as_path());
}

/// A root with no trash at all answers nothing and cannot be written into.
#[test]
fn a_root_without_that_trash_answers_nothing() {
    let root: PathBuf = fixture("trash-own-manifest-absent");
    let name: String = "d".repeat(40);
    std::fs::create_dir_all(root.join(TRASH_PARENT)).expect("the trash parent");
    assert_eq!(read(root.as_path(), name.as_str()), None);
    let manifest: TrashManifest = TrashManifest { name: name.clone(), trash: crate::fs_beneath::ObjectId { device: 1, inode: 1, birth: None }, entries: Vec::new() };
    assert!(write(root.as_path(), &manifest).is_err(), "a trash that is not there cannot hold a manifest");
    remove(root.as_path());
}
