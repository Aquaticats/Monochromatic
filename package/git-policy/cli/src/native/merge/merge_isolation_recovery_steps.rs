//! What:
//!  The steps both the recovery of an interrupted merge and the settling of a merge whose branch moved take:
//! where
//!  recovery runs Git,
//! what a recovery decides,
//! how a path of the merge stands in the index and the worktree,
//! and how an
//!  index entry or a file is put back.
//! Why:
//!  `merge_isolation_recovery.rs` recovers records and imports `merge_isolation_moved.rs`,
//! which settles a moved branch with
//!  these same steps;
//! the shared steps live in a file that imports neither,
//! so the two files no longer use each other.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const standing = standing(change, state); // 'merged' | 'untouched' | ...
//! ```

use crate::inner_git_words::record_words;
use crate::merge_isolation_changes::{Change, WorktreeState, parse_raw_changes, worktree_path};
use crate::merge_isolation_journal::{MergeIntent, remove_merge_intent};
use crate::recovery_error::{RecoveryError, io_failure};
use crate::transaction_git::{GitContext, GitRequest, run_git_checked};
use std::path::{Path, PathBuf};

/// What:
///  Where recovery runs Git:
/// the real Git and its environment,
/// the directory every command starts in,
/// and the
///  repository's common Git directory.
/// Why:
///   Recovery of a merge runs from the process directory with the invocation's global options,
/// whichever worktree the
///  record belongs to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RecoveryPlace = { gitPath: string; effectiveCwd: string; commonDir: string };
/// ```
#[derive(Clone, Debug)]
pub struct RecoveryPlace {
    /// Real Git and its environment.
    pub git: GitContext,
    /// Working directory of every Git command.
    pub cwd: PathBuf,
    /// The repository's common Git directory.
    pub common_dir: PathBuf,
}

/// What recovery did for one record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MergeRecoveryAction = 'owner-active' | 'discarded' | 'completed' | 'already-landed';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MergeRecoveryAction {
    /// The owner still runs;
    /// nothing was touched.
    OwnerActive,
    /// The merge never touched the worktree or the index;
    /// the record was removed.
    Discarded,
    /// Git had started updating the worktree;
    /// the merge was completed and the branch moved.
    Completed,
    /// The branch already names the merge commit;
    /// only the record was removed.
    AlreadyLanded,
    /// The branch names a commit that is neither the merge's parent nor the merge:
    /// it moved while the merge was prepared.
    /// What the merge had begun to install was put back to the moved head's content,
    /// so the checkout holds what it held
    ///  before the merge,
    /// and the record was removed;
    /// the merge can be computed again on the moved head.
    BranchMoved,
}

/// What:
///  The paths the merge changes,
/// with both sides of each.
/// Why:
///   They are exactly the paths Git updates for the merge and the only ones recovery may touch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function mergeChanges({ old, next }): Promise<Change[]>;
/// ```
pub fn merge_changes(place: &RecoveryPlace, root: &Path, intent: &MergeIntent) -> Result<Vec<Change>, RecoveryError> {
    let request: GitRequest = GitRequest::new(
        root,
        &["diff-tree", "-r", "-z", "--raw", "--no-renames", "--no-abbrev", intent.old_oid.as_str(), intent.new_oid.as_str()],
    );
    let output = match run_git_checked(&place.git, &request) {
        Ok(done) => done,
        Err(error) => return Err(RecoveryError(error.to_string())),
    };
    return parse_raw_changes(output.stdout.as_slice()).map_err(RecoveryError);
}

/// What:
///  Whether any index entry of a changed path already holds the merge's entry.
/// Why:
///   Only an entry equal to the merged one (or,
/// for a deleted path,
/// an absent one) shows that Git had begun to
///  install the merge's index;
/// another session's staged change to the same path is not that evidence,
/// and must
///  not make a merge that never began look like one that did.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function indexHoldsTheMerge({ place, root, oldOid, changes }): Promise<boolean>;
/// ```
pub fn index_holds_the_merge(
    place: &RecoveryPlace,
    root: &Path,
    old_oid: &str,
    changes: &[Change],
) -> Result<bool, RecoveryError> {
    let request: GitRequest = GitRequest::new(
        root,
        &["diff-index", "--cached", "-z", "--raw", "--no-renames", "--no-abbrev", old_oid, "--"],
    );
    let output = match run_git_checked(&place.git, &request) {
        Ok(done) => done,
        Err(error) => return Err(RecoveryError(error.to_string())),
    };
    let staged: Vec<Change> = parse_raw_changes(output.stdout.as_slice()).map_err(RecoveryError)?;
    for change in changes {
        let mut found: Option<&Change> = None;
        for candidate in &staged {
            if candidate.path == change.path {
                found = Some(candidate);
                break;
            }
        }
        let Some(entry) = found else {
            continue;
        };
        let merged_entry: bool = entry.new_mode == change.new_mode && entry.new_oid == change.new_oid;
        if merged_entry {
            return Ok(true);
        }
    }
    return Ok(false);
}

