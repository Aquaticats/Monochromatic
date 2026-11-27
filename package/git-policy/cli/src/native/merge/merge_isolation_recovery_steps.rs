//! What:
//!  The steps both the recovery of an interrupted merge and the settling of a merge whose branch moved take:
//!  the paths the merge changes,
//! whether the index already holds the merge,
//! and the removal of a record.
//!  What a recovery decides is `merge_isolation_recovery_record.rs`'s.
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
