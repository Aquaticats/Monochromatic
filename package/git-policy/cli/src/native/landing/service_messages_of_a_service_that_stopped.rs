//! What:
//!  The words for a commit whose service said nothing,
//! stopped reading its request,
//! or stopped answering it.
//! Why:
//!   These are the failures of the exchange with the commit service itself,
//! and each says the three things a person needs:
//!  whether the commit was made,
//! that their staged changes and working files are as they were,
//! and the steps that end a
//!  service that has stopped,
//! which are the same for all of them
//! (`service_messages.rs` keeps the failures about the repository
//!  itself).
//! They live in a file of their own because the rendering of every failure is at its file's line bound,
//! as the two ends
//!  of an interrupted commit are (`service_messages_of_an_interrupted_commit.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! render({ kind: 'service-not-reading', tookNothing: true })
//! ```

use crate::service_messages::{
    CHECK_COMMANDS, NOTHING_CHANGED, NOTHING_CHANGED_AND_THE_STAGED_WORK, RESTART_ONE_UNIT_COMMAND, STATUS_COMMAND, WhatTheBranchShows,
};
use crate::service_protocol::HEARTBEAT_SILENCE;
use crate::wait_ceilings::{REQUEST_WRITE_CEILING, REQUEST_WRITE_SILENCE};
use std::time::Duration;

/// What a person reads about a commit whose service stopped reading its request,
/// which is the one case where the request never reached the service as one line.
pub(crate) fn not_reading(took_nothing: bool) -> String {
    let what_happened: String = if took_nothing {
        format!(
            "For {} seconds the commit service of this worktree took no part of this commit's request, though a service that is \
             working takes it at once.",
            whole_seconds(REQUEST_WRITE_SILENCE)
        )
    } else {
        format!(
            "The commit service of this worktree did not take the whole of this commit's request within {} seconds, though it was \
             reading it.",
            whole_seconds(REQUEST_WRITE_CEILING)
        )
    };
    return format!(
        "The commit service of this worktree stopped reading, so this commit was not made. {what_happened} The request never \
         reached the service as one line, so nothing of it can land later by itself. {NOTHING_CHANGED_AND_THE_STAGED_WORK} \
         {NOTHING_CHANGED}\n{}",
        steps_for_a_commit_that_was_not_made()
    );
}

/// What a person reads about a commit whose service answered nothing at its address,
/// which is the case where the request was never sent at all.
pub(crate) fn nothing_answered(address: &str, waited: Duration) -> String {
    return format!(
        "Nothing has answered at the commit service's address for this worktree ({address}) for {} seconds, so this commit \
         was not made: the request was never sent, and nothing of it can land later by itself. \
         {NOTHING_CHANGED_AND_THE_STAGED_WORK} {NOTHING_CHANGED}\n{}",
        whole_seconds(waited),
        steps_for_a_commit_that_was_not_made()
    );
}

/// What:
///  What a person reads about a commit whose service is alive and has not greeted yet.
/// Why:
///   The service of this worktree was recovering the batches an earlier service left open,
/// and the command waited for it rather than starting a second one;
/// the person is told which process that is,
/// at which address,
/// and what the wait was,
/// because the next step is to let it finish rather than to start anything.
/// Nothing of the request was sent,
/// so nothing of it can land later by itself,
/// and the staged work is as they left it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// serviceIsPreparing('/run/user/1000/x.sock', 4711, 60) // the sentence and the steps
/// ```
pub(crate) fn service_is_preparing(address: &str, pid: i32, waited: Duration) -> String {
    return format!(
        "The commit service of this worktree is alive at {address} (process {pid}) and has no greeting yet: it is still preparing \
         the batches an earlier service left open, and it has not finished in {} seconds, the longest a command waits for it. This \
         command started no second service, because one service of this worktree is already there. This commit was not made, and \
         nothing of its request was sent. {NOTHING_CHANGED_AND_THE_STAGED_WORK} {NOTHING_CHANGED}\n{}",
        whole_seconds(waited),
        steps_for_a_commit_that_was_not_made()
    );
}

