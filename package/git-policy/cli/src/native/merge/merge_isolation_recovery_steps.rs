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

use crate::index_holder_wait::{IndexWait, wait_for_index_release_while_finishing_a_merge};
use crate::inner_git_words::record_words;
use crate::merge_isolation_changes::{Change, FileIdentity, WorktreeState, parse_raw_changes, worktree_path};
use crate::merge_isolation_journal::{MergeIntent, remove_merge_intent};
use crate::recovery_error::{RecoveryError, io_failure};
use crate::merge_isolation_take_asides::{
    Take, put_back, remove_empty_parents, restore_what_git_did_not_write, take_aside,
};

/// The take of a worktree file a merge must replace or delete,
/// and the private names it leaves,
/// re-exported so a caller of the recovery keeps one path.
pub use crate::merge_isolation_take_asides::{TakenOnDisk, finish_asides, put_back_stranded_takes};
use crate::transaction_git::{GitContext, GitRequest, run_git_checked};
use std::io::Write;
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
    /// The Git directory of the worktree whose index recovery writes.
    pub git_dir: PathBuf,
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
    /// Another program held `index.lock` until the wait's ceiling,
    /// so no index entry and no file was written,
    /// the record was kept,
    /// and the next command in the worktree finishes the merge.
    IndexHeld,
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

/// What came of writing the index and the files of the merge.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Installed = 'written' | 'index-held';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Installed {
    /// Every entry and every file of the merge is in place.
    Written,
    /// Another program held `index.lock` until the wait's ceiling,
    /// so nothing was written;
    /// the merge is finished by the next command.
    IndexHeld,
}

/// What:
///  Whether the merge may be written after this wait for the index lock.
/// Why:
///  A wait that ended with the lock still held means the index cannot be written,
/// and trying would fail with Git's
///  own sentence in place of the record;
/// leaving the record and writing nothing is safe,
/// because the recovery is idempotent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mayInstall = (waited: IndexWait) => waited !== 'gave-up' && waited !== 'owner-gone';
/// ```
pub fn may_install(waited: IndexWait) -> bool {
    return match waited {
        IndexWait::NotHeld | IndexWait::Released(_) => true,
        IndexWait::GaveUp(_) | IndexWait::OwnerGone(_) => false,
    };
}

/// What the worktree held at one changed path when recovery judged it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Judged = { standing: Standing; identity?: FileIdentity };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Judged {
    /// How the path stood against the two sides of the change.
    pub standing: Standing,
    /// The file that stood there,
    /// as the file system named it,
    /// or nothing when nothing stood there.
    pub identity: Option<FileIdentity>,
}

/// The refusal for a merge whose files changed after it judged them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `A merge in ${root} was interrupted, and these files changed after this command judged them: ${paths}. ...`
/// ```
fn changed_after_judging(root: &Path, paths: &[String]) -> RecoveryError {
    return RecoveryError(format!(
        "A merge in {} was interrupted while Git updated these files, and they changed after this command judged them: {}. \
         Nothing was changed by this command and the merge is still unfinished; writing the merge over them would have \
         discarded that edit. Save those edits elsewhere, restore the files with `git restore -- <path>`, then run any git \
         command again so recovery can finish the merge.",
        root.display(),
        paths.join(", ")
    ));
}

