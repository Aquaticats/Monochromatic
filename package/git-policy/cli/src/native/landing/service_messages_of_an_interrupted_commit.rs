//! What:
//!  The words for a commit the person interrupted,
//!  both when the service confirmed that it gave the request up
//!  and when it did not.
//! Why:
//!  A commit the person interrupts is not made,
//!  and native Git says nothing at all when its command is interrupted:
//!  the wrapper says what became of the commit,
//!  names what it waited for,
//!  and gives the two commands that show whether anything landed.
//!  The other failures of a commit through the service are worded in
//!  `service_messages.rs`,
//!  which asks this file for these two.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! interruptedWords(failure) // string | undefined
//! ```

use crate::service_messages::{CHECK_COMMANDS, LOG_COMMAND, NOTHING_CHANGED, RESTART_COMMAND, it_had_said, waiting_for};

/// What:
///  The words for a commit the person interrupted,
///  which the service confirmed will never land.
/// Why:
///  A commit that was given up is not made,
///  and the words say so,
///  name what the commit waited for,
///  and give the one remedy:
///  run the commit again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `This commit was interrupted with signal ${signal} before it landed, so it was not made, ...`
/// ```
pub(crate) fn interrupted(signal: i32, blocker: Option<&str>) -> String {
    return format!(
        "This commit was interrupted with signal {signal} before it landed, so it was not made, and nothing lands later by itself.{} \
         Your staged changes and your worktree are as they were. Run the commit again. {NOTHING_CHANGED}",
        waiting_for(blocker)
    );
}

/// What:
///  The words for a commit the person interrupted
///  whose withdrawal the service did not confirm.
/// Why:
///  Nothing was learned about the request,
///  so whether the commit was made is not known,
///  and the person is given the two commands that tell.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `This commit was interrupted with signal ${signal}, and the commit service did not confirm ...`
/// ```
pub(crate) fn not_known(signal: i32, blocker: Option<&str>) -> String {
    return format!(
        "This commit was interrupted with signal {signal}, and the commit service did not confirm that it gave the commit up, so \
         it is not known whether the commit was made.{} Run `{}` to see whether it was made, and `{}` to see what is still staged. \
         If it was not made, run the commit again; if the service is stuck, look at why with `{LOG_COMMAND}` and restart it with \
         `{RESTART_COMMAND}`.",
        it_had_said(blocker),
        CHECK_COMMANDS[0], CHECK_COMMANDS[1]
    );
}

/// The words of an interrupt are controlled in the test module.
#[cfg(test)]
#[path = "service_messages_of_an_interrupted_commit_tests.rs"]
mod tests;
