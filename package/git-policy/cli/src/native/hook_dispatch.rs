//! What:
//!  The hook dispatcher: a native entry started by a preparation hook name inside a transaction's
//!  `hooks` directory, which runs the repository's hooks for that event under the hook lock.
//! Why:
//!  Git starts the entry through `core.hooksPath`; the start name selects the event.
//!  The incumbent's `dispatch.mjs` (`src/hook-dispatch/hook-dispatch-program.ts`) does the same work,
//!  and both take the hook lock at `<common>/cli-git/hook.lock`, so they serialize on one lock.
//!  Any other start name, including `post-commit` or a hook name outside a transaction, behaves as `git`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await dispatch(event, process.argv.slice(2));
//! ```

/// Plan reading, the lock path and the plan's JSON spelling.
use super::hook_dispatch_plan::{
    ConfigParameters, CONFIG_PARAMETERS_VARIABLE, HookDispatchPlan, PLAN_FILENAME, PLAN_LIMIT,
    PREPARATION_HOOK_EVENTS, WORK_TREE_VARIABLE, parse_hook_dispatch_plan,
};
/// The hook entry naming rule on Windows.
use super::hook_dispatch_entries::{hook_entry_name, transaction_no_hooks_directory};
/// Lock acquisition and release shared with the other owner locks.
use super::live_lock::acquire_live_owner_lock;
use super::owner_lock::OwnerLock;
use super::start_retry::RetryStart;
/// Bounded no-follow reads of the plan.
use super::private_storage::read_regular_file;
/// Preparation lease variable name.
use super::transaction_lease::PREPARATION_LEASE_VARIABLE;
/// Registry naming of transaction directories.
use super::transaction_registry::{TRANSACTION_ROOT_NAME, is_transaction_id};
/// The wrapper's ordinary path when the start name is not a dispatch.
use super::entry::run_process;
/// `OsStr`/`OsString` are operating-system text; `Command` starts Git.
use std::ffi::{OsStr, OsString};
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};
/// Child process starting.
use std::process::Command;

/// What:
///  One recognized dispatch: the event the entry is named after and its transaction's hooks directory.
/// Why:
///   The dispatcher needs both, and the start name is the only proof that Git started this entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DispatchTarget = { event: CommitHookEvent; hooksDirectory: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DispatchTarget {
    /// Preparation event the entry is named after.
    pub event: &'static str,
    /// The transaction's `hooks` directory, holding `plan.json` beside the entry.
    pub hooks_directory: PathBuf,
}

/// What:
///  The event a start name names, if any.
/// Why:
///   Only the preparation events have entries; on Windows Git's `.exe` suffix is part of the lookup.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// PREPARATION_HOOK_EVENTS.find((event) => name === hookEntryName(event));
/// ```
fn event_of_name(name: &str) -> Option<&'static str> {
    for event in PREPARATION_HOOK_EVENTS {
        if name == hook_entry_name(event) {
            return Some(event);
        }
    }
    return None;
}

/// What:
///  Recognize a start name inside a transaction's `hooks` directory beside a `plan.json`.
/// Why:
///   The incumbent's dispatcher runs from `<transaction>/hooks`; recognizing any other name would
///  turn ordinary Git invocations into dispatches.
///  The name is read lexically, and the file system is touched only after every name matches.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // the dispatcher is the program written as <tx>/hooks/<event> beside <tx>/hooks/plan.json
/// ```
pub fn dispatch_target_for_start_name(start_name: &Path) -> Option<DispatchTarget> {
    let name: &str = start_name.file_name()?.to_str()?;
    let event: &'static str = event_of_name(name)?;
    let hooks_directory: &Path = start_name.parent()?;
    if hooks_directory.file_name()? != "hooks" {
        return None;
    }
    let transaction: &Path = hooks_directory.parent()?;
    if !is_transaction_id(transaction.file_name()?.to_str()?) {
        return None;
    }
    if transaction.parent()?.file_name()? != TRANSACTION_ROOT_NAME {
        return None;
    }
    if read_regular_file(hooks_directory.join(PLAN_FILENAME).as_path(), PLAN_LIMIT).is_err() {
        return None;
    }
    return Some(DispatchTarget {
        event,
        hooks_directory: hooks_directory.to_path_buf(),
    });
}

/// What:
///  The environment the dispatched Git sees: the caller's variables with the plan's three overrides.
/// Why:
///   The incumbent's dispatcher restores `GIT_CONFIG_PARAMETERS` (or removes it), exports the worktree root
///  and the preparation lease; nothing else changes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// environment.GIT_WORK_TREE = plan.worktreeRoot; environment.CLI_GIT_PREPARATION_LEASE = plan.lease;
/// ```
pub fn dispatch_environment(
    environment: &[(OsString, OsString)],
    plan: &HookDispatchPlan,
) -> Vec<(OsString, OsString)> {
    // `mut` lets the overrides replace the caller's values.
    let mut result: Vec<(OsString, OsString)> = Vec::with_capacity(environment.len() + 3);
    for (name, value) in environment {
        let overridden: bool = name == CONFIG_PARAMETERS_VARIABLE
            || name == WORK_TREE_VARIABLE
            || name == PREPARATION_LEASE_VARIABLE;
        if !overridden {
            result.push((name.clone(), value.clone()));
        }
    }
    if let ConfigParameters::Present(value) = &plan.config_parameters {
        result.push((OsString::from(CONFIG_PARAMETERS_VARIABLE), OsString::from(value)));
    }
    result.push((
        OsString::from(WORK_TREE_VARIABLE),
        OsString::from(plan.worktree_root.as_str()),
    ));
    result.push((
        OsString::from(PREPARATION_LEASE_VARIABLE),
        OsString::from(plan.lease.as_str()),
    ));
    return result;
}

