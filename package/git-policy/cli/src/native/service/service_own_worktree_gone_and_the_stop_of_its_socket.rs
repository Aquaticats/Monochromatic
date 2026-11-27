//! What:
//!  What a service knows about the worktree whose units are its own:
//! whether it is certainly gone,
//! what the answer means for the units,
//! and the stop of its own socket when it is.
//! Why:
//!  A removal Git ran with the wrapper replaced and a worktree deleted without Git
//! leave the socket unit listening and the service running for a directory that is gone,
//! so the service asks itself,
//! when it starts and while it serves (`service_server.rs`).
//! The answer is the same rule the sweep and `doctor` use (`worktree_is_gone`),
//! so an unmounted device and an instance nothing names keep the service serving.
//!  Only the socket is stopped,
//! and with the mode that leaves what requires it alone:
//! the service's own end is what stops its service unit,
//! and a stop of that unit from inside the service would signal this process mid-sentence.
//! The journal is the one file this process may still write,
//! so its removal waits for the sweep that proves the service gone
//! (`service_units_of_removed_worktrees.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const state = ownWorktreeState(instance, journal); // { gone } | { served } | 'unmapped'
//! if (state.gone) await stopOwnSocket(instance);
//! ```

use crate::service_files_of_a_stopped_instance::{
    journal_of_the_instance,
    work_a_person_may_want,
};
use crate::service_instance::socket_unit_instance;
use crate::service_journal_folder_of_an_instance::{
    JOURNAL_FOLDER,
    journal_of,
    state_dir_of_journal,
};
use crate::service_provision_host_and_manager::Manager;
use crate::service_unit_folders_and_the_worktree_of_an_instance::{
    UnitFolders,
    UnitWorktree,
    worktree_is_gone,
    worktree_of,
};
use crate::systemd_dbus::{
    Bus,
    CallError,
    EXCHANGE_WAIT,
    NO_SUCH_UNIT,
    session_bus_path,
};
use crate::wrapper_names_old_and_new::SERVICE_UNIT;
use std::path::{
    Path,
    PathBuf,
};

/// What:
/// The Git directory of an instance whose worktree is gone,
/// and nothing when it is still there,
/// when it cannot be told whose the units are,
/// or when the place that held the directory is not there either.
/// Why:
///   This is the question a service asks about itself before it serves:
/// a service of a worktree that no longer exists has nothing to commit to,
/// and the files of its socket and its note are the only record of whose they are.
/// The answer is the same one the survey and the sweep give,
/// so a service and the sweep never disagree about a gone worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ownWorktreeGone(folders, instance) // '/repo/.git/worktrees/topic' or undefined
/// ```
pub fn own_worktree_gone(
    folders: &UnitFolders<'_>,
    instance: &str,
) -> Option<PathBuf> {
    let UnitWorktree::Known(git_dir) = worktree_of(folders, instance) else {
        return None;
    };
    return worktree_is_gone(git_dir.as_path()).then_some(git_dir);
}

/// What a service's own units are to it,
/// once it asks about the worktree they serve.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OwnWorktree {
    /// The worktree's Git directory is certainly gone:
    /// the service has nothing to commit to and stops its own socket.
    Gone(PathBuf),
    /// The worktree is there,
    /// or it is on a device that cannot be reached right now,
    /// so the service serves it.
    Served(PathBuf),
    /// Nothing maps the instance to a worktree,
    /// so the service serves and nothing is stopped.
    Unmapped,
}

/// What:
/// What this service's own units are,
/// read from the runtime directory of the process and the journal it was started with.
/// Why:
///   This is the question a serving service asks of itself:
/// the note provisioning wrote beside the socket,
/// else the `gitdir=` records of its own journal,
/// name a directory that hashes to the instance,
/// and the same rule the sweep and `doctor` use decides whether it is gone.
/// A worktree on a device that is simply not mounted is served,
/// and so is one neither the note nor the journal names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ownWorktreeState(instance, journal) // { gone: gitDir } | { served: gitDir } | 'unmapped'
/// ```
pub fn own_worktree_state(
    instance: &str,
    journal: &Path,
) -> OwnWorktree {
    // What:     the runtime directory this session names, read in the one place this crate reads it.
    // Why:      a build made for tests answers only a runtime directory a control handed in, so a test process
    //           reads no note or unit of this account's (issue 6586).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const runtime = session.runtimeDirectory(); if (!runtime) return 'unmapped';
    // ```
    let Some(runtime_dir) = crate::session_runtime_directory_and_bus_address::runtime_directory()
    else {
        return OwnWorktree::Unmapped;
    };
    let state: Option<PathBuf> = state_dir_of_journal(journal, instance);
    let folders: UnitFolders<'_> = UnitFolders {
        runtime_dir: runtime_dir.as_path(),
        state_dir: state.as_deref(),
    };
    return match worktree_of(&folders, instance) {
        UnitWorktree::Unmapped => OwnWorktree::Unmapped,
        UnitWorktree::Known(git_dir) if worktree_is_gone(git_dir.as_path()) => {
            OwnWorktree::Gone(git_dir)
        }
        UnitWorktree::Known(git_dir) => OwnWorktree::Served(git_dir),
    };
}

