//! What:
//!  The hook lock and the preparation lease around the hooks the wrapper runs or lets Git run for a commit
//!  transaction: whether a run may find a hook, the lock it holds, and the lease its descendants inherit.
//! Why:
//!  The wrapper never has Git start the wrapper again for a hook.
//!  Git runs the configured preparation hooks (`pre-commit`, `prepare-commit-msg`, `commit-msg`) itself inside the
//!  native commit, because only Git knows the message file protocol (the `prepare-commit-msg` source argument, the
//!  editor between the two message hooks, the cleanup mode); the wrapper holds the hook lock around that one child.
//!  The re-run of `pre-commit` after a replay and the post-landing `post-commit` are Git's `hook run`, started by the
//!  wrapper under the same lock.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await using lock = mayRunPreparationHooks(plan) ? await acquireOwnerLock(plan.hookLockPath) : undefined;
//! ```

/// Plan facts the decisions read.
use super::hook_dispatch_plan::{HookDispatchPlan, PREPARATION_HOOK_EVENTS};
/// Lock acquisition and release shared with the other owner locks.
use super::live_lock::acquire_live_owner_lock;
use super::owner_lock::OwnerLock;
/// Preparation lease variable name.
use super::transaction_lease::PREPARATION_LEASE_VARIABLE;
/// The name of the absent hooks directory, shared with the shadow commands.
use super::shadow_git::NO_HOOKS_DIRECTORY;
/// `OsString` is owned operating-system text of raw bytes.
use std::ffi::OsString;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// What:
///  Why the hook lock could not be taken or released, naming the lock.
/// Why:
///   The caller stops the step and prints the sentence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// throw new Error(`cannot take ${path}`);
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HookLockError(pub String);

/// What:
///  One sentence naming the lock that failed.
/// Why:
///   The caller prints it as the diagnostic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// error.message
/// ```
impl std::fmt::Display for HookLockError {
    /// Writes the stored sentence.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(formatter, "{}", self.0);
    }
}

/// What:
///  The hooks directory that is deliberately absent, beside the transaction's directory.
/// Why:
///   The post-landing `post-commit` run points `core.hooksPath` here, so Git finds no traditional hook (they are
///  unsupported).
///  The directory is never created, because Git treats a missing hooks directory as holding no hook.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// join(transactionDirectory, 'no-hooks')
/// ```
pub fn transaction_no_hooks_directory(transaction_directory: &Path) -> PathBuf {
    return transaction_directory.join(NO_HOOKS_DIRECTORY);
}

/// What:
///  Whether the user disabled `event` with `hook.<event>.enabled=false`.
/// Why:
///   Git skips a disabled event's configured hooks, so a run of it needs neither the lock nor a process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// plan.disabledEvents.includes(event)
/// ```
pub fn event_disabled(plan: &HookDispatchPlan, event: &str) -> bool {
    return plan.disabled_events.iter().any(|name| { return name == event; });
}

/// What:
///  Whether a hook for `event` can run: the configuration may define a hook by name and the event is not disabled.
/// Why:
///   Traditional hooks are unsupported, so a configuration that defines no hook by name has nothing to run, and the
///  answer comes from the configuration snapshot without a file system call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// plan.namedHooksPossible && !plan.disabledEvents.includes(event)
/// ```
pub fn event_may_run(plan: &HookDispatchPlan, event: &str) -> bool {
    return plan.named_hooks_possible && !event_disabled(plan, event);
}

/// What:
///  Whether the native commit can run any hook of the preparation events.
/// Why:
///   Only then does the commit hold the hook lock, so a commit with no hook takes no lock and adds no system call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// PREPARATION_HOOK_EVENTS.some((event) => mayRun(plan, event))
/// ```
pub fn preparation_hooks_may_run(plan: &HookDispatchPlan) -> bool {
    return PREPARATION_HOOK_EVENTS.iter().any(|event| { return event_may_run(plan, event); });
}

/// What:
///  Take the hook lock at `lock_path`, waiting silently while a live owner holds it.
/// Why:
///   The lock lives under the common Git directory's `cli-git` directory; the owner lock creates one level only, so
///  its parent is made first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await acquireOwnerLock({ lockDirectory: plan.hookLockPath });
/// ```
pub fn acquire_hook_lock(lock_path: &str) -> Result<OwnerLock, HookLockError> {
    let directory: &Path = Path::new(lock_path);
    if let Some(parent) = directory.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            return HookLockError(format!("creating {} failed: {error}", parent.display()));
        })?;
    }
    return acquire_live_owner_lock(directory, "the hook lock")
        .map_err(|error| { return HookLockError(error.to_string()); });
}

/// What:
///  Take the hook lock for a run of `events` when one of them may find a hook and the plan does not skip the lock.
/// Why:
///   The hook lock serializes the hooks of concurrent commit transactions in one repository;
///  `hooks.concurrentCommits` and an inherited preparation lease (a hook's own nested commit) skip it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// return plan.skipHookLock || !events.some((event) => mayRun(plan, event)) ? undefined : await acquireHookLock(plan.hookLockPath);
/// ```
pub fn hold_hook_lock(plan: &HookDispatchPlan, events: &[&str]) -> Result<Option<OwnerLock>, HookLockError> {
    if plan.skip_hook_lock || !events.iter().any(|event| { return event_may_run(plan, event); }) {
        return Ok(None);
    }
    return acquire_hook_lock(plan.hook_lock_path.as_str()).map(Some);
}

/// What:
///  The environment pair that carries the transaction's preparation lease.
/// Why:
///   A wrapper started by a hook (a nested `git add`, `git commit`, `git worktree add`) validates the lease and
///  proceeds under its ancestor instead of waiting for locks the ancestor holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ['CLI_GIT_PREPARATION_LEASE', plan.lease]
/// ```
pub fn lease_variable(plan: &HookDispatchPlan) -> (OsString, OsString) {
    return (OsString::from(PREPARATION_LEASE_VARIABLE), OsString::from(plan.lease.as_str()));
}

/// Lock and lease decisions are controlled in the test module.
#[cfg(test)]
#[path = "hook_run_tests.rs"]
mod tests;
