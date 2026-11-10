//! What:
//!  `git monochromatic-git-policy-cli service settle-earlier-batches`:
//! finishes,
//! when the person asks,
//! the batches that the
//!  commit service of an earlier release left open for the worktree,
//! and the note `doctor` adds about them.
//! Why:
//!  The owner's rule is "never discard user data",
//! and the owner picked "Off,
//! settle on request" (2026-11-07):
//! the service
//!  never settles those batches by itself,
//! because nothing excludes an earlier-release wrapper that starts the earlier helper
//!  meanwhile (`earlier_service_journal_adopted.rs`).
//! This command is the one way in.
//! It acts only on the worktree it runs in
//!  (its instance's journal and the shared earlier journal's batches of that worktree),
//! checks that no earlier socket listens
//!  and no earlier helper runs,
//! says the condition it is safe under before it acts,
//! and runs the finish-only recovery.
//! It is
//!  not on the commit path,
//! so a commit never waits for it or fails for it.
//! It lives under `service` and not under `fix`
//!  because `fix` applies policies over files and demands a file scope,
//! while this works on the commit service's journals;
//!  and not under `doctor`,
//! which only reports.
//! `doctor` adds a read-only note that names this command.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await settleEarlierBatches({ selection, stateDir, git, procRoot }, conditionOut)
//! ```

use crate::action::Action;
use crate::child_environment::{child_environment_overlay, environment_value};
use crate::earlier_service_journal_adopted::{RequestedSettlement, SettledBatch, UnansweredBatch, earlier_journals_in_state, hold_on_earlier_helper, open_batch_count, settle_on_request};
use crate::earlier_service_process_look::{Hold, Holding};
use crate::earlier_service_unsure_program::UnsureProgram;
use crate::management_service_repository_selection::{Selection, status_git_dir};
use crate::service_instance::{earlier_service_unit_instance, earlier_socket_unit_instance, instance_of};
use crate::service_land::Handled;
use crate::transaction_git::GitContext;
use crate::wait_ceilings::SETTLE_LOCK_CEILING;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The command as a person types it,
/// named in every message about it.
const COMMAND: &str = "git monochromatic-git-policy-cli service settle-earlier-batches";

/// What a settling needs to know.
pub(crate) struct SettleRequest<'a> {
    /// The worktree the command runs in.
    pub selection: Selection,
    /// The state folder the earlier release's journals are in,
    /// when one is known.
    pub state_dir: Option<&'a Path>,
    /// Real Git for the recovery,
    /// when one was found.
    pub git: Option<GitContext>,
    /// The process table to look at.
    pub proc_root: &'a Path,
}

/// What:
///  The state folder of the user:
/// `XDG_STATE_HOME` when set and not empty,
/// else `$HOME/.local/state`.
/// Why:
///   The earlier unit's `StateDirectory=cli-git/service/%i` puts the journal there for a user manager (`systemd.exec(5)`,
///  `StateDirectory=`),
/// and the command runs as the same user.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stateDir = env.XDG_STATE_HOME || (env.HOME && join(env.HOME, '.local/state'))
/// ```
pub(crate) fn state_dir_of(environment: &[(OsString, OsString)]) -> Option<PathBuf> {
    if let Some(named) = environment_value(environment, "XDG_STATE_HOME")
        && !named.is_empty()
    {
        return Some(PathBuf::from(named));
    }
    return match environment_value(environment, "HOME") {
        Some(home) if !home.is_empty() => Some(PathBuf::from(home).join(".local").join("state")),
        _ => None,
    };
}

/// The noun that agrees with a count:
/// "batch" for one,
/// "batches" for any other number.
fn batch_noun(count: usize) -> &'static str {
    return if count == 1 { "batch" } else { "batches" };
}

/// The count with its noun,
/// as "1 batch" or "3 batches".
fn batches(count: usize) -> String {
    return format!("{count} {}", batch_noun(count));
}

/// The past-tense verb that agrees with a count,
/// as "was" for one and "were" for any other number.
fn was_or_were(count: usize) -> &'static str {
    return if count == 1 { "was" } else { "were" };
}

/// An exit that settled nothing,
/// with the reason on standard error.
fn refused(reason: &str) -> Action {
    return Action::Exit { code: 1, stdout: String::new(), stderr: format!("{COMMAND}: {reason}\n") };
}

