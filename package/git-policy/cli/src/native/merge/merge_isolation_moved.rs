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
use crate::merge_isolation_recovery_cleanup::an_installed_merge_behind_a_held_index;
use crate::merge_isolation_recovery_steps::{
    Installed, Judged, MergeRecoveryAction, MergeRefusal, RecoveryPlace, Standing, index_holds_the_merge_awaiting,
    install_entries_awaiting, judge_changes_awaiting, merge_changes_awaiting, put_back_stranded_takes, remove_record,
};
use crate::merge_isolation_index_reading::{entries_by_path, index_entries_awaiting, standing_of_index};
/// The maps of records a settling asks by path.
use std::collections::HashMap;
/// The fail-closed recovery failure,
/// and the refusal that says whether the merge began.
use crate::recovery_error::RecoveryError;
/// Running real Git.
use crate::transaction_git::GitRequest;
use crate::transaction_git_awaiting::run_git_checked_awaiting;
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
///  A checkout holding the merge's file for one path and the branch's content for the others is a writing that was
///  killed between two files,
/// which is settling's work too,
/// and a path the moved head already holds the merge's own
///  content for is no evidence of one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function settleMovedBranch({ place, record, intent, movedTo, keepRecord, stderr }): Promise<'branch-moved' | 'index-held'>;
/// ```
pub(crate) async fn settle_moved_branch_awaiting<W: std::io::Write + ?Sized>(
    place: &RecoveryPlace,
    record: &Path,
    intent: &MergeIntent,
    moved_to: &str,
    keep_record: bool,
    out: &mut W,
) -> Result<MergeRecoveryAction, MergeRefusal> {
    let root: &Path = intent.repository_root.as_path();
    // A merge whose own commits cannot be read may still be half installed, so the refusal says so.
    let changes: Vec<Change> = match merge_changes_awaiting(place, root, intent).await {
        Ok(found) => found,
        Err(error) => return Err(MergeRefusal { error, began: true }),
    };
    put_back_stranded_takes(root, changes.as_slice(), out);
    // The installed state is read once here too,
    // the index first and the worktree second,
    // so a worktree that cannot be read is reported by what
    //  the index holds.
    let index_began: bool = match index_holds_the_merge_awaiting(place, root, intent.old_oid.as_str(), changes.as_slice()).await {
        Ok(found) => found,
        Err(error) => return Err(MergeRefusal { error, began: true }),
    };
    // The same judgement the straight completion makes,
    // so a directory the branch's own files explain is read the same way here.
    let worktree: Vec<Judged> = match judge_changes_awaiting(place, root, changes.as_slice()).await {
        Ok(found) => found,
        Err(error) => return Err(MergeRefusal { error, began: index_began }),
    };
    // A path holding the merge is what writing began,
    // because nothing but this merge puts the merge's own content in the checkout:
    // Git writes one file at a time,
    //  so a kill between two of them leaves the first one holding the merge and the rest holding the branch's,
    // and requiring
    //  every path was what let such a checkout be read as untouched,
    // its record removed and its file left behind
    //  (a cross-vendor review of the ninth round,
    // its fourth finding).
    let worktree_shows_the_merge: bool = worktree.iter().any(holds_the_merge);
    // A merge that left nothing of itself in the index or the worktree has nothing to settle, so the moved head is not read.
    if !index_began && !worktree_shows_the_merge {
        return only_the_record_goes(record, false, keep_record);
    }
    // What each path ends as: the moved head's entry, which is the merge's own entry where the moved head left the path alone.
    let request: GitRequest = GitRequest::new(
        root,
        &["diff-tree", "-r", "-z", "--raw", "--no-renames", "--no-abbrev", intent.new_oid.as_str(), moved_to],
    );
    let output = match run_git_checked_awaiting(&place.git, &request).await {
        Ok(done) => done,
        // The moved head cannot be read, so a path holding the merge cannot be told from an independent commit of its content.
        Err(error) => return Err(MergeRefusal { error: RecoveryError(error.to_string()), began: index_began || worktree_shows_the_merge }),
    };
    let beyond: Vec<Change> = match parse_raw_changes(output.stdout.as_slice()) {
        Ok(found) => found,
        Err(error) => return Err(MergeRefusal { error: RecoveryError(error), began: index_began || worktree_shows_the_merge }),
    };
    // A file holding the merge is evidence of an install only where the moved head does not already hold that content:
    // an
    //  independent commit of the same content leaves the very file the merge would,
    // and writing it then would replace a
    //  person's own file for nothing.
    // One pass over the moved head's records answers every path asked about below, so neither list is walked per change
    //  (`AGENTS.md` `RG2`).
    let beyond_by_path: HashMap<&[u8], &Change> = entries_by_path(beyond.as_slice());
    let mut wrote_a_file: bool = false;
    for (judged, change) in worktree.iter().zip(changes.iter()) {
        if holds_the_merge(judged) && moved_head_holds_its_own_entry(&beyond_by_path, change.path.as_slice()) {
            wrote_a_file = true;
            break;
        }
    }
    let began: bool = index_began || wrote_a_file;
    if !began {
        return only_the_record_goes(record, began, keep_record);
    }
    let mut ends: Vec<Change> = Vec::with_capacity(changes.len());
    for change in &changes {
        let moved_entry: Option<&Change> = beyond_by_path.get(change.path.as_slice()).copied();
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
    let staged_index: Vec<Change> = match index_entries_awaiting(place, root, intent.old_oid.as_str()).await {
        Ok(found) => found,
        Err(error) => return Err(MergeRefusal { error, began }),
    };
    let staged_by_path: HashMap<&[u8], &Change> = entries_by_path(staged_index.as_slice());
    let mut staged: Vec<String> = Vec::new();
    for end in &ends {
        let entry: Option<&Change> = staged_by_path.get(end.path.as_slice()).copied();
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
    if install_entries_awaiting(place, root, ends.as_slice(), held.as_slice(), began, out).await? == Installed::IndexHeld {
        // A settling that had already begun has nothing to defer: the command is stopped rather than read the merge as
        //  another session's staged work.
        if began {
            return Err(partly_installed(an_installed_merge_behind_a_held_index(
                intent.target_ref.as_deref().unwrap_or("HEAD"),
                root,
                place.git_dir.join("index.lock").as_path(),
                record,
            )));
        }
        return Ok(MergeRecoveryAction::IndexHeld);
    }
    if !keep_record {
        remove_record(record).map_err(partly_installed)?;
    }
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
/// which is the evidence that Git wrote that path.
fn holds_the_merge(judged: &Judged) -> bool {
    return judged.standing == Standing::New;
}

/// What:
///  Whether the moved head holds an entry of its own for one path,
///  rather than the merge's own entry there.
/// Why:
///   Where the moved head left a path alone,
/// it holds the merge's entry and so its content,
/// and a file holding
///  that content is explained by the head without an install;
/// an entry of its own is what makes the file
///  evidence that the merge wrote it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function movedHeadHoldsItsOwnEntry(beyond, path): boolean; // beyond.has(path)
/// ```
fn moved_head_holds_its_own_entry(beyond: &HashMap<&[u8], &Change>, path: &[u8]) -> bool {
    crate::collected_elements_looked_at::note_looked_at(1);
    return beyond.contains_key(path);
}

/// What:
///  Drop the record of a merge that left nothing of itself in the checkout,
///  and say the branch moved.
/// Why:
///   The record is the only thing of such a merge,
/// so it goes and nothing else is written;
/// a record that cannot be
///  removed is reported with what the caller knows of the install.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function onlyTheRecordGoes(record, began): 'branch-moved'; // rm(record)
/// ```
fn only_the_record_goes(record: &Path, began: bool, keep_record: bool) -> Result<MergeRecoveryAction, MergeRefusal> {
    if !keep_record && let Err(error) = remove_record(record) {
        return Err(MergeRefusal { error, began });
    }
    return Ok(MergeRecoveryAction::BranchMoved);
}
