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
use crate::merge_isolation_changes::{
    Change, FileIdentity, WorktreeState, file_identity, parse_raw_changes, worktree_path, worktree_state,
};
use crate::merge_isolation_journal::{MergeIntent, remove_merge_intent};
use crate::recovery_error::{RecoveryError, io_failure};
/// The index reading,
/// and the take of a worktree file a merge must replace or delete,
/// re-exported so a caller of the recovery keeps one path.
pub use crate::merge_isolation_index_reading::{entries_by_path, index_entries, index_standings, standing_of_index};
pub use crate::merge_isolation_take_asides::{
    Take, TakeFailure, TakenOnDisk, finish_asides, put_back, put_back_stranded_takes, remove_empty_parents,
    restore_what_git_did_not_write, take_aside,
};
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

/// What:
///  The place recovery runs in when a command that started its own merge has to settle it.
/// Why:
///   The two directories stay apart:
///  the worktree's own Git directory is what recovery writes through,
///  and the repository's common one is where every worktree of it keeps what they share.
///  A linked worktree's own Git directory lives under the main repository's `worktrees` directory,
///  and only its `commondir` file names the repository's own
///  (`foundation/common_directory_of_a_git_directory.rs`),
///  so filling the common field with the worktree's directory would point every reader of it
///  at a directory that goes away with the worktree.
///  The context carries the invocation's common directory;
///  one built without it still knows the worktree's Git directory,
/// whose `commondir` names the rest.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const place = { git, cwd: root, commonDir: context.commonDir ?? commonDirectoryOf(gitDir), gitDir };
/// ```
impl RecoveryPlace {
    /// The place a settling command needs,
    /// from its context and the two directories.
    pub fn settling(git: &GitContext, git_dir: &Path, root: &Path) -> RecoveryPlace {
        let common_dir: PathBuf = match git.common_dir.as_ref() {
            Some(found) => found.clone(),
            None => crate::common_directory_of_a_git_directory::common_directory_of(git_dir),
        };
        return RecoveryPlace { git: GitContext { global_prefix: Vec::new(), ..git.clone() }, cwd: root.to_path_buf(), common_dir, git_dir: git_dir.to_path_buf() };
    }
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
    /// The branch advanced after the merge was saved on a side ref,
    /// so that merge cannot land as it was saved;
    /// the record went and the
    ///  side ref keeps the merge commit,
    /// which the line names.
    SavedBranchMoved,
    /// A merge that was saved on a side ref could not be finished now;
    /// nothing of it is in the checkout,
    /// so the command goes
    ///  on and the line says where the merge is and what blocked it.
    DeferredRefused,
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
///  Whether every changed path's index entry already holds the merge's entry.
/// Why:
///   Git installs the index for all the paths of a merge in one step,
/// so every entry being the merge's is what an install
///  leaves;
/// one entry equal to the merged one is just as much what an independent commit of the same content
///  leaves,
/// and it must not make a merge that never began look like one that did.
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
    let standings: Vec<Standing> = index_standings(place, root, old_oid, changes)?;
    return Ok(!standings.is_empty() && standings.iter().all(is_the_merges_entry));
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
    /// A directory that stands where the merge puts a file,
    /// and that holds only files of the branch,
    /// which the merge itself
    ///  deletes.
    /// Nothing of anyone's is there to set aside,
    /// and the merge's own deletions empty it.
    Directory,
    /// Anything else.
    Foreign,
}

/// What:
///  Compare a worktree state with the two sides of a change,
///  content and the mode Git records for it.
/// Why:
///   The old side wins a tie:
/// a change that does not alter the content is not evidence of a started merge.
/// A mode is read as part of the file,
/// because Git records it in the index entry:
/// a file whose bytes
///  are the branch's and whose executable bit is not has been changed by someone,
/// and replacing it would erase
///  that change
///  (a cross-vendor review of the ninth round,
/// its seventh finding).
/// A repository that records no
///  executable bit reads no mode difference at all,
/// as Git does there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function standing(change: Change, state: WorktreeState, recordsFileModes: boolean): Standing;
/// ```
pub fn standing(change: &Change, state: &WorktreeState, records_file_modes: bool) -> Standing {
    let old_matches: bool = match state {
        WorktreeState::Missing => !change.existed(),
        WorktreeState::Content { object, mode } => {
            change.existed() && *object == change.old_oid && (!records_file_modes || *mode == change.old_mode)
        }
        WorktreeState::Other => false,
    };
    if old_matches {
        return Standing::Old;
    }
    let new_matches: bool = match state {
        WorktreeState::Missing => !change.exists_after(),
        WorktreeState::Content { object, mode } => {
            change.exists_after() && *object == change.new_oid && (!records_file_modes || *mode == change.new_mode)
        }
        WorktreeState::Other => false,
    };
    if new_matches {
        return Standing::New;
    }
    return Standing::Foreign;
}