/// What:
///  What to do about the cause of a hold,
/// in plain words,
/// with the units that are the cause and no others.
/// Why:
///   The look tells this worktree's own socket and helper from the single service of the first releases,
/// which serves every
///  worktree;
/// the units to stop differ,
/// and stopping the wrong ones would leave the command refusing again.
/// A listening socket
///  does not stop by waiting,
/// so the wait for an idle exit is offered only for a helper that runs with no socket listening,
///  which the look reports after the sockets.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const wayOn = (holding: Holding, instance: string): string => ...
/// ```
fn way_on(holding: &Holding, instance: &str) -> String {
    let because: Hold = holding.because;
    let own_units: String = format!("`systemctl --user stop {} {}`", earlier_socket_unit_instance(instance), earlier_service_unit_instance(instance));
    let shared_units: String = format!("`systemctl --user stop {SHARED_SOCKET_UNIT} {SHARED_SERVICE_UNIT}`");
    let again: String = format!("Then run `{COMMAND}` again.");
    return match because {
        Hold::InstanceSocketListens => format!(
            "To go on, stop this worktree's earlier units with {own_units} when no commit by the earlier release is running here (the next commit made with this release stops the socket by itself unless its service is running). {again}"
        ),
        Hold::InstanceHelperRuns => format!(
            "To go on, either stop it with {own_units} when no commit by the earlier release is running here, or wait: nothing listens that could start it again, and it exits by itself when idle, three hours after its last client. {again}"
        ),
        Hold::SharedSocketListens => format!(
            "That service serves every worktree of this account, so stopping it ends it for all of them. The next commit made with this release, in any worktree, retires it by itself: it asks the service to finish its batch and exit, then stops its units (if the service is still working after ten seconds, that commit stops and says so). Or stop it now with {shared_units} when no commit by the earlier release is running in any worktree. {again}"
        ),
        Hold::SharedHelperRuns => format!(
            "That service serves every worktree of this account. The next commit made with this release, in any worktree, retires it by itself: it asks the service to finish its batch and exit, then stops its units (if the service is still working after ten seconds, that commit stops and says so). Or stop it now with {shared_units} when no commit by the earlier release is running in any worktree, or wait: nothing listens that could start it again, and it exits by itself when idle, three hours after its last client. {again}"
        ),
        Hold::HelperUnsure => match holding.program.as_ref() {
            Some(program) => unsure_way_on(program, own_units.as_str(), shared_units.as_str(), again.as_str()),
            None => format!(
                "It is not known which one, so to go on, stop this worktree's earlier units with {own_units} and the single service of the first releases with {shared_units} (that one serves every worktree), when no commit by the earlier release is running. {again}"
            ),
        },
        Hold::CannotLook => format!("To go on, try again in a moment. If it keeps happening, this program cannot read the list of running programs and sockets on this machine, and the batches stay open. {again}"),
    };
}

/// What:
///  What to do when a program that cannot be ruled out holds the command back:
/// wait for it,
/// stop it,
/// or stop the earlier units
///  when it is one,
/// then run the command again.
/// Why:
///   The program may be an earlier commit service or an unrelated tool that happens to take a `--journal` option,
/// and this check
///  cannot tell which.
/// Stopping the earlier units ends the first kind and leaves the second running,
/// so the text says which
///  way clears which case;
/// only waiting for the program or stopping it clears both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unsureWayOn = (program: UnsureProgram, ownUnits: string, sharedUnits: string, again: string): string => ...
/// ```
fn unsure_way_on(program: &UnsureProgram, own_units: &str, shared_units: &str, again: &str) -> String {
    let pid: u32 = program.pid;
    return format!(
        "To go on, do one of these: wait for process {pid} to end; stop it yourself if it is yours to stop (for example by closing the program that started it); or, if it is an earlier commit service, stop this worktree's earlier units with {own_units} and the single service of the first releases with {shared_units} (that one serves every worktree), when no commit by the earlier release is running. Stopping those units does not end an unrelated program, so for such a program only waiting for it or stopping it clears this. {again}"
    );
}

/// What:
///  The sentences that name the program behind a hold that rests on a folder this check could not resolve,
/// or nothing for any
///  other hold.
/// Why:
///   The person must be able to tell what blocks the command:
/// the process ID and the program it runs,
/// the folder that did not
///  resolve and the system's answer,
/// and why that is not enough to rule the program out.
/// The look
///  stops at the first such program,
/// so the text says another may be named once this one is gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const programSentences = (holding: Holding): string => holding.program ? `That program is process ...` : ''
/// ```
fn program_sentences(holding: &Holding) -> String {
    let Some(program) = holding.program.as_ref() else {
        return String::new();
    };
    let pid: u32 = program.pid;
    let folder: &str = program.journal_word.as_str();
    let relative: String = if program.journal_word_is_relative { format!(" (a relative path, looked up in the working directory of process {pid})") } else { String::new() };
    return format!(
        " That program is process {pid}, running `{}` (its other arguments are not shown, because any of them can be a secret). This check looked up the folder its `--journal` option names, `{folder}`{relative}, and the system answered: {}. A folder that cannot be looked up cannot be compared with this worktree's earlier journals, so process {pid} cannot be ruled out as the earlier commit service of this worktree, and an unrelated program that takes a `--journal` option looks the same. It is the first such program this check found; another may be named once it is gone.",
        program.program, program.reason
    );
}

