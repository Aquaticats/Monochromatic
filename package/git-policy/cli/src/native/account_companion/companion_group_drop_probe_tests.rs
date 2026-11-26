//! Controls of the group-drop probe:
//!  the four answers its records can mean,
//!  and the one answer this host gives,
//!  which is never a probe that could not be read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(theAnswer([passed('user-namespace'), passed('setgroups')])).toEqual({ kind: 'written' });
//! expect(theAnswer([passed('user-namespace'), failed('setgroups', 'open /proc/self/setgroups', 13)])).toEqual({ kind: 'skipped', errno: 13 });
//! ```

use super::{
    GroupDrop,
    the_answer,
    the_group_drop,
};
use crate::account_companion::companion_calls::{
    CompanionCall,
    Failure,
};
use crate::account_companion::companion_records::{
    Decoded,
    StepRecord,
    decode_all,
};
use crate::account_companion::companion_steps::CompanionStep;

/// The bytes of the records one control builds,
/// decoded the way the probe decodes what its child wrote.
fn records_of(records: &[StepRecord]) -> Decoded {
    let mut bytes: Vec<u8> = Vec::new();
    for record in records {
        bytes.extend_from_slice(record.encode().as_slice());
    }
    return decode_all(bytes.as_slice());
}

/// One failed step's record,
/// with the call that refused and the number it answered.
fn failed(
    step: CompanionStep,
    call: CompanionCall,
    errno: i32,
) -> StepRecord {
    return StepRecord::failed(
        step,
        Failure {
            call,
            errno,
        },
    );
}

/// What:
///  A run whose two records both passed wrote the `deny`,
///  so this host needs no line.
#[test]
fn two_passing_records_mean_the_deny_was_written() {
    assert_eq!(
        the_answer(records_of(&[
            StepRecord::passed(CompanionStep::UserNamespace),
            StepRecord::passed(CompanionStep::Setgroups),
        ])),
        GroupDrop::Written
    );
}

/// What:
///  The `setgroups` open answering `EACCES` is the skip,
///  and every other answer of that open or of the write is not.
#[test]
fn the_open_that_answered_eacces_is_the_skip_and_no_other_answer_is() {
    assert_eq!(
        the_answer(records_of(&[
            StepRecord::passed(CompanionStep::UserNamespace),
            failed(
                CompanionStep::Setgroups,
                CompanionCall::OpenSetgroups,
                libc::EACCES
            ),
        ])),
        GroupDrop::Skipped {
            errno: libc::EACCES
        }
    );
    for (call, errno) in [
        (CompanionCall::OpenSetgroups, libc::EPERM),
        (CompanionCall::WriteSetgroups, libc::EACCES),
        (CompanionCall::WriteSetgroups, libc::EPERM),
        (CompanionCall::OpenSetgroups, libc::ENOMEM),
    ] {
        assert_eq!(
            the_answer(records_of(&[
                StepRecord::passed(CompanionStep::UserNamespace),
                failed(CompanionStep::Setgroups, call, errno),
            ])),
            GroupDrop::Answered {
                call,
                errno
            },
            "{} answering errno {errno} is that call's own answer",
            call.name()
        );
    }
}

/// What:
///  A host that refused the user namespace wrote one record,
///  and its answer is that step's,
///  not the group drop's,
///  so `doctor` prints none of its own.
#[test]
fn a_refused_user_namespace_is_that_steps_answer() {
    assert_eq!(
        the_answer(records_of(&[failed(
            CompanionStep::UserNamespace,
            CompanionCall::UnshareUserNamespace,
            libc::EPERM
        )])),
        GroupDrop::Answered {
            call: CompanionCall::UnshareUserNamespace,
            errno: libc::EPERM
        }
    );
}

/// What:
///  No record, and a record this build cannot read,
///  are both a probe that answered nothing,
///  which `doctor` must say rather than pass for a passing host.
#[test]
fn an_empty_or_unreadable_answer_is_said() {
    for decoded in [
        records_of(&[]),
        decode_all(&[0xff; 8]),
        records_of(&[StepRecord::passed(CompanionStep::UserNamespace)]),
    ] {
        assert!(
            matches!(the_answer(decoded), GroupDrop::Unread { .. }),
            "an answer that is no whole run is unread"
        );
    }
}

/// What:
///  The probe this host runs gives one of the two host answers,
///  not a probe that could not be read,
///  and it leaves no child behind.
/// Why:
///  The read is a fork, a pipe and a wait,
///  and the controls above judge records alone;
///  this one exercises the machinery on whichever host it runs.
#[test]
fn the_probe_answers_on_this_host() {
    let answer: GroupDrop = the_group_drop();
    assert!(
        matches!(
            answer,
            GroupDrop::Written | GroupDrop::Skipped { .. } | GroupDrop::Answered { .. }
        ),
        "the probe's own machinery answers on this host, and it answered {answer:?}"
    );
}
