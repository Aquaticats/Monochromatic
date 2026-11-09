//! What:
//!  The notes the doctor's action gains on standard error:
//!  the commit service units of worktrees that no longer exist,
//!  and the batches an earlier release left open for this worktree.
//! Why:
//!  Both are informational and read-only,
//!  neither changes the status or standard output,
//!  and both leave a run that could not examine the repository alone;
//!  one file holds them so the management decision keeps within its line budget.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // management_action_notes.ts: the notes appended to a doctor action.
//! ```

/// The exit shapes a note is appended to.
use crate::action::{Action, ENGINE_FAILURE_EXIT_CODE};
/// The note about the batches an earlier release left open,
/// and where this worktree keeps them.
use crate::management_service_settle_earlier_batches::{earlier_batches_note, state_dir_of};
/// The commit service units of worktrees that no longer exist.
use crate::management_service_stop_units_of_removed_worktrees::units_note;
/// Which repository a Git start runs in.
use crate::management_service_repository_selection::{Selection, status_git_dir};
/// `OsString` is owned operating-system text of raw OS bytes.
use std::ffi::OsString;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// What:
///  The doctor's action with the note about commit service units that belong to worktrees that no longer exist.
/// Why:
///   Like the note about earlier batches it is informational and read-only,
/// and a run that could not examine the repository (status 2) gets none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// withUnitsNote(action, environment)
/// ```
pub fn with_units_note(action: Action, environment: &[(OsString, OsString)]) -> Action {
    // `match` picks the exit shape and gives its report back unchanged;
    //  `stderr: text` binds the field under a new name.
    match action {
        Action::Exit { code, stdout, stderr } => {
            if code == ENGINE_FAILURE_EXIT_CODE {
                return Action::Exit { code, stdout, stderr };
            }
            let note: String = units_note(environment);
            return Action::Exit { code, stdout, stderr: format!("{stderr}{note}") };
        }
        Action::ExitBytes { code, stdout, stderr } => {
            if code == ENGINE_FAILURE_EXIT_CODE {
                return Action::ExitBytes { code, stdout, stderr };
            }
            let note: String = units_note(environment);
            let mut explained: Vec<u8> = stderr;
            explained.extend_from_slice(note.as_bytes());
            return Action::ExitBytes { code, stdout, stderr: explained };
        }
        other => return other,
    }
}

/// What:
///  The doctor's action with the note about the batches the earlier release left open for this worktree on standard error.
/// Why:
///   The note is informational and read-only;
/// it changes neither the status nor standard output,
/// and a run that could not
///  examine the repository (status 2) gets none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// withEarlierBatchesNote(action, globalArguments, environment, realGit)
/// ```
pub fn with_earlier_batches_note(action: Action, global_arguments: &[OsString], environment: &[(OsString, OsString)], real_git: &Path) -> Action {
    // `match` picks the exit shape and gives its report back unchanged.
    match action {
        Action::Exit { code, stdout, stderr } => {
            if code == ENGINE_FAILURE_EXIT_CODE {
                return Action::Exit { code, stdout, stderr };
            }
            let selection: Selection = status_git_dir(global_arguments, environment, Some(real_git));
            let note: String = earlier_batches_note(&selection, state_dir_of(environment).as_deref());
            return Action::Exit { code, stdout, stderr: format!("{stderr}{note}") };
        }
        Action::ExitBytes { code, stdout, stderr } => {
            if code == ENGINE_FAILURE_EXIT_CODE {
                return Action::ExitBytes { code, stdout, stderr };
            }
            let selection: Selection = status_git_dir(global_arguments, environment, Some(real_git));
            let note: String = earlier_batches_note(&selection, state_dir_of(environment).as_deref());
            let mut explained: Vec<u8> = stderr;
            explained.extend_from_slice(note.as_bytes());
            return Action::ExitBytes { code, stdout, stderr: explained };
        }
        other => return other,
    }
}