/// What:
///  Put the index entries and the worktree files of the changed paths to the state each change ends in.
/// Why:
///   Git takes `index.lock` for the index write and dies when another program holds it,
/// so the write waits for
///  a holder first,
/// bounded and naming it (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`,
/// finding 2);
///  a lock that outlasts the bound leaves everything as it was and the record for the next command.
///   Every path that is not already at the state the change ends in is taken aside before anything is written,
/// and
///  the taken file is checked against what was judged,
/// so a file saved in between is never replaced with no notice
///  (finding 4);
/// one path that changed after it was judged puts all of them back and writes nothing.
///  The index entries are set first so a file written by `checkout-index` is recorded with the object it holds.
/// The
///  merge's completion and the roll back to a moved branch both end here,
/// with a different end state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function installEntries({ place, root, changes, judgements, stderr }): Promise<Installed>;
/// ```
pub fn install_entries(
    place: &RecoveryPlace,
    root: &Path,
    changes: &[Change],
    judgements: &[Judged],
    out: &mut dyn Write,
) -> Result<Installed, RecoveryError> {
    let waited: IndexWait = wait_for_index_release_while_finishing_a_merge(place.git_dir.join("index").as_path(), out);
    if !may_install(waited) {
        return Ok(Installed::IndexHeld);
    }
    // The take comes first: nothing of the merge is written while a file that was judged may already have changed.
    let mut taken_files: Vec<TakenOnDisk> = Vec::new();
    let mut checkout_input: Vec<u8> = Vec::new();
    let mut changed: Vec<String> = Vec::new();
    for (change, judged) in changes.iter().zip(judgements.iter()) {
        let location: PathBuf = worktree_path(root, change.path.as_slice());
        if judged.standing == Standing::New {
            // The path already holds the state the change ends in.
            continue;
        }
        if change.exists_after() {
            // Every path the merge writes is checked out, whether a file stood there or not.
            checkout_input.extend_from_slice(change.path.as_slice());
            checkout_input.push(0);
        }
        match take_aside(location.as_path(), judged.identity) {
            Ok(Take::Nothing) => {}
            Ok(Take::Changed) => changed.push(String::from_utf8_lossy(change.path.as_slice()).into_owned()),
            Ok(Take::Aside { aside, taken }) => taken_files.push(TakenOnDisk { location, aside, taken }),
            Err(error) => {
                // A take that stops must not leave the files it already took under their private names.
                let left: Vec<String> = put_back(&taken_files);
                let mut refusal: String = error.0;
                if !left.is_empty() {
                    refusal.push_str(format!(" Files that could not be put back: {}", left.join(", ")).as_str());
                }
                return Err(RecoveryError(refusal));
            }
        }
    }
    if !changed.is_empty() {
        let left: Vec<String> = put_back(&taken_files);
        let mut refusal: String = changed_after_judging(root, changed.as_slice()).0;
        refusal.push_str(left.join(", ").as_str());
        return Err(RecoveryError(refusal));
    }
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
        // The files are aside and the index is untouched: put them back rather than leave half a merge.
        let _restored = put_back(&taken_files);
        return Err(RecoveryError(error.to_string()));
    }
    // A deleted path's file was taken aside, so its directory can be reached even when the merge leaves it empty; the
    //  directories go once every aside name in them is gone.
    let mut deletions: Vec<PathBuf> = Vec::new();
    for (change, judged) in changes.iter().zip(judgements.iter()) {
        if change.exists_after() || judged.standing == Standing::New {
            continue;
        }
        deletions.push(worktree_path(root, change.path.as_slice()));
    }
    if !checkout_input.is_empty() {
        // Without `--force`: a file that appeared where the merge writes is skipped and Git says so,
        //  so the person's bytes are never replaced in the instant between the take and this write.
        let mut checkout: GitRequest = GitRequest::new(root, &["checkout-index", "-z", "--stdin"]);
        checkout.input = Some(checkout_input);
        let checked_out = match run_git_checked(&place.git, &checkout) {
            Ok(done) => done,
            Err(error) => {
                let restored: Result<(), RecoveryError> = restore_what_git_did_not_write(&taken_files, out);
                return Err(match restored {
                    Ok(()) => RecoveryError(error.to_string()),
                    Err(left) => RecoveryError(format!("{error} ({})", left.0)),
                });
            }
        };
        // The checkout carries no `--quiet`; what Git said of it is kept in the journal.
        record_words(&["checkout-index", "-z", "--stdin"], &checked_out);
    }
    finish_asides(&taken_files, out)?;
    for location in &deletions {
        // The take already moved the judged file away, so nothing of the merge's is left to delete here:
        // removing the name
        //  again would delete a file that appeared in this instant instead.
        remove_empty_parents(root, location.as_path());
    }
    return Ok(Installed::Written);
}
