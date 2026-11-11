//! Controls of the manifest a trash carries inside itself:
//! what it proves,
//! and what a directory that cannot show it is.

use super::{MANIFEST_FILE_NAME, its_own, read, write};
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
    assert!(its_own(&opened(root.as_path(), name.as_str()), name.as_str()).is_some(), "and the emptying takes the file for its own");
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
    // The emptying asks the same question of the opened trash, so a manifest that decodes but names another directory is
    //  no file of the emptying's either.
    assert!(its_own(&opened(root.as_path(), name.as_str()), name.as_str()).is_none(), "the bytes decode, and they name another directory");
    std::fs::write(root.join(TRASH_PARENT).join(&name).join(MANIFEST_FILE_NAME), b"not a manifest").expect("damage");
    assert_eq!(read(root.as_path(), name.as_str()), None, "a damaged manifest proves nothing");
    assert!(its_own(&opened(root.as_path(), name.as_str()), name.as_str()).is_none(), "and is no file of the emptying's");
    remove(root.as_path());
}

/// What:
///  A link at the manifest's name is refused instead of written through,
///  so nothing outside the trash takes the manifest's bytes.
/// Why:
///   Another process that may write the stage can put a link at the manifest's name before the write,
/// and a write that follows it replaces the file it points at with encoded identities,
/// which is that file's bytes lost.
/// The trash is the wrapper's own directory,
/// whose only file of that name is the one it writes itself.
#[test]
fn a_manifest_name_that_holds_a_link_is_refused() {
    let root: PathBuf = fixture("trash-own-manifest-link");
    let name: String = "e".repeat(40);
    let manifest: TrashManifest = planted(root.as_path(), name.as_str(), 1);
    let victim: PathBuf = root.join("victim");
    let original = b"user data that must survive\n";
    std::fs::write(&victim, original).expect("the person's file");
    let link: PathBuf = root.join(TRASH_PARENT).join(&name).join(MANIFEST_FILE_NAME);
    std::os::unix::fs::symlink(&victim, link.as_path()).expect("a link at the manifest's name");
    assert!(write(root.as_path(), &manifest).is_err(), "a link is not written through");
    assert_eq!(std::fs::read(&victim).expect("the person's file"), original, "and the file it points at keeps its bytes");
    remove(root.as_path());
}

/// What:
///  A named pipe at the manifest's name is refused at once,
///  so no command waits for a reader that may never come.
/// Why:
///   Another process that may write the stage can put a pipe at the manifest's name before the write,
/// and opening a pipe for writing waits for a reader.
#[test]
fn a_manifest_name_that_holds_a_pipe_is_refused_without_waiting() {
    let root: PathBuf = fixture("trash-own-manifest-pipe");
    let name: String = "f".repeat(40);
    let manifest: TrashManifest = planted(root.as_path(), name.as_str(), 1);
    let path: PathBuf = root.join(TRASH_PARENT).join(&name).join(MANIFEST_FILE_NAME);
    write(root.as_path(), &manifest).expect("positive control: an ordinary name is written");
    std::fs::remove_file(path.as_path()).expect("make room for the pipe");
    crate::subvolume_sync_attrs::make_fifo(path.as_path(), 0o600).expect("a named pipe");
    let written_root: PathBuf = root.clone();
    let written_manifest: TrashManifest = manifest.clone();
    let outcome = crate::test_support::finished_within(move || return write(written_root.as_path(), &written_manifest).map_err(|error| return error.to_string()));
    let Some(finished) = outcome else { panic!("the manifest write waited on a named pipe") };
    let refused: String = finished.expect_err("a named pipe is refused");
    assert!(refused.contains("not a plain file"), "{refused}");
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
