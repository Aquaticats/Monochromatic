//! What:
//!  The controls of the record's folder,
//! its name grammar and its read:
//!  what a listing classifies,
//!  what a read refuses about owner and mode,
//!  and what an absent record answers.
//! Why:
//!  The look destroys a snapshot on a record's word,
//!  so a folder or a file another account could write is refused before a byte is read,
//!  and a foreign entry is reported and never read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(isARecordName('git-worktree-clone-origin-00')).toBe(false);
//! ```

use crate::testing::test_support::fixture;
use crate::worktree_clone_origin_cleanup::origin_cleanup_fixtures::{OWN, an_owed_record};
use crate::worktree_clone_origin_cleanup::origin_cleanup_publication::CleanupFolder;
use crate::worktree_clone_origin_cleanup::origin_cleanup_reading::{
    an_existing_cleanup_record, is_a_record_name, is_a_staging_name, list_the_origin_cleanups, origin_cleanups_folder, read_the_cleanup_record,
};
use crate::worktree_clone_origin_cleanup::origin_cleanup_refusal::CleanupRefusalCause;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

/// One throwaway common directory with the wrapper's own folder made private.
fn a_common_dir(name: &str) -> PathBuf {
    let common: PathBuf = fixture(&format!("origin-cleanup-reading-{name}"));
    std::fs::create_dir_all(common.as_path()).expect("the common directory");
    return common;
}

#[test]
fn the_name_grammar_admits_one_shape_and_no_other() {
    assert!(is_a_record_name(OWN), "the prefix and the add's 32 lowercase digits");
    assert!(!is_a_record_name("origin"), "the design's example name is not this grammar");
    assert!(!is_a_record_name("git-worktree-clone-origin-445566778899AABBCCDDEEFF00112233"), "uppercase digits are not the add's spelling");
    assert!(!is_a_record_name("git-worktree-clone-origin-4455"));
    assert!(is_a_staging_name(".git-worktree-clone-origin-445566778899aabbccddeeff00112233.abc.partial"));
}

#[test]
fn a_listing_separates_records_from_foreign_and_staging_names() {
    let common: PathBuf = a_common_dir("listing");
    let folder: CleanupFolder = CleanupFolder::open_or_create(common.as_path()).expect("the folder is made");
    folder.publish(&an_owed_record()).expect("the record is written");
    std::fs::write(folder.path().join("foreign.txt"), b"x").expect("a foreign entry");
    std::fs::write(folder.path().join(format!(".{OWN}.abc.partial")), b"x").expect("a staging name");
    let listing = list_the_origin_cleanups(common.as_path()).expect("the folder lists").expect("the folder is there");
    assert_eq!(listing.records, vec![String::from(OWN)]);
    assert_eq!(listing.foreign, vec![String::from("foreign.txt")]);
    assert_eq!(listing.staging.len(), 1);
    assert_eq!(origin_cleanups_folder(common.as_path()), folder.path());
}

#[test]
fn a_folder_with_no_records_answers_no_listing_at_all() {
    let common: PathBuf = a_common_dir("absent");
    assert!(list_the_origin_cleanups(common.as_path()).expect("the read answers").is_none(), "no folder is the healthy state");
    assert!(an_existing_cleanup_record(common.as_path(), OWN).expect("the read answers").is_none(), "and no record stands");
}

#[test]
fn a_healthy_record_reads_back() {
    let common: PathBuf = a_common_dir("healthy");
    let folder: CleanupFolder = CleanupFolder::open_or_create(common.as_path()).expect("the folder is made");
    folder.publish(&an_owed_record()).expect("the record is written");
    let read = read_the_cleanup_record(common.as_path(), OWN).expect("the record reads");
    assert_eq!(read, an_owed_record());
    std::fs::set_permissions(folder.path(), std::fs::Permissions::from_mode(0o700)).expect("the folder's mode");
}

#[test]
fn a_record_any_account_may_write_refuses() {
    let common: PathBuf = a_common_dir("mode");
    let folder: CleanupFolder = CleanupFolder::open_or_create(common.as_path()).expect("the folder is made");
    let path: PathBuf = folder.publish(&an_owed_record()).expect("the record is written");
    std::fs::set_permissions(path.as_path(), std::fs::Permissions::from_mode(0o666)).expect("the file's mode");
    let refusal = read_the_cleanup_record(common.as_path(), OWN).expect_err("a file another account may write is not this account's record");
    assert_eq!(refusal.cause, CleanupRefusalCause::NotThisAccounts);
}

#[test]
fn a_record_named_another_snapshot_refuses() {
    let common: PathBuf = a_common_dir("mismatch");
    let folder: CleanupFolder = CleanupFolder::open_or_create(common.as_path()).expect("the folder is made");
    let other: String = String::from("git-worktree-clone-origin-00112233445566778899aabbccddeeff");
    let bytes: Vec<u8> = an_owed_record().encode().expect("the record encodes");
    std::fs::write(folder.path().join(other.as_str()), bytes).expect("the bytes are written under another name");
    let refusal = read_the_cleanup_record(common.as_path(), other.as_str()).expect_err("the name is the key");
    assert_eq!(refusal.cause, CleanupRefusalCause::NotCanonical);
}
