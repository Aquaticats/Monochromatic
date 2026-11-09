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
use crate::merge_isolation_changes::{Change, parse_raw_changes};
/// The intent record.
use crate::merge_isolation_journal::MergeIntent;
/// The shared steps of recovery.
use crate::merge_isolation_recovery_steps::{
    Installed, Judged, MergeRecoveryAction, MergeRefusal, RecoveryPlace, Standing, index_holds_the_merge, install_entries,
    judge_changes, merge_changes, put_back_stranded_takes, remove_record,
};
use crate::merge_isolation_index_reading::{index_entries, standing_of_index};
/// The fail-closed recovery failure,
/// and the refusal that says whether the merge began.
use crate::recovery_error::RecoveryError;
/// Running real Git.
use crate::transaction_git::{GitRequest, run_git_checked};
/// Writing the notices of the wait for the index lock.
use std::io::Write;
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
/// async function settleMovedBranch({ place, record, intent, movedTo, stderr }): Promise<'branch-moved' | 'index-held'>;
/// ```
pub(crate) fn settle_moved_branch(
    place: &RecoveryPlace,
    record: &Path,
    intent: &MergeIntent,
    moved_to: &str,
    out: &mut dyn Write,
) -> Result<MergeRecoveryAction, MergeRefusal> {
    let root: &Path = intent.repository_root.as_path();
    // A merge whose own commits cannot be read may still be half installed, so the refusal says so.
    let changes: Vec<Change> = match merge_changes(place, root, intent) {
        Ok(found) => found,
        Err(error) => return Err(MergeRefusal { error, began: true }),
    };
    put_back_stranded_takes(root, changes.as_slice(), out);
    // The installed state is read once here too,
    // the index first and the worktree second,
    // so a worktree that cannot be read is reported by what
    //  the index holds.
    let index_began: bool = match index_holds_the_merge(place, root, intent.old_oid.as_str(), changes.as_slice()) {
        Ok(found) => found,
        Err(error) => return Err(MergeRefusal { error, began: true }),
    };
    // The same judgement the straight completion makes,
    // so a directory the branch's own files explain is read the same way here.
    let worktree: Vec<Judged> = match judge_changes(place, root, changes.as_slice()) {
        Ok(found) => found,
        Err(error) => return Err(MergeRefusal { error, began: index_began }),
    };
    let worktree_began: bool = worktree.iter().any(holds_the_merge);
    let began: bool = index_began || worktree_began;
    if !began {
        if let Err(error) = remove_record(record) {
            return Err(MergeRefusal { error, began });
        }
        return Ok(MergeRecoveryAction::BranchMoved);
    }
    // What each path ends as: the moved head's entry, which is the merge's own entry where the moved head left the path alone.
    let request: GitRequest = GitRequest::new(
        root,
        &["diff-tree", "-r", "-z", "--raw", "--no-renames", "--no-abbrev", intent.new_oid.as_str(), moved_to],
    );
    let output = match run_git_checked(&place.git, &request) {
        Ok(done) => done,
        Err(error) => return Err(MergeRefusal { error: RecoveryError(error.to_string()), began }),
    };
    let beyond: Vec<Change> = match parse_raw_changes(output.stdout.as_slice()) {
        Ok(found) => found,
        Err(error) => return Err(MergeRefusal { error: RecoveryError(error), began }),
    };
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
    // A staged version of a path the merge changes has no other name, so it is never written over here either: the settling
    //  stops and names the paths, and the staged content stays in the index.
    let staged_index: Vec<Change> = match index_entries(place, root, intent.old_oid.as_str()) {
        Ok(found) => found,
        Err(error) => return Err(MergeRefusal { error, began }),
    };
    let mut staged: Vec<String> = Vec::new();
    for end in &ends {
        let entry: Option<&Change> = entry_of_path(staged_index.as_slice(), end.path.as_slice());
        if standing_of_index(entry, end, end) == Standing::Foreign {
            staged.push(String::from_utf8_lossy(end.path.as_slice()).into_owned());
        }
    }
    if !staged.is_empty() {
        return Err(MergeRefusal {
            error: RecoveryError(format!(
            "A merge in {} lost the branch to a newer commit, and the index holds staged content for these paths that the \
             settling would replace: {}. Nothing was changed, the record {} was kept, and the staged content is still in the \
             index. Commit or unstage it, then run any git command again.",
            root.display(),
            staged.join(", "),
            record.display()
            )),
            began,
        });
    }
    let mut foreign: Vec<String> = Vec::new();
    let mut held: Vec<Judged> = Vec::with_capacity(ends.len());
    for ((judged, end), change) in worktree.iter().zip(ends.iter()).zip(changes.iter()) {
        let holds_old_or_merged: bool = judged.standing != Standing::Foreign;
        let holds_end: bool = match &judged.content {
            Some(oid) => end.exists_after() && *oid == end.new_oid,
            None => judged.standing != Standing::Directory && !end.exists_after(),
        };
        if !holds_old_or_merged && !holds_end {
            foreign.push(String::from_utf8_lossy(change.path.as_slice()).into_owned());
        }
        // A directory the branch's own files explain stays a directory here too: the settling writes the moved head's
        //  end state, and nothing of anyone's is inside it.
        let settled: Standing = if holds_end {
            Standing::New
        } else if judged.standing == Standing::Directory {
            Standing::Directory
        } else {
            Standing::Old
        };
        held.push(Judged { standing: settled, identity: judged.identity, content: judged.content.clone() });
    }
    if !foreign.is_empty() {
        return Err(MergeRefusal {
            error: RecoveryError(format!(
            "A merge in {} lost the branch to a newer commit while Git updated these files, and they now hold neither \
             their old content, the merge's nor the newer commit's: {}. Nothing was changed and the record {} was kept. \
             Save your edits to them elsewhere, restore them with `git restore -- <path>`, then run any git command again.",
            root.display(),
            foreign.join(", "),
            record.display()
            )),
            began,
        });
    }
    if install_entries(place, root, ends.as_slice(), held.as_slice(), began, out)? == Installed::IndexHeld {
        return Ok(MergeRecoveryAction::IndexHeld);
    }
    remove_record(record).map_err(partly_installed)?;
    return Ok(MergeRecoveryAction::BranchMoved);
}

/// What:
///  The refusal of a step that ran after the index was written,
///  so the merge is partly installed.
/// Why:
///   Named once for both step types that can fail there,
/// and it says what the flag means.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const partlyInstalled = (error: RecoveryError): MergeRefusal => ({ error, began: true });
/// ```
fn partly_installed(error: RecoveryError) -> MergeRefusal {
    return MergeRefusal { error, began: true };
}

/// Whether one judgement holds the merge's content,
/// which is the only evidence that Git began.
fn holds_the_merge(judged: &Judged) -> bool {
    return judged.standing == Standing::New;
}

/// The index entry of one path,
/// or nothing when the index reports none for it.
fn entry_of_path<'a>(staged: &'a [Change], path: &[u8]) -> Option<&'a Change> {
    for candidate in staged {
        if candidate.path == path {
            return Some(candidate);
        }
    }
    return None;
}
