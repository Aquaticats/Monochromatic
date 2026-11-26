//! What:
//!  The controls of the folder's writes and its one removal:
//!  a record publishes and reads back,
//!  a bad name publishes nothing,
//!  and a removal of a name that is gone is the same completion.
//! Why:
//!  The writer proves its own bytes read before the rename,
//!  because a record its reader refuses would leave the cleanup with nothing that carries it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! folder.publish(record); folder.remove(OWN); folder.remove(OWN);
//! ```

use crate::testing::test_support::fixture;
use crate::worktree_clone_origin_cleanup::origin_cleanup_fixtures::{
    OWN,
    an_owed_record,
};
use crate::worktree_clone_origin_cleanup::origin_cleanup_publication::CleanupFolder;
use crate::worktree_clone_origin_cleanup::origin_cleanup_reading::read_the_cleanup_record;
use crate::worktree_clone_origin_cleanup::origin_cleanup_record::OriginCleanupRecord;
use crate::worktree_clone_origin_cleanup::origin_cleanup_refusal::CleanupRefusalCause;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

/// One throwaway common directory.
fn a_common_dir(name: &str) -> PathBuf {
    let common: PathBuf = fixture(&format!("origin-cleanup-publication-{name}"));
    std::fs::create_dir_all(common.as_path()).expect("the common directory");
    return common;
}

#[test]
fn a_record_publishes_into_a_private_folder_and_reads_back() {
    let common: PathBuf = a_common_dir("publish");
    let folder: CleanupFolder =
        CleanupFolder::open_or_create(common.as_path()).expect("the folder is made");
    let path: PathBuf = folder
        .publish(&an_owed_record())
        .expect("the record is written");
    let mode: u32 = std::fs::metadata(folder.path())
        .expect("the folder's metadata")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o700, "the folder is this account's own");
    let bytes: u32 = std::fs::metadata(path.as_path())
        .expect("the file's metadata")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(bytes, 0o600, "no other account may write the record");
    assert_eq!(
        read_the_cleanup_record(common.as_path(), OWN).expect("the record reads"),
        an_owed_record()
    );
}

#[test]
fn a_record_of_a_name_outside_the_grammar_publishes_nothing() {
    let common: PathBuf = a_common_dir("bad-name");
    let folder: CleanupFolder =
        CleanupFolder::open_or_create(common.as_path()).expect("the folder is made");
    let mut record: OriginCleanupRecord = an_owed_record();
    record.origin_snapshot = String::from("origin");
    let refusal = folder
        .publish(&record)
        .expect_err("the record's key is the add's own name");
    assert_eq!(refusal.cause, CleanupRefusalCause::NotCanonical);
    assert_eq!(
        std::fs::read_dir(folder.path())
            .expect("the folder lists")
            .count(),
        0,
        "nothing was written"
    );
}

#[test]
fn removing_a_record_that_is_gone_is_the_same_completion() {
    let common: PathBuf = a_common_dir("remove");
    let folder: CleanupFolder =
        CleanupFolder::open_or_create(common.as_path()).expect("the folder is made");
    let path: PathBuf = folder
        .publish(&an_owed_record())
        .expect("the record is written");
    folder.remove(OWN).expect("the record goes");
    assert!(std::fs::symlink_metadata(path.as_path()).is_err());
    folder
        .remove(OWN)
        .expect("a second removal is the same completion");
}
