//! What:
//!  The linked-worktree ignored-state copy around one forwarded Git invocation:
//! the public entry point the
//!  integrator calls,
//! the settlement and summary it writes,
//! and the exit status it returns.
//! Why:
//!  A worktree that Git creates (or a bare repository's) needs its ignored files from the source worktree.
//!  A snapshot of the main checkout also leaves directories out of the new worktree,
//! and those are emptied here beside the
//!  package repair instead of before it.
//!  No lock stands around the command:
//! the destination comes from the command line,
//! the copy is a transaction of its own
//!  (a journal and a stage claim that name their owner process),
//! and a copy a dead process left is recovered first by a command in the worktree it fills,
//!  under a ceiling, and never by a command in another worktree.
//! Concurrent adds in one repository never wait for one another (the owner's decision of 2026-11-03).
//!
//! Integrator contract:
//!  The caller decides the bypass facts (`--no-worktree-copy`,
//! short-circuit global help or version,
//! whether the
//!  command creates a worktree after alias resolution,
//! and the destination its command line names) with the native
//!  modules that already own them,
//! passes the repository identity and the effective working directory,
//! and supplies the
//!  runner that starts real Git.
//! This module never runs the commit refusal;
//! it is not wired yet.
//!  A main-checkout source is observed only when the command creates a worktree;
//! it then hands over its ignored
//!  state under the main checkout's leave-out rule (`worktree_copy_prune.rs`),
//! with its registered worktrees excluded.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await runGitWithWorktreeCopy({ args, gitPath, identity, command, environment });
//! ```

use crate::add_timing::{begin as begin_timing, begin_step, end_step, finish as finish_timing};
use crate::diagnostic_log::warn;
use crate::forwarding::{ChildStarted, NoChildRecorded};
use crate::json_record::quote;
use crate::transaction_git::GitContext;
use crate::worktree_copy_after_refusal::settle_refused_destination;
use crate::worktree_copy_error::{WorktreeCopyError, lifecycle_failure};
use crate::worktree_copy_recovery::{RecoveryDestinations, recover_pending_worktree_copies};
use crate::worktree_copy_selection::CopySelection;
use crate::worktree_copy_owed_copy::owed_selection_of;
use crate::worktree_copy_stage_claim::{StageClaim, claim_stage_for_creation};
use crate::worktree_copy_notices::nothing_but_a_reservation;
use crate::worktree_copy_registry::{WorktreeCopyObservation, destination_is_registered, observe_copy_source};
use crate::worktree_copy_synchronize::synchronize_after;
use crate::worktree_add_binding::OwedCopy;
use crate::worktree_identity::WorktreeIdentity;
use crate::worktree_add_plan::AddPlan;
use crate::worktree_ladder::{LadderResult, begin_ladder_holding};
use crate::worktree_subvolume::{CreatedSubvolume, Reservation, reserve_destination};
#[cfg(target_os = "linux")]
use crate::package_repair::RepairSwitch;
#[cfg(target_os = "linux")]
use crate::worktree_trash::PendingTrash;
pub(crate) use crate::worktree_copy_after_lock::{AfterLock, finish_trash_and_repair};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeCopy";

/// What:
///  How a real Git run ended.
/// Why:
///   Its numeric status is the exit status the wrapper keeps;
/// a signal has none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ForwardedGitExecution = { failure?: SubprocessError };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GitStatus {
    /// The exit code,
    /// absent when Git ended by a signal.
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
    /// The destination the creating command's line names,
    /// in the form Git records it;
    /// absent when the line has a form that is
    ///  not read,
    /// in which case no copy is made and the command says so.
    pub destination: Option<PathBuf>,
    /// Whether `--no-worktree-copy` appeared in flag position.
    pub opted_out: bool,
    /// Whether real Git only prints help or its version.
    pub short_circuit: bool,
    /// The real Git executable and the variables every Git child receives.
    pub context: GitContext,
    /// The destination of a `git worktree add` in a form the cheap-worktree ladder serves,
    /// when it is one.
    pub add_plan: Option<AddPlan>,
    /// The person's `--quiet` and `--no-checkout`,
    /// read from the arguments,
    /// which decide what the package-manager repair of
    ///  a new worktree may print and whether it runs.
    #[cfg(target_os = "linux")]
    pub repair: RepairSwitch,
}

