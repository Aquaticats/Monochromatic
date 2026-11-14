//! What:
//!  The service's handler for a trash work item,
//! awaited by a caller that is itself a task of the runtime:
//!  empty the one directory the record names,
//! after checking that it is the directory the snapshot made.
//! Why:
//!   The drain of the service's work is a task of the runtime
//! (`landing/service_work_drain_pending_claims.rs`),
//! and the synchronous disposal enters the runtime for its
//!  entry removals (`worktree_trash_entries.rs`,
//! `remove_entries`),
//! which is refused from inside it:
//! a trash item would be settled as failed and its
//!  directory left behind.
//! So the task asks this face,
//! which awaits the entries and the two lookups as jobs of the blocking
//!  pool,
//! and the synchronous face keeps its callers outside the runtime.
//! The record is decoded and every reason is worded by the synchronous face's
//! functions,
//! so the two cannot empty different things or report a different reason.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await emptyTrashAwaiting(item) // 'done' | { failed: reason }
//! ```

use crate::service_work::Verdict;
use crate::work_item::WorkItem;
use crate::worktree_trash_dispose::{EmptiedTrash, delete_trash_awaiting};
use crate::worktree_trash_manifest::TrashManifest;

/// A trash of more rounds than the tries a failure gets, driven one round at a time.
#[cfg(test)]
#[path = "worktree_trash_dispose_awaiting_tests.rs"]
mod tests;

/// The point a control holds to keep a trash running while it checks what the service did meanwhile;
/// it is reached on whichever thread runs the trash,
/// and inert unless a control asks for it.
#[cfg(test)]
pub(crate) const INSIDE_A_SLOW_STEP: &str = "inside-a-slow-step";

/// What:
///  Empty the trash directory the manifest of `item` describes,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   See the module description.
/// A record that cannot be read is refused before any path is built,
/// as the synchronous face refuses it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const manifest = TrashManifest.decode(item.oid, item.parent); await deleteTrashAwaiting({ repository: item.repository, gitDir: item.gitDir, manifest })
/// ```
pub async fn empty_trash_awaiting(item: &WorkItem) -> Verdict {
    // A control holds the service here, which is where this kind's slow step begins and before any path of the record is used,
    //  so a control can say what the round did while the trash ran. The release executable keeps nothing of it.
    #[cfg(test)]
    crate::service_fault::hooks::await_on_any_thread(INSIDE_A_SLOW_STEP).await;
    let manifest: TrashManifest = match TrashManifest::decode(item.oid.as_str(), item.parent.as_str()) {
        Ok(found) => found,
        Err(reason) => return Verdict::Failed(reason),
    };
    return match delete_trash_awaiting(item.repository.as_path(), item.git_dir.as_path(), &manifest, crate::wait_ceilings::trash_item_ceiling()).await {
        EmptiedTrash::Emptied => Verdict::Done,
        EmptiedTrash::Kept(left) => Verdict::Failed(left.reason),
        // What:
        //  A round that removed entries and ran out of its bound is queued again for a later look, and nothing of it failed;
        //  a round that could not remove any entry within its bound is a retry, which the claim's lease and `MAX_TRIES` bound.
        // Why:
        //  A trash of a removed worktree is emptied over many rounds by design (`TRASH_ITEM_CEILING`),
        //  and a retry spends one of the three tries the recovery gives a failure before the record is given up on in `failed`,
        //  where nothing reads it again: a trash of more than a few rounds was left on the disk that way.
        //  A round that removed nothing is a different thing, and three of those do end it.
        EmptiedTrash::LeftForALaterRound { left, removed: 0 } => Verdict::Retry(left.reason),
        EmptiedTrash::LeftForALaterRound { left, removed: _progress } => Verdict::Deferred(left.reason),
    };
}
