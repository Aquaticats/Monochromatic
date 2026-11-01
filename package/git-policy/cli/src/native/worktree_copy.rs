//! What:
//!  The linked-worktree ignored-state copy around one forwarded Git invocation: the public entry point the
//!  integrator calls, the settlement and summary it writes, and the exit status it returns.
//! Why:
//!  A worktree that Git creates (or a bare repository's) needs its ignored files from the source worktree,
//!  copied under one claim per destination (nothing waits for another copy) and recovered first when an earlier copy
//!  was interrupted (`doc/worktree-copy-claim.md`).
//!
//! Integrator contract:
//!  The caller decides the bypass facts (`--no-worktree-copy`, short-circuit global help or version, whether the
//!  command creates or moves worktrees after alias resolution) with the native modules that already own them,
//!  passes the repository identity and the effective working directory, and supplies the runner that starts real Git.
//!  This module never runs the commit refusal; it is not wired yet.
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
use super::worktree_copy_recovery::recover_pending_worktree_copies;
use super::worktree_copy_registry::{
    CreatedWorktrees, WorktreeCopyObservation, find_created_worktrees, is_initializing, observe_copy_source,
    refresh_before_admin_ids,
};
use super::worktree_copy_transaction::{Synchronized, synchronize_created_worktree};
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

/// Runs real Git once.
pub type GitRunner<'a> = dyn FnMut() -> Result<GitStatus, WorktreeCopyError> + 'a;

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
///   Every bypass forwards exactly the same way, so the runner is called once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await forwardOnly({ args, gitPath, environment });
/// ```
fn forward_only(run_git: &mut GitRunner) -> Result<Settlement, WorktreeCopyError> {
    let git: GitStatus = run_git()?;
    return Ok(Settlement {
        git,
        copy_failure: None,
    });
}

/// What:
///  The notice for a worktree another copy holds the claim of.
/// Why:
///   The user reads why this command copied nothing into it, and that the holder finishes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `cli-git: left the ignored-state copy into ${JSON.stringify(root)} to another process: ${reason}.`
/// ```
fn yielded_notice(destination: &Path, reason: &str) -> String {
    return format!(
        "cli-git: left the ignored-state copy into {} to another process: {reason}.\n",
        quote(destination.to_string_lossy().as_ref())
    );
}

/// What:
///  The notice for a worktree that was removed before anything was installed into it.
/// Why:
///   Nothing was created, so the user is told why no copy happened instead of seeing a failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `cli-git: skipped the ignored-state copy into ${JSON.stringify(root)}, which is no longer a registered worktree.`
/// ```
fn unregistered_notice(destination: &Path) -> String {
    return format!(
        "cli-git: skipped the ignored-state copy into {}, which is no longer a registered worktree.\n",
        quote(destination.to_string_lossy().as_ref())
    );
}

/// What:
///  Synchronizes every worktree the command registered and writes the summary line for the ones this command copied.
/// Why:
///   Each destination is settled in turn, and the first failure stops the copy with no summary written; a destination
///  another command already copied, or holds the claim of, is not this command's to count.
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
    let mut destinations: usize = found.created.len();
    // `match` on the source: a bare repository has nothing to copy, so only a worktree source is copied.
    match observation.source_root.as_ref() {
        None => {}
        Some(source) => {
            destinations = 0;
            for destination in &found.created {
                // `match` on how the destination ended: only a copy this command ran counts toward the summary.
                match synchronize_created_worktree(
                    context,
                    observation.common_dir.as_path(),
                    source.as_path(),
                    destination,
                    found.registered_roots.as_slice(),
                    &observation.selection,
                    stderr,
                )? {
                    Synchronized::Copied(entries) => {
                        copied += entries;
                        destinations += 1;
                    }
                    Synchronized::AlreadyCopied => {}
                    Synchronized::Yielded(reason) => {
                        stderr
                            .write_all(yielded_notice(destination.root.as_path(), reason.as_str()).as_bytes())
                            .map_err(|error| return raw_failure(&error))?;
                    }
                    Synchronized::Unregistered => {
                        stderr
                            .write_all(unregistered_notice(destination.root.as_path()).as_bytes())
                            .map_err(|error| return raw_failure(&error))?;
                    }
                }
            }
        }
    }
    if destinations == 0 {
        return Ok(());
    }
    let line: String = summary_line(
        copied,
        destinations,
        observation.source_root.as_deref(),
    );
    stderr
        .write_all(line.as_bytes())
        .map_err(|error| return raw_failure(&error))?;
    return Ok(());
}

/// What:
///  Whether the invocation runs from a linked worktree that Git is still creating.
/// Why:
///   Such a worktree has not been filled yet, so it is no source; a filter or hook of an enclosing `git worktree add`
///  that creates another worktree leaves the copy to the enclosing command, which copies from the real source.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// identity.kind === 'linked-worktree' && await isInitializing(identity.gitDir)
/// ```
fn source_is_being_created(identity: &WorktreeIdentity) -> Result<bool, WorktreeCopyError> {
    let WorktreeIdentity::LinkedWorktree { git_dir, .. } = identity else {
        return Ok(false);
    };
    let (Some(admin_root), Some(name)) = (git_dir.parent(), git_dir.file_name()) else {
        return Ok(false);
    };
    return is_initializing(admin_root, name.to_string_lossy().as_ref());
}

/// What:
///  Settles one forwarded invocation: bypasses forward untouched; creating commands recover what no running copy holds,
///  run real Git and copy into the worktrees it registered; other commands only recover, and only when something is pending.
/// Why:
///   The integrator calls this once per forwarded command and turns the settlement into the exit status with [`exit_code`].
///  Nothing here waits for another command: a running copy is skipped by recovery and yielded to by synchronization.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runGitWithWorktreeCopy({ args, gitPath, identity, command, environment });
/// ```
pub fn run_with_worktree_copy(
    invocation: &Invocation,
    run_git: &mut GitRunner,
    stderr: &mut dyn Write,
) -> Result<Settlement, WorktreeCopyError> {
    return settle_invocation(invocation, run_git, stderr).map_err(lifecycle_failure);
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
    // A creating command that runs inside the checkout of its own source (a filter or hook of an enclosing
    // `git worktree add`) has a source that does not hold its ignored files yet; the enclosing command copies instead.
    if invocation.creates_or_moves && source_is_being_created(&invocation.identity)? {
        return forward_only(run_git);
    }
    // Recovery of what no running copy holds comes before Git, so an interrupted copy is finished before the next
    // worktree is created; a command that creates nothing recovers and then forwards.
    recover_pending_worktree_copies(observation.common_dir.as_path(), stderr)?;
    if !invocation.creates_or_moves {
        return forward_only(run_git);
    }
    // The before-set is read after recovery, as close to Git as possible, so worktrees that other commands register
    // meanwhile are told apart from this command's by their claims and not by luck.
    let fresh: WorktreeCopyObservation = refresh_before_admin_ids(&observation)?;
    let git: GitStatus = run_git()?;
    let copy_failure: Option<WorktreeCopyError> =
        synchronize_after(&invocation.context, &fresh, stderr).err();
    return Ok(Settlement { git, copy_failure });
}

/// Lifecycle controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_lifecycle_tests.rs"]
mod tests;
