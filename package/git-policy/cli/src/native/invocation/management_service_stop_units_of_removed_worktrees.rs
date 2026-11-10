//! What:
//! `git monochromatic-git-policy-cli service stop-units-of-removed-worktrees`:
//! stops,
//! when the person asks,
//! the commit service units of every worktree that no longer exists,
//! and the note `doctor` adds about them.
//! Why:
//! Removing a worktree used to leave its socket unit listening and its service unit running
//! (`service_units_of_removed_worktrees.rs`);
//! the wrapper's own removals now stop them,
//! and this command stops what an earlier release,
//! a removal Git ran with the wrapper replaced (`--force`,
//! `prune`),
//! or a plain `rm` left.
//! It follows `service settle-earlier-batches`:
//! a command the person runs,
//! never something a commit waits for,
//! and `doctor` only reports a count and names this command.
//! It stops units only for a worktree whose Git directory is missing while the repository that held it is there,
//! never for an instance that maps to nothing,
//! and only through the user manager.
//! A stopped instance's journal goes with its note and its socket file when it holds nothing but the records of the
//! batches that landed;
//! a journal that holds work is kept and named,
//! never discarded.
//! Nothing here starts a unit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await stopUnitsOfRemovedWorktrees({ runtimeDir, stateDir }, manager)
//! ```

use crate::action::Action;
use crate::child_environment::environment_value;
use crate::management_service_settle_earlier_batches::state_dir_of;
use crate::service_provision_host_and_manager::Manager;
use crate::service_units_of_removed_worktrees::{SweepOutcome, UnitFolders, UnitWorktree, stop_units_of_gone_worktrees, survey, worktree_of};
use crate::systemd_dbus::{Bus, EXCHANGE_WAIT, session_bus_path};
use std::ffi::OsString;
use std::path::PathBuf;

/// The command as a person types it,
/// named in every message about it.
const COMMAND: &str = "git monochromatic-git-policy-cli service stop-units-of-removed-worktrees";

/// The user's runtime directory the environment names,
/// nothing when it is unset or empty.
fn runtime_dir_of(environment: &[(OsString, OsString)]) -> Option<PathBuf> {
    let value: OsString = environment_value(environment, "XDG_RUNTIME_DIR")?;
    if value.is_empty() {
        return None;
    }
    return Some(PathBuf::from(value));
}

/// The worktree an instance belongs to,
/// as a path for a line of output.
fn path_of(folders: &UnitFolders<'_>, instance: &str) -> String {
    return match worktree_of(folders, instance) {
        UnitWorktree::Known(git_dir) => git_dir.display().to_string(),
        UnitWorktree::Unmapped => String::from(UNKNOWN_WORKTREE),
    };
}

/// What a line says about a worktree the units of which could not be named.
const UNKNOWN_WORKTREE: &str = "an unknown worktree";

/// The count with its noun,
/// as "1 removed worktree" or "3 removed worktrees".
fn worktrees(count: usize) -> String {
    return format!("{count} removed {}", if count == 1 { "worktree" } else { "worktrees" });
}

/// What:
/// The note `doctor` adds:
/// how many removed worktrees still have commit service units,
/// how many instances cannot be told,
/// and the command that stops the first.
/// Empty when there is nothing to say,
/// or when no runtime directory is known.
/// Why:
///   The person learns about leftover units without listing them.
/// The count reads files only and changes none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// unitsNote(environment) // '' or one line
/// ```
pub(crate) fn units_note(environment: &[(OsString, OsString)]) -> String {
    let Some(runtime_dir) = runtime_dir_of(environment) else {
        return String::new();
    };
    let state: Option<PathBuf> = state_dir_of(environment);
    let found = survey(&UnitFolders { runtime_dir: runtime_dir.as_path(), state_dir: state.as_deref() });
    return note_of(found.gone.len(), found.unmapped.len());
}

/// The note for the counts,
/// empty when the worktrees' units are all in order.
fn note_of(gone: usize, unmapped: usize) -> String {
    let mut parts: Vec<String> = Vec::new();
    if gone > 0 {
        parts.push(format!(
            "the commit service units of {} are still set up; `{COMMAND}` stops them",
            worktrees(gone)
        ));
    }
    if unmapped > 0 {
        parts.push(format!(
            "{unmapped} commit service {} cannot be matched to a worktree, so none is touched",
            if unmapped == 1 { "instance" } else { "instances" }
        ));
    }
    if parts.is_empty() {
        return String::new();
    }
    return format!("git monochromatic-git-policy-cli doctor: {}; nothing was changed.\n", parts.join(", and "));
}