/// The socket unit of the single service of the first releases,
/// which served every worktree.
const SHARED_SOCKET_UNIT: &str = "cli-git.socket";

/// The service unit of the single service of the first releases.
const SHARED_SERVICE_UNIT: &str = "cli-git.service";

/// What is said before the command acts:
/// the condition it is safe under.
const CONDITION: &str = "Settling is safe only when no wrapper of the earlier release is in use in this worktree at this moment: such a wrapper sets its commit service up again and can start the earlier helper while this runs, and two services could then settle one batch. No earlier socket listens and no earlier helper runs now, so the command goes on.\n";

/// The exit for a settling that a hold stopped:
/// how many batches stay open,
/// the cause with the program behind it and the way on.
fn held_back(open: usize, instance: &str, because: Holding) -> Action {
    return Action::Exit {
        code: 1,
        stdout: String::new(),
        stderr: format!(
            "{} the earlier release's commit service left open for this worktree {} not settled, because {}.{} Nothing was changed. {}\n",
            batches(open),
            was_or_were(open),
            because.because.words(),
            program_sentences(&because),
            way_on(&because, instance)
        ),
    };
}

/// Say the safety condition on `out` and flush it,
/// so it reaches the person before anything is changed.
fn say_condition(out: &mut dyn std::io::Write) -> std::io::Result<()> {
    out.write_all(CONDITION.as_bytes())?;
    return out.flush();
}

/// Whether the settling left a batch open.
fn is_still_open(batch: &SettledBatch) -> bool {
    return batch.report.handled == Handled::StillOpen;
}

/// The Git facts of the real Git the invocation found,
/// or nothing when none was found.
fn git_context_of(environment: &[(OsString, OsString)], real_git: Option<&Path>) -> Option<GitContext> {
    let found: &Path = real_git?;
    return Some(GitContext { real_git: found.to_path_buf(), overlay: child_environment_overlay(environment, found), global_prefix: Vec::new(), common_dir: None });
}

/// A line about one batch the settling dealt with.
fn line_of(batch: &SettledBatch) -> String {
    let seq: u64 = batch.report.seq;
    let journal = batch.journal.display();
    return match batch.report.handled {
        Handled::Finished => format!("finished batch {seq} in {journal}\n"),
        Handled::StillOpen => format!("left open batch {seq} in {journal}: the index lock is held or a record cannot be read; run the command again\n"),
        Handled::Left => format!("left on disk batch {seq} in {journal}: its branch does not show it as landed, and a landing in progress writes its intent first, so the earlier helper settles it at its own start\n"),
        Handled::Dropped => format!("dropped the intent of batch {seq} in {journal}\n"),
        Handled::Held => format!("left batch {seq} in {journal} held behind a reference lock: its branch has not moved, and the next command that finds the lock gone moves it\n"),
    };
}

