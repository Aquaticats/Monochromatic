//! What:
//!  The emptying of the trash a snapshot's leave-out made:
//! what a trash is,
//! how the commit service's handler deletes it,
//! and how every later command
//!  names a trash nothing will empty.
//! The deletion takes only the directory the leave-out recorded and only the
//!  entries it recorded,
//! found by walking down from the worktree's handle without following a link,
//! and at the lowest I/O
//!  priority Linux offers.
//! Why:
//!  A name of forty hexadecimal digits proves nothing about who made a directory,
//! and a path string is resolved again by every
//!  call that takes it.
//! The manifest says which objects the leave-out made and moved;
//! the deletion opens each directory on the
//!  way by one plain name below the handle of the one above,
//! compares identities,
//! and leaves anything else alone with a
//!  reason.
//! A record stays until the trash is gone or the service has taken it,
//! so a failure is never a silent success.
//! The add never waits for any of this (`worktree_trash_hand_over.rs` hands the trash over):
//! the
//!  service empties one worktree's trashes one at a time and different worktrees' side by side,
//! at the idle I/O class,
//! so concurrent adds do not delete at
//!  once.
//! The parts of it are the files beside this one:
//!  the reason a trash stays (`worktree_trash_dispose_the_reason_a_trash_stays.rs`),
//!  the checks that prove the worktree is the recorded one
//! (`worktree_trash_dispose_the_worktree_is_the_recorded_one.rs`),
//!  the directories held open down to the trash (`worktree_trash_dispose_the_trash_held_open.rs`),
//!  and what the emptying does once the entries are gone
//! (`worktree_trash_dispose_after_the_entries.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! emptyTrash(item); // the service's handler, and the reason a trash stays
//! ```

use crate::blocking_jobs_within_the_budget::run_a_slow_job_or_in_turn;
use crate::parallel_budget::HostResources;
use crate::service_work::Verdict;
pub use crate::the_names_and_the_manifest_of_the_trash::TRASH_REFERENCE;
use crate::work_item::WorkItem;
use crate::worktree_trash_entries::{
    EntryRemoval,
    remove_entries,
    remove_entries_awaiting,
};
use crate::worktree_trash_manifest::TrashManifest;
use crate::worktree_trash_remove::remove_tree_beneath;
use std::fs::File;
use std::path::{
    Path,
    PathBuf,
};
use std::sync::Arc;

/// What the emptying does once the entries are gone,
/// re-exported by this file so its two faces name it here.
pub(crate) use crate::worktree_trash_dispose_after_the_entries::settle_trash_removal;
/// The reason a trash the emptying leaves gives for every step it would not take,
/// re-exported by this file so every caller names it here.
pub use crate::worktree_trash_dispose_the_reason_a_trash_stays::TrashLeft;
/// The directories one emptying holds open,
/// and the proof that the trash is the one the snapshot made,
/// re-exported by this file so its two faces name them here.
pub(crate) use crate::worktree_trash_dispose_the_trash_held_open::{
    ATrashToOpen,
    HeldTrash,
    prepare_trash_removal,
};

/// The most bytes of a work item record that is read:
/// the manifest of one trash,
/// which is short;
/// the scan that names what nothing will empty reads them with this bound.
pub const RECORD_LIMIT: u64 = 64 * 1024;

/// What:
///  The removal of one trash's entries as the state of one job:
/// what was opened,
/// and what the leave-out recorded.
/// Why:
///   The entries are removed by the one worker pool of the process,
/// and each entry is removed only when it is the object that was moved there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheRemovalOfTrashEntries = { inside: string; trash: Arc<File>; manifest: TrashManifest };
/// ```
struct TheRemovalOfTrashEntries {
    /// The trash's path,
    /// which every reason names.
    inside: PathBuf,
    /// The trash directory,
    /// held open.
    trash: Arc<File>,
    /// What the leave-out moved there.
    manifest: TrashManifest,
}

/// What:
///  Ask for one entry removal request of the trash's entries.
/// Why:
///   The synchronous and the awaiting removal take the same request,
/// so the two faces differ in the wait only.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = { inside, trash, manifest, host: hostResources(), remove: removeTreeBeneath };
/// ```
fn entry_removal_of(
    TheRemovalOfTrashEntries {
        inside,
        trash,
        manifest,
    }: TheRemovalOfTrashEntries
) -> EntryRemoval {
    return EntryRemoval {
        inside,
        trash,
        manifest: Arc::new(manifest),
        host: HostResources::read(),
        remove: Arc::new(remove_tree_beneath),
    };
}

