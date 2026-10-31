//! What:
//!  The linked-worktree ignored-state copy around one forwarded Git invocation: the public entry point the
//!  integrator calls, the settlement and summary it writes, and the exit status it returns.
//! Why:
//!  A worktree that Git creates (or a bare repository's) needs its ignored files from the source worktree,
//!  copied under one settlement lock per repository and recovered first when an earlier copy was interrupted.
//!
//! Integrator contract:
//!  The caller decides the bypass facts (`--no-worktree-copy`, short-circuit global help or version, whether the
//!  command creates or moves worktrees after alias resolution) with the native modules that already own them,
//!  passes the repository identity and the effective working directory, and supplies the runner that starts real Git
//!  with the lease token in its environment. This module never runs the commit refusal; it is not wired yet.
//!  A main-checkout source is a bypass unless the caller also names `Invocation::main_checkout_paths` (the configured
//!  forbidden-strings rules files, each entry of an array); then only those paths are copied, and only when Git ignores them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await runGitWithWorktreeCopy({ args, gitPath, identity, command, environment });
//! ```

use super::json_record::quote;
use super::transaction_git::GitContext;
use super::worktree_copy_error::{WorktreeCopyError, lifecycle_failure, raw_failure};
use super::worktree_copy_lock::{acquire_worktree_copy_lock, validate_inherited_lease};
use super::worktree_copy_recovery::{
    recover_pending_worktree_copies, recover_worktree_copy_transactions, report_recovered,
};
use super::worktree_copy_registry::{
    CreatedWorktrees, WorktreeCopyObservation, find_created_worktrees, observe_copy_source,
};
use super::worktree_copy_transaction::synchronize_created_worktree;
use super::worktree_identity::WorktreeIdentity;
use std::io::Write;
use std::path::{Path, PathBuf};

/// What:
///  How a real Git run ended.
/// Why:
///   Its numeric status is the exit status the wrapper keeps; a signal has none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ForwardedGitExecution = { failure?: SubprocessError };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GitStatus {
    /// The exit code, absent when Git ended by a signal.
    pub code: Option<i32>,
}

impl GitStatus {
    /// Whether Git exited with status zero.
    pub fn succeeded(&self) -> bool {
        return self.code == Some(0);
    }
}

/// What:
///  Everything the lifecycle needs to decide how one invocation is settled.
/// Why:
///   The decisions that depend on argv and global options are made by their owners and passed in as facts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// runGitWithWorktreeCopy({ args, gitPath, identity, command, environment })
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Invocation {
    /// The effective repository identity captured before real Git.
    pub identity: WorktreeIdentity,
    /// The working directory the repository queries run in.
    pub effective_cwd: PathBuf,
    /// Whether the command creates or moves worktrees after alias resolution.
    pub creates_or_moves: bool,
    /// Whether `--no-worktree-copy` appeared in flag position.
    pub opted_out: bool,
    /// Whether real Git only prints help or its version.
    pub short_circuit: bool,
    /// The lease token inherited from an outer settlement, when the environment carried one.
    pub inherited_token: Option<String>,
    /// Repository paths a main-checkout source hands over (the configured rules file);
    ///  empty keeps the main checkout a bypass, and linked and bare sources ignore it.
    pub main_checkout_paths: Vec<String>,
    /// The real Git executable and the variables every Git child receives.
    pub context: GitContext,
}

/// What:
///  The result of one settled invocation: Git's status and the copy failure, if any.
/// Why:
///   Both facts decide the exit status, so they travel together.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a resolved run, or a ForwardedGitWorktreeCopyError carrying both facts
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Settlement {
    /// How real Git ended.
    pub git: GitStatus,
    /// The ignored-state copy failure, when copying failed.
    pub copy_failure: Option<WorktreeCopyError>,
}

/// Runs real Git once, with the given lease token (absent when the run inherits none).
pub type GitRunner<'a> = dyn FnMut(Option<&str>) -> Result<GitStatus, WorktreeCopyError> + 'a;

