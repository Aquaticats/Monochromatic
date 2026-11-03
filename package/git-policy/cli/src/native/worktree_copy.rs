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
//!  A main-checkout source is observed only when the command creates or moves a worktree; it then hands over its ignored
//!  state under the main checkout's leave-out rule (`worktree_copy_prune.rs`), with its registered worktrees excluded.
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
use super::worktree_copy_selection::CopySelection;
use super::worktree_copy_registry::{
    CreatedWorktrees, WorktreeCopyObservation, find_created_worktrees, observe_copy_source,
};
use super::worktree_copy_transaction::synchronize_created_worktree_noting;
use super::worktree_identity::WorktreeIdentity;
use super::worktree_add_plan::AddPlan;
use super::worktree_ladder::{LadderResult, begin_ladder};
#[cfg(target_os = "linux")]
use super::package_repair::{RepairEnvironment, RepairRequest, RepairSwitch, repair_worktree};
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

/// Runs one real Git step, with the given lease token (absent when the run inherits none).
pub type GitRunner<'a> = dyn FnMut(Option<&str>, &GitStep<'_>) -> Result<GitStatus, WorktreeCopyError> + 'a;

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
    let git: GitStatus = run_git(None, &GitStep::Person)?;
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
///  Synchronizes every worktree the command registered and writes the summary line for them; the roots of every created
///  worktree, a snapshot's included, come back for the package-manager repair.
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
    carried: Option<&Path>,
    stderr: &mut dyn Write,
) -> Result<Vec<PathBuf>, WorktreeCopyError> {
    // A main checkout that names no path hands over nothing: no stage, no journal and no summary line.
    if matches!(&observation.selection, CopySelection::Paths(paths) if paths.is_empty()) {
        return Ok(carried.map(Path::to_path_buf).into_iter().collect());
    }
    let mut found: CreatedWorktrees = find_created_worktrees(context, observation)?;
    let created_roots: Vec<PathBuf> = found.created.iter().map(|worktree| { return worktree.root.clone(); }).collect();
    // A destination that received the source's ignored state with a snapshot needs no copy.
    found.created.retain(|worktree| { return carried != Some(worktree.root.as_path()); });
    if found.created.is_empty() {
        return Ok(created_roots);
    }
    let mut copied: usize = 0;
    // `match` on the source: a bare repository has nothing to copy, so only a worktree source is copied.
    match observation.source_root.as_ref() {
        None => {}
        Some(source) => {
            for destination in &found.created {
                copied += synchronize_created_worktree_noting(
                    context,
                    observation.common_dir.as_path(),
                    source.as_path(),
                    destination.root.as_path(),
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
    return Ok(created_roots);
}

/// What:
///  What is left to do once the settlement lock is released: the source the ignored files came from and the worktrees the
///  command created.
/// Why:
///   The package-manager repairs touch only the new worktrees and read the source, so no other add has to wait for them; they
///  run after the release, with the facts the locked part found.
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
///  It runs after the settlement lock is released, so a slow tool never holds another add.
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
) -> Result<(Settlement, AfterLock), WorktreeCopyError> {
    let refreshed: Option<WorktreeCopyObservation> = observe_copy_source(
        &identity_after_lock()?,
        invocation.effective_cwd.as_path(),
    )?;
    let observation: WorktreeCopyObservation = match refreshed {
        Some(observed) if observed.common_dir == first.common_dir => observed,
        _ => return Err(repository_changed()),
    };
    let report = recover_worktree_copy_transactions(observation.common_dir.as_path())?;
    report_recovered(&report, stderr)?;
    let ladder = begin_ladder(invocation.add_plan.as_ref(), &invocation.context, &invocation.identity);
    let ran: Result<GitStatus, WorktreeCopyError> = run_git(Some(lease_token), &ladder.first_step());
    let settled: Result<LadderResult, WorktreeCopyError> = match ran {
        Ok(status) => Ok(ladder.finish(status, Some(lease_token), run_git, &invocation.context, &mut std::io::stdout())),
        Err(error) => {
            // Git did not start: the ladder only has its empty subvolume to remove, whatever status it is given.
            ladder.finish(GitStatus { code: None }, Some(lease_token), run_git, &invocation.context, &mut std::io::sink());
            Err(error)
        }
    };
    let LadderResult { git, carried } = settled?;
    let (copy_failure, created): (Option<WorktreeCopyError>, Vec<PathBuf>) =
        match synchronize_after(&invocation.context, &observation, carried.as_deref(), stderr) {
            Ok(roots) => (None, roots),
            // A copy that failed leaves a worktree that is not whole; repairing it would hide that.
            Err(failure) => (Some(failure), Vec::new()),
        };
    return Ok((Settlement { git, copy_failure }, AfterLock { source_root: observation.source_root, created }));
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
    let settled: Result<(Settlement, AfterLock), WorktreeCopyError> = settle_under_lock(
        invocation,
        &observation,
        run_git,
        identity_after_lock,
        lease.lease_token(),
        stderr,
    );
    lease.release()?;
    let (settlement, after) = settled?;
    // The lock is released: the repairs touch only the new worktrees, so another add never waits for them.
    #[cfg(target_os = "linux")]
    repair_created(invocation, &after, stderr);
    #[cfg(not(target_os = "linux"))]
    let _after = after;
    return Ok(settlement);
}

/// Lifecycle controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_lifecycle_tests.rs"]
mod tests;