/// What a person reads about a commit whose service answered nothing after the request was sent,
/// where `shows` is what the branch showed when the command looked afterwards.
/// It is what tells a commit that was made from one that was not.
pub(crate) fn silent_after_preparing(shows: WhatTheBranchShows) -> String {
    let what_it_said: String = format!(
        "The commit service of this worktree stopped answering: it said nothing for {} seconds, the longest a service that is \
         working stays silent.",
        whole_seconds(HEARTBEAT_SILENCE)
    );
    return match shows {
        WhatTheBranchShows::Made => format!(
            "{what_it_said} The commit was made: the branch is at the commit this command prepared, and your staged changes are \
             the ones it took. There is nothing to run."
        ),
        WhatTheBranchShows::NotMade => format!(
            "{what_it_said} The branch is not at the commit this command prepared, so this commit was not made by this command. \
             The whole request did reach the service, so a service that wakes after this command ended may still land it: run \
             `{}` to see whether it has, and `{}` to see what is still staged, before running the commit again. \
             {NOTHING_CHANGED_AND_THE_STAGED_WORK}\n{}",
            CHECK_COMMANDS[0],
            CHECK_COMMANDS[1],
            steps_for_a_result_that_is_not_known()
        ),
        WhatTheBranchShows::Unread => format!(
            "{what_it_said} The branch could not be read afterwards, so whether the commit was made is not known: run \
             `{}` to see, and `{}` to see what is still staged. {NOTHING_CHANGED_AND_THE_STAGED_WORK}\n{}",
            CHECK_COMMANDS[0],
            CHECK_COMMANDS[1],
            steps_for_a_result_that_is_not_known()
        ),
    };
}

/// What a person reads about a commit whose service stopped answering before the branch was read,
/// which the commit path turns into one of the three above once it has looked at the branch.
pub(crate) fn silent() -> String {
    return format!(
        "The commit service of this worktree stopped answering: it said nothing for {} seconds, the longest a service that is \
         working stays silent. Whether this commit was made is not known until the branch is read: `{}` shows it, and `{}` shows \
         what is still staged. {NOTHING_CHANGED_AND_THE_STAGED_WORK}\n{}",
        whole_seconds(HEARTBEAT_SILENCE),
        CHECK_COMMANDS[0],
        CHECK_COMMANDS[1],
        steps_for_a_result_that_is_not_known()
    );
}

/// What:
///  What to do about a service that stopped,
///  for a commit of which the branch shows that it was not made.
/// Why:
///  The steps are what ends it,
/// in the order that ends it,
/// and each is a command this wrapper and this machine have:
/// the
///  status names the unit of this worktree's service,
/// the restart names that one unit,
/// and the commit is safe to run again because
///  the request of the failed one cannot land.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stepsForACommitThatWasNotMade() // the three steps, and why running again makes one commit
/// ```
fn steps_for_a_commit_that_was_not_made() -> String {
    return format!(
        "What to do: the commit service is the background program that lands the commits of this worktree. \
         1. `{STATUS_COMMAND}` says whether that service answers, and names the unit that serves it. \
         2. A service that is running and answers nothing has stopped inside itself: it was frozen, a debugger holds it, or it \
         waits for a disk that never answers. Starting it again does not help, because the service manager believes it runs; \
         restart that one unit with `{RESTART_ONE_UNIT_COMMAND}` and the unit name the status printed. What it was doing is in \
         its log: `journalctl --user --unit` and the same unit name, from the start of this commit. \
         3. Run the same commit again, which is safe: this command's request never reached the service as one whole line, \
         because a service reads a request only when its framing newline arrived, so the commit is made once and never \
         twice."
    );
}

/// What:
///  What to do about a service that stopped,
///  for a commit of which nobody knows yet whether it was made.
/// Why:
///  The steps are the same,
/// and the last one waits until `git log -1` has shown that the commit was not made,
/// because that
///  is the one fact the command could not read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stepsForAResultThatIsNotKnown() // the three steps, and why a commit is never made twice
/// ```
fn steps_for_a_result_that_is_not_known() -> String {
    return format!(
        "What to do: the commit service is the background program that lands the commits of this worktree. \
         1. `{STATUS_COMMAND}` says whether that service answers, and names the unit that serves it. \
         2. A service that is running and answers nothing has stopped inside itself: it was frozen, a debugger holds it, or it \
         waits for a disk that never answers. Starting it again does not help, because the service manager believes it runs; \
         restart that one unit with `{RESTART_ONE_UNIT_COMMAND}` and the unit name the status printed. What it was doing is in \
         its log: `journalctl --user --unit` and the same unit name, from the start of this commit. \
         3. Run the commit again once `{CHECK}` has shown that it was not made. A commit is never made twice, so running it \
         again makes the one commit.",
        CHECK = CHECK_COMMANDS[0]
    );
}

/// A span as the whole seconds a person counts in,
/// where a part of a second is one second waited.
fn whole_seconds(span: Duration) -> u64 {
    return span.as_secs().max(1);
}
