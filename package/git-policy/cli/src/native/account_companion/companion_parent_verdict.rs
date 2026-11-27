//! What:
//!  What one ended companion run means for the add:
//!  a signal death,
//!  a status line that could not be read,
//!  a step that failed and whether the host's refusal is on the skip's list,
//!  a run that wrote fewer records than the child has steps,
//!  and the step that was in flight when it ended.
//! Why:
//!  Each ending has its own words,
//!  and a host refusal is told apart from a failure by the call and the errno
//!  the design names,
//!  so the deciding lives apart from the run that produced the records
//!  (`crate::account_companion::companion_parent_run`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! return theVerdict(decoded, ended, settings); // passed, skipped with its line, or refused with its reason
//! ```

use super::companion_calls::CompanionCall;
use super::companion_calls::{
    Failure,
    failed,
};
use super::companion_lines::{
    CompanionVerdict,
    Refusal,
    Skip,
};
use super::companion_process_calls::Exit;
use super::companion_records::Decoded;
use super::companion_settings::{
    self,
    Settings,
};
use super::companion_steps::{
    CompanionStep,
    STEP_COUNT,
    STEPS_IN_ORDER,
    is_skip,
};

/// What:
///  The verdict of one ended run.
/// Why:
///  A signal death,
/// a missing record and a failed step each have their own
///  words,
///  and a host refusal on the skip's list is told apart from a failure by the
///  call and the errno the design names.
pub(super) fn the_verdict(
    decoded: Decoded,
    ended: Exit,
    settings: &Settings,
) -> CompanionVerdict {
    let step_in_flight: CompanionStep = step_after(decoded.records.len());
    if let Exit::Signaled {
        signal,
    } = ended
    {
        return CompanionVerdict::Refused {
            refusal: Refusal::ChildSignaled {
                step_in_flight,
                signal,
            },
        };
    }
    if decoded.unreadable {
        return CompanionVerdict::Refused {
            refusal: Refusal::NoStatusLine {
                step_in_flight,
            },
        };
    }
    if let Some(failed_record) = decoded
        .records
        .iter()
        .find(|record| return record.failure.is_some())
    {
        let failure: Failure = failed_record
            .failure
            .unwrap_or_else(|| return failed(CompanionCall::WriteRecord));
        if is_skip(
            failed_record.step,
            failure.call,
            failure.errno,
            companion_settings::user_namespace_entry_exists(),
        ) {
            return CompanionVerdict::Skipped {
                skip: Skip {
                    call: failure.call,
                    errno: failure.errno,
                    settings: settings.clone(),
                },
            };
        }
        return CompanionVerdict::Refused {
            refusal: Refusal::ArmFailed {
                step: failed_record.step,
                failure,
            },
        };
    }
    if decoded.records.len() != STEP_COUNT {
        return CompanionVerdict::Refused {
            refusal: Refusal::NoStatusLine {
                step_in_flight,
            },
        };
    }
    return CompanionVerdict::Passed;
}

/// The step after the last whole record,
/// which is the step that was in flight.
fn step_after(records: usize) -> CompanionStep {
    return STEPS_IN_ORDER
        .get(records)
        .copied()
        .unwrap_or(CompanionStep::MoveUnmount);
}