/// What:
///  The result of one settled invocation:
/// Git's status and the copy failure,
/// if any.
/// Why:
///   Both facts decide the exit status,
/// so they travel together.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a resolved run, or a ForwardedGitWorktreeCopyError carrying both facts
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Settlement {
    /// How real Git ended.
    pub git: GitStatus,
    /// The ignored-state copy failure,
    /// when copying failed.
    pub copy_failure: Option<WorktreeCopyError>,
}

/// What:
///  Which Git run the settlement asks its runner to start.
/// Why:
///   The person's command is one run;
/// the cheap-worktree ladder adds `--no-checkout` to it and starts the commands that
///  complete the worktree itself,
/// each with the caller's streams.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GitStep = { kind: 'person' } | { kind: 'without-checkout'; insertAt: number } | { kind: 'other'; args: string[]; env: [string, string][] };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GitStep<'a> {
    /// The person's arguments,
    /// unchanged.
    Person,
    /// The person's arguments with `--no-checkout` inserted at this index.
    WithoutCheckout(usize),
    /// Another Git command with these arguments and these variables added.
    Other {
        /// The arguments,
        /// global options and subcommand included.
        arguments: &'a [std::ffi::OsString],
        /// The variables added to the run's environment.
        environment: &'a [(std::ffi::OsString, std::ffi::OsString)],
    },
}

/// What:
///  Runs one real Git step.
///  A `trait` is a named set of methods that different types can each supply,
/// like a TypeScript `interface`.
/// Why:
///   The ladder calls it once per Git step;
/// the state a step needs (the invocation,
/// the real Git,
/// the overlay,
/// the parallel-checkout
///  decision) is a small struct that implements this.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface GitRunner { run(step: GitStep): GitStatus } // throws WorktreeCopyError
/// ```
pub trait GitRunner {
    /// Run `step` and report how Git ended,
    /// naming the process number of the Git it started to `child_started` as soon as it has one,
    /// so the add's record can say which Git may still be at work if this command is killed.
    fn run(&mut self, step: &GitStep<'_>, child_started: &mut dyn ChildStarted) -> Result<GitStatus, WorktreeCopyError>;
}

/// What:
///  The one summary line a successful copy writes.
/// Why:
///   Users and agents read the count,
/// the source,
/// and the destination count from it.
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
///   A copy-only failure is 2;
/// Git's own status is kept when Git failed;
/// a signal without a status is 1.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // copy failure 2, Git status kept, signal 1
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
///  Runs one forwarded invocation with no synchronization,
/// and settles its status.
/// Why:
///   Every bypass forwards exactly the same way,
/// so the runner is called once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await forwardOnly({ args, gitPath, environment });
/// ```
fn forward_only(run_git: &mut dyn GitRunner) -> Result<Settlement, WorktreeCopyError> {
    // Nothing is copied here, so there is no record for the child's name.
    let git: GitStatus = run_git.run(&GitStep::Person, &mut NoChildRecorded)?;
    return Ok(Settlement {
        git,
        copy_failure: None,
    });
}

/// What:
///  The record of the Git a creating command started:
///  the claim the command made before Git ran,
///  and the destination its Git fills.
/// Why:
///   The run reports the child's number while that child is still unreaped,
///  and the only thing to do with the number is name it in the claim;
///  so the claim and the destination are the report's own fields
///  instead of closures over them.
///  Both are absent when the command recorded nothing or named no destination,
///  and the report then does nothing.
struct ChildOfACreatingCommand<'a> {
    /// The claim made before Git ran,
    /// when the command made one.
    claim: Option<&'a StageClaim>,
    /// The destination the person's command line named.
    destination: Option<&'a Path>,
}

impl ChildStarted for ChildOfACreatingCommand<'_> {
    /// Name the Git in the claim,
    /// when the command has both a claim and a destination.
    fn started(&mut self, pid: u32) {
        if let (Some(claim), Some(destination)) = (self.claim, self.destination) {
            claim.remember_running_child(destination, pid);
        }
    }
}

