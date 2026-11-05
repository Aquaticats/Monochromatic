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

use crate::add_timing::{begin as begin_timing, begin_step, end_step, finish as finish_timing};
use crate::json_record::quote;
use crate::transaction_git::GitContext;
use crate::worktree_copy_error::{WorktreeCopyError, lifecycle_failure, raw_failure_of};
use crate::worktree_copy_recovery::recover_pending_worktree_copies;
use crate::worktree_copy_selection::CopySelection;
use crate::worktree_copy_registry::{
    CreatedWorktrees, WorktreeCopyObservation, destination_is_registered, find_created_worktree, observe_copy_source,
};
use crate::worktree_copy_transaction::synchronize_created_worktree_noting;
use crate::worktree_identity::WorktreeIdentity;
use crate::worktree_add_plan::AddPlan;
use crate::worktree_ladder::{LadderResult, begin_ladder};
#[cfg(target_os = "linux")]
use crate::package_repair::{RepairEnvironment, RepairRequest, RepairSwitch, repair_worktree};
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
    /// The person's `--quiet` and `--no-checkout`, read from the arguments, which decide what the package-manager repair of
    ///  a new worktree may print and whether it runs.
    #[cfg(target_os = "linux")]
    pub repair: RepairSwitch,
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

/// What:
///  Runs one real Git step.
///  A `trait` is a named set of methods that different types can each supply, like a TypeScript `interface`.
/// Why:
///   The ladder calls it once per Git step; the state a step needs (the invocation, the real Git, the overlay, the parallel-checkout
///  decision) is a small struct that implements this.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface GitRunner { run(step: GitStep): GitStatus } // throws WorktreeCopyError
/// ```
pub trait GitRunner {
    /// Run `step` and report how Git ended.
    fn run(&mut self, step: &GitStep<'_>) -> Result<GitStatus, WorktreeCopyError>;
}

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
        "monochromatic-git-policy-cli: copied {copied} ignored filesystem {entry_noun} from {source_text} into {destinations} new {worktree_noun}.\n"
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
fn forward_only(run_git: &mut dyn GitRunner) -> Result<Settlement, WorktreeCopyError> {
    let git: GitStatus = run_git.run(&GitStep::Person)?;
    return Ok(Settlement {
        git,
        copy_failure: None,
    });
}

/// The line a successful `git worktree add` prints when its command line has a form whose destination is not read.
const UNKNOWN_DESTINATION_NOTICE: &str = "monochromatic-git-policy-cli: this `git worktree add` names its worktree in a form monochromatic-git-policy-cli does not read, so the worktree received no ignored-state copy. Name the path as a plain operand (`git worktree add [-b <branch>] <path> [<commit-ish>]`, with `-C <directory>` or `-c <name>=<value>` before `worktree`) to have it copied.\n";

/// What:
///  The line for a destination Git made whose registration could not be confirmed.
/// Why:
///   A worktree that gets no copy must say so; here the pointer in its `.git` file and the registration it names do not
///  name each other, so monochromatic-git-policy-cli cannot tell the worktree is the command's own and copies nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: ${quote(destination)} was made by git worktree add, but ...`
/// ```
fn unconfirmed_destination_notice(destination: &Path) -> String {
    return format!(
        "monochromatic-git-policy-cli: {} was made, but its registration could not be confirmed as this command's, so it received no ignored-state copy. `git worktree list` shows whether Git registered it.\n",
        quote(destination.to_string_lossy().as_ref())
    );
}

/// What:
///  Synchronizes the worktree this command registered at its destination and writes the summary line for it; the root of
///  the created worktree, a snapshot's included, comes back for the package-manager repair.
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
    git_succeeded: bool,
    carried: Option<&Path>,
    stderr: &mut dyn Write,
) -> Result<Vec<PathBuf>, WorktreeCopyError> {
    // A main checkout that names no path hands over nothing: no stage, no journal and no summary line.
    if matches!(&observation.selection, CopySelection::Paths(paths) if paths.is_empty()) {
        return Ok(carried.map(Path::to_path_buf).into_iter().collect());
    }
    let Some(destination) = invocation.destination.as_deref() else {
        stderr
            .write_all(UNKNOWN_DESTINATION_NOTICE.as_bytes())
            .map_err(raw_failure_of)?;
        return Ok(Vec::new());
    };
    let mut found: CreatedWorktrees =
        find_created_worktree(&invocation.context, observation, destination, registered_before)?;
    // A destination Git made but whose registration cannot be confirmed is told, not skipped without a word.
    if found.created.is_empty()
        && git_succeeded
        && !registered_before
        && std::fs::symlink_metadata(destination.join(".git")).is_ok()
    {
        let line: String = unconfirmed_destination_notice(destination);
        stderr
            .write_all(line.as_bytes())
            .map_err(raw_failure_of)?;
        return Ok(Vec::new());
    }
    let mut created_roots: Vec<PathBuf> = Vec::new();
    for worktree in &found.created {
        created_roots.push(worktree.root.clone());
    }
    // A destination that received the source's ignored state with a snapshot needs no copy.
    let mut kept = Vec::new();
    for worktree in found.created {
        if carried != Some(worktree.root.as_path()) {
            kept.push(worktree);
        }
    }
    found.created = kept;
    if found.created.is_empty() {
        return Ok(created_roots);
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
        .map_err(raw_failure_of)?;
    return Ok(created_roots);
}

/// What:
///  What is left to do once the ignored state is in place: the source the ignored files came from and the worktrees the
///  command created.
/// Why:
///   The package-manager repairs touch only the new worktrees and read the source, so no other add has to wait for them; they
///  run last, with the facts the copy found.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AfterLock = { sourceRoot?: string; created: string[] };
/// ```
struct AfterLock {
    /// The checkout the ignored files came from; none for a bare repository, which copies nothing.
    source_root: Option<PathBuf>,
    /// The roots of every worktree the command created, a snapshot's included; empty after a failed copy.
    created: Vec<PathBuf>,
}

/// What:
///  Repair the package-manager state of every worktree the command created, once its ignored state is in place.
/// Why:
///   A copy or a snapshot keeps links and build state exactly as the source had them; each tool's own offline repair makes
///  them work at the new location (`package_repair.rs`). It never fails the add: a repair that cannot finish says so in one
///  line that names the command to run. A bare repository has no source worktree, so nothing was copied and nothing is repaired.
///  It runs on the worktree the registration binding confirmed, and no lock stands around it, so a slow tool never holds
///  another add.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const root of roots) repairWorktree({ git, source, destination: root, switch, environment }, stderr);
/// ```
#[cfg(target_os = "linux")]
fn repair_created(invocation: &Invocation, after: &AfterLock, stderr: &mut dyn Write) {
    let Some(source) = after.source_root.as_deref() else { return };
    let environment: RepairEnvironment = RepairEnvironment::from_process();
    for root in &after.created {
        let request: RepairRequest<'_> = RepairRequest { git: &invocation.context, source, destination: root.as_path(), switch: invocation.repair, environment: &environment };
        repair_worktree(&request, stderr);
    }
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
    run_git: &mut dyn GitRunner,
    stderr: &mut dyn Write,
) -> Result<(Settlement, AfterLock), WorktreeCopyError> {
    let registered_before: bool = match invocation.destination.as_deref() {
        Some(destination) => destination_is_registered(observation.common_dir.as_path(), destination),
        None => false,
    };
    let choose_rung_entered: bool = begin_step("chooseRung");
    let ladder = begin_ladder(invocation.add_plan.as_ref(), &invocation.context, &invocation.identity);
    end_step(choose_rung_entered);
    let git_add_entered: bool = begin_step("gitAdd");
    let ran: Result<GitStatus, WorktreeCopyError> = run_git.run(&ladder.first_step());
    end_step(git_add_entered);
    let settled: Result<LadderResult, WorktreeCopyError> = match ran {
        Ok(status) => {
            let finish_rung_entered: bool = begin_step("finishRung");
            let finished = ladder.finish(status, run_git, &invocation.context, &mut std::io::stdout());
            end_step(finish_rung_entered);
            Ok(finished)
        }
        Err(error) => {
            // Git did not start: the ladder only has its empty subvolume to remove, whatever status it is given.
            ladder.finish(GitStatus { code: None }, run_git, &invocation.context, &mut std::io::sink());
            Err(error)
        }
    };
    let LadderResult { git, carried } = settled?;
    let (copy_failure, created): (Option<WorktreeCopyError>, Vec<PathBuf>) =
        if git.succeeded() || invocation.destination.is_some() {
            let copy_ignored_entered: bool = begin_step("copyIgnored");
            let copy_ignored_outcome = synchronize_after(invocation, observation, registered_before, git.succeeded(), carried.as_deref(), stderr);
            end_step(copy_ignored_entered);
            match copy_ignored_outcome {
                Ok(roots) => (None, roots),
                // A copy that failed leaves a worktree that is not whole; repairing it would hide that.
                Err(failure) => (Some(failure), Vec::new()),
            }
        } else {
            (None, Vec::new())
        };
    return Ok((Settlement { git, copy_failure }, AfterLock { source_root: observation.source_root.clone(), created }));
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
    run_git: &mut dyn GitRunner,
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
    run_git: &mut dyn GitRunner,
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
    // The record of an add's steps covers a creating command the ladder serves; every other command is forwarded unrecorded.
    let recorded: bool = invocation.creates && invocation.add_plan.is_some();
    if recorded {
        begin_timing(observation.common_dir.clone());
    }
    let settled: Result<Settlement, WorktreeCopyError> = settle_observed(invocation, &observation, run_git, stderr);
    if recorded {
        finish_timing(settled.as_ref().map_or(1, exit_code));
    }
    return settled;
}

/// What:
///  The part of a forwarded command's settlement that follows the observation of its source: recovery, then, for a creating
///  command, the ladder, the copy of the ignored state and the repairs.
/// Why:
///   The caller records the steps of a creating command around this whole part, whichever way it ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await settleObserved({ invocation, observation, runGit, stderr });
/// ```
fn settle_observed(
    invocation: &Invocation,
    observation: &WorktreeCopyObservation,
    run_git: &mut dyn GitRunner,
    stderr: &mut dyn Write,
) -> Result<Settlement, WorktreeCopyError> {
    // Recovery never waits: it takes over only journals and claims whose owner is dead, and one rename picks its single winner.
    let recover_entered: bool = begin_step("recover");
    let recover_outcome = recover_pending_worktree_copies(observation.common_dir.as_path(), stderr);
    end_step(recover_entered);
    recover_outcome?;
    if !invocation.creates {
        return forward_only(run_git);
    }
    let (settlement, after) = settle_creation(invocation, observation, run_git, stderr)?;
    // No lock stands around the add: the repairs touch only the new worktree and read the source, and another add never waits
    //  for them.
    #[cfg(target_os = "linux")]
    {
        let repair_entered: bool = begin_step("repair");
        repair_created(invocation, &after, stderr);
        end_step(repair_entered);
    }
    #[cfg(not(target_os = "linux"))]
    let _after = after;
    return Ok(settlement);
}

/// Lifecycle controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_lifecycle_tests.rs"]
mod tests;