/// What:
///  Empty the trash directory the manifest describes,
/// in the worktree `repository` whose administrative directory is
///  `git_dir`;
/// a trash that is already gone is done.
/// Why:
///   Every directory from the worktree down is opened by one plain name below the handle of the one above,
/// so a link at any
///  level is an error and never followed.
/// The trash is the directory with the recorded identity,
/// each entry in it is removed
///  only when it is the object that was moved there,
/// and anything else in the trash stays with a reason.
/// The callers of this face run on a thread with no runtime of its own,
/// and the synchronous handler face is one of
///  them;
/// it does the lookups here and the entries through the one removal face that enters the runtime.
/// A caller that is
///  itself a task of the runtime awaits `delete_trash_awaiting` instead:
/// the drain of a round asks
///  `empty_trash_awaiting` (`worktree_trash_dispose_awaiting.rs`),
/// because the synchronous face is refused from inside the
///  runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await deleteTrash({ repository, gitDir, manifest }); // rejects with the reason something stays
/// ```
pub fn delete_trash(
    repository: &Path,
    git_dir: &Path,
    manifest: &TrashManifest,
) -> Result<(), TrashLeft> {
    let opened: ATrashToOpen = ATrashToOpen {
        repository: repository.to_path_buf(),
        git_dir: git_dir.to_path_buf(),
        manifest: manifest.clone(),
    };
    let Some(held) = prepare_trash_removal(opened).map_err(TrashLeft::kept)? else {
        return Ok(());
    };
    let left: Vec<String> = remove_entries(&entry_removal_of(TheRemovalOfTrashEntries {
        inside: held.inside.clone(),
        trash: Arc::clone(&held.trash),
        manifest: manifest.clone(),
    }));
    if !left.is_empty() {
        // An entry that stayed is either not the recorded object or one whose removal failed, and the line cannot tell
        //  them apart, so what the trash holds is not shown to be the snapshot's own.
        return Err(TrashLeft::kept(left.join("; ")));
    }
    return settle_trash_removal(held);
}

/// What:
///  `delete_trash` on the process's runtime:
/// the lookups and the last removal run as jobs of the blocking pool,
///  and the entries are removed by the awaited ordered runner's own jobs.
/// Why:
///   A caller that is itself a task of the runtime cannot enter it,
/// and a synchronous lookup on a task that other work shares would hold that work;
/// so every step of this removal is a job of the pool and the task only waits.
/// The steps are the synchronous face's:
/// the same `prepare_trash_removal`,
/// the same request and the same `settle_trash_removal`,
///  so the two faces cannot remove different things,
/// and both answer with the same `TrashLeft`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function deleteTrashAwaiting({ repository, gitDir, manifest }): Promise<void>; // rejects with the reason
/// ```
pub async fn delete_trash_awaiting(
    repository: &Path,
    git_dir: &Path,
    manifest: &TrashManifest,
) -> Result<(), TrashLeft> {
    let opened: ATrashToOpen = ATrashToOpen {
        repository: repository.to_path_buf(),
        git_dir: git_dir.to_path_buf(),
        manifest: manifest.clone(),
    };
    let opened_trash: Option<HeldTrash> =
        match run_a_slow_job_or_in_turn(opened, prepare_trash_removal).await {
            Ok(Ok(held)) => held,
            // A directory that is not the recorded one is left alone with its reason, and nothing was removed.
            Ok(Err(reason)) => return Err(TrashLeft::kept(reason)),
            // Nothing was opened and nothing was removed, and the reason the system gave is why.
            Err(refused) => {
                return Err(TrashLeft::kept(format!(
                    "the directories of the trash {} could not be examined ({refused})",
                    manifest.name
                )));
            }
        };
    let Some(held) = opened_trash else {
        return Ok(());
    };
    let left: Vec<String> = remove_entries_awaiting(&entry_removal_of(TheRemovalOfTrashEntries {
        inside: held.inside.clone(),
        trash: Arc::clone(&held.trash),
        manifest: manifest.clone(),
    }))
    .await;
    if !left.is_empty() {
        // An entry that stayed is either not the recorded object or one whose removal failed, and the line cannot tell
        //  them apart, so what the trash holds is not shown to be the snapshot's own.
        return Err(TrashLeft::kept(left.join("; ")));
    }
    return match run_a_slow_job_or_in_turn(held, settle_trash_removal).await {
        Ok(outcome) => outcome,
        // The entries are gone and the directory above them stays, which the words say.
        Err(refused) => Err(TrashLeft::kept(format!(
            "the trash directory {} stays ({refused})",
            manifest.name
        ))),
    };
}

/// What:
///  The service's handler:
/// empty the trash directory a work item names,
/// after checking that it is the one the snapshot made.
/// Why:
///   The record is a file in the Git directory;
/// the handler trusts nothing in it beyond what it can verify:
/// the worktree's
///  `.git` file points into the record's Git directory,
/// every directory down to the trash is a real one,
/// the trash has the
///  recorded identity,
/// and each entry removed is the recorded object.
///
/// A caller that is itself a task of the runtime,
/// which the drain of a round is,
/// asks `empty_trash_awaiting`
/// (`worktree_trash_dispose_awaiting.rs`) instead:
/// this face enters the runtime for the entry removals and is refused from
/// inside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// emptyTrash(item) // 'done' | { failed: reason }
/// ```
pub fn empty_trash(item: &WorkItem) -> Verdict {
    let manifest: TrashManifest =
        match TrashManifest::decode(item.oid.as_str(), item.parent.as_str()) {
            Ok(found) => found,
            Err(reason) => return Verdict::Failed(reason),
        };
    return match delete_trash(item.repository.as_path(), item.git_dir.as_path(), &manifest) {
        Ok(()) => Verdict::Done,
        Err(left) => Verdict::Failed(left.reason),
    };
}

/// The emptying's own controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_trash_dispose_tests.rs"]
mod tests;
