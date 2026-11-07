//! What:
//!  `git monochromatic-git-policy-cli service settle-earlier-batches`: finishes, when the person asks, the batches that the
//!  commit service of an earlier release left open for the worktree, and the note `doctor` adds about them.
//! Why:
//!  Stub until the controls for it are written.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await settleEarlierBatches({ selection, stateDir, git, procRoot }, conditionOut)
//! ```

use crate::action::Action;
use crate::management_service_repository_selection::Selection;
use crate::transaction_git::GitContext;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

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

/// The state folder of the user: `XDG_STATE_HOME`, else `$HOME/.local/state`.
pub(crate) fn state_dir_of(environment: &[(OsString, OsString)]) -> Option<PathBuf> {
    let _unused = environment;
    return None;
}

/// Settles on request.
pub(crate) fn settle_earlier_batches_action(request: &SettleRequest<'_>, condition_out: &mut dyn std::io::Write) -> Action {
    let _unused = (request, condition_out);
    return Action::Exit { code: 0, stdout: String::new(), stderr: String::new() };
}

/// The note `doctor` adds, empty when no batch is left.
pub(crate) fn earlier_batches_note(selection: &Selection, state_dir: Option<&Path>) -> String {
    let _unused = (selection, state_dir);
    return String::new();
}

/// Plans the command from the invocation's facts.
pub(crate) fn plan_settle_earlier_batches(global_arguments: &[OsString], environment: &[(OsString, OsString)], inputs: &crate::real_git::ResolutionInputs) -> Action {
    let _unused = (global_arguments, environment, inputs);
    return Action::Exit { code: 0, stdout: String::new(), stderr: String::new() };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "management_service_settle_earlier_batches_tests.rs"]
mod tests;
