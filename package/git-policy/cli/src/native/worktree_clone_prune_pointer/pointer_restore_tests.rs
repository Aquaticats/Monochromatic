//! Controls for the one write that puts the recorded pointer back:
//! what it writes,
//! what it leaves alone,
//! and every shape it refuses.

use super::*;
use crate::test_support::{fixture, remove};
use std::os::unix::fs::PermissionsExt as _;
use std::path::PathBuf;

/// The bytes one record holds in these controls.
const RECORDED: &[u8] = b"gitdir: /repo/.git/worktrees/one\n";

/// One slot folder with nothing at `.git`.
fn empty_slot(name: &str) -> PathBuf {
    let root: PathBuf = fixture(name);
    let slot: PathBuf = root.join("one");
    std::fs::create_dir(slot.as_path()).expect("slot");
    return slot;
}

/// What:
///  A missing pointer is written from the record,
///  with Git's own mode,
///  and no staging name is left.
/// Why:
///  This is the arm a prune runs,
///  and a leftover staging file would be a name Git never writes.
#[test]
fn a_missing_pointer_is_written_from_the_record() {
    let slot: PathBuf = empty_slot("pointer-restore-missing");
    let outcome: RestoreOutcome = restore_the_slot_pointer(slot.as_path(), &PointerSite::Underlay, Some(RECORDED));
    assert_eq!(outcome, RestoreOutcome::Written { bytes: RECORDED.len() });
    assert_eq!(std::fs::read(slot.join(".git")).expect("pointer reads"), RECORDED);
    let mode: u32 = std::fs::metadata(slot.join(".git")).expect("pointer metadata").permissions().mode() & 0o777;
    assert_eq!(mode, POINTER_FILE_MODE, "the pointer is written with Git's own mode");
    let left: Vec<PathBuf> = std::fs::read_dir(slot.as_path())
        .expect("slot lists")
        .flatten()
        .map(|entry| return entry.file_name().into())
        .filter(|name: &PathBuf| return name.to_string_lossy().starts_with(".git.pointer-restore."))
        .collect();
    assert!(left.is_empty(), "no staging name is left: {left:?}");
    remove(slot.parent().expect("root"));
}

/// What:
///  A pointer with other bytes is replaced,
///  and the bytes the record holds are the ones left.
/// Why:
///  A changed pointer names another registration,
///  and the record's bytes are what Git was given.
#[test]
fn a_different_pointer_is_replaced() {
    let slot: PathBuf = empty_slot("pointer-restore-different");
    std::fs::write(slot.join(".git"), b"gitdir: /repo/.git/worktrees/two\n").expect("other pointer");
    let outcome: RestoreOutcome = restore_the_slot_pointer(slot.as_path(), &PointerSite::AtTheCloneRoot, Some(RECORDED));
    assert_eq!(outcome, RestoreOutcome::Written { bytes: RECORDED.len() });
    assert_eq!(std::fs::read(slot.join(".git")).expect("pointer reads"), RECORDED);
    remove(slot.parent().expect("root"));
}

/// What:
///  A pointer that is already the record's is left as it stands.
/// Why:
///  A prune runs this write for every slot it protects,
///  and rewriting a file that is already right would take a lock for nothing.
#[test]
fn an_intact_pointer_is_not_rewritten() {
    let slot: PathBuf = empty_slot("pointer-restore-intact");
    std::fs::write(slot.join(".git"), RECORDED).expect("pointer");
    assert_eq!(restore_the_slot_pointer(slot.as_path(), &PointerSite::Underlay, Some(RECORDED)), RestoreOutcome::AlreadyIntact);
    remove(slot.parent().expect("root"));
}

/// What:
///  Nothing is written without recorded bytes,
///  from a mount this route did not make,
///  over a directory,
///  or through a link.
/// Why:
///  Each of those would write a file the record does not prove,
///  and a copied administration must survive untouched.
#[test]
fn every_other_shape_refuses_and_changes_nothing() {
    let slot: PathBuf = empty_slot("pointer-restore-refused");
    assert!(matches!(restore_the_slot_pointer(slot.as_path(), &PointerSite::Underlay, None), RestoreOutcome::NothingToWrite { .. }));
    assert!(!slot.join(".git").exists(), "nothing was written without bytes");
    assert!(matches!(
        restore_the_slot_pointer(slot.as_path(), &PointerSite::AnotherMount { described: String::from("`tank/other` stands at the slot") }, Some(RECORDED)),
        RestoreOutcome::Refused { .. }
    ));
    assert!(!slot.join(".git").exists());
    std::fs::create_dir(slot.join(".git")).expect("copied administration");
    assert!(matches!(restore_the_slot_pointer(slot.as_path(), &PointerSite::Underlay, Some(RECORDED)), RestoreOutcome::Refused { .. }));
    assert!(slot.join(".git").is_dir(), "the copied administration stands as it was");
    std::fs::remove_dir(slot.join(".git")).expect("directory removed");
    let other: PathBuf = slot.parent().expect("root").join("elsewhere");
    std::fs::write(other.as_path(), b"untouched").expect("target");
    std::os::unix::fs::symlink(other.as_path(), slot.join(".git")).expect("planted link");
    assert!(matches!(restore_the_slot_pointer(slot.as_path(), &PointerSite::Underlay, Some(RECORDED)), RestoreOutcome::Refused { .. }));
    assert_eq!(std::fs::read(other.as_path()).expect("target reads"), b"untouched");
    // A link to a file that holds the recorded bytes is still refused: the read never follows it, so an intact-looking
    // target is not mistaken for the slot's own pointer.
    let pretending: PathBuf = slot.parent().expect("root").join("pretending");
    std::fs::write(pretending.as_path(), RECORDED).expect("pretending pointer");
    std::fs::remove_file(slot.join(".git")).expect("planted link removed");
    std::os::unix::fs::symlink(pretending.as_path(), slot.join(".git")).expect("planted link to the recorded bytes");
    assert!(matches!(restore_the_slot_pointer(slot.as_path(), &PointerSite::Underlay, Some(RECORDED)), RestoreOutcome::Refused { .. }));
    assert_eq!(std::fs::read(pretending.as_path()).expect("pretending reads"), RECORDED, "the target is not written through");
    remove(slot.parent().expect("root"));
}
