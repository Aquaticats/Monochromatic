//! What:
//!  Controls of the record one removal writes before it hands a folder to Git:
//!  what it holds,
//!  what the mark adds,
//!  and what a reader answers.
//! Why:
//!  The record is the only thing that outlives a removal whose process died,
//!  and the winner of a merge against a branch that changes the record's shape cannot see a wrong reading here:
//!  a name that is not text,
//!  a path holding a zero byte's neighbour,
//!  and a record without an owner are the states a person's own filesystem makes.

use super::{LeftBehindRecord, RECORD_FOLDER, forget, owner_may_be_running, read_record, record_before_git, record_returned};
use crate::test_support::{fixture, remove};
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

/// A folder holding `names`,
/// each of them a file.
fn folder_with(root: &Path, names: &[&[u8]]) -> PathBuf {
    let folder: PathBuf = root.join("worktree");
    std::fs::create_dir_all(folder.as_path()).expect("folder");
    for name in names {
        std::fs::write(folder.join(OsStr::from_bytes(name)), b"x").expect("entry");
    }
    return folder;
}

/// What:
///  One record names the entries a folder held and the folder itself,
///  and putting it back together answers the same entries.
/// Why:
///   A reader that loses a name would let a finish delete a file that appeared while the removal ran.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordBeforeGit(common, 'topic', target); expect(readRecord(bytes).seen).toEqual(['a', 'b']);
/// ```
#[test]
fn a_record_names_the_entries_and_the_folder() {
    let root: PathBuf = fixture("left-behind-record-round-trip");
    let common: PathBuf = root.join("common");
    std::fs::create_dir_all(common.as_path()).expect("common");
    let target: PathBuf = folder_with(root.as_path(), &[b"README.md", b"node_modules"]);
    record_before_git(common.as_path(), OsStr::new("topic"), target.as_path()).expect("the record is written");
    let bytes: Vec<u8> = std::fs::read(common.join(RECORD_FOLDER).join("topic")).expect("the record");
    let record: LeftBehindRecord = read_record(bytes.as_slice()).expect("the record is read");
    let mut seen: Vec<&[u8]> = record.seen.iter().map(|name: &OsString| { return name.as_bytes(); }).collect();
    seen.sort_unstable();
    assert_eq!(seen, vec![b"README.md".as_slice(), b"node_modules".as_slice()]);
    assert_eq!(record.target, target);
    assert!(record.owner.is_some(), "a record this process writes names itself: {:?}", record.owner);
    assert!(!record.returned, "Git has not returned when the record is written");
    assert!(owner_may_be_running(record.owner.as_ref()), "this process is still running");
    remove(root.as_path());
}

/// What:
///  A name that is not text and a folder whose name holds a space round-trip byte for byte.
/// Why:
///   A record that decoded a name would name another entry,
///  and a finish that deletes "another entry" deletes a file nobody asked about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = readRecord(bytes); expect(record.seen).toEqual([Buffer.from([0xff, 0x20])]);
/// ```
#[test]
fn a_name_that_is_not_text_survives_the_record() {
    let root: PathBuf = fixture("left-behind-record-bytes");
    let common: PathBuf = root.join("common");
    std::fs::create_dir_all(common.as_path()).expect("common");
    let target: PathBuf = folder_with(root.as_path(), &[&[0xff, 0x20, 0x61][..]]);
    record_before_git(common.as_path(), OsStr::new("topic"), target.as_path()).expect("the record is written");
    let bytes: Vec<u8> = std::fs::read(common.join(RECORD_FOLDER).join("topic")).expect("the record");
    let record: LeftBehindRecord = read_record(bytes.as_slice()).expect("the record is read");
    assert_eq!(record.seen.len(), 1);
    assert_eq!(record.seen[0].as_bytes(), &[0xff, 0x20, 0x61]);
    remove(root.as_path());
}