/// An exit that stopped nothing,
/// with the reason on standard error.
fn refused(reason: &str) -> Action {
    return Action::Exit { code: 1, stdout: String::new(), stderr: format!("{COMMAND}: {reason}\n") };
}

/// What:
/// Stops the units of every removed worktree through `manager`
/// and says what was stopped,
/// what was refused,
/// and what could not be matched.
/// Why:
///   Status 0 means every unit of a gone worktree was stopped or there was none;
/// a refusal of the manager is status 1 with its reason,
/// and the stopped ones stay stopped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stopUnitsAction(folders, manager) // { code, stdout, stderr }
/// ```
pub(crate) fn stop_units_action(folders: &UnitFolders<'_>, manager: &mut dyn Manager) -> Action {
    // Named before the stop, which removes a note the name may come from.
    let mut named: Vec<(String, String)> = Vec::new();
    for instance in survey(folders).gone {
        let path: String = path_of(folders, instance.as_str());
        named.push((instance, path));
    }
    let outcome: SweepOutcome = stop_units_of_gone_worktrees(manager, folders);
    let mut stdout: String = String::new();
    for instance in &outcome.stopped {
        let mut path: &str = UNKNOWN_WORKTREE;
        for (known, known_path) in &named {
            if known == instance {
                path = known_path.as_str();
            }
        }
        stdout.push_str(format!("stopped the commit service units of the removed worktree {path} (instance {instance})\n").as_str());
    }
    if outcome.stopped.is_empty() && outcome.failed.is_empty() {
        stdout.push_str("no commit service unit belongs to a removed worktree\n");
    }
    let mut stderr: String = String::new();
    for (instance, reason) in &outcome.failed {
        stderr.push_str(format!("{COMMAND}: the units of instance {instance} were not stopped: {reason}; run the command again\n").as_str());
    }
    if !outcome.survey.unmapped.is_empty() {
        stderr.push_str(
            format!(
                "{COMMAND}: {} commit service {} cannot be matched to a worktree and {} left running\n",
                outcome.survey.unmapped.len(),
                if outcome.survey.unmapped.len() == 1 { "instance" } else { "instances" },
                if outcome.survey.unmapped.len() == 1 { "was" } else { "were" }
            )
            .as_str(),
        );
    }
    // A journal that holds work is kept and named with where it is and what kept it, never discarded, and the command's
    // own status stays 0: the units it was asked about were stopped.
    for (instance, kept) in &outcome.journals_kept {
        stderr.push_str(
            format!(
                "{COMMAND}: the journal of instance {instance} is kept at {}, because it holds {}, {}; look at it and remove it yourself if nothing in it is wanted\n",
                kept.journal.display(),
                kept.entry,
                kept.reason
            )
            .as_str(),
        );
    }
    return Action::Exit { code: i32::from(!outcome.failed.is_empty()), stdout, stderr };
}

/// What:
/// Plans the command from the process's environment:
/// connects to the session bus and stops the units.
/// Why:
///   The units belong to the user's manager,
/// so a missing runtime directory or an unreachable bus is a refusal with its remedy,
/// and nothing is stopped by any other means.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// planStopUnitsOfRemovedWorktrees(environment)
/// ```
pub(crate) fn plan_stop_units_of_removed_worktrees(environment: &[(OsString, OsString)]) -> Action {
    let Some(runtime_dir) = runtime_dir_of(environment) else {
        return refused("XDG_RUNTIME_DIR is not set, so there is no user runtime directory whose units could be looked at; run it from a login session");
    };
    let state: Option<PathBuf> = state_dir_of(environment);
    let folders: UnitFolders<'_> = UnitFolders { runtime_dir: runtime_dir.as_path(), state_dir: state.as_deref() };
    let address: Option<OsString> = environment_value(environment, "DBUS_SESSION_BUS_ADDRESS");
    let address_text: Option<&str> = match &address {
        Some(value) => value.to_str(),
        None => None,
    };
    let socket: PathBuf = match session_bus_path(address_text, Some(runtime_dir.as_path())) {
        Some(path) if path.exists() => path,
        _ => return refused("the user's session bus was not found, so the units cannot be stopped through the manager; run it from a login session"),
    };
    return match Bus::connect(socket.as_path(), EXCHANGE_WAIT) {
        Ok(mut bus) => stop_units_action(&folders, &mut bus),
        Err(error) => refused(format!("the user's session bus could not be reached ({error:?})").as_str()),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "management_service_stop_units_of_removed_worktrees_tests.rs"]
mod tests;
