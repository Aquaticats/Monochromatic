//! Controls of one step's record:
//!  the eight bytes a pass and a failure take,
//!  the decode that reads them back,
//!  and the two edges a parent must not read as an answer.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(decode(encode({ step: 'walk' }))).toEqual({ step: 'walk' });
//! ```

use super::{Decoded, RECORD_BYTES, StepRecord, decode_all};
use crate::account_companion::companion_calls::{CompanionCall, Failure};
use crate::account_companion::companion_steps::CompanionStep;

/// What:
///  A pass and a failure both survive their own round trip.
#[test]
fn a_record_decodes_to_the_step_and_failure_it_was_written_from() {
    let passed: StepRecord = StepRecord::passed(CompanionStep::Walk);
    assert_eq!(StepRecord::decode(passed.encode().as_slice()), Some(passed));
    let failed: StepRecord = StepRecord::failed(CompanionStep::ScratchTmpfs, Failure { call: CompanionCall::MountTmpfs, errno: libc::EPERM });
    assert_eq!(StepRecord::decode(failed.encode().as_slice()), Some(failed));
    assert_eq!(passed.encode().len(), RECORD_BYTES);
}

/// What:
///  A number this build does not know is no record,
///  so a child that sent one did not say what happened.
#[test]
fn a_step_or_call_this_build_does_not_know_is_no_record() {
    let unknown_step: [u8; RECORD_BYTES] = [200, 0, 0, 0, 0, 0, 0, 0];
    assert_eq!(StepRecord::decode(unknown_step.as_slice()), None, "a step number past this build's is no record");
    let unknown_call: [u8; RECORD_BYTES] = [0, 1, 200, 0, 1, 0, 0, 0];
    assert_eq!(StepRecord::decode(unknown_call.as_slice()), None, "a failure naming no call of this build is no record");
    assert_eq!(StepRecord::decode(unknowable(7).as_slice()), None, "bytes short of a record are no record");
}

/// What:
///  A short tail is the design's own status line the parent cannot parse,
///  while two whole records with no tail read as two.
#[test]
fn a_partial_record_is_unreadable_and_two_whole_ones_are_not() {
    let one: StepRecord = StepRecord::passed(CompanionStep::Setgroups);
    let two: StepRecord = StepRecord::passed(CompanionStep::UidMap);
    let mut bytes: Vec<u8> = Vec::new();
    bytes.extend_from_slice(one.encode().as_slice());
    bytes.extend_from_slice(two.encode().as_slice());
    let whole: Decoded = decode_all(bytes.as_slice());
    assert_eq!(whole.records, vec![one, two]);
    assert!(!whole.unreadable);
    bytes.extend_from_slice(&[0, 0, 0]);
    let cut: Decoded = decode_all(bytes.as_slice());
    assert_eq!(cut.records, vec![one, two], "the whole records before the tail are still read");
    assert!(cut.unreadable, "a tail short of a record is what the parent cannot parse");
}

/// Some bytes of one length,
/// for the shortness control.
fn unknowable(length: usize) -> Vec<u8> {
    return vec![0; length];
}