/// What:
///  Print one dispatcher diagnostic and return exit status 1, as an uncaught error does in the incumbent.
/// Why:
///   Failures stop the hook with a message; the lock is released before this returns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// throw new Error('cli-git hook dispatcher: ' + message);
/// ```
fn dispatch_failure(message: &str) -> i32 {
    eprintln!("cli-git hook dispatcher: {message}");
    return 1;
}

/// What:
///  Run the real Git hook runner for one event under the plan's lock, and return its status.
/// Why:
///   The incumbent's `dispatch`:
///  a disabled event exits 0 untouched, the lock is taken unless the plan skips it,
///  Git runs `hook run --ignore-missing <event>` with the repository's hooks path,
///  a signal or a failure to report a status is exit 1, and the lock is released afterwards.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function dispatch(event, args): Promise<void>;
/// ```
pub fn run_hook_dispatch(
    target: &DispatchTarget,
    arguments: &[OsString],
    environment: &[(OsString, OsString)],
) -> i32 {
    let plan_path: PathBuf = target.hooks_directory.join(PLAN_FILENAME);
    let Ok(bytes) = read_regular_file(plan_path.as_path(), PLAN_LIMIT) else {
        return dispatch_failure(
            format!("cannot read the plan {}", plan_path.display()).as_str(),
        );
    };
    let Some(plan) = String::from_utf8(bytes)
        .ok()
        .and_then(|text| { return parse_hook_dispatch_plan(text.as_str()); })
    else {
        return dispatch_failure(
            format!("the plan {} is malformed", plan_path.display()).as_str(),
        );
    };
    if plan.disabled_events.iter().any(|name| { return name == target.event; }) {
        return 0;
    }
    let child_environment: Vec<(OsString, OsString)> = dispatch_environment(environment, &plan);
    let lock: Option<OwnerLock> = if plan.skip_hook_lock {
        None
    } else {
        match acquire_live_owner_lock(Path::new(plan.hook_lock_path.as_str()), "the hook lock") {
            Ok(acquired) => Some(acquired),
            Err(error) => return dispatch_failure(error.to_string().as_str()),
        }
    };
    // The transaction directory is the parent of the `hooks` directory the entry started from.
    let no_hooks: PathBuf = match target.hooks_directory.parent() {
        Some(transaction) => transaction_no_hooks_directory(transaction),
        None => return dispatch_failure("the hooks directory has no transaction directory"),
    };
    let status: i32 = run_git_hook(&plan, target.event, no_hooks.as_path(), arguments, child_environment.as_slice());
    if let Some(held) = lock
        && let Err(error) = held.release()
    {
        return dispatch_failure(error.to_string().as_str());
    }
    return status;
}

/// What:
///  Start Git's hook runner for one event with the plan's paths, and wait for it.
/// Why:
///   The incumbent's `spawn` with `stdio: 'inherit'`; a missing status (a signal or a failure to start) is 1.
///  `core.hooksPath` names the absent directory, so Git runs only the hooks the configuration defines by name:
///  traditional hooks are unsupported, and the transaction's own entries must not start themselves again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// spawn(plan.gitPath, ['-c', `hook.${event}.enabled=true`, '-c', `core.hooksPath=${noHooks}`, 'hook', 'run', '--ignore-missing', event, '--', ...args], { stdio: 'inherit', env });
/// ```
fn run_git_hook(
    plan: &HookDispatchPlan,
    event: &str,
    no_hooks: &Path,
    arguments: &[OsString],
    environment: &[(OsString, OsString)],
) -> i32 {
    let enabled: String = format!("hook.{event}.enabled=true");
    // Built from raw OS text, so a transaction directory that is not UTF-8 stays exact.
    let mut hooks_path: OsString = OsString::from("core.hooksPath=");
    hooks_path.push(no_hooks.as_os_str());
    let status = Command::new(plan.git_path.as_str())
        .args(["-c", enabled.as_str()])
        .args([OsStr::new("-c"), hooks_path.as_os_str()])
        .args(["hook", "run", "--ignore-missing", event, "--"])
        .args(arguments)
        .env_clear()
        .envs(environment.iter().map(|(name, value)| { return (name.clone(), value.clone()); }))
        .status_retrying();
    let Ok(finished) = status else {
        return dispatch_failure(
            format!("cannot start {}", plan.git_path).as_str(),
        );
    };
    return finished.code().unwrap_or(1);
}

/// What:
///  Dispatch when the start name names a transaction's entry, otherwise run the wrapper.
/// Why:
///   The executable calls this with its own start name first; every other start name, and
///  an absent one, reaches the ordinary path unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (startName is a dispatch) await dispatch(event, args); else await runProcess(args, env);
/// ```
pub fn run_executable(
    start_name: Option<&OsStr>,
    arguments: &[OsString],
    environment: &[(OsString, OsString)],
) -> i32 {
    if let Some(name) = start_name
        && let Some(target) = dispatch_target_for_start_name(Path::new(name))
    {
        return run_hook_dispatch(&target, arguments, environment);
    }
    return run_process(arguments, environment);
}

#[cfg(test)]
#[path = "hook_dispatch_tests.rs"]
mod tests;
