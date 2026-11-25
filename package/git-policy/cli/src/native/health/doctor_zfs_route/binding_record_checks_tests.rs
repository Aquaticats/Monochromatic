//! What:
//!  Controls of one slot's binding-record line:
//!  a slot with no record,
//!  a record that cannot be read,
//!  a filled record with every field a removal needs,
//!  a record of an earlier phase,
//!  and a settling one.
//! Why:
//!  The removal's own refusals send a person to this record,
//!  so the line must name the clone, the origin and the registration a removal
//!  proves itself against,
//!  and it must never turn a record nobody could read into an absence.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(checkBindingRecord(reading)).toMatchObject({ kind: 'pass', detail: expect.stringContaining('phase=filled') });
//! ```

use super::{NOT_RECORDED, check_binding_record};
use crate::health::doctor_zfs_route::CheckOutcome;
use crate::worktree_clone_binding_record::binding_words::Phase;
use crate::worktree_clone_slot_state::slot_state_readings::BindingReading;
use crate::worktree_clone_slot_state::slot_state_test_records::{CLONE_DATASET, POOL, record_at};
use std::path::PathBuf;

/// The slot's whole path.
fn slot_path() -> PathBuf {
    return PathBuf::from("/repo/.claude/worktrees/topic");
}

/// The record one phase holds at the slot.
fn record(phase: Phase) -> BindingReading {
    return BindingReading::Record(Box::new(record_at(phase, slot_path().as_path())));
}

/// A slot with no record passes,
/// because no add claims it.
#[test]
fn a_slot_without_a_record_passes() {
    let answer: CheckOutcome = check_binding_record(&BindingReading::Absent);
    let CheckOutcome::Pass { detail } = answer else {
        panic!("a slot without a record passes: {answer:?}");
    };
    assert!(detail.contains("no binding record"), "{detail}");
}

/// A record that cannot be read is unknown,
/// with the reason the record's own reader wrote.
#[test]
fn an_unreadable_record_is_unknown() {
    let answer: CheckOutcome = check_binding_record(&BindingReading::Unreadable { reason: String::from("the record is longer than its bound") });
    let CheckOutcome::Unknown { reason } = answer else {
        panic!("an unreadable record is unknown: {answer:?}");
    };
    assert!(reason.contains("longer than its bound"), "{reason}");
}

/// A filled record passes,
/// naming the phase,
/// the clone and its GUID,
/// the origin and the registration.
#[test]
fn a_filled_record_names_every_field() {
    let answer: CheckOutcome = check_binding_record(&record(Phase::Filled));
    let CheckOutcome::Pass { detail } = answer else {
        panic!("a filled record passes: {answer:?}");
    };
    assert!(detail.contains("phase=filled"), "{detail}");
    assert!(detail.contains(CLONE_DATASET), "{detail}");
    assert!(detail.contains(POOL), "{detail}");
    assert!(detail.contains("4821193048571903102"), "{detail}");
    assert!(detail.contains("origin snapshot"), "{detail}");
    assert!(detail.contains("/repo/.git/worktrees/topic"), "{detail}");
}

/// A record of an earlier phase prints the absent words for the fields that
/// phase has not proved,
/// rather than an empty value.
#[test]
fn an_earlier_phase_prints_the_absent_words() {
    let answer: CheckOutcome = check_binding_record(&record(Phase::Snapshotted));
    let CheckOutcome::Pass { detail } = answer else {
        panic!("an earlier phase passes: {answer:?}");
    };
    assert!(detail.contains("phase=snapshotted"), "{detail}");
    assert!(detail.contains(NOT_RECORDED), "{detail}");
    assert!(detail.contains("GUID not recorded at this phase"), "{detail}");
}

/// A settling record names its settle reason,
/// which is what the add's resume reads.
#[test]
fn a_settling_record_names_its_reason() {
    let answer: CheckOutcome = check_binding_record(&record(Phase::Settling));
    let CheckOutcome::Pass { detail } = answer else {
        panic!("a settling record passes: {answer:?}");
    };
    assert!(detail.contains("settle reason"), "{detail}");
    assert!(detail.contains("enospc"), "{detail}");
}
