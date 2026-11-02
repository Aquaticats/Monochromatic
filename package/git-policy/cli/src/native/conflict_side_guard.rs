//! What:
//!  The linked-worktree policy's carve-out for conflicted files: judge a `git checkout` or `git restore` command line
//!  during a merge, rebase, cherry-pick or revert, and save the files it would replace.
//! Why:
//!  The owner decided (2026-11-02, "save, then allow") that taking a side of a conflicted file is allowed once the
//!  current file is copied beside the audit log, and that every path that is not conflicted keeps the rejection. This
//!  file joins the pieces that decide and save (`conflict_sides.rs`, `command_conflict_side.rs`) to the repository: it
//!  finds the layout and the index in process, starts no Git, and returns what the policy should do. The copy is made
//!  here, before the policy lets the command run, so Git never starts before the copy is on disk.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const verdict = judgeConflictSides({ context, arguments }); // 'not-applicable' | allowed | refused | copy-failed
//! ```

use super::command_conflict_side::{SideRequest, conflict_side_request};
use super::conflict_sides::{
    Operation, SAVED_DIRECTORY, SaveFailure, SavedCopy, SideDecision, clean_paths_note, conflicted_paths, decide_sides, operation_in_progress,
    save_copies, saved_notice,
};
use super::discarded_prune::{directory_name_of, prune, read_limits};
use super::global_arguments::command_tokens;
use super::private_storage::read_regular_file;
use super::repo_read_index::{IndexEntry, parse_index};
use super::repo_read_layout::{RepoLayout, read_layout};
use super::transaction_git::GitContext;
use std::ffi::OsString;

/// The largest index file this reads.
const INDEX_LIMIT_BYTES: u64 = 1 << 30;

/// What the policy does with a command line during an operation in progress.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SideVerdict {
    /// Not a conflict-side form, no operation in progress, or a layout this reader declines: the usual rules decide.
    NotApplicable,
    /// Every path is conflicted and its copy is saved: the command runs.
    Allowed {
        /// The notice to show the person before Git runs.
        notice: String,
        /// What was saved, for the audit record.
        copies: Vec<SavedCopy>,
    },
    /// Some path is not conflicted, or the index cannot be read in process: the usual rejection stays, with this sentence added.
    Refused(String),
    /// A copy could not be written: the command is rejected with this text, because it would discard unsaved work.
    CopyFailed(String),
}

/// The sentence for a copy that could not be written.
fn copy_failed_text(failure: &SaveFailure) -> String {
    return format!(
        "cli-git: this command would replace a conflicted file, and your copy of it could not be saved first ({}: {}), so it is \
         rejected and nothing was changed. Free the space or fix the permissions of the Git directory and run it again, or \
         save the file yourself and pass `--no-enforce-worktree` directly after the subcommand.",
        failure.subject, failure.reason
    );
}

/// The conflicted paths of the index at `layout`, or the sentence that says why they cannot be read in process.
fn read_conflicts(layout: &RepoLayout) -> Result<std::collections::BTreeSet<Vec<u8>>, String> {
    let declined = || -> String {
        return String::from(
            " The index could not be read in process (it is split, sparse, unreadable, or chosen by `GIT_INDEX_FILE`), so no \
             file can be shown to be conflicted.",
        );
    };
    if std::env::var_os("GIT_INDEX_FILE").is_some() {
        return Err(declined());
    }
    let bytes: Vec<u8> = read_regular_file(layout.git_dir.join("index").as_path(), INDEX_LIMIT_BYTES).map_err(|_| { return declined(); })?;
    let entries: Vec<IndexEntry> = parse_index(bytes.as_slice(), layout.object_format).ok_or_else(declined)?;
    return Ok(conflicted_paths(entries.as_slice()));
}

/// What:
///  Applies the age and size limits to the folder of saved copies, now that a new save is complete and durable.
/// Why:
///   Pruning after the save, never before, means a kill while copying costs no older copy; the directory just written is
///  never chosen, whatever its name says. The pass takes no lock and nothing waits on it, and a failure of it never changes
///  the verdict (`discarded_prune` says it on standard error when a setting is no limit).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await pruneAfterSave(layout, copies, milliseconds)
/// ```
fn prune_after_save(layout: &RepoLayout, copies: &[SavedCopy], milliseconds: i64) {
    let saved_root: std::path::PathBuf = layout.common_dir.join(SAVED_DIRECTORY);
    let keep: Option<String> = copies.iter().find_map(|copy| { return copy.saved_to.as_deref().and_then(|place| { return directory_name_of(saved_root.as_path(), place); }); });
    if keep.is_none() {
        return;
    }
    let limits = read_limits(layout.common_dir.as_path(), layout.git_dir.as_path());
    let _removed = prune(layout.common_dir.as_path(), milliseconds, limits, keep.as_deref());
}

/// What:
///  Judge a command line against the operation in progress, saving the copies when it is allowed.
/// Why:
///   The order is the owner's: the form must qualify, an operation must be in progress, every named path must be
///  conflicted, and the copies must be on disk before the command runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function judgeConflictSides(context: GitContext, arguments: string[], milliseconds: number): SideVerdict;
/// ```
pub fn judge_conflict_sides(context: &GitContext, arguments: &[OsString], milliseconds: i64) -> SideVerdict {
    let Some((word, region)) = command_tokens(arguments) else {
        return SideVerdict::NotApplicable;
    };
    let Some(request): Option<SideRequest> = conflict_side_request(word.as_encoded_bytes(), region) else {
        return SideVerdict::NotApplicable;
    };
    let Some(layout) = read_layout(context.global_prefix.as_slice(), context.overlay.as_slice()) else {
        return SideVerdict::NotApplicable;
    };
    let Some(operation): Option<Operation> = operation_in_progress(layout.git_dir.as_path()) else {
        return SideVerdict::NotApplicable;
    };
    let conflicted = match read_conflicts(&layout) {
        Ok(found) => found,
        Err(note) => return SideVerdict::Refused(note),
    };
    match decide_sides(request.paths.as_slice(), layout.prefix.as_slice(), layout.worktree_root.as_path(), &conflicted) {
        SideDecision::Refuse(others) => return SideVerdict::Refused(clean_paths_note(others.as_slice(), operation)),
        SideDecision::Allow(paths) => {
            match save_copies(layout.worktree_root.as_path(), layout.common_dir.as_path(), paths.as_slice(), milliseconds) {
                Ok(copies) => {
                    prune_after_save(&layout, copies.as_slice(), milliseconds);
                    return SideVerdict::Allowed { notice: saved_notice(copies.as_slice()), copies };
                }
                Err(failure) => return SideVerdict::CopyFailed(copy_failed_text(&failure)),
            }
        }
    }
}