/// What:
///  Settles on request:
/// counts,
/// looks,
/// says the condition,
/// then runs the finish-only recovery.
///  `condition_out` is where the condition is said before anything is changed.
/// Why:
///   Every refusal changes nothing and says what to do.
/// The condition is written before the recovery starts,
/// and only when
///  the look passed,
/// so a held-back run does not claim to be acting.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const action = settleEarlierBatchesAction(request, process.stderr)
/// ```
pub(crate) fn settle_earlier_batches_action(request: &SettleRequest<'_>, condition_out: &mut dyn std::io::Write) -> Action {
    let git_dir: &Path = match &request.selection {
        Selection::Found(found) => found.as_path(),
        Selection::Outside => return refused("this is not inside a repository, so there is no worktree whose leftover batches could be settled."),
        Selection::Unresolved(reason) => return refused(format!("it is not known which worktree this invocation selects ({reason}), so nothing was settled.").as_str()),
    };
    let Some(state) = request.state_dir else {
        return refused("neither XDG_STATE_HOME nor HOME is set, so the state folder that holds the earlier release's journals is not known, and nothing was settled.");
    };
    let instance: String = instance_of(git_dir);
    let earlier = earlier_journals_in_state(state, instance.as_str());
    let open: usize = open_batch_count(&earlier, instance.as_str());
    if open == 0 {
        return Action::Exit { code: 0, stdout: String::from("No batch of the earlier release is left open for this worktree. Nothing to settle.\n"), stderr: String::new() };
    }
    if let Some(because) = hold_on_earlier_helper(&earlier, instance.as_str(), request.proc_root) {
        return held_back(open, instance.as_str(), because);
    }
    let Some(git) = request.git.as_ref() else {
        return refused("no real Git was found to finish the batches with, so nothing was settled.");
    };
    // The condition reaches the person before anything is changed; when it cannot be delivered, nothing is.
    if let Err(error) = say_condition(condition_out) {
        return refused(format!("the safety condition could not be written to standard error ({error}), so nothing was settled.").as_str());
    }
    let (settled, unanswered) = match settle_on_request(git, &earlier, instance.as_str(), request.proc_root, SETTLE_LOCK_CEILING) {
        RequestedSettlement::Settled { batches, unanswered } => (batches, unanswered),
        RequestedSettlement::Held { because, .. } => return held_back(open, instance.as_str(), because),
        RequestedSettlement::NothingOpen => return Action::Exit { code: 0, stdout: String::from("No batch of the earlier release is left open for this worktree. Nothing to settle.\n"), stderr: String::new() },
    };
    let mut lines: String = settled.iter().map(line_of).collect();
    for batch in &unanswered {
        lines.push_str(unanswered_line(batch).as_str());
    }
    let still_open: bool = settled.iter().any(is_still_open);
    let problems: bool = still_open || !unanswered.is_empty();
    return Action::Exit {
        code: i32::from(problems),
        stdout: lines,
        stderr: if problems { format!("Some batches were not settled and stay on disk; nothing was discarded. Run `{COMMAND}` again.\n") } else { String::new() },
    };
}

/// A line about a batch the recovery could not examine,
/// naming it and its journal.
fn unanswered_line(batch: &UnansweredBatch) -> String {
    return format!(
        "batch {} in {} could not be examined: Git gave no answer about its branch (its repository may be unavailable); it stays on disk, nothing was discarded\n",
        batch.seq,
        batch.journal.display()
    );
}

/// What:
///  The note `doctor` adds:
/// how many batches the earlier release's service left open for this worktree,
/// and the command that
///  settles them.
/// Empty when none are left,
/// or when the worktree or the state folder is not known.
/// Why:
///   The person learns about leftover batches without reading journals.
/// The count is read without changing a file,
/// and it may
///  include a batch that is finished already (its intent is still on disk until the recovery sees it).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// earlierBatchesNote(selection, stateDir) // '' or one line
/// ```
pub(crate) fn earlier_batches_note(selection: &Selection, state_dir: Option<&Path>) -> String {
    let (Selection::Found(git_dir), Some(state)) = (selection, state_dir) else {
        return String::new();
    };
    let instance: String = instance_of(git_dir.as_path());
    let open: usize = open_batch_count(&earlier_journals_in_state(state, instance.as_str()), instance.as_str());
    if open == 0 {
        return String::new();
    }
    return format!(
        "git monochromatic-git-policy-cli doctor: {} the earlier release's commit service left open for this worktree (the count includes any that landed and only lack their record); nothing was changed. `{COMMAND}` finishes them when no wrapper of the earlier release is in use in this worktree.\n",
        batches(open)
    );
}

/// What:
///  Plans the command from the invocation's facts:
/// the worktree Git selects,
/// the state folder,
/// real Git,
/// the live `/proc`.
/// Why:
///   The condition is said on standard error before anything is changed;
/// the result comes back as the action.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// planSettleEarlierBatches(globalArguments, environment, realGit)
/// ```
pub(crate) fn plan_settle_earlier_batches(global_arguments: &[OsString], environment: &[(OsString, OsString)], real_git: Option<&Path>) -> Action {
    let state: Option<PathBuf> = state_dir_of(environment);
    let git: Option<GitContext> = git_context_of(environment, real_git);
    let request: SettleRequest<'_> = SettleRequest {
        selection: status_git_dir(global_arguments, environment, real_git),
        state_dir: state.as_deref(),
        git,
        proc_root: Path::new("/proc"),
    };
    return settle_earlier_batches_action(&request, &mut std::io::stderr());
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "management_service_settle_earlier_batches_tests.rs"]
mod tests;
