//! What:
//! Stops the commit service units of a worktree when the wrapper has removed that worktree:
//! the instance is read from the registration before the removal,
//! and the units are stopped after it,
//! once the registration is gone.
//! Why:
//! A unit instance is named by a hash of its worktree's own Git directory (`service_instance.rs`),
//! so after the removal nothing could say whose the units were,
//! and the socket unit,
//! which has no idle exit,
//! kept listening.
//! The name is therefore taken first,
//! and the stop happens only when the Git directory is certainly gone:
//! a removal that Git refused leaves the worktree and its units alone.
//! Both removals the wrapper serves (the subvolume destroy and Git's own removal run as a child)
//! end in `ForwardedRun::Ran(0)`,
//! so one hook covers both;
//! a removal handed to Git with the process replaced (`--force`,
//! `prune`) cannot be looked at afterwards,
//! and `git monochromatic-git-policy-cli service stop-units-of-removed-worktrees` stops those units.
//! A failure to stop is logged and never fails the removal the person asked for.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const about = beforeRemoval(commonDir, plan); // read the registration
//! await runRemoval();
//! if (about) await stopUnitsAfterRemoval(about, folders, manager);
//! ```

use crate::diagnostic_log::{debug, warn};
use crate::service_instance::{socket_path, try_instance_of};
use crate::service_provision_host_and_manager::Manager;
use crate::service_units_of_removed_worktrees::{UnitFolders, forget_stopped_instance, note_path, remove_journal_of_a_stopped_instance, stop_units, worktree_is_gone};
use crate::systemd_dbus::{Bus, CallError, EXCHANGE_WAIT, session_bus_path};
use crate::worktree_remove_plan::RemovePlan;
use crate::worktree_remove_registry::registration_of;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeRemovalStopsUnits";

/// The worktree a removal is about to remove,
/// as its units know it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorktreeAboutToGo {
    /// The instance name of the worktree.
    pub instance: String,
    /// The canonical administrative directory of the worktree,
    /// which the removal deletes.
    pub git_dir: PathBuf,
}

/// What:
/// Reads the instance of the worktree a removal names,
/// from its registration in the repository the command runs in.
/// Why:
///   The registration is deleted by the removal,
/// and the instance is a hash of its path:
/// this is the last moment the name can be made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const about = beforeRemoval(commonDir, plan); // undefined when Git decides which worktree goes
/// ```
pub fn before_removal(common_dir: &Path, plan: &RemovePlan) -> Option<WorktreeAboutToGo> {
    let registration = match registration_of(common_dir, plan.target.as_path(), plan.operand.as_os_str()) {
        Ok(found) => found,
        Err(reason) => {
            debug(TAG, format!("{} has no registration to name units by ({reason})", plan.target.display()).as_str());
            return None;
        }
    };
    let instance: String = match try_instance_of(registration.admin.as_path()) {
        Ok(name) => name,
        Err(unreachable) => {
            debug(TAG, format!("{unreachable}").as_str());
            return None;
        }
    };
    return Some(WorktreeAboutToGo { instance, git_dir: registration.admin });
}

/// What:
/// Whether any unit of the worktree could exist:
/// its socket file or its note is there.
/// Why:
///   A worktree that never committed has no unit,
/// and then the removal need not connect to the manager at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// hasUnits(folders, instance) // socket file or note
/// ```
fn has_units(folders: &UnitFolders<'_>, instance: &str) -> bool {
    return socket_path(folders.runtime_dir, instance).exists() || note_path(folders.runtime_dir, instance).exists();
}

