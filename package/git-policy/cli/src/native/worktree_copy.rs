//! What:
//!  The linked-worktree ignored-state copy around one forwarded Git invocation: the public entry point the
//!  integrator calls, the settlement and summary it writes, and the exit status it returns.
//! Why:
//!  A worktree that Git creates (or a bare repository's) needs its ignored files from the source worktree.
//!  No lock stands around the command: the destination comes from the command line, the copy is a transaction of its own
//!  (a journal and a stage claim that name their owner process), and a copy a dead process left is recovered first by the
//!  next command. Concurrent adds in one repository never wait for one another (the owner's decision of 2026-11-03).
//!
//! Integrator contract:
//!  The caller decides the bypass facts (`--no-worktree-copy`, short-circuit global help or version, whether the
//!  command creates a worktree after alias resolution, and the destination its command line names) with the native
//!  modules that already own them, passes the repository identity and the effective working directory, and supplies the
//!  runner that starts real Git. This module never runs the commit refusal; it is not wired yet.
//!  A main-checkout source is observed only when the command creates a worktree; it then hands over its ignored
//!  state under the main checkout's leave-out rule (`worktree_copy_prune.rs`), with its registered worktrees excluded.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await runGitWithWorktreeCopy({ args, gitPath, identity, command, environment });
//! ```

use super::json_record::quote;
use super::transaction_git::GitContext;
use super::worktree_copy_error::{WorktreeCopyError, lifecycle_failure, raw_failure};
use super::worktree_copy_recovery::recover_pending_worktree_copies;
use super::worktree_copy_selection::CopySelection;
use super::worktree_copy_registry::{
    CreatedWorktrees, WorktreeCopyObservation, destination_is_registered, find_created_worktree, observe_copy_source,
};
use super::worktree_copy_transaction::synchronize_created_worktree_noting;
use super::worktree_identity::WorktreeIdentity;
use super::worktree_add_plan::AddPlan;
use super::worktree_ladder::{LadderResult, begin_ladder};
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
    /// Whether the command creates a worktree after alias resolution.
    pub creates: bool,
    /// The destination the creating command's line names, in the form Git records it; absent when the line has a form that is
    ///  not read, in which case no copy is made and the command says so.
    pub destination: Option<PathBuf>,
    /// Whether `--no-worktree-copy` appeared in flag position.
    pub opted_out: bool,
    /// Whether real Git only prints help or its version.
    pub short_circuit: bool,
    /// The real Git executable and the variables every Git child receives.
    pub context: GitContext,
    /// The destination of a `git worktree add` in a form the cheap-worktree ladder serves, when it is one.
    pub add_plan: Option<AddPlan>,
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

/// What:
///  Which Git run the settlement asks its runner to start.
/// Why:
///   The person's command is one run; the cheap-worktree ladder adds `--no-checkout` to it and starts the commands that
///  complete the worktree itself, each with the caller's streams.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GitStep = { kind: 'person' } | { kind: 'without-checkout'; insertAt: number } | { kind: 'other'; args: string[]; env: [string, string][] };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GitStep<'a> {
    /// The person's arguments, unchanged.
    Person,
    /// The person's arguments with `--no-checkout` inserted at this index.
    WithoutCheckout(usize),
    /// Another Git command with these arguments and these variables added.
    Other {
        /// The arguments, global options and subcommand included.
        arguments: &'a [std::ffi::OsString],
        /// The variables added to the run's environment.
        environment: &'a [(std::ffi::OsString, std::ffi::OsString)],
    },
}

/// Runs one real Git step.
pub type GitRunner<'a> = dyn FnMut(&GitStep<'_>) -> Result<GitStatus, WorktreeCopyError> + 'a;

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
    let git: GitStatus = run_git(&GitStep::Person)?;
    return Ok(Settlement {
        git,
        copy_failure: None,
    });
}

/// The line a successful `git worktree add` prints when its command line has a form whose destination is not read.
const UNKNOWN_DESTINATION_NOTICE: &str = "cli-git: this `git worktree add` names its worktree in a form cli-git does not read, so the worktree received no ignored-state copy. Name the path as a plain operand (`git worktree add [-b <branch>] <path> [<commit-ish>]`, with `-C <directory>` or `-c <name>=<value>` before `worktree`) to have it copied.\n";

