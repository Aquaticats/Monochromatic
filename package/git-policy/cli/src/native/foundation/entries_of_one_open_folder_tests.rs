//! Controls for one folder held open and the entries below it read and listed by name:
//! a record reads back through the held folder,
//! a link or a folder planted at an entry's name is refused,
//! a file larger than the bound is refused rather than read into memory,
//! and a missing folder or a link at the folder's own name is refused.

use super::*;
use crate::publishing_entries_in_one_open_folder::create_new_entry;
use crate::test_support::{
    fixture,
    remove,
};
use std::io::Write;
use std::path::PathBuf;

/// What:
///  A file written by one call reads back through the folder that holds it,
///  and the folder's own listing holds its name.
/// Why:
///  Every record the store reads is one entry below one held folder,
///  so a read that resolved a path again would judge a name the account could change.
#[test]
fn a_file_written_through_a_held_folder_reads_back() {
    let root: PathBuf = fixture("held-folder-read");
    let folder: std::fs::File = open_folder(root.as_path()).expect("the fixture folder opens");
    let mut written: std::fs::File =
        create_new_entry(&folder, OsStr::new("record"), 0o644).expect("the entry is created");
    written
        .write_all(b"two lines\nand a second\n")
        .expect("the bytes are written");
    drop(written);
    let mut read: std::fs::File =
        open_entry_for_reading(&folder, OsStr::new("record")).expect("the entry opens");
    let bytes: Vec<u8> = read_entry_bytes(&mut read, 1024).expect("the entry reads");
    assert_eq!(bytes, b"two lines\nand a second\n");
    let names: Vec<OsString> = entry_names(&folder).expect("the folder lists");
    assert_eq!(
        names,
        vec![OsString::from("record")],
        "the folder lists exactly the entry it holds"
    );
    remove(root.as_path());
}

/// What:
///  An absent entry,
/// a link at an entry's name and a folder at an entry's name are told apart,
///  and none of them is read through.
/// Why:
///  A refusal that cannot tell "nothing is there" from "something unsafe is there" sends a person to the wrong repair.
#[test]
fn an_absent_link_or_folder_entry_is_refused_by_its_own_cause() {
    let root: PathBuf = fixture("held-folder-refusals");
    let folder: std::fs::File = open_folder(root.as_path()).expect("the fixture folder opens");
    let absent: ReadRefusal =
        open_entry_for_reading(&folder, OsStr::new("absent")).expect_err("nothing is there");
    assert!(matches!(absent, ReadRefusal::Missing), "{absent:?}");
    let elsewhere: PathBuf = root.join("elsewhere");
    std::fs::write(elsewhere.as_path(), b"not this name").expect("the link's target");
    std::os::unix::fs::symlink(elsewhere.as_path(), root.join("link"))
        .expect("a link at an entry's name");
    let linked: ReadRefusal = open_entry_for_reading(&folder, OsStr::new("link"))
        .expect_err("a link is not read through");
    assert!(matches!(linked, ReadRefusal::NotRegular), "{linked:?}");
    std::fs::create_dir(root.join("folder")).expect("a folder at an entry's name");
    let nested: ReadRefusal = open_entry_for_reading(&folder, OsStr::new("folder"))
        .expect_err("a folder is not a record");
    assert!(matches!(nested, ReadRefusal::NotRegular), "{nested:?}");
    remove(root.as_path());
}

/// What:
///  A file larger than the bound is refused,
///  and one of exactly the bound reads.
/// Why:
///  A runaway or replaced file must be named rather than read into memory,
///  and the bound must not refuse the widest file a writer makes.
#[test]
fn a_file_larger_than_the_read_accepts_is_refused() {
    let root: PathBuf = fixture("held-folder-bound");
    let folder: std::fs::File = open_folder(root.as_path()).expect("the fixture folder opens");
    let mut exact: std::fs::File =
        create_new_entry(&folder, OsStr::new("exact"), 0o644).expect("the entry is created");
    exact
        .write_all("0".repeat(64).as_bytes())
        .expect("the bytes are written");
    drop(exact);
    let mut read: std::fs::File =
        open_entry_for_reading(&folder, OsStr::new("exact")).expect("the entry opens");
    assert_eq!(
        read_entry_bytes(&mut read, 64)
            .expect("the bound itself reads")
            .len(),
        64
    );
    let mut over: std::fs::File =
        create_new_entry(&folder, OsStr::new("over"), 0o644).expect("the entry is created");
    over.write_all("0".repeat(65).as_bytes())
        .expect("the bytes are written");
    drop(over);
    let mut read_over: std::fs::File =
        open_entry_for_reading(&folder, OsStr::new("over")).expect("the entry opens");
    let refused: ReadRefusal =
        read_entry_bytes(&mut read_over, 64).expect_err("one byte over the bound");
    assert!(matches!(refused, ReadRefusal::TooLarge), "{refused:?}");
    remove(root.as_path());
}

/// What:
///  A missing folder and a link planted at the folder's own name are refused,
///  and a plain folder opens.
/// Why:
///  The folder is opened once and every entry is named relative to it,
///  so a link at its own name would make every entry below it the account's choice.
#[test]
fn a_missing_folder_or_a_link_at_its_name_is_refused() {
    let root: PathBuf = fixture("held-folder-top");
    let absent: std::io::Error =
        open_folder(root.join("absent").as_path()).expect_err("nothing is there");
    assert_eq!(absent.kind(), std::io::ErrorKind::NotFound, "{absent}");
    std::os::unix::fs::symlink(root.as_path(), root.join("link"))
        .expect("a link at the folder's name");
    let linked: std::io::Error =
        open_folder(root.join("link").as_path()).expect_err("a link is not opened as a folder");
    assert!(
        matches!(linked.raw_os_error(), Some(code) if code == libc::ELOOP || code == libc::ENOTDIR),
        "the link is refused rather than followed: {linked}"
    );
    let plain: std::fs::File = open_folder(root.as_path()).expect("a plain folder opens");
    assert!(plain.metadata().expect("the folder's metadata").is_dir());
    remove(root.as_path());
}
