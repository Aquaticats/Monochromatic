//! Controls for creating, filling and publishing one entry below a folder held open:
//! a link publishes a filled staging file and answers `EEXIST` onto a taken name,
//! the rename that never replaces does the same on a folder without links,
//! a replacing rename replaces in one step,
//! and the removal and the flush leave the folder without the staging name.

use super::*;
use crate::entries_of_one_open_folder::open_entry_for_reading;
use crate::entries_of_one_open_folder::open_folder;
use crate::test_support::{fixture, remove};
use std::io::Read;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;

/// The bytes an entry below `folder` holds.
fn bytes_of(folder: &std::fs::File, name: &OsStr) -> Vec<u8> {
    let mut file: std::fs::File = open_entry_for_reading(folder, name).expect("the entry opens");
    let mut bytes: Vec<u8> = Vec::new();
    file.read_to_end(&mut bytes).expect("the entry reads");
    return bytes;
}

/// A staging entry below `folder` holding `bytes`.
fn staged(folder: &std::fs::File, name: &str, bytes: &[u8]) -> std::fs::File {
    let mut file: std::fs::File = create_new_entry(folder, OsStr::new(name), 0o644).expect("the staging entry is created");
    fill_entry_and_sync(&mut file, bytes, 0o644).expect("the bytes are durable");
    return file;
}

/// What:
///  One link publishes a filled staging file under its real name,
///  the record keeps its bytes with the staging name gone,
///  and the same call onto a taken name answers `EEXIST` and replaces nothing.
/// Why:
///  This is the one-winner step two recorders of one name meet at,
///  so the loser's name must never be written and the winner's bytes must never be replaced.
#[test]
fn one_link_publishes_and_a_taken_name_is_answered_with_already_exists() {
    let root: PathBuf = fixture("publish-link");
    let folder: std::fs::File = open_folder(root.as_path()).expect("the fixture folder opens");
    let staged_file: std::fs::File = staged(&folder, ".a1b2c3d4", b"record\n");
    link_entry(&folder, OsStr::new(".a1b2c3d4"), OsStr::new("add-cache-layer")).expect("the staging name becomes the record's");
    remove_entry(&folder, OsStr::new(".a1b2c3d4")).expect("the staging name is removed");
    sync_folder(&folder).expect("the folder flushes");
    drop(staged_file);
    assert_eq!(bytes_of(&folder, OsStr::new("add-cache-layer")), b"record\n");
    let metadata: std::fs::Metadata = std::fs::metadata(root.join("add-cache-layer")).expect("the record's metadata");
    assert_eq!(metadata.nlink(), 1, "the record keeps one link after the staging name is removed");
    assert!(!root.join(".a1b2c3d4").exists(), "the staging name is gone");
    let _second: std::fs::File = staged(&folder, ".e5f6", b"second\n");
    let taken: std::io::Error = link_entry(&folder, OsStr::new(".e5f6"), OsStr::new("add-cache-layer")).expect_err("the record exists");
    assert_eq!(taken.raw_os_error(), Some(libc::EEXIST), "{taken}");
    assert_eq!(bytes_of(&folder, OsStr::new("add-cache-layer")), b"record\n", "the winner's bytes are not replaced");
    assert!(root.join(".e5f6").exists(), "the loser's staging name stays until its writer removes it");
    remove(root.as_path());
}

/// What:
///  The rename that never replaces answers `EEXIST` and leaves the staging name in place,
///  and publishes onto an absent name in one step with the staging name gone.
/// Why:
///  It is the publication's fallback on a folder without hard links,
///  so it must keep the same one-winner property and the same cleanup.
#[test]
fn the_rename_that_never_replaces_publishes_once_and_keeps_the_staging_name_on_a_taken_name() {
    let root: PathBuf = fixture("publish-rename");
    let folder: std::fs::File = open_folder(root.as_path()).expect("the fixture folder opens");
    let _occupied: std::fs::File = staged(&folder, ".1111", b"first\n");
    rename_entry_never_replacing(&folder, OsStr::new(".1111"), OsStr::new("taken")).expect("the staging name becomes the record's");
    let _second: std::fs::File = staged(&folder, ".2222", b"second\n");
    let taken: std::io::Error = rename_entry_never_replacing(&folder, OsStr::new(".2222"), OsStr::new("taken")).expect_err("the record exists");
    assert_eq!(taken.raw_os_error(), Some(libc::EEXIST), "{taken}");
    assert!(root.join(".2222").exists(), "a refused rename leaves the staging name in place");
    assert_eq!(bytes_of(&folder, OsStr::new("taken")), b"first\n");
    rename_entry_never_replacing(&folder, OsStr::new(".2222"), OsStr::new("free")).expect("an absent name is published");
    assert!(!root.join(".2222").exists(), "the rename moved the staging name");
    assert_eq!(bytes_of(&folder, OsStr::new("free")), b"second\n");
    remove(root.as_path());
}

/// What:
///  A replacing rename replaces what a name holds in one step,
///  and the staging name is gone afterwards.
/// Why:
///  The adoption replaces the parent record with one rename,
///  so a reader sees either whole record and never a third state.
#[test]
fn the_replacing_rename_replaces_what_the_name_held() {
    let root: PathBuf = fixture("publish-replace");
    let folder: std::fs::File = open_folder(root.as_path()).expect("the fixture folder opens");
    let _old: std::fs::File = staged(&folder, "record", b"old\n");
    let _new: std::fs::File = staged(&folder, "..record.abcdef.partial", b"new\n");
    rename_entry_replacing(&folder, OsStr::new("..record.abcdef.partial"), OsStr::new("record")).expect("the record is replaced");
    assert_eq!(bytes_of(&folder, OsStr::new("record")), b"new\n");
    assert!(!root.join("..record.abcdef.partial").exists(), "the staging name is gone");
    remove(root.as_path());
}

/// What:
///  A folder that refuses a hard link is told apart from one whose name is taken,
///  and an exclusive creation at a taken name is refused.
/// Why:
///  A publication must fall back to the rename only where the folder makes no links,
///  and its staging name must never be shared with a second writer.
#[test]
fn a_refused_link_is_told_apart_and_an_exclusive_creation_refuses_a_taken_name() {
    let root: PathBuf = fixture("publish-told-apart");
    let folder: std::fs::File = open_folder(root.as_path()).expect("the fixture folder opens");
    let taken: std::io::Error = std::io::Error::from_raw_os_error(libc::EEXIST);
    assert!(!link_refused_because_the_folder_makes_none(&taken), "a taken name is not a folder without links");
    for code in [libc::EPERM, libc::EOPNOTSUPP] {
        let no_links: std::io::Error = std::io::Error::from_raw_os_error(code);
        assert!(link_refused_because_the_folder_makes_none(&no_links), "{code} is a folder without links");
    }
    let _first: std::fs::File = staged(&folder, ".3333", b"one\n");
    let refused: std::io::Error = create_new_entry(&folder, OsStr::new(".3333"), 0o644).expect_err("the staging name is taken");
    assert_eq!(refused.raw_os_error(), Some(libc::EEXIST), "{refused}");
    remove(root.as_path());
}