/// What:
///  Synchronizes the worktree this command registered at its destination and writes the summary line for it.
/// Why:
///   The destination is the one the command line named, so a worktree another add registered at the same time is never
///  copied into here; a failure stops the copy with no summary written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const found = await findCreatedWorktree(...); const copied = await synchronizeCreatedWorktree(...); process.stderr.write(renderSummary(copied));
/// ```
fn synchronize_after(
    invocation: &Invocation,
    observation: &WorktreeCopyObservation,
    registered_before: bool,
    carried: Option<&Path>,
    stderr: &mut dyn Write,
) -> Result<(), WorktreeCopyError> {
    // A main checkout that names no path hands over nothing: no stage, no journal and no summary line.
    if matches!(&observation.selection, CopySelection::Paths(paths) if paths.is_empty()) {
        return Ok(());
    }
    let Some(destination) = invocation.destination.as_deref() else {
        stderr
            .write_all(UNKNOWN_DESTINATION_NOTICE.as_bytes())
            .map_err(|error| return raw_failure(&error))?;
        return Ok(());
    };
    let mut found: CreatedWorktrees =
        find_created_worktree(&invocation.context, observation, destination, registered_before)?;
    // A destination that received the source's ignored state with a snapshot needs no copy.
    found.created.retain(|worktree| { return carried != Some(worktree.root.as_path()); });
    if found.created.is_empty() {
        return Ok(());
    }
    let mut copied: usize = 0;
    // `match` on the source: a bare repository has nothing to copy, so only a worktree source is copied.
    match observation.source_root.as_ref() {
        None => {}
        Some(source) => {
            for created in &found.created {
                copied += synchronize_created_worktree_noting(
                    &invocation.context,
                    observation.common_dir.as_path(),
                    source.as_path(),
                    created.root.as_path(),
                    found.registered_roots.as_slice(),
                    &observation.selection,
                    stderr,
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
///  Runs one creating command: whether its destination was registered already, the ladder and real Git, and the copy.
/// Why:
///   Nothing here waits for another command: the destination, the stage claim and the journal belong to this command alone,
///  so any number of adds run side by side in one repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const before = isRegistered(destination); const status = await runLadder(); await synchronize(destination, before);
/// ```
fn settle_creation(
    invocation: &Invocation,
    observation: &WorktreeCopyObservation,
    run_git: &mut GitRunner,
    stderr: &mut dyn Write,
) -> Result<Settlement, WorktreeCopyError> {
    let registered_before: bool = invocation
        .destination
        .as_deref()
        .is_some_and(|destination| { return destination_is_registered(observation.common_dir.as_path(), destination); });
    let ladder = begin_ladder(invocation.add_plan.as_ref(), &invocation.context, &invocation.identity);
    let ran: Result<GitStatus, WorktreeCopyError> = run_git(&ladder.first_step());
    let settled: Result<LadderResult, WorktreeCopyError> = match ran {
        Ok(status) => Ok(ladder.finish(status, run_git, &invocation.context, &mut std::io::stdout())),
        Err(error) => {
            // Git did not start: the ladder only has its empty subvolume to remove, whatever status it is given.
            ladder.finish(GitStatus { code: None }, run_git, &invocation.context, &mut std::io::sink());
            Err(error)
        }
    };
    let LadderResult { git, carried } = settled?;
    let copy_failure: Option<WorktreeCopyError> = if git.succeeded() || invocation.destination.is_some() {
        synchronize_after(invocation, observation, registered_before, carried.as_deref(), stderr).err()
    } else {
        None
    };
    return Ok(Settlement { git, copy_failure });
}

/// What:
///  Settles one forwarded invocation: bypasses forward untouched; every other command first recovers what dead owners left,
///  and a creating command then copies the ignored state into the worktree it created.
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
    )?
    else {
        return forward_only(run_git);
    };
    // Recovery never waits: it takes over only journals and claims whose owner is dead, and one rename picks its single winner.
    recover_pending_worktree_copies(observation.common_dir.as_path(), stderr)?;
    if !invocation.creates {
        return forward_only(run_git);
    }
    return settle_creation(invocation, &observation, run_git, stderr);
}

/// Lifecycle controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_lifecycle_tests.rs"]
mod tests;