/// What:
///  Runs one creating command:
/// whether its destination was registered already,
/// the ladder and real Git,
/// and the copy.
/// Why:
///   Nothing here waits for another command:
/// the destination,
/// the stage claim and the journal belong to this command alone,
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
    stderr: &mut (dyn Write + Send),
) -> Result<(Settlement, AfterLock), WorktreeCopyError> {
    let registered_before: bool = match invocation.destination.as_deref() {
        Some(destination) => destination_is_registered(observation.common_dir.as_path(), destination),
        None => false,
    };
    let choose_rung_entered: bool = begin_step("chooseRung");
    // The record comes before real Git runs, so a command killed after Git returned still leaves a record of the worktree
    // it made; a command that will copy nothing records nothing. A destination that is registered already is no worktree
    // this command will make, so a record naming it would tell every other command that worktree is still being filled.
    let copies: bool = !registered_before
        && observation.source_root.is_some()
        && !matches!(&observation.selection, CopySelection::Paths(paths) if paths.is_empty());
    let record_destination_entered: bool = begin_step("recordDestination");
    // The record names the source and the selection as well as the destination, so a command killed before its copy ran
    // leaves everything the next command needs to make that copy itself. A selection of named paths is not one of the
    // two listings a record can name, so such a command records nothing and its worktree is named instead.
    let owed: Option<OwedCopy> = observation.source_root.as_ref().and_then(|source| {
        return owed_selection_of(&observation.selection).map(|selection| { return OwedCopy { source: source.clone(), selection }; });
    });
    let recorded = claim_stage_for_creation(observation.common_dir.as_path(), invocation.destination.as_deref(), copies, owed.as_ref());
    end_step(record_destination_entered);
    let mut held: Option<StageClaim> = match recorded {
        Ok(found) => found,
        Err(error) => {
            warn(TAG, format!("the worktree add could not be recorded durably ({}); an interrupted copy into it will not be reported", error.0).as_str());
            None
        }
    };
    let ladder = begin_ladder_holding(invocation.add_plan.as_ref(), &mut held, &invocation.context, &invocation.identity);
    // Who owns the destination is decided here, before real Git runs, in one exclusive creation: the command whose
    // directory Git fills copies into it, whatever its own Git then returns, and every command that finds the directory
    // there copies nothing. A rung that already made its destination settles it itself, so its identity is not reserved
    // again here.
    let reserve_destination_entered: bool = begin_step("reserveDestination");
    // A rung may have made the destination already, as an empty subvolume Git fills: that object is this command's too,
    // and its rung settles it, so it is carried forward rather than reserved again. Otherwise one exclusive creation
    // decides the owner, and no other command's Git can be the one whose worktree this command fills.
    let mut made_here: bool = false;
    let reserved_before_git: Reservation = match (copies, ladder.destination_identity()) {
        (true, Some(identity)) => match invocation.destination.as_deref() {
            Some(destination) => Reservation::Mine(CreatedSubvolume { path: destination.to_path_buf(), identity: Some(identity) }),
            None => Reservation::NotAttempted,
        },
        (true, None) => match invocation.destination.as_deref() {
            Some(destination) => {
                let reservation: Reservation = reserve_destination(destination);
                made_here = matches!(reservation, Reservation::Mine(_));
                reservation
            }
            None => Reservation::NotAttempted,
        },
        _ => Reservation::NotAttempted,
    };
    end_step(reserve_destination_entered);
    end_step(choose_rung_entered);
    let git_add_entered: bool = begin_step("gitAdd");
    // The record names the Git the person's command started, so a command that runs after this one was killed can tell
    //  an add whose Git is still making the destination from one whose Git is gone.
    let naming_destination: Option<&Path> = invocation.destination.as_deref();
    let ran: Result<GitStatus, WorktreeCopyError> = run_git.run(
        &ladder.first_step(),
        &mut ChildOfACreatingCommand {
            claim: held.as_ref(),
            destination: naming_destination,
        },
    );
    end_step(git_add_entered);
    let settled: Result<LadderResult, WorktreeCopyError> = match ran {
        Ok(status) => {
            let finish_rung_entered: bool = begin_step("finishRung");
            // The line Git's checkout prints goes to the standard output, and every note about what the add did instead goes to the stream this command was given, so a caller that hands in its own report reads the whole of it.
            let finished = ladder.finish(status, run_git, &invocation.context, &mut std::io::stdout(), stderr);
            end_step(finish_rung_entered);
            Ok(finished)
        }
        Err(error) => {
            // Git did not start: the ladder only has its empty subvolume to remove, whatever status it is given.
            ladder.finish(GitStatus { code: None }, run_git, &invocation.context, &mut std::io::sink(), stderr);
            Err(error)
        }
    };
    let finished: LadderResult = settled?;
    let git: GitStatus = finished.git;
    let carried: Option<PathBuf> = finished.carried;
    // A rung that served the add swapped its own snapshot into the destination, so the directory Git made is not there
    // any more and the object at the path is the one the rung made. The reservation names that object, or every later
    // question about whose destination it is compares against an object this command moved away itself and answers, wrongly,
    // that another command replaced it (a service of a snapshot of the main checkout would report no ignored-state copy and
    // would leave its copy record behind).
    let reserved: Reservation = match (finished.placed, invocation.destination.as_deref()) {
        (Some(placed), Some(destination)) => Reservation::Mine(CreatedSubvolume { path: destination.to_path_buf(), identity: Some(placed) }),
        _ => reserved_before_git,
    };
    #[cfg(target_os = "linux")]
    let trash: Option<PendingTrash> = finished.trash;
    // The claim the ladder took is handed back here, not released: the copy owns it until its journal names the stage,
    // so no command killed between the end of the checkout and that journal can leave a worktree whose ignored state was
    // never copied with no record naming it.
    let handed_over: Option<StageClaim> = finished.claim;
    // A snapshot that served the add swapped a subvolume into place, so its record is settled without a flush.
    let swapped: bool = carried.is_some();
    if handed_over.is_some() {
        held = handed_over;
    }
    // Set when the copy deliberately left the destination unfilled, as a source that is still being filled does.
    let mut left_unfilled: bool = false;
    // Set when the destination is another command's worktree, which carries its proof.
    let mut another_commands_worktree: bool = false;
    let (copy_failure, created): (Option<WorktreeCopyError>, Vec<PathBuf>) =
        if git.succeeded() || invocation.destination.is_some() {
            let copy_ignored_entered: bool = begin_step("copyIgnored");
            let copy_ignored_outcome = synchronize_after(invocation, observation, registered_before, copies, &reserved, git.succeeded(), carried.as_deref(), &mut held, &mut left_unfilled, &mut another_commands_worktree, stderr);
            end_step(copy_ignored_entered);
            match copy_ignored_outcome {
                Ok(roots) => (None, roots),
                // A copy that failed leaves a worktree that is not whole; repairing it would hide that.
                Err(failure) => (Some(failure), Vec::new()),
            }
        } else {
            (None, Vec::new())
        };
    // A destination this command made and real Git refused is removed again while it is still the same object and still
    // empty, so a refused add leaves no empty folder inside the enclosing checkout for a later Git command to act on as if
    // the enclosing checkout were that worktree (the owner met four of them on 2026-11-10 with `git worktree add -b
    // <branch> <path> main`). `rmdir` cannot remove a directory that holds anything, so a destination another program
    // wrote into stays with everything in it, and the person is told which path this command left there. Nothing is
    // removed while the copy failed: what stands at the path is not known then.
    if made_here && copy_failure.is_none() && !git.succeeded() && let Reservation::Mine(made) = &reserved {
        // The person's stream carries this line, and a closed one keeps the status Git gave.
        if let Some(line) = settle_refused_destination(made) {
            let _written = stderr.write_all(line.as_bytes());
        }
    }
    // A record whose copy did not run (a destination Git did not register, a snapshot that carried the state itself) is
    // settled here: the add is whole, so the record says so before it goes. A copy that failed is not that: the worktree
    // was never filled, so the record stays and the next command reports it as an interrupted add.
    // A destination that still exists after a Git that failed is a worktree this command asked for and did not fill: the
    // record stays, so a later command reports it instead of finding a worktree that looks whole.
    /// Whether the path may hold something:
    /// absent is the only answer that says it does not,
    /// so a path this command
    /// may not look at keeps the record rather than losing it.
    fn may_hold_something(path: &Path) -> bool {
        return !matches!(
            std::fs::symlink_metadata(path),
            Err(ref error) if error.kind() == std::io::ErrorKind::NotFound
        );
    }
    /// A destination that holds anything other than this command's empty reservation:
    /// a worktree that was not filled.
    fn holds_a_worktree(path: &Path) -> bool {
        return may_hold_something(path) && !nothing_but_a_reservation(path);
    }
    let unfilled: bool = left_unfilled
        || (copies
            && !another_commands_worktree
            && !git.succeeded()
            && invocation
                .destination
                .as_deref()
                .is_some_and(holds_a_worktree));
    if copy_failure.is_none()
        && !unfilled
        && let Some(unused) = held.take()
    {
        let release_unused_entered: bool = begin_step("releaseClaim");
        let released: Result<(), WorktreeCopyError> = if swapped { unused.settle_unsynced() } else { unused.settle() };
        end_step(release_unused_entered);
        if let Err(error) = released {
            warn(TAG, format!("the record of the worktree add could not be removed ({})", error.0).as_str());
        }
    }
    #[cfg(target_os = "linux")]
    return Ok((Settlement { git, copy_failure }, AfterLock { source_root: observation.source_root.clone(), created, trash }));
    #[cfg(not(target_os = "linux"))]
    return Ok((Settlement { git, copy_failure }, AfterLock { source_root: observation.source_root.clone(), created }));
}

