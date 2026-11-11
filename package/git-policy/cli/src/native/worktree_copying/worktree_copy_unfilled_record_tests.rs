//! Controls for the record that names a worktree left without part of its source's ignored state:
//! what it names,
//! when it settles,
//! and that a copy still running is left alone.

use super::*;
use crate::json_record::quote;
use crate::test_support::{dead_copy_owner, fixture, remove};
use crate::worktree_copy_owner::current_copy_owner;
use crate::worktree_copy_recovery::recover_pending_worktree_copies;
use std::os::unix::fs::PermissionsExt;

/// The journal root of a fixture repository whose common directory is `common`,
/// made private as the records' reader requires.
fn record_root(common: &Path) -> PathBuf {
    let root: PathBuf = journal_root(common);
    std::fs::create_dir_all(root.as_path()).expect("record root");
    let parent: &Path = root.parent().expect("the record root's parent");
    std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700)).expect("private parent");
    std::fs::set_permissions(root.as_path(), std::fs::Permissions::from_mode(0o700)).expect("private root");
    return root;
}

/// The journal path of one copy in that root,
/// named as the copy names it,
/// with a dead owner unless the control says otherwise.
fn journal_of(common: &Path, token: &str) -> PathBuf {
    return record_root(common).join(format!("0123456789abcdef.{token}.json"));
}

/// A directory a record can name,
/// with the identity it has now.
fn worktree(root: &Path) -> (PathBuf, crate::fs_beneath::ObjectId) {
    let destination: PathBuf = root.join("worktree");
    std::fs::create_dir_all(destination.as_path()).expect("destination");
    let identity: crate::fs_beneath::ObjectId = crate::worktree_add_binding::path_identity(destination.as_path()).expect("identity");
    return (destination, identity);
}

/// The record one copy writes names its worktree,
/// its source,
/// what the copy ended with,
/// and its owner.
#[test]
fn a_record_names_the_worktree_the_source_and_the_owner() {
    let root: PathBuf = fixture("unfilled-record-encode");
    let (destination, identity) = worktree(root.as_path());
    let journal: PathBuf = root.join("0123456789abcdef.json");
    let owner: String = dead_copy_owner().token();
    remember_unfilled(journal.as_path(), destination.as_path(), identity, Path::new("/source/tree"), Some(owner.as_str())).expect("remembered");
    let copy: UnfilledCopy = decode_unfilled(std::fs::read_to_string(unfilled_path_of(journal.as_path())).expect("record").as_str()).expect("decoded");
    assert_eq!(copy.destination, destination);
    assert_eq!(copy.destination_identity, identity);
    assert_eq!(copy.source, PathBuf::from("/source/tree"));
    assert_eq!(copy.owner, Some(owner));
    assert!(!copy.reason.is_empty());
    remove(root.as_path());
}

/// The record takes the journal's identifier and no owner,
/// so a resumed copy that renames its journal keeps writing one record.
#[test]
fn the_record_keeps_the_journals_identifier_across_owners() {
    let first: PathBuf = unfilled_path_of(Path::new("/records/0123456789abcdef.100.1.2.json"));
    let again: PathBuf = unfilled_path_of(Path::new("/records/0123456789abcdef.200.1.2.json"));
    assert_eq!(first, again);
    assert_eq!(first, PathBuf::from("/records/0123456789abcdef.unfilled"));
}

/// A copy that finished removes the record it wrote before its first entry.
#[test]
fn a_finished_installation_forgets_the_record() {
    let root: PathBuf = fixture("unfilled-record-forget");
    let (destination, identity) = worktree(root.as_path());
    let journal: PathBuf = root.join("0123456789abcdef.json");
    remember_unfilled(journal.as_path(), destination.as_path(), identity, Path::new("/source/tree"), Some(dead_copy_owner().token().as_str())).expect("remembered");
    assert!(unfilled_path_of(journal.as_path()).exists());
    forget_unfilled(journal.as_path()).expect("forgotten");
    assert!(!unfilled_path_of(journal.as_path()).exists());
    remove(root.as_path());
}

/// A worktree that still stands and was not replaced is named again from every pass,
/// with the commands that settle it.
#[test]
fn a_standing_worktree_is_named_on_every_pass() {
    let root: PathBuf = fixture("unfilled-record-standing");
    let common: PathBuf = root.join("common");
    let (destination, identity) = worktree(root.as_path());
    let journal: PathBuf = journal_of(common.as_path(), dead_copy_owner().token().as_str());
    remember_unfilled(journal.as_path(), destination.as_path(), identity, Path::new("/source/tree"), Some(dead_copy_owner().token().as_str())).expect("remembered");
    let first: Vec<String> = report_unfilled_records(common.as_path()).expect("reported");
    let again: Vec<String> = report_unfilled_records(common.as_path()).expect("reported again");
    assert_eq!(first.len(), 1, "{first:?}");
    assert_eq!(first, again, "the line comes again until the worktree is settled");
    assert!(first[0].contains(quote(destination.to_string_lossy().as_ref()).as_str()), "{}", first[0]);
    assert!(first[0].contains("holds less than"), "{}", first[0]);
    assert!(first[0].contains("git worktree remove --force"), "{}", first[0]);
    assert!(first[0].contains("/source/tree"), "{}", first[0]);
    assert!(unfilled_path_of(journal.as_path()).exists(), "the record stays while the worktree stands");
    remove(root.as_path());
}

