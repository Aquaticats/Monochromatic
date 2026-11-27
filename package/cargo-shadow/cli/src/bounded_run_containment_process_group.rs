//! What:
//! The stop of a bare process group,
//! the weaker path,
//! and the words its refusal carries.
//! Why:
//! The weaker path can show only that no member of the group is left
//! and that the program it led was reaped,
//! so each of those two facts gets its own sentence (issue 6528's review,
//! Low 2).
//! Split out of `bounded_run_containment.rs`,
//! which keeps the stop of a scope,
//! to keep that file under the code-line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // bounded-run-containment-process-group.ts
//! export type GroupStop = 'gone' | 'members-left' | { not-reaped: string } | 'unnamed'
//! export function stopProcessGroup(child: ChildProcess, group: number): GroupStop
//! ```

/// What:
/// `use super::reap_of_the_child::{ReapShortfall, Reaped, reap};` imports the wait for the direct child and its answers.
/// Why:
/// The process group's members are one question and the child's reaping is another,
/// so the refusal says which of them it could not show.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ReapShortfall, Reaped, reap } from './bounded_run_containment_reap_of_the_child.ts';
/// ```
use super::reap_of_the_child::{
    ReapShortfall,
    Reaped,
    reap,
};
/// What:
/// `use crate::bounded_run::{POLL_INTERVAL, QUIESCE_BOUND, REAP_BOUND};` imports the three waits the two sentences name.
/// Why:
/// The group's loop asks at one interval and ends at one bound,
/// and the reaping has a bound of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { POLL_INTERVAL, QUIESCE_BOUND, REAP_BOUND } from 'crate/bounded_run';
/// ```
use crate::bounded_run::{
    POLL_INTERVAL,
    QUIESCE_BOUND,
    REAP_BOUND,
};
/// What:
/// `use std::process::Child;` imports the handle of the started child.
/// Why:
/// The stop kills the group and then reaps that one child through its handle.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ChildProcess } from 'node:child_process';
/// ```
use std::process::Child;
/// What:
/// `use std::thread;` imports the thread sleep between two looks at the group.
/// Why:
/// The loop waits one interval between signal and test,
/// so a group that ends at once is not looked at in a tight loop.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // await new Promise((resolve) => setTimeout(resolve, ms))
/// ```
use std::thread;
/// What:
/// `use std::time::Instant;` imports a point on the clock (its sibling `Duration` is a span of time).
/// Why:
/// The loop computes its deadline from the moment it started.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Date.now()
/// ```
use std::time::Instant;

/// What:
/// `use rustix::io::Errno;` imports the error code that says no member of the group is left.
/// Why:
/// `ESRCH` is the one answer that shows the group gone,
/// and every other answer keeps the loop going.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Errno } from 'rustix/io';
/// ```
use rustix::io::Errno;
/// What:
/// `use rustix::process::{Pid, Signal, kill_process_group, test_kill_process_group};` imports the group id,
/// the signal and the
/// two calls:
/// one sends it and one asks whether the group exists.
/// Why:
/// This is all the weaker path can do,
/// so both calls are named here rather than reached through a helper.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // process.kill(-pid, 'SIGKILL') and process.kill(-pid, 0)
/// ```
use rustix::process::{
    Pid,
    Signal,
    kill_process_group,
    test_kill_process_group,
};

/// What the weaker path's stop found when it gave up.
pub(crate) enum GroupStop {
    /// No member of the process group is left.
    Gone,
    /// Members of the process group were still there when its bound passed.
    MembersLeft,
    /// No member of the process group is left,
    /// and the direct child could not be reaped,
    /// with the reason.
    NotReaped(
        /// Why the direct child was not reaped.
        ReapShortfall,
    ),
    /// The program's process id could not be named as a process group,
    /// so nothing of it was stopped.
    Unnamed,
}

/// What:
/// `impl GroupStop` answers whether the stop settled the run,
/// and the words a refusal carries when it did not.
/// Why:
/// Both process-group paths (a run's own end and the start that failed to record the group)
/// need one sentence each,
/// and the sentence must state the wait that was waited or none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // settled(): boolean; refusal(): string
/// ```
impl GroupStop {
    /// What:
    /// `pub(crate) fn settled(&self) -> bool` says whether every member of the group is shown gone.
    /// Why:
    /// Only `Gone` lets the run be treated as quiescent.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// settled(): boolean { return this === 'gone' }
    /// ```
    pub(crate) fn settled(&self) -> bool {
        return matches!(self, GroupStop::Gone);
    }

    /// What:
    /// `pub(crate) fn refusal(&self) -> String` is why this stop could not settle the run.
    /// Why:
    /// A group already shown gone must not be worded as a bound that was waited,
    /// and an unnamed group must not be worded as a wait that never started (issue 6528's review,
    /// Low 2).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// refusal(): string
    /// ```
    pub(crate) fn refusal(&self) -> String {
        return match self {
            GroupStop::Gone => "the process group is gone".to_string(),
            GroupStop::MembersLeft => format!(
                "the members of the process group could not be shown gone within {QUIESCE_BOUND:?}"
            ),
            GroupStop::NotReaped(ReapShortfall::Bound) => format!(
                "no member of the process group is left, and the program it led did not end within {REAP_BOUND:?} after it was killed"
            ),
            GroupStop::NotReaped(ReapShortfall::Unreadable(reason)) => format!(
                "no member of the process group is left, and the exit of the program it led could not be read ({reason})"
            ),
            GroupStop::Unnamed => {
                "the process group of the program could not be named, so nothing of it was stopped"
                    .to_string()
            }
        };
    }
}

/// What:
/// `fn stop_process_group(child: &mut Child, group: Pid) -> GroupStop` kills the process group,
/// reaps the direct child,
/// and says whether no member of the group is left.
/// Why:
/// This is all the weaker path can show,
/// and the two questions it answers (the group's members and the direct child) are what a refusal has to tell apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stopProcessGroup(child: ChildProcess, group: number): GroupStop
/// ```
pub(crate) fn stop_process_group(
    child: &mut Child,
    group: Pid,
) -> GroupStop {
    let reaped: Reaped = {
        let _ = kill_process_group(group, Signal::KILL);
        reap(child)
    };
    let deadline: Instant = Instant::now() + QUIESCE_BOUND;
    loop {
        if test_kill_process_group(group) == Err(Errno::SRCH) {
            return match reaped {
                Reaped::Ended => GroupStop::Gone,
                Reaped::NotReaped(shortfall) => GroupStop::NotReaped(shortfall),
            };
        }
        if Instant::now() >= deadline {
            return GroupStop::MembersLeft;
        }
        let _ = kill_process_group(group, Signal::KILL);
        thread::sleep(POLL_INTERVAL);
    }
}
