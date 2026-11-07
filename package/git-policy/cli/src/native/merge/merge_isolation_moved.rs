//! What:
//!  The settling of an isolated merge whose branch moved while the merge was prepared or applied:
//! what the merge had begun
//!  to install is put back to the moved head's content,
//! and the record goes,
//! so the merge can be computed again on the
//!  moved head.
//! Why:
//!  Git's fast-forward installs the checkout of the paths the merge changes and then moves the branch from the commit the
//!  merge was computed on.
//! When another session (or the auto-push's reconcile merge) moved the branch first,
//! the move is
//!  refused:
//! nothing was installed if Git refused before it began,
//! and the merge's paths are installed beside a branch
//!  that does not hold them if it lost the race in the instant between.
//! Both end with the checkout showing what the
//!  moved head holds for those paths,
//! never a half-applied merge and never a record that fails every later command.
//!  Only paths that hold the old,
//! the merged or the moved head's content are written;
//! any other content belongs to
//!  someone else and stops the settling,
//! naming the paths,
//! with the record kept.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await settleMovedBranch({ place, record, intent, movedTo }); // 'branch-moved'
//! ```

/// The raw differences and the worktree states.
use crate::merge_isolation_changes::{Change, WorktreeState, parse_raw_changes, worktree_state};
/// The intent record.
use crate::merge_isolation_journal::MergeIntent;
/// The shared steps of recovery.
use crate::merge_isolation_recovery_steps::{
    MergeRecoveryAction, RecoveryPlace, Standing, index_holds_the_merge, install_entries, merge_changes, remove_record,
    standing,
};
/// The fail-closed recovery failure.
use crate::recovery_error::RecoveryError;
/// Running real Git.
use crate::transaction_git::{GitRequest, run_git_checked};
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// What:
///  Settle a merge whose branch is at `moved_to`,
/// which is neither its old nor its new commit.
/// Why:
///   If neither the index nor the worktree holds anything of the merge,
/// the record only goes.
/// Otherwise each path the merge
///  changes is set to the moved head's content,
/// in the index and in the worktree,
/// so the checkout matches the head again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function settleMovedBranch({ place, record, intent, movedTo }): Promise<'branch-moved'>;
/// ```
pub(crate) fn settle_moved_branch(
    place: &RecoveryPlace,
    record: &Path,
    intent: &MergeIntent,
    moved_to: &str,
) -> Result<MergeRecoveryAction, RecoveryError> {
    let root: &Path = Path::new(intent.repository_root.as_str());
    let changes: Vec<Change> = merge_changes(place, root, intent)?;
    let mut states: Vec<WorktreeState> = Vec::with_capacity(changes.len());
    for change in &changes {
        states.push(worktree_state(&place.git, root, change.path.as_slice()).map_err(RecoveryError)?);
    }
    let mut worktree_began: bool = false;
    for (change, state) in changes.iter().zip(states.iter()) {
        if standing(change, state) == Standing::New {
            worktree_began = true;
            break;
        }
    }
    // The index is read first and the worktree standings only decide when it holds nothing of the merge, as `||` ran.
    let began: bool = index_holds_the_merge(place, root, intent.old_oid.as_str(), changes.as_slice())? || worktree_began;
    if !began {
        remove_record(record)?;
        return Ok(MergeRecoveryAction::BranchMoved);
    }
    // What each path ends as: the moved head's entry, which is the merge's own entry where the moved head left the path alone.
    let request: GitRequest = GitRequest::new(
        root,
        &["diff-tree", "-r", "-z", "--raw", "--no-renames", "--no-abbrev", intent.new_oid.as_str(), moved_to],
    );
    let output = match run_git_checked(&place.git, &request) {
        Ok(done) => done,
        Err(error) => return Err(RecoveryError(error.to_string())),
    };
    let beyond: Vec<Change> = parse_raw_changes(output.stdout.as_slice()).map_err(RecoveryError)?;
    let mut ends: Vec<Change> = Vec::with_capacity(changes.len());
    for change in &changes {
        let mut moved_entry: Option<&Change> = None;
        for moved in &beyond {
            if moved.path == change.path {
                moved_entry = Some(moved);
                break;
            }
        }
        let (mode, oid): (String, String) = match moved_entry {
            Some(moved) => (moved.new_mode.clone(), moved.new_oid.clone()),
            None => (change.new_mode.clone(), change.new_oid.clone()),
        };
        ends.push(Change {
            path: change.path.clone(),
            old_mode: change.new_mode.clone(),
            new_mode: mode,
            old_oid: change.new_oid.clone(),
            new_oid: oid,
        });
    }
    let mut foreign: Vec<String> = Vec::new();
    let mut held: Vec<Standing> = Vec::with_capacity(ends.len());
    for ((change, end), state) in changes.iter().zip(ends.iter()).zip(states.iter()) {
        let holds_old_or_merged: bool = standing(change, state) != Standing::Foreign;
        let holds_end: bool = match state {
            WorktreeState::Missing => !end.exists_after(),
            WorktreeState::Content(oid) => end.exists_after() && *oid == end.new_oid,
            WorktreeState::Other => false,
        };
        if !holds_old_or_merged && !holds_end {
            foreign.push(String::from_utf8_lossy(change.path.as_slice()).into_owned());
        }
        held.push(if holds_end { Standing::New } else { Standing::Old });
    }
    if !foreign.is_empty() {
        return Err(RecoveryError(format!(
            "A merge in {} lost the branch to a newer commit while Git updated these files, and they now hold neither \
             their old content, the merge's nor the newer commit's: {}. Nothing was changed and the record {} was kept. \
             Save your edits to them elsewhere, restore them with `git restore -- <path>`, then run any git command again.",
            root.display(),
            foreign.join(", "),
            record.display()
        )));
    }
    install_entries(place, root, ends.as_slice(), held.as_slice())?;
    remove_record(record)?;
    return Ok(MergeRecoveryAction::BranchMoved);
}
