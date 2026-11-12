//! What:
//!  The second measurement of an abort or a skip,
//!  made immediately before the wrapper hands the command to real Git.
//! Why:
//!  The first measurement happens when the policy decides,
//!  and Git changes files later,
//!  so a file can be edited in between and go with the abort unseen.
//!  Nothing in the wrapper can close that window:
//!  only the process that changes files can decide against the state it is about to change.
//!  What the wrapper can do is ask the question again as late as it controls,
//!  and refuse there when the answer changed:
//!  one round is all a refusal needs,
//!  because any difference ends the command,
//!  so a second round could only find the same difference again.
//!  `SPEC.md` ("The window between the last measurement and Git") states the window that remains.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! planTheSecondLook({ context, command, identity, seen });
//! // immediately before Git starts:
//! const refusal = recheckBeforeTheCommand(); // a line for stderr, or nothing
//! ```

use crate::discard_audit::{AuditOutcome, record_audit};
use crate::repository_facts::git_facts;
use crate::rule_linked_worktree::GuardedCommand;
use crate::transaction_git::GitContext;
use crate::worktree_identity::WorktreeIdentity;
use std::ffi::OsString;
use std::sync::Mutex;

/// What:
///  Everything the second measurement needs to ask the same question again and to record its answer.
/// Why:
///   The measurement happens where the policy decides,
///  and the command starts several layers below it;
///  carrying these four values keeps every layer between them from having to know about aborts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AbortRecheckPlan = { context: GitContext; command: GuardedCommand; identity: WorktreeIdentity; seen: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AbortRecheckPlan {
    /// The real Git, its global options and its overlay,
    ///  which the same question is asked with.
    pub context: GitContext,
    /// The abort or skip form the person's command is.
    pub command: GuardedCommand,
    /// The repository identity the audit record is written under.
    pub identity: WorktreeIdentity,
    /// The argument list Git will run,
    ///  for the audit record.
    pub seen: Vec<OsString>,
}

/// What:
///  The measurement one invocation still owes,
///  which is at most one:
///  one command runs once.
/// Why:
///   The second look has to happen where the wrapper starts the command the person typed,
///  and taking the plan out means a start the system refused and the wrapper repeats measures once.
static PENDING: Mutex<Option<AbortRecheckPlan>> = Mutex::new(None);

/// What:
///  Record the second measurement this command owes before Git starts.
/// Why:
///   The policy pass has no way to run code after it returns,
///  so the plan waits here for the start that hands the command to Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function planTheSecondLook(plan: AbortRecheckPlan): void;
/// ```
pub fn plan_the_second_look(plan: AbortRecheckPlan) {
    let mut pending: std::sync::MutexGuard<'_, Option<AbortRecheckPlan>> = PENDING.lock().unwrap_or_else(|poisoned| { return poisoned.into_inner(); });
    *pending = Some(plan);
}

/// What:
///  Take the pending plan, if there is one.
/// Why:
///   Only the first start of the person's command may measure:
///  a command the wrapper starts again reads the same repository and would ask twice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const plan = takeTheSecondLook();
/// ```
fn take_the_second_look() -> Option<AbortRecheckPlan> {
    let mut pending: std::sync::MutexGuard<'_, Option<AbortRecheckPlan>> = PENDING.lock().unwrap_or_else(|poisoned| { return poisoned.into_inner(); });
    return pending.take();
}

/// What:
///  The line that says when the work the refusal names appeared.
/// Why:
///   A person who read the wrapper's decision a moment ago has to know the state moved since,
///  or the refusal reads as if the first measurement had missed something it could not have seen.
///  It stands on its own line after the refusal, because the refusal's own last sentence is its advice
///  and a clause inside a sentence would separate advice from the reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const LATE = 'This work appeared between the wrapper\'s measurement and Git\'s start.';
/// ```
pub const LATE_WORK: &str = "This work appeared between the wrapper's measurement and Git's start.";

/// What:
///  Ask again what the command would discard,
///  immediately before Git starts,
///  and refuse it when the answer holds work the first measurement did not see.
/// Why:
///   This is the last moment the wrapper controls before Git discards the files,
///  so it is where an edit that landed after the policy pass is caught.
///  A refusal here names the work and the two ways forward,
///  and the record says `rejected`,
///  because the command did not run.
///  A plan that is not there means no abort or skip decided to run, and nothing is done.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusal = recheckBeforeTheCommand(); // one line for stderr, or nothing
/// ```
pub fn recheck_before_the_command() -> Option<String> {
    let Some(plan) = take_the_second_look() else {
        return None;
    };
    let mut facts = git_facts(
        plan.context.real_git.as_path(),
        plan.context.global_prefix.as_slice(),
        plan.context.overlay.as_slice(),
    );
    let crate::abort_discard_guard::AbortVerdict::Refused(text) = crate::abort_discard_guard::judge_abort_discard(&mut facts, plan.command) else {
        return None;
    };
    record_audit(&plan.identity, plan.command, AuditOutcome::Rejected, plan.seen.as_slice(), None);
    return Some(format!("{text}\n  {LATE_WORK}\n"));
}

/// The second measurement's own record.
#[cfg(test)]
#[path = "abort_recheck_tests.rs"]
mod tests;
