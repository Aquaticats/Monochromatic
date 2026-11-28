//! Controls for the table of kinds:
//!  every kind has a row,
//!  the one kind no round shows an ending for is refused with the design's own word,
//!  a locked holder that is this account's own unit is stopped through its
//!  manager,
//!  the taskless namespace's ending is built and names the round that showed it,
//!  every row names its evidence and its owed arm,
//!  and a refusal and a unit's stop end no blocker.

use crate::worktree_clone_forced_removal::forced_removal_words::RefusalWord;
use crate::worktree_clone_forced_removal::holder_endings::*;

/// Every kind the design names has one row,
/// and no row appears twice.
#[test]
fn every_kind_has_one_row() {
    let kinds: [HolderKind; 6] = [
        HolderKind::SessionTask,
        HolderKind::OtherAccountTask,
        HolderKind::UnitTask,
        HolderKind::TasklessNamespace,
        HolderKind::PathHolder,
        HolderKind::LockedHolder,
    ];
    let mut seen: Vec<HolderKind> = Vec::new();
    for kind in kinds {
        let row: KindEvidence = ending_of(kind).expect("every kind has a row");
        assert_eq!(row.kind, kind);
        assert!(!seen.contains(&kind), "kind `{}` has two rows", kind.word());
        seen.push(kind);
        assert!(
            !row.shown_by.is_empty(),
            "row `{}` names no evidence",
            kind.word()
        );
    }
    assert_eq!(seen.len(), THE_KINDS.len());
}

/// A locked holder that is this account's own unit is stopped through that
/// manager,
/// and the taskless namespace's ending is the one the walk measured.
#[test]
fn the_kind_no_round_shows_is_refused_and_a_unit_of_ours_is_stopped() {
    let locked: KindEvidence = ending_of(HolderKind::LockedHolder).expect("the locked row");
    assert_eq!(locked.ending, Ending::StopTheAccountsOwnUnitThenGoOn);
    // The row the owner's answer of 2026-11-26 refused now names the ending the walk measured, and the text that
    // said why it was refused stays beside it as the record of what changed.
    let taskless: KindEvidence =
        ending_of(HolderKind::TasklessNamespace).expect("the taskless row");
    assert_eq!(taskless.ending, Ending::EnterAndTakeTheCopyOut);
    assert_eq!(taskless.shown_by, TASKLESS_SHOWN_BY);
    assert!(
        taskless.shown_by.contains("7.3.5-1-cachyos"),
        "{}",
        taskless.shown_by
    );
    assert!(
        taskless.shown_by.contains("ENOSYS"),
        "the control's guest is named on the same row: {}",
        taskless.shown_by
    );
    assert!(
        TASKLESS_NOT_BUILT.contains("was refused until this round"),
        "the superseded reason says what it now is: {TASKLESS_NOT_BUILT}"
    );
    // The rows name the evidence they have and the arm that would settle what is still open.
    assert!(locked.shown_by.contains("guest arm"), "{}", locked.shown_by);
    assert!(
        locked.owed_arm.contains("not a service unit"),
        "{}",
        locked.owed_arm
    );
    // The taskless row's owed arm is no longer the guest that measures the walk, because this round measured it.
    assert!(!taskless.owed_arm.contains("6.19 or newer"));
}

/// The kinds whose ending a round shows say where,
/// and the two that only a schedule of one account's own child reaches say that instead.
#[test]
fn every_rows_evidence_says_what_is_shown_and_what_is_owed() {
    for kind in [HolderKind::SessionTask, HolderKind::PathHolder] {
        let row: KindEvidence = ending_of(kind).expect("a shown row");
        assert_eq!(row.ending, Ending::EndBlockersThenUnmount);
        assert!(
            row.shown_by.contains("this round's guest arm"),
            "{}",
            row.shown_by
        );
    }
    for kind in [HolderKind::OtherAccountTask, HolderKind::UnitTask] {
        let row: KindEvidence = ending_of(kind).expect("a row still owed");
        assert!(row.shown_by.contains("no round shows"), "{}", row.shown_by);
        assert!(row.owed_arm.contains("another account"), "{}", row.owed_arm);
    }
}

/// A refusal ends no blocker,
/// a unit's stop ends none either,
/// a taskless namespace's ending sends no signal either,
/// and each ending that does answers true,
/// which is what keeps a locked holder's jobs from being signalled.
#[test]
fn a_refusal_ends_no_blocker() {
    assert!(!ends_blockers(Ending::Refuse(RefusalWord::HolderLocked)));
    assert!(!ends_blockers(Ending::Refuse(RefusalWord::HolderUnendable)));
    assert!(!ends_blockers(Ending::StopTheAccountsOwnUnitThenGoOn));
    assert!(!ends_blockers(Ending::EnterAndTakeTheCopyOut));
    assert!(ends_blockers(Ending::EndBlockersThenUnmount));
    assert!(ends_blockers(Ending::UnmountInPlaceByHandle));
    assert!(ends_blockers(Ending::MakeTheParentASlaveThenMoveAndUnmount));
}

/// A kind the table lacks is a named error and not a default ending.
#[test]
fn a_kind_without_a_row_is_a_named_error() {
    // The table is the design's six kinds, so a seventh has no row; the function's own signature is what a caller
    // relies on, and the message names the kind it could not answer for.
    let missing: MissingKind = MissingKind {
        kind: HolderKind::LockedHolder,
    };
    let sentence: String = missing.to_string();
    assert!(sentence.contains("locked-holder"), "{sentence}");
    assert!(sentence.contains("no measured ending"), "{sentence}");
}