/// One stop of the named unit and of nothing that requires it,
/// where a unit the manager does not hold is stopped already.
fn stop_one_alone(
    manager: &mut dyn Manager,
    unit: &str,
) -> Result<(), CallError> {
    return match manager.stop_alone(unit) {
        Err(CallError::Manager {
            name,
            ..
        }) if name == NO_SUCH_UNIT => Ok(()),
        other => other,
    };
}

/// One stop of an instance's socket through the manager.
fn stop_own_socket(
    instance: &str,
    manager: &mut dyn Manager,
) -> Result<(), CallError> {
    manager.within(EXCHANGE_WAIT);
    return stop_one_alone(manager, socket_unit_instance(instance).as_str());
}

/// What:
/// Stops the socket unit of one instance through the manager,
/// ends the pushes it kept for a repository that is gone,
/// and keeps the journal,
/// the note and the socket file for the sweep that follows this process's end,
/// when the worktree of that instance is gone,
/// and nothing when it is not.
/// Why:
///   This is what a service of a gone worktree does about itself:
/// it serves nothing,
/// and no connection starts it again;
/// the retry plan of every branch of the gone repository ends
/// (`service_plans_of_a_gone_repository.rs`),
/// so a service started later has no try left to spend;
/// nothing of the journal is removed here,
/// because this process is still one of that journal's writers,
/// so `doctor` counts the instance until the sweep removes it,
/// which is the price of never taking a record this process may still publish.
/// Only the socket is stopped,
/// and with the mode that leaves what requires it alone:
/// the service's own end is what stops its service unit,
/// and a stop of that unit from inside the service would signal this process mid-sentence.
/// The socket goes first so no connection starts the service again.
/// The journal is the one file this process may still write,
/// so its removal waits for a sweep that proves the service gone,
/// and the files that name the instance stay until then.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await stopOwnSocketOfAGoneWorktree(folders, instance, manager) // the Git directory, or undefined
/// ```
pub fn stop_own_socket_of_a_gone_worktree(
    folders: &UnitFolders<'_>,
    instance: &str,
    manager: &mut dyn Manager,
) -> Result<Option<PathBuf>, CallError> {
    let Some(gone) = own_worktree_gone(folders, instance) else {
        return Ok(None);
    };
    stop_own_socket(instance, manager)?;
    // What: The pushes this service kept for the repository that is gone end here.
    // Why:  Nothing is left to deliver, and a plan that stayed would be read by a service of this instance started later, which
    //       would spend a try on a directory nothing can reach: the owner's "nothing should ever wait forever or indefinitely".
    //       A plan of a repository that is still there, and every other record, stays and keeps the journal.
    if let Some(state) = folders.state_dir {
        let journal: PathBuf = journal_of(state.join(SERVICE_UNIT.current).as_path(), instance);
        let ended: Vec<String> =
            crate::service_plans_of_a_gone_repository::end_plans_of_gone_repositories(
                journal.as_path(),
            );
        if !ended.is_empty() {
            crate::service_log!(
                "the pushes this commit service kept for {} end without another try, because its repository is gone: {}; nothing is left there for a try to reach",
                gone.display(),
                ended.join(", ")
            );
        }
    }
    // Nothing of the journal is removed here, because this process is one of its writers and still runs:
    //  the task that runs the after-commit work writes the note that names a repository to look at and the plan of a push
    //  to try again, on a thread of its own and from jobs of the blocking pool, and both may write between the listing that
    //  says the journal holds nothing unresolved and the removal of the whole tree.
    // Removing it there took a record published in that window: the durable record of a pending push, of a repository still
    //  to look at, or of a batch a client whose answer was lost still reads.
    // So the journal, the note and the socket file stay for the sweep that runs after this process has ended and the manager
    //  reports its service gone (`stop_units_of_gone_worktrees`), and what the journal holds is still named here.
    match work_a_person_may_want(folders.state_dir, instance) {
        Some(kept) => crate::service_warn!(
            "the journal of this commit service is kept at {}, because it holds {}, {}; look at it and remove it yourself if nothing in it is wanted",
            kept.journal.display(),
            kept.entry,
            kept.reason
        ),
        // Landed records a client may still read, and the landing record of nothing in flight, ask nothing of a person:
        //  the sweep removes them together with the files that name this instance once this process is gone.
        None => crate::service_log!(
            "the journal of this commit service is kept at {} for the sweep that follows this process's end, because this service is still one of its writers",
            journal_folder_in_words(folders.state_dir, instance)
        ),
    }
    return Ok(Some(gone));
}

