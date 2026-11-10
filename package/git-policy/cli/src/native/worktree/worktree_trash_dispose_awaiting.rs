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
use crate::worktree_trash_dispose::delete_trash_awaiting;
use crate::worktree_trash_manifest::TrashManifest;

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
    let manifest: TrashManifest = match TrashManifest::decode(item.oid.as_str(), item.parent.as_str()) {
        Ok(found) => found,
        Err(reason) => return Verdict::Failed(reason),
    };
    return match delete_trash_awaiting(item.repository.as_path(), item.git_dir.as_path(), &manifest).await {
        Ok(()) => Verdict::Done,
        Err(left) => Verdict::Failed(left.reason),
    };
}