/// What:
///  The judgement of every changed path,
///  with a directory the branch's own files explain read as the old side.
/// Why:
///  `worktree_state` reads a directory as neither side of a change,
/// so a merge that replaces a directory with a
///  file was refused for good,
/// although the directory holds only the branch's files,
/// which the merge's own
///  deletions take
/// (a cross-vendor review of the second round):
/// a path whose old side has no entry of its own is
///  the merge's to replace when the branch stores files below it and the merge deletes them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function judgeChanges({ place, root, changes }): Judged[]; // standing(...), then explainedDirectory(...) ? 'directory' : one
/// ```
pub fn judge_changes(place: &RecoveryPlace, root: &Path, changes: &[Change]) -> Result<Vec<Judged>, RecoveryError> {
    // Whether a mode is part of the file here is one question for the whole judgement,
    // and it is the repository's answer, not this
    //  command's.
    let records_file_modes: bool = crate::merge_isolation_changes::the_repository_records_file_modes(&place.git, root);
    let mut judgements: Vec<Judged> = Vec::with_capacity(changes.len());
    for change in changes {
        let location: PathBuf = worktree_path(root, change.path.as_slice());
        // The identity is read before the content, so a file written in between does not match the judgement and is never
        //  replaced.
        let identity: Option<FileIdentity> = file_identity(location.as_path());
        let state: WorktreeState = worktree_state(&place.git, root, change.path.as_slice()).map_err(RecoveryError)?;
        let content: Option<String> = match &state {
            WorktreeState::Content { object, .. } => Some(object.clone()),
            WorktreeState::Missing | WorktreeState::Other => None,
        };
        judgements.push(Judged { standing: standing(change, &state, records_file_modes), identity, content });
    }
    for (index, change) in changes.iter().enumerate() {
        if judgements[index].standing != Standing::Foreign || change.existed() || !change.exists_after() {
            continue;
        }
        let location: PathBuf = worktree_path(root, change.path.as_slice());
        let stands_as_a_directory: bool = std::fs::symlink_metadata(location.as_path()).map(is_a_directory).unwrap_or(false);
        if !stands_as_a_directory
            || !the_branchs_files_stand_below(change, changes)
            || !crate::merge_isolation_changes::the_directory_holds_only_what_the_merge_deletes(change, changes, root)
        {
            continue;
        }
        judgements[index] = Judged { standing: Standing::Directory, identity: None, content: None };
    }
    return Ok(judgements);
}

/// Whether an index entry is the merge's own.
fn is_the_merges_entry(standing: &Standing) -> bool {
    return *standing == Standing::New;
}

/// Whether one file system entry is a directory.
fn is_a_directory(found: std::fs::Metadata) -> bool {
    return found.is_dir();
}

/// Whether one change deletes a file of the branch below `prefix`.
fn takes_a_file_below(other: &Change, prefix: &[u8]) -> bool {
    return other.path.starts_with(prefix) && other.existed() && !other.exists_after();
}

