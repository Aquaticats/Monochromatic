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
///  The point at which the trash `item` names begins,
///  for a control that holds it while it checks what the service did meanwhile
///  (`service_fault_hooks_test_support.rs`).
/// Why:
///   The point names the record whose trash is emptied,
///  so two controls of one process hold the beginning of their own trash and neither answers the other (issue 6543),
///  as `collection_point` names the directory it collects and `instance_read_point` the directory it reads.
///  It is reached on whichever task runs the trash,
///  and inert unless a control asks for it.
/// Gotcha:
///  The point names the record's object and not its repository:
/// a trash record's id is its kind and its object alone (`WorkItem::id`),
/// so the second trash of one repository reaches a point of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const insideASlowStepPoint = (item: WorkItem) => `inside-a-slow-step-of-${item.oid}`;
/// ```
#[cfg(test)]
pub(crate) fn inside_a_slow_step_point(item: &WorkItem) -> String {
    // What:     `format!` builds an owned `String`, a heap-allocated growable UTF-8 buffer whose sibling is the
    //           borrowed `&str`, from the template and the record's object.
    // Why:      the hold registry keeps the point after this call returns, so the text cannot be a slice of `item`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return `inside-a-slow-step-of-${item.oid}`;
    // ```
    return format!("inside-a-slow-step-of-{}", item.oid);
}

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
    crate::service_fault::hooks::await_on_any_thread(inside_a_slow_step_point(item).as_str()).await;
    let manifest: TrashManifest =
        match TrashManifest::decode(item.oid.as_str(), item.parent.as_str()) {
            Ok(found) => found,
            Err(reason) => return Verdict::Failed(reason),
        };
    return match delete_trash_awaiting(item.repository.as_path(), item.git_dir.as_path(), &manifest)
        .await
    {
        Ok(()) => Verdict::Done,
        Err(left) => Verdict::Failed(left.reason),
    };
}
