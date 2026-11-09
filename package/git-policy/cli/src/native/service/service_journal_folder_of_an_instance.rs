//! What:
//!  Where a worktree's service keeps its journal:
//!  the folder the unit's `--journal` names,
//!  and the state folder that holds it,
//!  read back from a journal folder.
//! Why:
//!   The unit writes the folder,
//! the sweep of removed worktrees reads it back,
//! and a client whose answer was lost looks in it for the
//!  commit its request made,
//! so its one shape lives here instead of in each of them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const journal = journalOf('/s/monochromatic-git-policy-cli', '3f9a'); // '/s/monochromatic-git-policy-cli/service/3f9a'
//! ```

use crate::wrapper_names_old_and_new::SERVICE_UNIT;
use std::path::{Path, PathBuf};

/// The folder below the wrapper's state folder that holds the journals of every instance.
pub const JOURNAL_FOLDER: &str = "service";

/// What:
///  The journal folder the unit gives an instance:
///  `<state>/monochromatic-git-policy-cli/service/<instance>`.
/// Why:
///   The unit's `--journal` names it,
/// the sweep reads it back,
/// and a client whose answer was lost looks in it for its commit,
///  so its shape lives in one function.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// journalOf('/s/monochromatic-git-policy-cli', '3f9a') // '/s/monochromatic-git-policy-cli/service/3f9a'
/// ```
pub fn journal_of(wrapper_state: &Path, instance: &str) -> PathBuf {
    return wrapper_state.join(JOURNAL_FOLDER).join(instance);
}

/// What:
///  The state directory whose service folder holds `journal`,
/// and nothing when the path has not the shape the unit passes.
/// Why:
///   The unit starts the service with `--journal <state>/monochromatic-git-policy-cli/service/<instance>`,
/// so the service reads its worktree from its own journal records as well as from its note,
/// by the same rule the sweep and `doctor` use.
/// A journal of another shape (a development run) names no state directory,
/// and then only the note maps the instance.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stateDirOfJournal('/s/monochromatic-git-policy-cli/service/3f9a', '3f9a') // '/s'
/// ```
pub fn state_dir_of_journal(journal: &Path, instance: &str) -> Option<PathBuf> {
    let state: &Path = journal.parent()?.parent()?.parent()?;
    let expected: PathBuf = journal_of(state.join(SERVICE_UNIT.current).as_path(), instance);
    if journal != expected {
        return None;
    }
    return Some(state.to_path_buf());
}

/// The folder controls stay out of the release executable.
#[cfg(test)]
#[path = "service_journal_folder_of_an_instance_tests.rs"]
mod tests;