/// What:
///  The one summary line a successful copy writes.
/// Why:
///   Users and agents read the count, the source, and the destination count from it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// renderSummary({ copiedEntries, destinationCount, sourceRoot })
/// ```
pub fn summary_line(copied: usize, destinations: usize, source: Option<&Path>) -> String {
    let source_text: String = match source {
        None => String::from("bare repository with an empty source set"),
        Some(path) => quote(path.to_string_lossy().as_ref()),
    };
    let worktree_noun: &str = if destinations == 1 {
        "worktree"
    } else {
        "worktrees"
    };
    let entry_noun: &str = if copied == 1 { "entry" } else { "entries" };
    return format!(
        "cli-git: copied {copied} ignored filesystem {entry_noun} from {source_text} into {destinations} new {worktree_noun}.\n"
    );
}

/// What:
///  The exit status of a settled invocation.
/// Why:
///   A copy-only failure is 2; Git's own status is kept when Git failed; a signal without a status is 1.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // src/bin.ts: copy failure 2, Git status kept, signal 1
/// ```
pub fn exit_code(settlement: &Settlement) -> i32 {
    let git_code: i32 = settlement.git.code.unwrap_or(1);
    // `match` on whether Git and the copy each succeeded.
    match (
        settlement.git.succeeded(),
        settlement.copy_failure.is_some(),
    ) {
        (true, false) => return 0,
        (false, false) => return git_code,
        (true, true) => return 2,
        (false, true) => return git_code,
    }
}

/// What:
///  Runs one forwarded invocation with no synchronization, and settles its status.
/// Why:
///   Every bypass forwards exactly the same way, so the runner is called once with no lease.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await forwardOnly({ args, gitPath, environment });
/// ```
fn forward_only(run_git: &mut GitRunner) -> Result<Settlement, WorktreeCopyError> {
    let git: GitStatus = run_git(None)?;
    return Ok(Settlement {
        git,
        copy_failure: None,
    });
}

/// What:
///  The error for a repository whose common directory changed while the lock was acquired.
/// Why:
///   The lock belongs to one repository; a different one means the copy would settle the wrong state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new WorktreeCopyError('cli-git: effective repository changed while acquiring worktree-copy settlement lock.')
/// ```
fn repository_changed() -> WorktreeCopyError {
    return WorktreeCopyError(String::from(
        "cli-git: effective repository changed while acquiring worktree-copy settlement lock.",
    ));
}

/// What:
///  Synchronizes every worktree the command registered and writes the summary line for them.
/// Why:
///   Each destination is settled in turn, and the first failure stops the copy with no summary written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const found = await findCreatedWorktrees(...); const summary = await synchronizeCreatedWorktrees(...); process.stderr.write(renderSummary(summary));
/// ```
fn synchronize_after(
    context: &GitContext,
    observation: &WorktreeCopyObservation,
    stderr: &mut dyn Write,
) -> Result<(), WorktreeCopyError> {
    let found: CreatedWorktrees = find_created_worktrees(context, observation)?;
    if found.created.is_empty() {
        return Ok(());
    }
    let mut copied: usize = 0;
    // `match` on the source: a bare repository has nothing to copy, so only a worktree source is copied.
    match observation.source_root.as_ref() {
        None => {}
        Some(source) => {
            for destination in &found.created {
                copied += synchronize_created_worktree(
                    context,
                    observation.common_dir.as_path(),
                    source.as_path(),
                    destination.root.as_path(),
                    found.registered_roots.as_slice(),
                    &observation.selection,
                )?;
            }
        }
    }
    let line: String = summary_line(
        copied,
        found.created.len(),
        observation.source_root.as_deref(),
    );
    stderr
        .write_all(line.as_bytes())
        .map_err(|error| return raw_failure(&error))?;
    return Ok(());
}

