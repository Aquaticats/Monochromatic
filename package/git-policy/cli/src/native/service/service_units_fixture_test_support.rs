//! What:
//! One disposable machine for the controls of the commit service units of worktrees that no longer exist:
//! a repository and one runtime directory and state directory on disk,
//! with the notes,
//! the socket files and the journals an instance leaves there.
//! Why:
//! The controls of the units,
//! of the files an instance keeps and of the stop share the same world:
//! a checkout's Git directory with linked worktrees' administrative directories under it,
//! a socket file per instance the manager keeps while its socket unit listens,
//! and the journal folder the unit's `--journal` names.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const machine = machine('units-survey'); // { root, runtime, state, folders() }
//! ```

use crate::service_instance::{SOCKET_DIRECTORY, socket_path};
use crate::service_journal::DONE_EXTENSION;
use crate::service_journal_folder_of_an_instance::journal_of;
use crate::service_units_of_removed_worktrees::UnitFolders;
use crate::test_support::fixture;
use crate::wrapper_names_old_and_new::SERVICE_UNIT;
use std::path::PathBuf;

/// The record the journal keeps of a landing that is in flight.
const LANDING_RECORD_DIRECTORY: &str = "busy";

/// One repository and one runtime directory on disk:
/// a checkout's Git directory with linked worktrees' administrative directories under it.
pub struct Machine {
    /// The fixture root.
    pub root: PathBuf,
    /// The user's runtime directory,
    /// which holds the socket files and the notes.
    pub runtime: PathBuf,
    /// The user's state directory,
    /// which holds the journals.
    pub state: PathBuf,
}

/// A machine in the agent scratch directory,
/// with the socket directory and the state directory made.
pub fn machine(name: &str) -> Machine {
    let root: PathBuf = fixture(name);
    let runtime: PathBuf = root.join("run");
    let state: PathBuf = root.join("state");
    std::fs::create_dir_all(runtime.join(SOCKET_DIRECTORY)).expect("socket directory");
    std::fs::create_dir_all(&state).expect("state directory");
    return Machine { root, runtime, state };
}

impl Machine {
    /// The folders the units and the survey read.
    pub fn folders(&self) -> UnitFolders<'_> {
        return UnitFolders { runtime_dir: self.runtime.as_path(), state_dir: Some(self.state.as_path()) };
    }

    /// The administrative directory a linked worktree named `name` has,
    /// made when `present` and otherwise only named.
    pub fn linked(&self, name: &str, present: bool) -> PathBuf {
        let common: PathBuf = self.root.join("repository/.git");
        let admin: PathBuf = common.join("worktrees").join(name);
        if present {
            std::fs::create_dir_all(&admin).expect("administrative directory");
        } else {
            std::fs::create_dir_all(&common).expect("common directory");
        }
        return admin;
    }

    /// A socket file for an instance,
    /// as the manager keeps one while the socket unit listens.
    pub fn socket_of(&self, instance: &str) {
        std::fs::write(socket_path(self.runtime.as_path(), instance), b"listening").expect("socket file");
    }

    /// The journal folder of an instance,
    /// made as the unit's `--journal` names it.
    pub fn journal_of(&self, instance: &str) -> PathBuf {
        let wrapper_state: PathBuf = self.state.join(SERVICE_UNIT.current);
        let journal: PathBuf = journal_of(wrapper_state.as_path(), instance);
        std::fs::create_dir_all(&journal).expect("journal folder");
        return journal;
    }

    /// A journal as a service of a worktree that committed leaves it:
    /// the finished batches it remembers,
    /// and the landing record that is empty once nothing is in flight.
    pub fn settled_journal_of(&self, instance: &str) -> PathBuf {
        let journal: PathBuf = self.journal_of(instance);
        std::fs::write(journal.join(format!("1.{DONE_EXTENSION}")), b"seq=1\n").expect("a finished batch");
        std::fs::write(journal.join(format!("2.{DONE_EXTENSION}")), b"seq=2\n").expect("a finished batch");
        std::fs::create_dir_all(journal.join(LANDING_RECORD_DIRECTORY)).expect("the landing's record");
        return journal;
    }
}
