//! What:
//!  `git monochromatic-git-policy-cli service settle-earlier-batches`: finishes, when the person asks, the batches that the
//!  commit service of an earlier release left open for the worktree, and the note `doctor` adds about them.
//! Why:
//!  The owner's rule is "never discard user data", and the owner picked "Off, settle on request" (2026-11-07): the service
//!  never settles those batches by itself, because nothing excludes an earlier-release wrapper that starts the earlier helper
//!  meanwhile (`earlier_service_journal_adopted.rs`). This command is the one way in. It acts only on the worktree it runs in
//!  (its instance's journal and the shared earlier journal's batches of that worktree), checks that no earlier socket listens
//!  and no earlier helper runs, says the condition it is safe under before it acts, and runs the finish-only recovery. It is
//!  not on the commit path, so a commit never waits for it or fails for it. It lives under `service` and not under `fix`
//!  because `fix` applies policies over files and demands a file scope, while this works on the commit service's journals;
//!  and not under `doctor`, which only reports. `doctor` adds a read-only note that names this command.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await settleEarlierBatches({ selection, stateDir, git, procRoot }, conditionOut)
//! ```

use crate::action::Action;
use crate::child_environment::{child_environment_overlay, environment_value};
use crate::earlier_service_journal_adopted::{RequestedSettlement, SettledBatch, earlier_journals_in_state, hold_on_earlier_helper, open_batch_count, settle_on_request};
use crate::earlier_service_process_look::Hold;
use crate::management_service_repository_selection::{Selection, status_git_dir};
use crate::service_instance::{earlier_service_unit_instance, earlier_socket_unit_instance, instance_of};
use crate::service_land::Handled;
use crate::transaction_git::GitContext;
use crate::wait_ceilings::LOCK_CEILING;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The command as a person types it, named in every message about it.
const COMMAND: &str = "git monochromatic-git-policy-cli service settle-earlier-batches";

/// What a settling needs to know.
pub(crate) struct SettleRequest<'a> {
    /// The worktree the command runs in.
    pub selection: Selection,
    /// The state folder the earlier release's journals are in, when one is known.
    pub state_dir: Option<&'a Path>,
    /// Real Git for the recovery, when one was found.
    pub git: Option<GitContext>,
    /// The process table to look at.
    pub proc_root: &'a Path,
}

/// What:
///  The state folder of the user: `XDG_STATE_HOME` when set and not empty, else `$HOME/.local/state`.
/// Why:
///   The earlier unit's `StateDirectory=cli-git/service/%i` puts the journal there for a user manager (`systemd.exec(5)`,
///  `StateDirectory=`), and the command runs as the same user.
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

/// "1 batch" or "3 batches".
fn batches(count: usize) -> String {
    return if count == 1 { String::from("1 batch") } else { format!("{count} batches") };
}

/// An exit that settled nothing, with the reason on standard error.
fn refused(reason: &str) -> Action {
    return Action::Exit { code: 1, stdout: String::new(), stderr: format!("{COMMAND}: {reason}\n") };
}

/// What to do about an earlier helper that may run or start, in plain words.
fn way_on(instance: &str) -> String {
    return format!(
        "To go on, stop the earlier helper when no commit by the earlier release is running (`systemctl --user stop {} {}`), or wait for it to exit by itself, which it does when idle, three hours after its last client. Then run `{COMMAND}` again.",
        earlier_socket_unit_instance(instance),
        earlier_service_unit_instance(instance)
    );
}

/// What is said before the command acts: the condition it is safe under.
const CONDITION: &str = "Settling is safe only when no wrapper of the earlier release is in use in this worktree at this moment: such a wrapper sets its commit service up again and can start the earlier helper while this runs, and two services could then settle one batch. No earlier socket listens and no earlier helper runs now, so the command goes on.\n";

/// A line about one batch the settling dealt with.
fn line_of(batch: &SettledBatch) -> String {
    let seq: u64 = batch.report.seq;
    let journal = batch.journal.display();
    return match batch.report.handled {
        Handled::Finished => format!("finished batch {seq} in {journal}\n"),
        Handled::StillOpen => format!("left open batch {seq} in {journal}: the index lock is held or a record cannot be read; run the command again\n"),
        Handled::Left => format!("left on disk batch {seq} in {journal}: its branch does not show it as landed, and a landing in progress writes its intent first, so the earlier helper settles it at its own start\n"),
        Handled::Dropped => format!("dropped the intent of batch {seq} in {journal}\n"),
    };
}

/// What:
///  Settles on request: counts, looks, says the condition, then runs the finish-only recovery.
///  `condition_out` is where the condition is said before anything is changed.
/// Why:
///   Every refusal changes nothing and says what to do. The condition is written before the recovery starts, and only when
///  the look passed, so a held-back run does not claim to be acting.
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
    let held = |because: Hold| -> Action {
        return Action::Exit {
            code: 1,
            stdout: String::new(),
            stderr: format!("{} the earlier release's commit service left open for this worktree were not settled, because {}. Nothing was changed. {}\n", batches(open), because.words(), way_on(instance.as_str())),
        };
    };
    if let Some(because) = hold_on_earlier_helper(&earlier, instance.as_str(), request.proc_root) {
        return held(because);
    }
    let Some(git) = request.git.as_ref() else {
        return refused("no real Git was found to finish the batches with, so nothing was settled.");
    };
    let _said = condition_out.write_all(CONDITION.as_bytes());
    let _flushed = condition_out.flush();
    let settled = match settle_on_request(git, &earlier, instance.as_str(), request.proc_root, LOCK_CEILING) {
        RequestedSettlement::Settled(batches) => batches,
        RequestedSettlement::Held { because, .. } => return held(because),
        RequestedSettlement::NothingOpen => return Action::Exit { code: 0, stdout: String::from("No batch of the earlier release is left open for this worktree. Nothing to settle.\n"), stderr: String::new() },
    };
    if settled.is_empty() {
        return Action::Exit {
            code: 1,
            stdout: String::new(),
            stderr: format!("{} could not be settled now: Git gave no answer about them, so nothing was discarded and they stay open. Run `{COMMAND}` again.\n", batches(open)),
        };
    }
    let lines: String = settled.iter().map(line_of).collect();
    let still_open: bool = settled.iter().any(|batch| { return batch.report.handled == Handled::StillOpen; });
    return Action::Exit {
        code: i32::from(still_open),
        stdout: lines,
        stderr: if still_open { format!("Some batches stay open; run `{COMMAND}` again.\n") } else { String::new() },
    };
}

/// What:
///  The note `doctor` adds: how many batches the earlier release's service left open for this worktree, and the command that
///  settles them. Empty when none are left, or when the worktree or the state folder is not known.
/// Why:
///   The person learns about leftover batches without reading journals. The count is read without changing a file, and it may
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
///  Plans the command from the invocation's facts: the worktree Git selects, the state folder, real Git, the live `/proc`.
/// Why:
///   The condition is said on standard error before anything is changed; the result comes back as the action.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// planSettleEarlierBatches(globalArguments, environment, realGit)
/// ```
pub(crate) fn plan_settle_earlier_batches(global_arguments: &[OsString], environment: &[(OsString, OsString)], real_git: Option<&Path>) -> Action {
    let state: Option<PathBuf> = state_dir_of(environment);
    let git: Option<GitContext> = real_git.map(|found| {
        return GitContext { real_git: found.to_path_buf(), overlay: child_environment_overlay(environment, found), global_prefix: Vec::new(), common_dir: None };
    });
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
