//! What:
//!  The steps both the recovery of an interrupted merge and the settling of a merge whose branch moved take:
//!  the paths the merge changes,
//! what a recovery decides,
//! whether the index already holds the merge,
//!  the removal of a record,
//! and the index and file writes both steps reach.
//! Where those steps run is `merge_isolation_recovery_place.rs`,
//! and how they judge one changed path is `merge_isolation_the_merges_paths_judged.rs`.
//! Why:
//!  `merge_isolation_recovery.rs` recovers records and imports `merge_isolation_moved.rs`,
//! which settles a moved branch with
//!  these same steps;
//! the shared steps live in a file that imports neither,
//! so the two files no longer use each other.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const changes = await mergeChanges({ place, root, intent });
//! const began = await indexHoldsTheMerge({ place, root, oldOid, changes });
//! ```

/// The process's one runtime,
/// entered by the synchronous face of a read.
use crate::lazy_runtime::run_on_runtime;
/// The paths the merge changes and the raw reading of them.
use crate::merge_isolation_changes::{
    Change,
    parse_raw_changes,
};
/// The index reading,
/// and the take of a worktree file a merge must replace or delete,
/// re-exported so a caller of the recovery keeps one path.
pub use crate::merge_isolation_index_reading::{
    entries_by_path,
    index_entries,
    index_entries_awaiting,
    index_standings,
    index_standings_awaiting,
    standing_of_index,
};
use crate::merge_isolation_journal::{
    MergeIntent,
    remove_merge_intent,
};
/// The write of a merge into the index and the worktree,
/// named here so a caller of the recovery keeps one import.
pub use crate::merge_isolation_recovery_install::{
    install_entries,
    install_entries_awaiting,
    remove_empty_parents,
};
/// Where recovery runs Git,
/// named here so a caller of the recovery keeps one path.
pub use crate::merge_isolation_recovery_place::RecoveryPlace;
pub use crate::merge_isolation_take_asides::{
    Take,
    TakeFailure,
    TakenOnDisk,
    finish_asides,
    finish_asides_awaiting,
    put_back_awaiting,
    put_back_stranded_takes_awaiting,
    restore_what_git_did_not_write_awaiting,
    take_aside_awaiting,
};
/// How each path of the merge was judged,
/// named here so a caller of the recovery keeps one path.
pub use crate::merge_isolation_the_merges_paths_judged::{
    Judged,
    Standing,
    judge_changes_awaiting,
    standing,
};
use crate::recovery_error::{
    RecoveryError,
    io_failure,
};
/// The request one Git command of these steps is written as.
use crate::transaction_git::GitRequest;
use crate::transaction_git_awaiting::run_git_checked_awaiting;
/// `Path` is a borrowed filesystem path;
/// the steps own no path of their own.
use std::path::Path;

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
    /// Another program held `index.lock`,
    /// so nothing more of the merge could be written now,
    /// the record was kept,
    /// and the next command in the worktree settles it,
    /// in the state the settle read (`IndexHeldAt`).
    IndexHeld(IndexHeldAt),
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
///  What the checkout and the branch held when an index lock stopped the settle of a merge.
/// Why:
///  A lock can stop the settle before Git wrote anything,
/// after Git wrote the merge's own entries,
/// or while the branch already names the merge,
/// and each of those ends in different words.
/// The settle reads the three facts,
/// so the line a command prints for the merge it could not finish states what stands instead of
///  asserting one state for every one of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type IndexHeldAt = { theMergesEntriesInTheCheckout: boolean; landed: boolean; branchMoved: boolean };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IndexHeldAt {
    /// Some index entry or file of the merge is in the checkout.
    pub the_merges_entries_in_the_checkout: bool,
    /// The branch names the merge commit.
    pub landed: bool,
    /// The branch names a commit that is neither the merge's parent nor the merge.
    pub branch_moved: bool,
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
pub async fn merge_changes_awaiting(
    place: &RecoveryPlace,
    root: &Path,
    intent: &MergeIntent,
) -> Result<Vec<Change>, RecoveryError> {
    let request: GitRequest = GitRequest::new(
        root,
        &[
            "diff-tree",
            "-r",
            "-z",
            "--raw",
            "--no-renames",
            "--no-abbrev",
            intent.old_oid.as_str(),
            intent.new_oid.as_str(),
        ],
    );
    let output = match run_git_checked_awaiting(&place.git, &request).await {
        Ok(done) => done,
        Err(error) => return Err(RecoveryError(error.to_string())),
    };
    return parse_raw_changes(output.stdout.as_slice()).map_err(RecoveryError);
}

/// What:
///  The same paths of a merge,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   `merge_changes` enters the runtime around the tree difference,
/// which a task cannot do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function mergeChanges({ old, next }): Promise<Change[]>;
/// ```
pub fn merge_changes(
    place: &RecoveryPlace,
    root: &Path,
    intent: &MergeIntent,
) -> Result<Vec<Change>, RecoveryError> {
    return match run_on_runtime(merge_changes_awaiting(place, root, intent)) {
        Ok(found) => found,
        Err(refusal) => Err(RecoveryError(format!("{refusal}"))),
    };
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
pub async fn index_holds_the_merge_awaiting(
    place: &RecoveryPlace,
    root: &Path,
    old_oid: &str,
    changes: &[Change],
) -> Result<bool, RecoveryError> {
    let standings: Vec<Standing> = index_standings_awaiting(place, root, old_oid, changes).await?;
    return Ok(!standings.is_empty() && standings.iter().all(is_the_merges_entry));
}

/// Whether an index entry is the merge's own.
fn is_the_merges_entry(standing: &Standing) -> bool {
    return *standing == Standing::New;
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