/// What:
///  Settles one forwarded invocation:
/// bypasses forward untouched;
/// every other command first recovers what dead owners left,
///  and a creating command then copies the ignored state into the worktree it created.
/// Why:
///   The integrator calls this once per forwarded command and turns the settlement into the exit status with [`exit_code`].
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runGitWithWorktreeCopy({ args, gitPath, identity, command, environment });
/// ```
///
/// The stream is also the one the disposal of a snapshot's trash writes to from a thread of its own,
/// so it is asked
///  for what a thread may carry.
pub fn run_with_worktree_copy(
    invocation: &Invocation,
    run_git: &mut dyn GitRunner,
    stderr: &mut (dyn Write + Send),
) -> Result<Settlement, WorktreeCopyError> {
    return settle_invocation(invocation, run_git, stderr).map_err(lifecycle_failure);
}

/// What:
///  The settlement of one invocation,
/// before the lifecycle words its raw failures.
/// Why:
///   Only the top of the lifecycle words raw failures,
/// as the incumbent's `asWorktreeCopyError` does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // body of runGitWithWorktreeCopy
/// ```
fn settle_invocation(
    invocation: &Invocation,
    run_git: &mut dyn GitRunner,
    stderr: &mut (dyn Write + Send),
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
    // An add that is still running after a few seconds says which step it is in, on a thread of its own, so the wait is
    // never silent and the add never waits for the line (owner: "nothing should ever wait forever or indefinitely").
    let standing: Option<crate::add_standing_notice::StandingNotice> = if recorded { Some(crate::add_standing_notice::begin()) } else { None };
    let settled: Result<Settlement, WorktreeCopyError> = settle_observed(invocation, &observation, run_git, stderr);
    if recorded {
        finish_timing(settled.as_ref().map_or(1, exit_code));
    }
    drop(standing);
    return settled;
}

