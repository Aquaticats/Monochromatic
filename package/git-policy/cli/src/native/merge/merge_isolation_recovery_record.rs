//! What:
//!  The vocabulary a merge's recovery record is written in:
//!  what a recovery did with one record,
//!  what the checkout and the branch held when an index lock stopped it,
//!  why a merge stopped and whether anything of it was written,
//!  what came of writing its index entries and files,
//!  and the record of one merge that recovery visited.
//! Why:
//!  The recovery of an interrupted merge,
//!  the settling of a merge whose branch moved,
//!  the take of a worktree file and the copy a merge kept beside a path
//!  all answer with these,
//!  so they live in one file that names none of them
//!  (`AGENTS.md` rule LYF:
//! a folder's files never use each other in a loop).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type MergeRecoveryAction = 'owner-active' | 'discarded' | 'completed' | 'already-landed' | 'branch-moved' | 'index-held' | 'saved-branch-moved' | 'deferred-refused';
//! ```

/// Why a merge stopped and whether any of it was written,
/// which every caller answers a refusal with.
use crate::recovery_error::RecoveryError;
/// The record's own file,
/// which a merged outcome names back to the command.
use std::path::PathBuf;

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
///  The refusal of a step that ran after the index was written,
///  so the merge is partly installed.
/// Why:
///   The branch write and the record's removal come after the index and the files,
/// and a failure there leaves a
///  merge a command must not read past.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const partlyInstalled = (error: RecoveryError): MergeRefusal => ({ error, began: true });
/// ```
pub fn partly_installed(error: RecoveryError) -> MergeRefusal {
    return MergeRefusal {
        error,
        began: true,
    };
}

/// What:
///  A refusal about a merge that wrote nothing,
/// so a command that meets it may go on when the merge was saved.
/// Why:
///  Every refusal before the index write leaves the checkout as it was,
/// and the flag says so once for all of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const untouched = (error: RecoveryError): MergeRefusal => ({ error, began: false });
/// ```
pub fn untouched(error: RecoveryError) -> MergeRefusal {
    return MergeRefusal {
        error,
        began: false,
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

/// One record recovery visited.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MergeRecoveryOutcome = { record: string; action: MergeRecoveryAction; targetRef?: string; newOid: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MergeRecoveryOutcome {
    /// The record's file.
    pub record: PathBuf,
    /// What recovery did.
    pub action: MergeRecoveryAction,
    /// The ref the record names,
    /// or nothing when the merge ran on a detached `HEAD`.
    pub target_ref: Option<String>,
    /// The merge commit the record names.
    pub new_oid: String,
    /// The side ref that keeps the merge commit,
    /// for a merge that was saved because the index was locked.
    pub side_ref: Option<String>,
}
