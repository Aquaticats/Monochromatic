//! What:
//!  What a shadow creation records for its controls as it starts one `init` attempt:
//!  how many attempts this thread's creations have made,
//!  and which attempt the start it is about to make is,
//!  in test builds alone.
//! Why:
//!  A control that counts the starts of the shadow's `init` needs a count the wrapper makes itself:
//!  the program a start runs may be ended at its ceiling before it writes anything.
//!  The same holds for a control's Git stand-in,
//!  which has to know which of its own starts it is hanging:
//!  a file the cut start writes cannot tell it,
//!  because a start ended at its ceiling may never write.
//!  The record is per thread,
//!  so a control counts the creation it makes and no other test's
//!  (`doc/troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md`,
//!  cause M).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // noteInitAttempt(request, attempt) at each attempt; initAttempts() and resetInitAttempts() in the controls.
//! ```

/// The request a start is made from,
/// whose environment the attempt number is added to in test builds.
use crate::transaction_git::GitRequest;
/// `OsString` is owned operating-system text of raw bytes,
/// for the attempt number a control's stand-in reads.
#[cfg(test)]
use std::ffi::OsString;
/// A span of time,
/// for the ceiling a control plants a cut at.
#[cfg(test)]
use std::time::Duration;

#[cfg(test)]
thread_local! {
    /// How many `init` starts this thread's shadow creations have made,
    /// counted by the controls only.
    /// It is per thread,
    /// so a control that counts the attempts of its own creation
    ///  is not disturbed by another test's creation.
    static INIT_ATTEMPTS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };

    /// How long a control ends the first `init` start of this thread's next creation at.
    static CUT_AT: std::cell::Cell<Option<Duration>> = const { std::cell::Cell::new(None) };
}

/// What:
///  The variable a control's Git stand-in reads to know which attempt it is.
/// Why:
///   A stand-in that decides from a file the cut start writes gets no answer when the ceiling ends that start
///  before it writes,
///  so the second start then reads the file's absence as the first attempt and hangs as well.
///  The number the wrapper hands the start says which attempt it is before the program runs at all.
///  It exists in test builds alone:
///  `note_init_attempt` adds it to a request only there,
///  and no shipped build sets or reads it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const INIT_ATTEMPT_VARIABLE = 'MONOCHROMATIC_TEST_SHADOW_INIT_ATTEMPT';
/// ```
#[cfg(test)]
pub(crate) const INIT_ATTEMPT_VARIABLE: &str = "MONOCHROMATIC_TEST_SHADOW_INIT_ATTEMPT";

/// What:
///  One attempt at the shadow's `init`,
/// recorded for the controls only.
/// Why:
///   The count a control reads from here is the attempt the wrapper made,
/// which exists whether or not the program it starts writes anything
///  before the ceiling ends it.
///  A file the started program writes cannot promise that:
/// the program is ended at the ceiling wherever it has got to.
///  The attempt number in the request's environment is what a control's stand-in reads,
/// and no shipped build carries it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// noteInitAttempt(request, attempt)
/// ```
pub(crate) fn note_init_attempt(request: &mut GitRequest, attempt: u32) {
    #[cfg(test)]
    {
        INIT_ATTEMPTS.with(|counter| {
            counter.set(counter.get() + 1);
        });
        // A later attempt replaces the number an earlier one left, so the request the start is made from holds one value.
        request.environment.retain(|(name, _)| {
            return name.as_os_str() != INIT_ATTEMPT_VARIABLE;
        });
        request.environment.push((OsString::from(INIT_ATTEMPT_VARIABLE), OsString::from(attempt.to_string())));
        apply_the_planted_cut(attempt);
    }
    #[cfg(not(test))]
    let _ = (request, attempt);
}

/// What:
///  Ends the first `init` start of this thread's next creation at `ceiling`,
/// and leaves the start made after it to the ceiling production gives it.
/// Why:
///   A control holds that a start cut at its ceiling is made again in a fresh directory,
/// and that the start made again is what makes the shadow.
///  A shortened ceiling over that start fails the control whenever the program is slower than the ceiling,
///  which is a fact about the host and never about what the control holds:
///  a control may depend on what the product guarantees,
///  never on how fast a program it started reaches a line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// cutTheFirstInitAttemptAt(700)
/// ```
#[cfg(test)]
pub(crate) fn cut_the_first_init_attempt_at(ceiling: Duration) {
    CUT_AT.with(|cell| {
        cell.set(Some(ceiling));
    });
}

/// What:
///  Forgets a planted cut,
/// and gives this thread's steps back the ceiling of their class.
/// Why:
///   A plant that outlives its control would shorten another test's start on the same thread,
///  so a control clears it once its creation returned.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// clearThePlantedCut()
/// ```
#[cfg(test)]
pub(crate) fn clear_the_planted_cut() {
    CUT_AT.with(|cell| {
        cell.set(None);
    });
    crate::transaction_git::STEP_CEILING_FOR_THIS_THREAD.with(|cell| {
        cell.set(None);
    });
}

/// What:
///  Shortens the ceiling of the attempt a control planted its cut in,
/// and gives the ceiling back before the next attempt.
/// Why:
///   The ceiling is read when a step is watched,
///  so it has to be set before this start and cleared before the start made after it;
///  the plant covers the first attempt alone,
///  because that is the attempt the control means to cut.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// applyPlantedCut(attempt) // attempt === 1 ? stepCeiling.set(planted) : stepCeiling.set(undefined)
/// ```
#[cfg(test)]
fn apply_the_planted_cut(attempt: u32) {
    let Some(ceiling) = CUT_AT.with(std::cell::Cell::get) else {
        return;
    };
    if attempt == 1 {
        crate::transaction_git::STEP_CEILING_FOR_THIS_THREAD.with(|cell| {
            cell.set(Some(ceiling));
        });
        return;
    }
    crate::transaction_git::STEP_CEILING_FOR_THIS_THREAD.with(|cell| {
        cell.set(None);
    });
    CUT_AT.with(|cell| {
        cell.set(None);
    });
}

/// What:
///  How many `init` attempts this thread's shadow creations have made since they were last reset.
/// Why:
///   A control reads it to count the attempts the wrapper made,
/// and never what a program the wrapper started got to write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// initAttempts()
/// ```
#[cfg(test)]
pub(crate) fn init_attempts() -> u32 {
    return INIT_ATTEMPTS.with(std::cell::Cell::get);
}

/// What:
///  Forgets the `init` attempts made so far on this thread.
/// Why:
///   A control starts counting at the creation it makes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// resetInitAttempts()
/// ```
#[cfg(test)]
pub(crate) fn reset_init_attempts() {
    INIT_ATTEMPTS.with(|counter| {
        counter.set(0);
    });
}