/// What:
///  The part of a forwarded command's settlement that follows the observation of its source:
/// recovery,
/// then,
/// for a creating
///  command,
/// the ladder,
/// the copy of the ignored state and the repairs.
/// Why:
///   The caller records the steps of a creating command around this whole part,
/// whichever way it ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await settleObserved({ invocation, observation, runGit, stderr });
/// ```
fn settle_observed(
    invocation: &Invocation,
    observation: &WorktreeCopyObservation,
    run_git: &mut dyn GitRunner,
    stderr: &mut (dyn Write + Send),
) -> Result<Settlement, WorktreeCopyError> {
    // Recovery never waits: it takes over only journals and claims whose owner is dead, and one rename picks its single winner.
    // A copy is finished only into the worktree this command runs in, so no command pays for another worktree's ignored
    // state ("Agents should never wait on each other on commits or merges").
    let destinations: RecoveryDestinations<'_> = match crate::worktree_identity::worktree_root(&invocation.identity) {
        Some(root) => RecoveryDestinations::OnlyThisWorktree(root),
        None => RecoveryDestinations::NoWorktree,
    };
    let recover_entered: bool = begin_step("recover");
    let recover_outcome = recover_pending_worktree_copies(observation.common_dir.as_path(), destinations, &invocation.context, stderr);
    end_step(recover_entered);
    recover_outcome?;
    if !invocation.creates {
        return forward_only(run_git);
    }
    let (settlement, after) = settle_creation(invocation, observation, run_git, stderr)?;
    // No lock stands around the add: the repairs touch only the new worktree and read the source, and another add never waits
    //  for them.
    #[cfg(target_os = "linux")]
    finish_trash_and_repair(invocation, &after, stderr);
    #[cfg(not(target_os = "linux"))]
    let _after = after;
    return Ok(settlement);
}

/// Lifecycle controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_lifecycle_tests.rs"]
mod tests;
