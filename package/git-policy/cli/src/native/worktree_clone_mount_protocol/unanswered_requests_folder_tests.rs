//! What:
//!  Controls of the folder of unanswered requests:
//!  a wait's entry published whole,
//!  the folder that goes with its last entry,
//!  the attempt a reader takes when one above it is empty,
//!  and every file the reader must refuse.
//! Why:
//!  A command reads this folder before it runs,
//!  so a file it believed wrongly is a clone nudged for ever or an entry
//!  lost,
//!  and a folder it could not leave would make every command pay a listing
//!  (`doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
//!  "The record:
//! one file per clone").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const folder = UnansweredRequests.at(fixture('requests'), process.geteuid());
//! folder.recordTheRequest(entry);
//! ```

use super::{
    REQUESTS_FOLDER_NAME,
    UnansweredRequests,
    this_account,
};
use crate::testing::test_support::fixture;
use crate::worktree_clone_mount_protocol::clone_mount_intent::MountOperation;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use crate::worktree_clone_mount_protocol::unanswered_request_entry::{
    RequestEntry,
    RequestOutcome,
};
use crate::worktree_clone_mount_protocol::unanswered_request_names::{
    EntryName,
    fresh_staging_name,
};
use std::os::unix::fs::PermissionsExt;
use std::path::{
    Path,
    PathBuf,
};

/// The clone every control writes.
fn guid() -> DatasetGuid {
    return DatasetGuid::parse("1234567890123456789").expect("the control GUID is canonical");
}

/// One waiting entry of this clone.
fn waiting() -> RequestEntry {
    return RequestEntry {
        guid: guid(),
        nonce: RequestNonce::parse("0123456789abcdef".repeat(2).as_str())
            .expect("the control nonce is 32 hexadecimal digits"),
        operation: MountOperation::Mount,
        written_at: String::from("2026-11-27T21:14:14.000Z"),
        outcome: RequestOutcome::Waiting,
        dataset: String::from("zpcachyos/nosnap/clones/one"),
    };
}

/// A fixture folder the folder of unanswered requests lives in.
fn root(name: &str) -> PathBuf {
    return fixture(name);
}

/// The folder object at a fixture root,
/// owned by this account as the live one is.
fn folder_at(root: &Path) -> UnansweredRequests {
    return UnansweredRequests::at(root.join(REQUESTS_FOLDER_NAME), this_account());
}

/// A wait's entry is published whole,
/// the folder is made,
/// and the entry reads back as the attempt the wait took.
#[test]
fn a_wait_s_entry_is_published_whole_and_read_back() {
    let root: PathBuf = root("requests-published");
    let folder: UnansweredRequests = folder_at(root.as_path());
    assert!(
        !folder.stands(),
        "a repository with no request has no folder, which is the one `stat` a command pays"
    );
    let entry: RequestEntry = waiting();
    let name: EntryName = folder
        .record_the_request(&entry)
        .expect("the entry is written");
    assert_eq!(name.attempt, 1);
    assert!(folder.stands());
    let mode: u32 = std::fs::metadata(folder.path())
        .expect("the folder exists")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o700, "the folder is the account's alone");
    let entry_mode: u32 = std::fs::metadata(folder.path().join(name.file_name()))
        .expect("the entry exists")
        .permissions()
        .mode()
        & 0o001;
    assert_eq!(entry_mode, 0, "the entry is the account's alone");
    assert_eq!(
        folder.the_entries(),
        vec![(name, entry)],
        "the entry reads back as the one the wait wrote, and the staging name is gone"
    );
    assert_eq!(
        std::fs::read_dir(folder.path())
            .expect("the folder lists")
            .flatten()
            .count(),
        1,
        "the staging name a finished writer made is removed"
    );
}

/// The folder goes with its last entry,
/// so a repository whose request was answered returns to the one-`stat` case.
#[test]
fn the_folder_goes_with_its_last_entry() {
    let root: PathBuf = root("requests-gone");
    let folder: UnansweredRequests = folder_at(root.as_path());
    let _first: EntryName = folder
        .record_the_request(&waiting())
        .expect("the entry is written");
    let _second: EntryName = folder
        .record_the_request(&waiting())
        .expect("the second attempt is written");
    assert_eq!(folder.the_entries().len(), 1, "one entry per clone is read");
    folder.remove_every_attempt(guid());
    assert!(!folder.stands(), "the folder went with its last entry");
    assert!(folder.the_entries().is_empty());
    folder.remove_every_attempt(guid());
}

/// An empty attempt above a readable one is skipped,
/// and the number a claim takes is one past the highest name present,
/// so a crash's empty file stops neither the read nor the next claim.
#[test]
fn an_empty_attempt_above_the_readable_one_is_skipped() {
    let root: PathBuf = root("requests-empty-above");
    let folder: UnansweredRequests = folder_at(root.as_path());
    let first: EntryName = folder
        .record_the_request(&waiting())
        .expect("the entry is written");
    let empty: EntryName = EntryName {
        guid: guid(),
        attempt: 2,
    };
    std::fs::write(folder.path().join(empty.file_name()), b"")
        .expect("the control writes an empty claim");
    assert_eq!(
        folder.the_entries(),
        vec![(first, waiting())],
        "the reader takes the highest readable attempt and skips the empty one above it"
    );
    assert_eq!(folder.attempts_of(guid()), vec![1, 2]);
    assert_eq!(
        super::super::unanswered_request_names::one_past_the_highest(
            folder.attempts_of(guid()).as_slice()
        ),
        3,
        "a claim after a crash's empty file takes the number one past the highest name present"
    );
}

/// A staging name a killed writer left blocks no later writer and is read as
/// no entry.
#[test]
fn a_killed_writer_s_staging_name_blocks_nothing() {
    let root: PathBuf = root("requests-staging-left");
    let folder: UnansweredRequests = folder_at(root.as_path());
    let lower: EntryName = folder
        .record_the_request(&waiting())
        .expect("the entry is written");
    let leftover: String = fresh_staging_name().expect("the random source answers");
    std::fs::write(folder.path().join(leftover.as_str()), b"half an entry")
        .expect("the control leaves a killed writer's file");
    let written: EntryName = folder
        .record_the_request(&waiting())
        .expect("a later writer's own name is free");
    assert_eq!(
        written.attempt, 2,
        "the number counts the entries and not a staging name"
    );
    assert_eq!(
        folder.the_entries(),
        vec![(written.clone(), waiting())],
        "the reader takes the highest readable attempt and reads no staging name"
    );
    assert!(
        folder.path().join(leftover.as_str()).exists(),
        "a staging name another writer made is left for `doctor` to remove"
    );
    assert_ne!(lower, written);
}
