//! What:
//!  The linked-worktree policy's carve-out for conflicted files:
//! judge a `git checkout` or `git restore` command line
//!  during a merge,
//! rebase,
//! cherry-pick or revert,
//! and save the files it would replace.
//! Why:
//!  The owner decided (2026-11-02,
//! "save,
//! then allow") that taking a side of a conflicted file is allowed once the
//!  current file is copied beside the audit log,
//! and that every path that is not conflicted keeps the rejection.
//! This
//!  file joins the pieces that decide and save (`conflict_sides.rs`,
//! `command_conflict_side.rs`) to the repository:
//! it
//!  finds the layout and the index in process,
//! starts no Git,
//! and returns what the policy should do.
//! The copy is made
//!  here,
//! before the policy lets the command run,
//! so Git never starts before the copy is on disk.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const verdict = judgeConflictSides({ context, arguments }); // 'not-applicable' | allowed | refused | copy-failed
//! ```

use crate::command_conflict_side::{SideRequest, conflict_side_request};
use crate::conflict_side_decision::magic_on_conflicted;
use crate::conflict_side_modes::{MODE_OPTIONS, MODE_VARIABLES, magic_note, mode_note, modes_in_effect};
use crate::conflict_sides::{
    FailureKind, Operation, SAVED_DIRECTORY, SaveFailure, SavedCopy, SideDecision, clean_paths_note, conflicted_paths, decide_sides, operation_in_progress,
    save_copies, saved_notice, still_being_written_note,
};
use crate::diagnostic_log::debug;
use crate::discarded_prune::{Limits, Problem, directory_name_of, prune};
use crate::global_arguments::command_tokens;
use crate::private_storage::read_regular_file;
use crate::repo_read_index::{IndexEntry, parse_index};
use crate::repo_read_layout::{RepoLayout, read_layout};
use crate::repo_read_layout_without::read_layout_without;
use crate::transaction_git::GitContext;
use std::ffi::OsString;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// Module tag for diagnostics.
const TAG: &str = "judgeConflictSides";

/// The largest index file this reads.
const INDEX_LIMIT_BYTES: u64 = 1 << 30;

/// What the policy does with a command line during an operation in progress.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SideVerdict {
    /// Not a conflict-side form,
    /// no operation in progress,
    /// or a layout this reader declines:
    /// the usual rules decide.
    NotApplicable,
    /// Every path is conflicted and its copy is saved:
    /// the command runs.
    Allowed {
        /// The notice to show the person before Git runs.
        notice: String,
        /// What was saved,
        /// for the audit record.
        copies: Vec<SavedCopy>,
    },
    /// Some path is not conflicted,
    /// or the index cannot be read in process:
    /// the usual rejection stays,
    /// with this sentence added.
    Refused(String),
    /// A copy could not be written:
    /// the command is rejected with this text,
    /// because it would discard unsaved work.
    CopyFailed(String),
}

/// The sentence for a copy that could not be written,
/// with the advice that fits what went wrong.
fn copy_failed_text(failure: &SaveFailure) -> String {
    return match failure.kind {
        FailureKind::Io => format!(
            "monochromatic-git-policy-cli: this command would replace a conflicted file, and your copy of it could not be saved first ({}: {}), so it is \
             rejected and nothing was changed. Free the space or fix the permissions of the Git directory and run it again, or \
             save the file yourself and pass `--no-enforce-worktree` directly after the subcommand.",
            failure.subject, failure.reason
        ),
        FailureKind::Ancestor => format!(
            "monochromatic-git-policy-cli: this command would replace a conflicted file, but `{}` in front of it {}. Git would replace `{}` with a real \
             directory to write the file, and nothing of it would be saved first, so the command is rejected and nothing was \
             changed. Move `{}` out of the way or save it yourself and run the command again, or pass `--no-enforce-worktree` \
             directly after the subcommand to go ahead without a copy.",
            failure.subject, failure.reason, failure.subject, failure.subject
        ),
        FailureKind::Changed => format!(
            "monochromatic-git-policy-cli: the conflicted file `{}` {}, so the copy that was being made may be stale or cut short and was not kept. The \
             command is rejected and nothing was changed. Run it again once nothing is writing to the file (an editor or a build may \
             be), or save the file yourself and pass `--no-enforce-worktree` directly after the subcommand.",
            failure.subject, failure.reason
        ),
    };
}