/// What:
///  Runs the settled part of one applicable creating command: recovery, real Git, and synchronization, under the lock.
/// Why:
///   Recovery must finish before the new Git command can create another worktree in the same repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // inside the acquired lease: recover, executeRealGit, synchronize, report
/// ```
fn settle_under_lock(
    invocation: &Invocation,
    first: &WorktreeCopyObservation,
    run_git: &mut GitRunner,
    identity_after_lock: &mut dyn FnMut() -> Result<WorktreeIdentity, WorktreeCopyError>,
    lease_token: &str,
    stderr: &mut dyn Write,
) -> Result<Settlement, WorktreeCopyError> {
    let refreshed: Option<WorktreeCopyObservation> = observe_copy_source(
        &identity_after_lock()?,
        invocation.effective_cwd.as_path(),
        invocation.main_checkout_paths.as_slice(),
    )?;
    let observation: WorktreeCopyObservation = match refreshed {
        Some(observed) if observed.common_dir == first.common_dir => observed,
        _ => return Err(repository_changed()),
    };
    let report = recover_worktree_copy_transactions(observation.common_dir.as_path())?;
    report_recovered(&report, stderr)?;
    let git: GitStatus = run_git(Some(lease_token))?;
    let copy_failure: Option<WorktreeCopyError> =
        synchronize_after(&invocation.context, &observation, stderr).err();
    return Ok(Settlement { git, copy_failure });
}

/// What:
///  Settles one forwarded invocation: bypasses forward untouched; applicable creating commands run under the settlement lock;
///  other commands only recover pending journals, and only when one exists.
/// Why:
///   The integrator calls this once per forwarded command and turns the settlement into the exit status with [`exit_code`].
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runGitWithWorktreeCopy({ args, gitPath, identity, command, environment });
/// ```
pub fn run_with_worktree_copy(
    invocation: &Invocation,
    run_git: &mut GitRunner,
    identity_after_lock: &mut dyn FnMut() -> Result<WorktreeIdentity, WorktreeCopyError>,
    stderr: &mut dyn Write,
) -> Result<Settlement, WorktreeCopyError> {
    return settle_invocation(invocation, run_git, identity_after_lock, stderr)
        .map_err(lifecycle_failure);
}

/// What:
///  The settlement of one invocation, before the lifecycle words its raw failures.
/// Why:
///   Only the top of the lifecycle words raw failures, as the incumbent's `asWorktreeCopyError` does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // body of runGitWithWorktreeCopy
/// ```
fn settle_invocation(
    invocation: &Invocation,
    run_git: &mut GitRunner,
    identity_after_lock: &mut dyn FnMut() -> Result<WorktreeIdentity, WorktreeCopyError>,
    stderr: &mut dyn Write,
) -> Result<Settlement, WorktreeCopyError> {
    // `if` chain in the incumbent's order: each bypass forwards without touching the repository.
    if invocation.opted_out || invocation.short_circuit {
        return forward_only(run_git);
    }
    let Some(observation) = observe_copy_source(
        &invocation.identity,
        invocation.effective_cwd.as_path(),
        invocation.main_checkout_paths.as_slice(),
    )?
    else {
        return forward_only(run_git);
    };
    let inherited: bool = match invocation.inherited_token.as_deref() {
        Some(token) => validate_inherited_lease(observation.common_dir.as_path(), token)?,
        None => false,
    };
    if inherited {
        return forward_only(run_git);
    }
    if !invocation.creates_or_moves {
        recover_pending_worktree_copies(observation.common_dir.as_path(), stderr)?;
        return forward_only(run_git);
    }
    let lease = acquire_worktree_copy_lock(observation.common_dir.as_path(), stderr)?;
    let settled: Result<Settlement, WorktreeCopyError> = settle_under_lock(
        invocation,
        &observation,
        run_git,
        identity_after_lock,
        lease.lease_token(),
        stderr,
    );
    lease.release()?;
    return settled;
}

/// Lifecycle controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_lifecycle_tests.rs"]
mod tests;
