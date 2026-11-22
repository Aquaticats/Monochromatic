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

#[cfg(test)]
thread_local! {
    /// How many `init` starts this thread's shadow creations have made,
    /// counted by the controls only.
    /// It is per thread,
    /// so a control that counts the attempts of its own creation
    ///  is not disturbed by another test's creation.
    static INIT_ATTEMPTS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
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
    }
    #[cfg(not(test))]
    let _ = (request, attempt);
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
