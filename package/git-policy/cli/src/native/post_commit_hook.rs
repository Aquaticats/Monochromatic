//! What:
//!  The single `post-commit` run after a landing: its author environment, the hook lock around it,
//!  and the real Git hook runner with inherited standard streams.
//! Why:
//!  The incumbent's `commit-landing-post-commit-hook.ts` runs it outside both landing locks,
//!  under the hook lock unless `hooks.concurrentCommits` is `true` or an outer preparation lease is inherited.
//!  Its exit status is ignored, as native Git ignores it, and the user sees the hook's output on the terminal.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await runPostCommitHook({ gitPath, cwd, globalArgs, realIndexPath, landedOid, plan });
//! ```

use super::diagnostic_log::debug;
use super::forwarding::git_command;
use super::hook_dispatch_plan::HookDispatchPlan;
use super::hook_presence::event_may_have_user_hook;
use super::live_lock::acquire_live_owner_lock;
use super::owner_lock::OwnerLock;
use super::transaction_git::{GitContext, GitRequest, run_git};
use std::ffi::OsString;
use std::path::Path;
use std::process::{Command, Stdio};

/// What:
///  A failure of the post-commit run: the author lookup, the hook lock, or the hook's start.
/// Why:
///   The caller reports the message and exits 1, as the incumbent reports a thrown error after a landing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// throw new CommitTransactionGitError(message);
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PostCommitHookError(pub String);

/// What:
///  Whether `post-commit` ran, and the status its hook returned when it did.
/// Why:
///   A disabled event runs nothing, and a non-zero status is recorded but never fails the landed commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PostCommitHookOutcome = 'disabled' | { exitCode: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PostCommitHookOutcome {
    /// `hook.post-commit.enabled=false` disabled the event; nothing ran.
    Disabled,
    /// No user hook existed for the event when it came due; no author lookup, lock or hook runner started.
    Absent,
    /// The hook runner's status, which the caller ignores.
    Ran(i32),
}

/// What:
///  The inputs of one post-commit run.
/// Why:
///   Named fields keep the paths apart, and the plan carries the disabled events and the hook lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// { gitPath, cwd, globalArgs, realIndexPath, landedOid, plan }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct PostCommitHookRequest<'a> {
    /// Real Git and the caller's global options.
    pub context: &'a GitContext,
    /// Owning worktree root, where the hook runs.
    pub worktree: &'a Path,
    /// Real index native Git names in `GIT_INDEX_FILE`.
    pub real_index: &'a Path,
    /// Landed commit whose author the hook sees.
    pub landed_oid: &'a str,
    /// Dispatch plan holding the disabled events and the hook lock.
    pub plan: &'a HookDispatchPlan,
}

/// What:
///  Run `post-commit` once for a landed commit.
/// Why:
///   The order matches the incumbent: disabled check, author lookup, hook lock, hook run, lock release;
///  an event with no user hook (looked up now, after the landing) runs none of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (plan.disabledEvents.includes('post-commit') || !mayHaveHook(plan, 'post-commit')) return; await using lock = plan.skipHookLock ? undefined : await acquireOwnerLock(...); await runTransactionGit(...);
/// ```
pub fn run_post_commit_hook(
    request: PostCommitHookRequest,
) -> Result<PostCommitHookOutcome, PostCommitHookError> {
    if request.plan.disabled_events.iter().any(|name| {
        return name.as_str() == "post-commit";
    }) {
        debug(
            "runPostCommitHook",
            "post-commit disabled by hook.post-commit.enabled=false",
        );
        return Ok(PostCommitHookOutcome::Disabled);
    }
    if !event_may_have_user_hook(request.plan, "post-commit") {
        debug("runPostCommitHook", "no user post-commit hook; nothing to run");
        return Ok(PostCommitHookOutcome::Absent);
    }
    let mut environment: Vec<(OsString, OsString)> =
        read_author_environment(request.context, request.worktree, request.landed_oid)?;
    environment.push((
        OsString::from("GIT_INDEX_FILE"),
        request.real_index.as_os_str().to_os_string(),
    ));
    environment.push((OsString::from("GIT_EDITOR"), OsString::from(":")));
    // Held until the end of this function, so an early return releases it through `Drop`.
    let lock: Option<OwnerLock> = if request.plan.skip_hook_lock {
        None
    } else {
        Some(acquire_hook_lock(request.plan.hook_lock_path.as_str())?)
    };
    let status: i32 = run_hook(request.context, request.worktree, environment.as_slice())?;
    if let Some(held) = lock {
        held.release()
            .map_err(|error| return PostCommitHookError(error.to_string()))?;
    }
    return Ok(PostCommitHookOutcome::Ran(status));
}