/// What:
///  The folder the service journals of the user's instances sit in,
/// in words.
/// Why:
///   A line that asks a person to look at a journal has to say where the journals are,
/// and the folder is the state directory's,
///  which the command may have been given none of.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// journalsFolderInWords(stateDir) // '<state>/monochromatic-git-policy-cli/service'
/// ```
pub fn journals_folder_in_words(state_dir: Option<&Path>) -> String {
    let Some(state) = state_dir else {
        return String::from("a folder this process was given no state directory for");
    };
    return state
        .join(SERVICE_UNIT.current)
        .join(JOURNAL_FOLDER)
        .display()
        .to_string();
}

/// The journal folder of an instance in words,
/// for a line that names where the journal a service leaves is kept.
fn journal_folder_in_words(
    state_dir: Option<&Path>,
    instance: &str,
) -> String {
    let Some(journal) = journal_of_the_instance(state_dir, instance) else {
        return String::from("a folder this process was given no state directory for");
    };
    return journal.display().to_string();
}

/// What:
/// Stops this service's own socket when the worktree it serves is gone,
/// with the line that says why,
/// and says whether the worktree is gone.
/// Why:
///   A removal Git ran with the wrapper replaced (`worktree remove --force`,
/// `worktree prune`),
/// and a worktree deleted without Git,
/// leave the socket unit listening and the service unit running for a directory that is gone,
/// and a service has nothing to commit to there.
/// Nothing else asks,
/// so the service asks itself:
/// when it starts,
/// and while it serves (`service_server.rs`).
/// It stops its own socket and nothing that requires it,
/// because its own end is what stops its service unit,
/// and a stop of that unit from inside the service would signal it before it removed the note and the socket file.
/// A manager that cannot be reached only logs:
/// the files stay,
/// so `doctor` counts the leftover and the next start tries again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stopOwnSocketWhenTheWorktreeIsGone(instance, journal) // true: gone, so serve nothing
/// ```
pub fn stop_own_socket_when_the_worktree_is_gone(
    instance: &str,
    journal: &Path,
) -> bool {
    // What:     the two facts a manager is reached by, read in the one place this crate reads them.
    // Why:      a build made for tests answers only what a control handed in, so a test process stops no socket of
    //           this account's (issue 6586).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const runtime = session.runtimeDirectory(); if (!runtime) return false;
    // ```
    let Some(runtime_dir) = crate::session_runtime_directory_and_bus_address::runtime_directory()
    else {
        return false;
    };
    let state: Option<PathBuf> = state_dir_of_journal(journal, instance);
    let folders: UnitFolders<'_> = UnitFolders {
        runtime_dir: runtime_dir.as_path(),
        state_dir: state.as_deref(),
    };
    // A worktree that is there, or one nothing names, costs no bus connection and no question to the manager: the files answer it.
    let OwnWorktree::Gone(gone) = own_worktree_state(instance, journal) else {
        return false;
    };
    let address: Option<String> = crate::session_runtime_directory_and_bus_address::bus_address();
    let Some(socket) = session_bus_path(address.as_deref(), Some(runtime_dir.as_path())) else {
        crate::service_warn!(
            "the worktree of this commit service is gone ({}), so it serves nothing, but no session bus was found to stop its socket; `git monochromatic-git-policy-cli doctor` counts it and the sweep stops it",
            gone.display()
        );
        return true;
    };
    let mut bus: Bus = match Bus::connect(socket.as_path(), EXCHANGE_WAIT) {
        Ok(connected) => connected,
        Err(error) => {
            crate::service_warn!(
                "the worktree of this commit service is gone ({}), so it serves nothing, but the session bus could not be reached ({error:?}) to stop its socket; `git monochromatic-git-policy-cli doctor` counts it and the sweep stops it",
                gone.display()
            );
            return true;
        }
    };
    match stop_own_socket_of_a_gone_worktree(&folders, instance, &mut bus) {
        Ok(Some(stopped)) => {
            crate::service_log!(
                "the worktree of this commit service is gone ({}), so it served nothing and stopped its own socket",
                stopped.display()
            );
        }
        // The directory came back between the two looks, so this service is a live worktree's again.
        Ok(None) => {}
        Err(error) => {
            crate::service_warn!(
                "the worktree of this commit service is gone ({}), so it serves nothing, but its socket was not stopped ({error:?}); `git monochromatic-git-policy-cli doctor` counts it and the sweep stops it",
                gone.display()
            );
        }
    }
    return true;
}
