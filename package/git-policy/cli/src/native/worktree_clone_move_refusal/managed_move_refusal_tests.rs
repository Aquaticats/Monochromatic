//! Controls for the `git worktree move` refusal:
//!  a bound slot refuses,
//!  mounted or not,
//!  at every phase,
//!  a slot with no record is left to Git,
//!  a record no reader may act on refuses without naming a phase,
//!  a record naming another path refuses when Git's list names no worktree at
//!  the move's source,
//!  that record's slot move is still refused when it names the same folder by
//!  another spelling,
//!  a path no record can be keyed by is left to Git,
//!  and the sentence says the slot is fixed.

use crate::testing::test_support::fixture;
use crate::worktree_clone_binding_record::binding_words::Phase;
use crate::worktree_clone_move_refusal::managed_move_refusal::{
    ManagedMoveRefusal,
    ManagedMoveRefusalCause,
    refuse_move_of_a_bound_slot,
};
use crate::worktree_clone_slot_state::slot_state_test_records::{
    publish,
    record_at,
};
use std::path::{
    Path,
    PathBuf,
};

/// The fixture's slot path.
fn slot_path() -> PathBuf {
    return PathBuf::from("/repo/.claude/worktrees/topic");
}

/// One fixture common directory with the slot's record already published.
fn with_record(
    name: &str,
    phase: Phase,
) -> PathBuf {
    let root: PathBuf = fixture(name);
    let common: PathBuf = root.join(".git");
    publish(
        common.as_path(),
        &record_at(phase, slot_path().as_path()),
        "topic",
    );
    return common;
}

/// A bound slot refuses at every phase the record can hold,
/// which is the mounted and the unmounted arm alike.
#[test]
fn a_bound_slot_refuses_at_every_phase() {
    let phases: [Phase; 8] = [
        Phase::Claimed,
        Phase::Slot,
        Phase::Registered,
        Phase::Snapshotted,
        Phase::Cloned,
        Phase::Mounted,
        Phase::Filled,
        Phase::Settling,
    ];
    for phase in phases {
        let common: PathBuf = with_record("move-refusal-bound", phase);
        let refusal: ManagedMoveRefusal =
            refuse_move_of_a_bound_slot(common.as_path(), slot_path().as_path())
                .unwrap_or_else(|| panic!("phase `{}` must refuse", phase.word()));
        assert_eq!(refusal.slot, "topic");
        match refusal.cause {
            ManagedMoveRefusalCause::Bound {
                phase: found,
                clone_dataset,
            } => {
                assert_eq!(found, phase, "phase `{}`", phase.word());
                assert!(clone_dataset.contains("topic"), "{clone_dataset}");
            }
            found => panic!(
                "phase `{}` refused for another cause: {found:?}",
                phase.word()
            ),
        }
        assert!(
            refusal.sentence.contains("is refused"),
            "{}",
            refusal.sentence
        );
        assert!(
            refusal.sentence.contains("is fixed"),
            "{}",
            refusal.sentence
        );
        assert!(
            refusal.sentence.contains("does not move the clone"),
            "{}",
            refusal.sentence
        );
        assert!(
            refusal.sentence.contains("Nothing was moved"),
            "{}",
            refusal.sentence
        );
    }
}

/// A slot with no record is not the wrapper's to refuse.
#[test]
fn a_slot_with_no_record_is_left_to_git() {
    let root: PathBuf = fixture("move-refusal-absent");
    let common: PathBuf = root.join(".git");
    std::fs::create_dir_all(common.as_path()).expect("the fixture common directory");
    assert_eq!(
        refuse_move_of_a_bound_slot(common.as_path(), slot_path().as_path()),
        None
    );
}

/// A record no reader may act on refuses,
/// and the sentence carries the reader's reason.
#[test]
fn an_unreadable_record_refuses_with_its_reason() {
    let root: PathBuf = fixture("move-refusal-unreadable");
    let common: PathBuf = root.join(".git");
    let home: PathBuf =
        crate::worktree_clone_binding_record::registration_home::home_folder(common.as_path());
    std::fs::create_dir_all(home.as_path()).expect("the fixture home");
    std::fs::write(home.join("topic.binding"), b"not a record\n").expect("the foreign file writes");
    let refusal: ManagedMoveRefusal =
        refuse_move_of_a_bound_slot(common.as_path(), slot_path().as_path())
            .expect("an unreadable record refuses");
    match &refusal.cause {
        ManagedMoveRefusalCause::Unreadable {
            reason,
        } => assert!(reason.contains("topic.binding"), "{reason}"),
        found => panic!("expected the unreadable cause, got {found:?}"),
    }
    assert!(
        refusal.sentence.contains("cannot be read"),
        "{}",
        refusal.sentence
    );
}