/// The sentence for an index that cannot be read in process.
fn index_declined() -> String {
    return String::from(
        " The index could not be read in process (it is split, sparse, unreadable, or chosen by `GIT_INDEX_FILE`), so no \
         file can be shown to be conflicted.",
    );
}

/// The conflicted paths of the index at `layout`,
/// or the sentence that says why they cannot be read in process.
fn read_conflicts(layout: &RepoLayout) -> Result<std::collections::BTreeSet<Vec<u8>>, String> {
    if std::env::var_os("GIT_INDEX_FILE").is_some() {
        return Err(index_declined());
    }
    let Ok(bytes) = read_regular_file(layout.git_dir.join("index").as_path(), INDEX_LIMIT_BYTES) else {
        return Err(index_declined());
    };
    let entries: Vec<IndexEntry> = parse_index(bytes.as_slice(), layout.object_format).ok_or_else(index_declined)?;
    return Ok(conflicted_paths(entries.as_slice()));
}

/// What:
///  Applies the age and size limits to the folder of saved copies,
/// now that a new save is complete and durable,
/// and gives
///  the sentences for whatever the pass could not do (empty when it did all it should).
/// Why:
///   Pruning after the save,
/// never before,
/// means a kill while copying costs no older copy;
/// the directory just written is
///  never chosen,
/// whatever its name says.
/// Nothing waits on the pass (another pass holding the folder makes this one do
///  nothing),
/// and a failure of it never changes the verdict;
/// it is said in the notice,
/// so the person sees that the limits
///  did not apply and why.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await pruneAfterSave(commonDir, copies, milliseconds)
/// ```
pub(crate) fn prune_after_save(common_dir: &Path, copies: &[SavedCopy], milliseconds: i64, limits: Limits) -> String {
    let saved_root: std::path::PathBuf = common_dir.join(SAVED_DIRECTORY);
    let mut keep: Option<String> = None;
    for copy in copies {
        if let Some(place) = copy.saved_to.as_deref() {
            keep = directory_name_of(saved_root.as_path(), place);
            if keep.is_some() {
                break;
            }
        }
    }
    if keep.is_none() {
        return String::new();
    }
    let outcome = prune(common_dir, milliseconds, limits, keep.as_deref());
    return outcome.problems.iter().map(Problem::message).collect();
}

/// What:
///  The verdict for a conflict-side line whose layout the reader declined:
/// nothing,
/// unless a pathspec mode is the only reason
///  and the same line without it would be allowed,
/// when it is the sentence that says so.
/// Why:
///   The rejection stays either way.
/// The reader is asked again without the mode only here,
/// only for a line that qualifies,
///  and the line must name only conflicted files under an operation in progress,
/// so the sentence never promises what the
///  carve-out would refuse.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function verdictForDeclinedLayout(context: GitContext, request: SideRequest): SideVerdict;
/// ```
fn verdict_for_declined_layout(context: &GitContext, request: &SideRequest) -> SideVerdict {
    let modes: Vec<String> = modes_in_effect(context.global_prefix.as_slice(), context.overlay.as_slice());
    if modes.is_empty() {
        return SideVerdict::NotApplicable;
    }
    let Some(layout) = read_layout_without(context.global_prefix.as_slice(), context.overlay.as_slice(), &MODE_OPTIONS, &MODE_VARIABLES) else {
        debug(TAG, &format!("not applicable: the layout reader declines this invocation even without {modes:?}"));
        return SideVerdict::NotApplicable;
    };
    let Some(operation): Option<Operation> = operation_in_progress(layout.git_dir.as_path()) else {
        return SideVerdict::NotApplicable;
    };
    let Ok(conflicted) = read_conflicts(&layout) else {
        return SideVerdict::NotApplicable;
    };
    if let SideDecision::Allow(_) = decide_sides(request.paths.as_slice(), layout.prefix.as_slice(), layout.worktree_root.as_path(), &conflicted) {
        debug(TAG, &format!("refused during {operation:?}: only {modes:?} stands between this line and the carve-out"));
        return SideVerdict::Refused(mode_note(modes.as_slice(), operation));
    }
    return SideVerdict::NotApplicable;
}