/// What the worktree holds at a changed path,
/// against the two sides of the merge.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Standing = 'old' | 'new' | 'foreign';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Standing {
    /// The old content,
    /// or no file where the old side has none.
    Old,
    /// The new content,
    /// or no file where the new side has none.
    New,
    /// Anything else.
    Foreign,
}

/// What:
///  Compare a worktree state with the two sides of a change.
/// Why:
///   The old side wins a tie:
/// a change that does not alter the content is not evidence of a started merge.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function standing(change: Change, state: WorktreeState): Standing;
/// ```
pub fn standing(change: &Change, state: &WorktreeState) -> Standing {
    let old_matches: bool = match state {
        WorktreeState::Missing => !change.existed(),
        WorktreeState::Content(oid) => change.existed() && *oid == change.old_oid,
        WorktreeState::Other => false,
    };
    if old_matches {
        return Standing::Old;
    }
    let new_matches: bool = match state {
        WorktreeState::Missing => !change.exists_after(),
        WorktreeState::Content(oid) => change.exists_after() && *oid == change.new_oid,
        WorktreeState::Other => false,
    };
    if new_matches {
        return Standing::New;
    }
    return Standing::Foreign;
}

/// What:
///  Remove a record,
/// naming it when that fails.
/// Why:
///   A record that cannot be removed would be recovered again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rm(record, { force: true });
/// ```
pub fn remove_record(record: &Path) -> Result<(), RecoveryError> {
    return match remove_merge_intent(record) {
        Ok(()) => Ok(()),
        Err(error) => Err(io_failure("removing", record, &error)),
    };
}

/// What:
///  Put the index entries and the worktree files of the changed paths to the state each change ends in.
/// Why:
///   The index entries are set first so a file written by `checkout-index` is recorded with the object it holds;
///  deletions go before writes so a file replaced by a directory of the same name can be created;
/// a path whose
///  worktree already holds the end state is not written again.
/// The merge's completion and the roll back to a moved
///  branch both end here,
/// with a different end state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function installEntries({ place, root, changes, standings }): Promise<void>;
/// ```
pub fn install_entries(
    place: &RecoveryPlace,
    root: &Path,
    changes: &[Change],
    standings: &[Standing],
) -> Result<(), RecoveryError> {
    let mut index_input: Vec<u8> = Vec::new();
    for change in changes {
        if change.exists_after() {
            index_input.extend_from_slice(format!("{} {}\t", change.new_mode, change.new_oid).as_bytes());
        } else {
            index_input.extend_from_slice(format!("0 {}\t", "0".repeat(change.old_oid.len())).as_bytes());
        }
        index_input.extend_from_slice(change.path.as_slice());
        index_input.push(0);
    }
    let mut update: GitRequest = GitRequest::new(root, &["update-index", "-z", "--index-info"]);
    update.input = Some(index_input);
    if let Err(error) = run_git_checked(&place.git, &update) {
        return Err(RecoveryError(error.to_string()));
    }
    let mut checkout_input: Vec<u8> = Vec::new();
    for (change, found) in changes.iter().zip(standings.iter()) {
        let location: PathBuf = worktree_path(root, change.path.as_slice());
        if !change.exists_after() {
            remove_file_and_empty_parents(root, location.as_path())?;
        } else if *found != Standing::New {
            checkout_input.extend_from_slice(change.path.as_slice());
            checkout_input.push(0);
        }
    }
    if !checkout_input.is_empty() {
        let mut checkout: GitRequest = GitRequest::new(root, &["checkout-index", "--force", "-z", "--stdin"]);
        checkout.input = Some(checkout_input);
        let checked_out = match run_git_checked(&place.git, &checkout) {
            Ok(done) => done,
            Err(error) => return Err(RecoveryError(error.to_string())),
        };
        // The checkout carries no `--quiet`; what Git said of it is kept in the journal.
        record_words(&["checkout-index", "--force", "-z", "--stdin"], &checked_out);
    }
    return Ok(());
}

/// What:
///  Delete a file the merge deletes,
/// and the directories that it leaves empty up to the worktree top.
/// Why:
///   Git removes a deleted file's emptied directories too;
/// a directory that still holds anything stays.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rm(file, { force: true }); for (const directory of parents(file)) { try { await rmdir(directory); } catch { break; } }
/// ```
pub fn remove_file_and_empty_parents(root: &Path, file: &Path) -> Result<(), RecoveryError> {
    match std::fs::remove_file(file) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(io_failure("removing", file, &error)),
    }
    let mut directory: Option<&Path> = file.parent();
    while let Some(current) = directory {
        if current == root || !current.starts_with(root) || std::fs::remove_dir(current).is_err() {
            break;
        }
        directory = current.parent();
    }
    return Ok(());
}