/// What:
///  Whether the branch stores files below a path that the merge deletes them from,
///  which is what a directory standing
///  there means.
/// Why:
///   A directory where the merge puts a file is the branch's own only when the merge takes every file in it;
/// a
///  directory nobody's files explain stays someone else's and is refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// changes.some((other) => other.path.startsWith(`${change.path}/`) && other.existed() && !other.existsAfter());
/// ```
fn the_branchs_files_stand_below(change: &Change, changes: &[Change]) -> bool {
    let mut prefix: Vec<u8> = change.path.clone();
    prefix.push(b'/');
    for other in changes {
        if takes_a_file_below(other, prefix.as_slice()) {
            return true;
        }
    }
    return false;
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
///  Why a merge stopped,
///  and whether anything of it is already in the index or the worktree.
/// Why:
///   A merge that was saved on a side ref with nothing installed may be left for a later command,
/// while one whose index
///  has been written or whose file has been moved aside must stop the command:
/// a read-only command that went on
///  would read a half-applied merge as another session's work
///  (a cross-vendor review of the third round,
/// finding 5).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MergeRefusal = { error: RecoveryError; began: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MergeRefusal {
    /// What stopped the merge.
    pub error: RecoveryError,
    /// Whether the index or the worktree already holds part of the merge.
    pub began: bool,
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Judged {
    /// How the path stood against the two sides of the change.
    pub standing: Standing,
    /// The file that stood there,
    /// as the file system named it,
    /// or nothing when nothing stood there.
    pub identity: Option<FileIdentity>,
    /// The object the worktree held there,
    /// or nothing when it held no file:
    /// the bytes a take must still find when it finishes the file.
    pub content: Option<String>,
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
///  Every file a take set aside,
///  the merge's writes and its deletions together,
///  for the paths that put them all back.
/// Why:
///  A statement that puts files back must not know which of them the merge writes and which it deletes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// [...written, ...deleted];
/// ```
fn everything_taken(written: &[TakenOnDisk], deleted: &[TakenOnDisk]) -> Vec<TakenOnDisk> {
    let mut all: Vec<TakenOnDisk> = Vec::with_capacity(written.len() + deleted.len());
    for entry in written.iter().chain(deleted.iter()) {
        all.push(TakenOnDisk {
            location: entry.location.clone(),
            path: entry.path.clone(),
            aside: entry.aside.clone(),
            taken: entry.taken,
            expected: entry.expected.clone(),
        });
    }
    return all;
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
    installed_before: bool,
    out: &mut dyn Write,
) -> Result<Installed, MergeRefusal> {
    let waited: IndexWait = wait_for_index_release_while_finishing_a_merge(place.git_dir.join("index").as_path(), out);
    if !may_install(waited) {
        return Ok(Installed::IndexHeld);
    }
    // The take comes first: nothing of the merge is written while a file that was judged may already have changed.
    let mut taken_writes: Vec<TakenOnDisk> = Vec::new();
    let mut taken_deletions: Vec<TakenOnDisk> = Vec::new();
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
        if judged.standing == Standing::Directory {
            // The directory holds only the branch's files, which the merge's own deletions take, so there is nothing of
            //  anyone's to set aside; the deletions empty it before the checkout writes the file.
            continue;
        }
        match take_aside(place, root, change.path.as_slice(), location.as_path(), judged) {
            Ok(Take::Nothing) => {}
            Ok(Take::Changed) => changed.push(String::from_utf8_lossy(change.path.as_slice()).into_owned()),
            Ok(Take::Aside { aside, taken, expected }) => {
                let entry: TakenOnDisk = TakenOnDisk { location, path: change.path.clone(), aside, taken, expected };
                if change.exists_after() {
                    taken_writes.push(entry);
                } else {
                    taken_deletions.push(entry);
                }
            }
            Err(failure) => {
                // A take that stops must not leave the files it already took under their private names; the one it stopped on is
                //  in the failure, because the rename came before the read that failed.
                let left: Vec<String> = put_back(&everything_taken(&taken_writes, &taken_deletions));
                let mut refusal: String = failure.error.0;
                if !left.is_empty() {
                    refusal.push_str(format!(" Files that could not be put back: {}", left.join(", ")).as_str());
                }
                let began: bool = installed_before || failure.stranded.is_some() || !left.is_empty();
                return Err(MergeRefusal { error: RecoveryError(refusal), began });
            }
        }
    }
    if !changed.is_empty() {
        let left: Vec<String> = put_back(&everything_taken(&taken_writes, &taken_deletions));
        let mut refusal: String = changed_after_judging(root, changed.as_slice()).0;
        refusal.push_str(left.join(", ").as_str());
        return Err(MergeRefusal { error: RecoveryError(refusal), began: installed_before || !left.is_empty() });
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
        let left: Vec<String> = put_back(&everything_taken(&taken_writes, &taken_deletions));
        return Err(MergeRefusal { error: RecoveryError(error.to_string()), began: installed_before || !left.is_empty() });
    }
    // From here the index holds the merge's entries, so a command that stops must not go on reading the checkout.
    let began: bool = true;
    // A deleted path's file was taken aside, so its directory can be reached even when the merge leaves it empty; the
    //  directories go once every aside name in them is gone,
    // and they go before the checkout,
    // because a merge may put a file
    //  where one of them stood.
    let mut deletions: Vec<PathBuf> = Vec::new();
    for change in changes {
        if change.exists_after() {
            continue;
        }
        // Every path the merge deletes has its emptied directories pruned,
        // whether or not it already stands at its end state:
        // a
        //  recovery killed between finishing a child's private name and removing the directory it emptied leaves that
        //  directory behind,
        // and only an empty directory is ever removed here.
        deletions.push(worktree_path(root, change.path.as_slice()));
    }
    if let Err(error) = finish_asides(place, root, &taken_deletions, out) {
        return Err(MergeRefusal { error, began });
    }
    for location in &deletions {
        remove_empty_parents(root, location.as_path());
    }
    if !checkout_input.is_empty() {
        // Without `--force`: a file that appeared where the merge writes is skipped and Git says so,
        //  so the person's bytes are never replaced in the instant between the take and this write.
        let mut checkout: GitRequest = GitRequest::new(root, &["checkout-index", "-z", "--stdin"]);
        checkout.input = Some(checkout_input);
        let checked_out = match run_git_checked(&place.git, &checkout) {
            Ok(done) => done,
            Err(error) => {
                let restored: Result<(), RecoveryError> = restore_what_git_did_not_write(place, root, &taken_writes, out);
                return Err(match restored {
                    Ok(()) => MergeRefusal { error: RecoveryError(error.to_string()), began },
                    Err(left) => MergeRefusal { error: RecoveryError(format!("{error} ({})", left.0)), began },
                });
            }
        };
        // The checkout carries no `--quiet`; what Git said of it is kept in the journal.
        record_words(&["checkout-index", "-z", "--stdin"], &checked_out);
    }
    if let Err(error) = finish_asides(place, root, &taken_writes, out) {
        return Err(MergeRefusal { error, began });
    }
    return Ok(Installed::Written);
}
