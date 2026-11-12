//! What:
//!  What one forwarded `git worktree remove` does with the folder Git's own walk could not empty:
//!  the record it writes before Git runs,
//!  the finish after Git returned,
//!  and the line the person reads.
//! Why:
//!  Git deletes a worktree's registration whether or not its walk emptied the folder
//! (`builtin/worktree.c`,
//! `remove_worktree`:
//!  "continue on even if ret is non-zero,
//! there's no going back from here"),
//!  and something writing into the folder while the walk runs makes the walk fail
//!  with `Directory not empty`.
//!  The folder that stays is the one the person asked to be gone,
//!  so the removal remembers what it held and finishes it
//!  (`worktree_removal_left_behind_folder.rs`).
//!  The run and the words are separated here because a forwarded run's file is the lifecycle's,
//!  and this is the removal's own part of it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const remembered = rememberedBeforeGit(facts);       // only a removal
//! const finished = finishedAfterGit(remembered, run);  // Git came back
//! ```

use super::worktree_copy_forward::ForwardedRun;
use crate::worktree_removal_left_behind_folder::{LeftBehind, Remembered, finish_after_git as finish_the_recorded_folder, finished_notice, kept_notice, remember_before_git};
use crate::worktree_remove_plan::RemovePlan;
use std::io::Write;
use std::path::Path;

/// What:
///  The folder of a `git worktree remove` this run is about to hand to Git,
///  recorded with the entries it holds now,
///  or nothing when the command is not a removal or the repository does not register its folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const remembered = rememberedBeforeGit(facts.removePlan, commonDirectory(facts.identity));
/// ```
pub(super) fn remembered_before_git(the_removal: Option<&RemovePlan>, the_repository: Option<&Path>) -> Option<Remembered> {
    let plan: &RemovePlan = the_removal?;
    let common_dir: &Path = the_repository?;
    return remember_before_git(common_dir, plan.target.as_path());
}

/// What:
///  Finish the folder the removal recorded once Git returned,
///  say what happened on standard error,
///  and keep Git's own status.
/// Why:
///   A folder Git could not empty is the removal's to finish,
///  and the person has to read why their worktree needed it:
///  a removal whose notice says nothing is the failure this answers.
///  The status stays Git's whatever the finish did,
///  because the command the person ran is the one Git answered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (remembered && finished.kind !== 'ran-ok') { const outcome = finishAfterGit(remembered); stderr.write(noticeOf(outcome)); }
/// return finished;
/// ```
pub(super) fn finished_after_git(remembered: Option<Remembered>, finished: ForwardedRun) -> ForwardedRun {
    let Some(recorded_folder) = remembered else { return finished };
    // A removal this program could not start did not hand the folder to Git, so its record names a folder it never walked.
    if matches!(finished, ForwardedRun::Failed { .. }) {
        return finished;
    }
    let outcome: LeftBehind = finish_the_recorded_folder(&recorded_folder);
    for line in notice_of(&outcome, recorded_folder.target.as_path()) {
        // A closed terminal is not a reason to stop the command the person asked for.
        let _written = writeln!(std::io::stderr(), "{line}");
    }
    return finished;
}

/// What:
///  The lines the person reads about the folder a removal left,
///  and none when nothing was left.
/// Why:
///   The words belong to the completion module,
///  and a control that takes a different route through the run reads the same ones.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lines = noticeOf(outcome, target);
/// ```
fn notice_of(outcome: &LeftBehind, target: &Path) -> Vec<String> {
    return match outcome {
        LeftBehind::Nothing => Vec::new(),
        LeftBehind::Removed => vec![finished_notice(target)],
        LeftBehind::Kept { names, reason } => vec![kept_notice(target, names.as_slice(), reason.as_str())],
    };
}