/// What:
///  Take the hook lock for the post-commit run, waiting silently while a live owner holds it.
/// Why:
///   The incumbent's `acquireOwnerLock` on `plan.hookLockPath`, shared with the preparation dispatchers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await acquireOwnerLock({ lockDirectory: plan.hookLockPath });
/// ```
fn acquire_hook_lock(lock_path: &str) -> Result<OwnerLock, PostCommitHookError> {
    let directory: &Path = Path::new(lock_path);
    // The incumbent creates the lock's parents recursively; the native owner lock creates one level only.
    if let Some(parent) = directory.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            return PostCommitHookError(format!("creating {} failed: {error}", parent.display()));
        })?;
    }
    return acquire_live_owner_lock(directory, "the hook lock")
        .map_err(|error| return PostCommitHookError(error.to_string()));
}

/// What:
///  Read the landed commit's author as `GIT_AUTHOR_*` variables, through a Git start without the global options.
/// Why:
///   The incumbent's `authorEnvironment` runs `show` with the bare command; a refusal fails the run before the hook starts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runTransactionGit({ gitPath, cwd, args: ['show', '--no-patch', '--date=raw', '--format=%an%x00%ae%x00%ad', landedOid] });
/// ```
fn read_author_environment(
    context: &GitContext,
    worktree: &Path,
    landed_oid: &str,
) -> Result<Vec<(OsString, OsString)>, PostCommitHookError> {
    let request: GitRequest = GitRequest {
        without_prefix: true,
        ..GitRequest::new(
            worktree,
            &[
                "show",
                "--no-patch",
                "--date=raw",
                "--format=%an%x00%ae%x00%ad",
                landed_oid,
            ],
        )
    };
    let output = run_git(context, &request).map_err(|error| {
        return PostCommitHookError(format!("git show could not start: {error}"));
    })?;
    if !output.succeeded() {
        return Err(PostCommitHookError(output.error_text()));
    }
    let text: String = String::from_utf8(output.stdout).map_err(|error| {
        return PostCommitHookError(format!(
            "the landed commit's author fields are not UTF-8: {error}"
        ));
    })?;
    return Ok(author_environment(text.as_str())
        .into_iter()
        .map(|(name, value)| {
            return (OsString::from(name), OsString::from(value));
        })
        .collect());
}

/// What:
///  Start the real Git hook runner for `post-commit` with inherited standard streams, and return its status.
/// Why:
///   The user sees the hook's output on the terminal, as the incumbent's `stdio: 'inherit'` gives it;
///  a missing status (a signal) is 1, which the caller ignores.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runTransactionGit({ gitPath, cwd, args: [...globalArgs, 'hook', 'run', '--ignore-missing', 'post-commit'], environment, stdio: 'inherit', allowFailure: true });
/// ```
fn run_hook(
    context: &GitContext,
    worktree: &Path,
    environment: &[(OsString, OsString)],
) -> Result<i32, PostCommitHookError> {
    let mut arguments: Vec<OsString> = context.global_prefix.clone();
    arguments.extend(
        ["hook", "run", "--ignore-missing", "post-commit"]
            .into_iter()
            .map(OsString::from),
    );
    let mut command: Command = git_command(
        context.real_git.as_path(),
        arguments.as_slice(),
        context.overlay.as_slice(),
    );
    command
        .current_dir(worktree)
        .envs(environment.iter().cloned())
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    let finished = command.status().map_err(|error| {
        return PostCommitHookError(format!(
            "{} could not start: {error}",
            context.real_git.display()
        ));
    })?;
    return Ok(finished.code().unwrap_or(1));
}

/// What:
///  The environment native `git commit` gives the hook, from the landed commit's `%an`, `%ae` and raw `%ad`.
/// Why:
///   Hooks read `GIT_AUTHOR_*`; the incumbent takes the NUL-separated fields and drops one trailing newline.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [name = '', email = '', date = ''] = output.replace(/\n$/, '').split('\0');
/// ```
pub fn author_environment(output: &str) -> Vec<(String, String)> {
    let trimmed: &str = output.strip_suffix('\n').unwrap_or(output);
    let mut fields = trimmed.split('\0');
    let name: &str = fields.next().unwrap_or("");
    let email: &str = fields.next().unwrap_or("");
    let date: &str = fields.next().unwrap_or("");
    return vec![
        (String::from("GIT_AUTHOR_NAME"), String::from(name)),
        (String::from("GIT_AUTHOR_EMAIL"), String::from(email)),
        (String::from("GIT_AUTHOR_DATE"), format!("@{date}")),
    ];
}

/// Post-commit controls stay out of the release executable.
#[cfg(test)]
#[path = "post_commit_hook_tests.rs"]
mod tests;