/// What:
/// Stops the units of the worktree after a removal,
/// when its Git directory is gone and it had units,
/// and removes the note,
/// the socket file and the journal of an instance nothing serves again.
/// Whether the units were stopped.
/// Why:
///   A removal that Git refused leaves the Git directory,
/// and the worktree's units are then still its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await stopUnitsAfterRemoval(about, folders, manager) // false: nothing to stop
/// ```
pub fn stop_units_after_removal(about: &WorktreeAboutToGo, folders: &UnitFolders<'_>, manager: &mut dyn Manager) -> Result<bool, CallError> {
    if !worktree_is_gone(about.git_dir.as_path()) {
        debug(TAG, format!("{} is still there; its units stay", about.git_dir.display()).as_str());
        return Ok(false);
    }
    if !has_units(folders, about.instance.as_str()) {
        return Ok(false);
    }
    manager.within(EXCHANGE_WAIT);
    stop_units(manager, about.instance.as_str())?;
    forget_stopped_instance(folders.runtime_dir, about.instance.as_str());
    if let Some(kept) = remove_journal_of_a_stopped_instance(folders.state_dir, about.instance.as_str()) {
        warn(
            TAG,
            format!(
                "the journal of the removed worktree {} is kept at {}, because it holds {}, {}; look at it and remove it yourself if nothing in it is wanted",
                about.git_dir.display(),
                kept.journal.display(),
                kept.entry,
                kept.reason
            )
            .as_str(),
        );
    }
    return Ok(true);
}

/// What:
/// Stops the units of a removed worktree through the session manager,
/// and only logs a failure.
/// Why:
///   The removal the person asked for has already happened,
/// and a manager that cannot be reached is not a reason to fail it;
/// the units are stopped by the next sweep.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await stopUnitsOfRemovedWorktree(about, env); // never throws
/// ```
pub fn stop_units_of_removed_worktree(about: &WorktreeAboutToGo) {
    let Some(runtime) = non_empty_variable("XDG_RUNTIME_DIR") else { return };
    let runtime_dir: PathBuf = PathBuf::from(runtime);
    // The journal's state directory, which the units' `StateDirectory=` names and the sweep reads back: without it a
    // removed worktree's journal outlives its units.
    let state_dir: Option<PathBuf> = state_dir_of_the_user();
    let folders: UnitFolders<'_> = UnitFolders { runtime_dir: runtime_dir.as_path(), state_dir: state_dir.as_deref() };
    if !worktree_is_gone(about.git_dir.as_path()) || !has_units(&folders, about.instance.as_str()) {
        return;
    }
    let address: Option<String> = non_empty_variable("DBUS_SESSION_BUS_ADDRESS");
    let Some(socket) = session_bus_path(address.as_deref(), Some(runtime_dir.as_path())) else {
        warn(TAG, format!("no session bus was found, so the units of the removed worktree {} keep running", about.git_dir.display()).as_str());
        return;
    };
    let mut bus: Bus = match Bus::connect(socket.as_path(), EXCHANGE_WAIT) {
        Ok(connected) => connected,
        Err(error) => {
            warn(TAG, format!("the session bus could not be reached ({error:?}), so the units of the removed worktree {} keep running", about.git_dir.display()).as_str());
            return;
        }
    };
    match stop_units_after_removal(about, &folders, &mut bus) {
        Ok(true) => debug(TAG, format!("stopped the units of the removed worktree {}", about.git_dir.display()).as_str()),
        Ok(false) => {}
        Err(error) => warn(TAG, format!("the units of the removed worktree {} were not stopped ({error:?}); a later sweep tries again", about.git_dir.display()).as_str()),
    }
}

/// The text of an environment variable,
/// nothing when it is unset or empty.
fn non_empty_variable(name: &str) -> Option<String> {
    return match std::env::var(name) {
        Ok(text) if !text.is_empty() => Some(text),
        _ => None,
    };
}

/// What:
/// The state directory of this user,
/// whose `monochromatic-git-policy-cli/service/<instance>` folders hold the service journals.
/// Why:
///   The unit's `StateDirectory=` puts the journal under `$XDG_STATE_HOME`,
/// else under `$HOME/.local/state` (`systemd.exec(5)`),
/// and the removal hook runs as the same user.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stateDirOfTheUser() // $XDG_STATE_HOME, else $HOME/.local/state
/// ```
fn state_dir_of_the_user() -> Option<PathBuf> {
    if let Some(named) = non_empty_variable("XDG_STATE_HOME") {
        return Some(PathBuf::from(named));
    }
    return non_empty_variable("HOME").map(state_dir_under_home);
}

/// The state directory of a user whose home is `home`,
/// as `systemd` places it.
fn state_dir_under_home(home: String) -> PathBuf {
    return PathBuf::from(home).join(".local").join("state");
}

/// Removal hook controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_removal_stops_units_tests.rs"]
mod tests;