/// A record whose worktree is gone settles without a word,
/// and so does one whose path now names a different object.
#[test]
fn a_gone_or_replaced_worktree_settles_its_record() {
    let root: PathBuf = fixture("unfilled-record-settles");
    let common: PathBuf = root.join("common");
    let (gone, gone_identity) = worktree(root.as_path());
    let journal: PathBuf = journal_of(common.as_path(), dead_copy_owner().token().as_str());
    remember_unfilled(journal.as_path(), gone.as_path(), gone_identity, Path::new("/source/tree"), Some(dead_copy_owner().token().as_str())).expect("remembered");
    std::fs::remove_dir_all(gone.as_path()).expect("removed");
    assert_eq!(report_unfilled_records(common.as_path()).expect("reported"), Vec::<String>::new());
    assert!(!unfilled_path_of(journal.as_path()).exists(), "a worktree that is gone settles its record");
    let (replaced, before) = worktree(root.as_path());
    remember_unfilled(journal.as_path(), replaced.as_path(), before, Path::new("/source/tree"), Some(dead_copy_owner().token().as_str())).expect("remembered");
    std::fs::remove_dir_all(replaced.as_path()).expect("removed");
    std::fs::create_dir_all(replaced.as_path()).expect("made again");
    assert_eq!(report_unfilled_records(common.as_path()).expect("reported"), Vec::<String>::new());
    assert!(!unfilled_path_of(journal.as_path()).exists(), "a worktree made again settles the record of the one before it");
    remove(root.as_path());
}

/// A record whose owner still runs belongs to a copy filling that worktree now,
/// and is left without a word.
#[test]
fn a_record_of_a_running_copy_is_left_alone() {
    let root: PathBuf = fixture("unfilled-record-running");
    let common: PathBuf = root.join("common");
    let (destination, identity) = worktree(root.as_path());
    let owner: String = current_copy_owner().expect("owner").token();
    let journal: PathBuf = journal_of(common.as_path(), owner.as_str());
    remember_unfilled(journal.as_path(), destination.as_path(), identity, Path::new("/source/tree"), Some(owner.as_str())).expect("remembered");
    assert_eq!(report_unfilled_records(common.as_path()).expect("reported"), Vec::<String>::new(), "a copy that runs owns its record");
    // The same record, once its owner is gone, names the worktree.
    remember_unfilled(journal.as_path(), destination.as_path(), identity, Path::new("/source/tree"), Some(dead_copy_owner().token().as_str())).expect("remembered");
    assert_eq!(report_unfilled_records(common.as_path()).expect("reported").len(), 1);
    remove(root.as_path());
}

/// A record this release does not write is kept,
/// and named,
/// because nothing it says can be acted on.
#[test]
fn a_record_this_release_does_not_write_is_kept_and_named() {
    let root: PathBuf = fixture("unfilled-record-foreign");
    let common: PathBuf = root.join("common");
    let record: PathBuf = record_root(common.as_path()).join("0123456789abcdef.unfilled");
    std::fs::write(record.as_path(), b"destination=nothing-useful\n").expect("foreign record");
    std::fs::set_permissions(record.as_path(), std::fs::Permissions::from_mode(0o600)).expect("private record");
    let refusal: WorktreeCopyError = report_unfilled_records(common.as_path()).expect_err("refused");
    assert!(refusal.0.contains("is not one this release writes"), "{}", refusal.0);
    assert!(refusal.0.contains("check it and remove the file"), "{}", refusal.0);
    assert!(record.exists(), "a record that cannot be read is kept");
    remove(root.as_path());
}

/// The pending check sees the record,
/// so a command in the repository runs the pass that names the worktree.
#[test]
fn a_record_makes_the_pending_check_run() {
    let root: PathBuf = fixture("unfilled-record-pending");
    let common: PathBuf = root.join("common");
    let record: PathBuf = record_root(common.as_path()).join("0123456789abcdef.unfilled");
    assert!(!unfilled_records_exist(common.as_path()).expect("answer"));
    std::fs::write(record.as_path(), b"record\n").expect("record");
    assert!(unfilled_records_exist(common.as_path()).expect("answer"));
    remove(root.as_path());
}

/// The check every forwarded command runs names the worktree,
/// so nothing about it is left in one command's output.
#[test]
fn the_pending_check_names_the_worktree() {
    let root: PathBuf = fixture("unfilled-record-forwarded");
    let common: PathBuf = root.join("common");
    let (destination, identity) = worktree(root.as_path());
    let journal: PathBuf = journal_of(common.as_path(), dead_copy_owner().token().as_str());
    remember_unfilled(journal.as_path(), destination.as_path(), identity, Path::new("/source/tree"), Some(dead_copy_owner().token().as_str())).expect("remembered");
    let mut told: Vec<u8> = Vec::new();
    recover_pending_worktree_copies(common.as_path(), &crate::test_support::git_context(), &mut told).expect("the pass");
    let text: String = String::from_utf8(told).expect("text");
    assert!(text.contains(quote(destination.to_string_lossy().as_ref()).as_str()), "{text}");
    assert!(text.contains("holds less than"), "{text}");
    remove(root.as_path());
}