/// What:
///  Judge a command line against the operation in progress,
/// saving the copies when it is allowed.
/// Why:
///   The order is the owner's:
/// the form must qualify,
/// an operation must be in progress,
/// every named path must be
///  conflicted,
/// and the copies must be on disk before the command runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function judgeConflictSides(context: GitContext, arguments: string[], milliseconds: number): SideVerdict;
/// ```
pub fn judge_conflict_sides(context: &GitContext, arguments: &[OsString], milliseconds: i64, limits: Limits) -> SideVerdict {
    let Some((word, region)) = command_tokens(arguments) else {
        return SideVerdict::NotApplicable;
    };
    let Some(request): Option<SideRequest> = conflict_side_request(word.as_encoded_bytes(), region) else {
        return SideVerdict::NotApplicable;
    };
    let Some(layout) = read_layout(context.global_prefix.as_slice(), context.overlay.as_slice()) else {
        // A global option other than `-C` (a pathspec mode such as `--icase-pathspecs` among them) or a `GIT_` variable the
        // layout reader does not tolerate (such as `GIT_ICASE_PATHSPECS`) lands here: what a pathspec names is then not known.
        debug(TAG, "the layout reader declined this invocation (a global option other than -C, a GIT_ variable it does not tolerate, or a layout it does not read), so the usual rejection stands");
        return verdict_for_declined_layout(context, &request);
    };
    let Some(operation): Option<Operation> = operation_in_progress(layout.git_dir.as_path()) else {
        return SideVerdict::NotApplicable;
    };
    let conflicted = match read_conflicts(&layout) {
        Ok(found) => found,
        Err(note) => {
            debug(TAG, &format!("refused during {operation:?}: the conflicted paths cannot be read in process:{note}"));
            return SideVerdict::Refused(note);
        }
    };
    match decide_sides(request.paths.as_slice(), layout.prefix.as_slice(), layout.worktree_root.as_path(), &conflicted) {
        SideDecision::Refuse(others) => {
            debug(TAG, &format!("refused during {operation:?}: {others:?} are not conflicted files among {} conflicted", conflicted.len()));
            let magic = magic_on_conflicted(others.as_slice(), layout.prefix.as_slice(), layout.worktree_root.as_path(), &conflicted);
            if let Some(names) = magic {
                debug(TAG, &format!("the only reason is pathspec magic on conflicted files: {names:?}"));
                return SideVerdict::Refused(magic_note(names.as_slice(), operation));
            }
            return SideVerdict::Refused(clean_paths_note(others.as_slice(), operation));
        }
        SideDecision::Allow(paths) => {
            match save_copies(layout.worktree_root.as_path(), layout.common_dir.as_path(), paths.as_slice(), milliseconds) {
                Ok(saved) => {
                    let unpruned: String = prune_after_save(layout.common_dir.as_path(), saved.copies.as_slice(), milliseconds, limits);
                    // A copy whose source was still being written is kept and named as such, and the command runs:
                    //  a writer that never stops must not hold the command the person asked for.
                    return SideVerdict::Allowed {
                        notice: format!(
                            "{}{unpruned}{}",
                            saved_notice(saved.copies.as_slice()),
                            still_being_written_note(saved.still_being_written.as_slice())
                        ),
                        copies: saved.copies,
                    };
                }
                Err(failure) => {
                    debug(TAG, &format!("copy failed during {operation:?}, command rejected: {failure:?}"));
                    return SideVerdict::CopyFailed(copy_failed_text(&failure));
                }
            }
        }
    }
}

/// The sentences for a copy that could not be made.
#[cfg(test)]
#[path = "conflict_side_guard_tests.rs"]
mod tests;