/// A record filed under the same leaf name for another path refuses too,
/// because the two statements disagree and Git's list names no worktree at the
/// move's source.
#[test]
fn a_record_naming_another_path_refuses() {
    let root: PathBuf = fixture("move-refusal-another-path");
    let common: PathBuf = root.join(".git");
    let recorded: PathBuf = PathBuf::from("/other/repo/.claude/worktrees/topic");
    publish(
        common.as_path(),
        &record_at(Phase::Filled, recorded.as_path()),
        "topic",
    );
    let refusal: ManagedMoveRefusal =
        refuse_move_of_a_bound_slot(common.as_path(), slot_path().as_path())
            .expect("a disagreeing record refuses");
    match &refusal.cause {
        ManagedMoveRefusalCause::NamesAnotherPath {
            recorded: found,
        } => assert_eq!(found, &recorded),
        found => panic!("expected the other-path cause, got {found:?}"),
    }
    assert!(
        refusal
            .sentence
            .contains("/other/repo/.claude/worktrees/topic"),
        "{}",
        refusal.sentence
    );
}

/// What:
///  A record that names this very slot through a symbolic link refuses as the
///  slot's move,
///  not as another path's.
/// Why:
///  The record holds one spelling of the slot folder and a move may name
///  another,
///  so reading the second spelling as another path would let the slot's own move
///  through the allowance a plain same-leaf worktree has,
///  which the answer keeps refused.
#[test]
fn a_record_naming_the_slot_through_a_link_still_refuses_as_the_slot() {
    let root: PathBuf = fixture("move-refusal-linked-slot");
    let common: PathBuf = root.join(".git");
    let real: PathBuf = root.join("slots/topic");
    std::fs::create_dir_all(real.as_path()).expect("the slot folder");
    let linked: PathBuf = root.join("linked-topic");
    std::os::unix::fs::symlink(real.as_path(), linked.as_path()).expect("the link");
    publish(
        common.as_path(),
        &record_at(Phase::Filled, linked.as_path()),
        "topic",
    );
    let refusal: ManagedMoveRefusal = refuse_move_of_a_bound_slot(common.as_path(), real.as_path())
        .expect("the slot's own move refuses");
    assert!(
        matches!(refusal.cause, ManagedMoveRefusalCause::Bound { .. }),
        "the other spelling is still the slot: {:?}",
        refusal.cause
    );
}

/// A source whose last component is no plain slot name cannot hold a record,
/// so the move is left to Git.
#[test]
fn a_path_no_record_can_be_keyed_by_is_left_to_git() {
    let root: PathBuf = fixture("move-refusal-no-slot");
    let common: PathBuf = root.join(".git");
    std::fs::create_dir_all(common.as_path()).expect("the fixture common directory");
    let no_name: &Path = Path::new("/");
    assert_eq!(refuse_move_of_a_bound_slot(common.as_path(), no_name), None);
    let upper: &Path = Path::new("/repo/.claude/worktrees/Topic");
    assert_eq!(refuse_move_of_a_bound_slot(common.as_path(), upper), None);
}

/// The refusal reads only the record,
/// so the mounted and unmounted arms are the same call:
/// one fixture,
/// two answers from the same code because only the mount differs.
#[test]
fn the_refusal_never_reads_a_mount() {
    let common: PathBuf = with_record("move-refusal-same-code", Phase::Filled);
    let first: Option<ManagedMoveRefusal> =
        refuse_move_of_a_bound_slot(common.as_path(), slot_path().as_path());
    let second: Option<ManagedMoveRefusal> =
        refuse_move_of_a_bound_slot(common.as_path(), slot_path().as_path());
    assert_eq!(first, second);
    assert!(first.is_some());
}
