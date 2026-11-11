//! What:
//!  The plan of the pushes waiting for a new try and the repositories an earlier service was woken for,
//! read from this instance's journal on a thread of its own,
//! and nothing learned when that thread gives no answer within its ceiling.
//! Why:
//!  Both are storage:
//! the plan is one file per waiting branch and the watch one file per repository,
//! and a filesystem that
//!  stopped answering holds a read of either in the kernel with no end.
//! The landing thread reads neither itself,
//! so a journal that is slow delays
//!  the after-commit work and never the greeting of a client or the answer to a commit.
//! A read that gives no answer loses no commit:
//! the plan only repeats a push the person's failure record already names,
//! and the
//!  watch only saves a wake.
//! It may not lose the journal itself:
//! the plan's directory and the watch's directory are kept,
//! so a retry planned after the timeout is still kept for a restart,
//! and a read that gave no answer is made again rather than given up on,
//! because a plan nobody read is a push nobody tries
//!  (`PushRetries::needs_restore`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const [retries, watch] = readTheJournalWithin(ceiling, planDirectory, watchDirectory, instance);
//! ```

use crate::push_retry::PushRetries;
use crate::service_claim_watch::ClaimWatch;
use std::path::PathBuf;
use std::time::Duration;

/// What:
///  `plan_directory`'s plan and `watch_directory`'s notes,
/// read on a thread of their own and awaited no longer than `ceiling`,
/// and a service that starts without them when that thread gives no answer.
/// Why:
///   See the module description.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const read = workWithin(ceiling, () => [PushRetries.restored(planDirectory), ClaimWatch.restored(watchDirectory, instance)]);
/// const [retries, watch] = read ?? [PushRetries.unreadFromEnvironment(planDirectory), ClaimWatch.unread(watchDirectory, instance)];
/// ```
pub fn restored_within(ceiling: Duration, plan_directory: PathBuf, watch_directory: PathBuf, instance: Option<String>) -> (PushRetries, ClaimWatch) {
    // The paths this call was given, kept for the answer a read that gives no answer leaves: the service keeps them either way.
    let asked: (PathBuf, PathBuf, Option<String>) = (plan_directory.clone(), watch_directory.clone(), instance.clone());
    let restored = crate::work_awaited_within_a_ceiling::work_within(ceiling, move || {
        let retries: PushRetries = PushRetries::restored_from_environment(plan_directory.as_path());
        let watch: ClaimWatch = ClaimWatch::restored(watch_directory.as_path(), instance.as_deref());
        return (retries, watch);
    });
    return match restored {
        Some(pair) => pair,
        None => {
            crate::service_warn!(
                "the plan of the pushes waiting for a new try and the repositories an earlier service was woken for gave no answer within {ceiling:?}, so this service starts before them: their directories are kept, so a retry planned now is kept for a restart, and the read is made again after a pause and by the next service that starts; until then a push that was waiting for a new try is not tried again"
            );
            let (plan, watch, instance): (PathBuf, PathBuf, Option<String>) = asked;
            (PushRetries::unread_from_environment(plan.as_path()), ClaimWatch::unread(watch.as_path(), instance.as_deref()))
        }
    };
}

/// Controls for the journal read at a service's start stay out of the release executable.
#[cfg(test)]
#[path = "service_journal_restored_at_start_tests.rs"]
mod tests;