/// What:
///  The mark says Git returned,
///  and keeps the entries and the folder it was written over.
/// Why:
///   The mark is what lets a later command finish a removal whose process ended,
///  so a mark that dropped the entries would leave it unable to say what is the removal's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordReturned(common, 'topic'); expect(readRecord(bytes)).toMatchObject({ returned: true, seen: ['a'] });
/// ```
#[test]
fn the_mark_says_git_returned_and_keeps_what_the_record_names() {
    let root: PathBuf = fixture("left-behind-record-mark");
    let common: PathBuf = root.join("common");
    std::fs::create_dir_all(common.as_path()).expect("common");
    let target: PathBuf = folder_with(root.as_path(), &[b"a"]);
    record_before_git(common.as_path(), OsStr::new("topic"), target.as_path()).expect("the record is written");
    let owner: Option<String> = read_record(std::fs::read(common.join(RECORD_FOLDER).join("topic")).expect("bytes").as_slice()).expect("read").owner;
    record_returned(common.as_path(), OsStr::new("topic"));
    let bytes: Vec<u8> = std::fs::read(common.join(RECORD_FOLDER).join("topic")).expect("the record");
    let record: LeftBehindRecord = read_record(bytes.as_slice()).expect("the marked record is read");
    assert!(record.returned, "the mark is read back");
    assert_eq!(record.seen.len(), 1, "the entries stay: {:?}", record.seen);
    assert_eq!(record.target, target);
    assert_eq!(record.owner, owner, "the owner line stays");
    remove(root.as_path());
}

/// What:
///  A record an earlier build wrote,
///  which names no process,
///  is read and counts as one whose owner may still be running.
/// Why:
///   Nothing proves such a removal ended,
///  and finishing a removal another command is still walking deletes what it is still proving.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(readRecord(Buffer.from('/tmp/topic'))).toMatchObject({ seen: [], owner: undefined });
/// expect(ownerMayBeRunning(undefined)).toBe(true);
/// ```
#[test]
fn a_record_without_an_owner_counts_as_one_still_running() {
    let record: LeftBehindRecord = read_record(b"/nowhere/topic").expect("a record without a header is read");
    assert_eq!(record.target, PathBuf::from("/nowhere/topic"));
    assert!(record.seen.is_empty());
    assert_eq!(record.owner, None);
    assert!(!record.returned);
    assert!(owner_may_be_running(record.owner.as_ref()), "a record that names no process proves nothing about one");
}

/// What:
///  A record that names no folder is refused with the reason rather than read as one.
/// Why:
///   An empty record would name the common directory itself,
///  and a finish that acted on it would delete the repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(() => readRecord(Buffer.alloc(0))).toThrow(/names no folder/);
/// ```
#[test]
fn a_record_that_names_no_folder_is_refused() {
    assert!(read_record(b"").is_err(), "an empty record names nothing");
    assert!(read_record(b"owner=1:2\n").is_err(), "a record of a header alone names nothing");
}

/// What:
///  Forgetting a record removes it and the folder once it is the last.
/// Why:
///   A record that stayed after its removal finished would be a folder the next command keeps reporting.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// forget(common, 'topic'); expect(existsSync(join(common, RECORD_FOLDER))).toBe(false);
/// ```
#[test]
fn forgetting_a_record_takes_the_folder_with_it() {
    let root: PathBuf = fixture("left-behind-record-forget");
    let common: PathBuf = root.join("common");
    std::fs::create_dir_all(common.as_path()).expect("common");
    let target: PathBuf = folder_with(root.as_path(), &[b"a"]);
    record_before_git(common.as_path(), OsStr::new("topic"), target.as_path()).expect("the record is written");
    assert!(common.join(RECORD_FOLDER).join("topic").is_file());
    forget(common.as_path(), OsStr::new("topic"));
    assert!(!common.join(RECORD_FOLDER).join("topic").exists(), "the record is gone");
    assert!(!common.join(RECORD_FOLDER).exists(), "the folder goes with the last record");
    remove(root.as_path());
}
