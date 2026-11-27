//! What:
//!  The snapshots the wrapper makes ahead of an add,
//! and the add's claim of one instead of a snapshot of its own.
//! Why:
//!  One `BTRFS_IOC_SNAP_CREATE_V2` forces a btrfs transaction commit on the caller's thread,
//! and waits for the
//!  source's writeback first (`doc/planning/git-policy-cli-btrfs-transaction-waits.md`,
//! "A worktree add"):
//! on this volume it took 7,443.8 ms at the median,
//! 18,006.2 ms at the ninetieth percentile and 39,828.5 ms at its
//!  worst,
//! while an add that finds a snapshot ready only renames one directory there ("Step 2:
//! snapshots made
//!  ahead").
//! The commit service of the repository makes them on its own time,
//! from a work item the add writes
//! (`worktree_snapshot_ahead_job.rs`),
//! so it is the service's thread that waits for the commit and not the
//!  person's.
//! The owner's bound for an add is 60 s ("60s is fine for adding worktrees") and nothing may wait indefinitely,
//! so an add waited no longer than `SNAPSHOT_AHEAD_CEILING` for a copy.
//! The owner answered "Fresh snapshot every add" on 2026-11-11 when he was asked to keep that copy
//!  or to take one on every add,
//! so no add claims a copy from this pool and no add waits for one:
//! the modules stay for a record an earlier release queued and a removal clears a folder it filled.
//!
//! A snapshot made ahead is a copy of the source as it was when the service made it,
//! so an add may not trust it:
//! the
//!  files the new worktree receives must equal the source's index,
//! and an entry that fell behind it would give a
//!  worktree whose tracked files carry an older state beside freshly recorded stat data,
//! which Git would then report
//!  as clean.
//! Every candidate is therefore proven by the same walk the add makes over its own snapshot
//! (`worktree_snapshot_index_step.rs`,
//! `prove_the_copy`) before it is renamed into the stage name;
//! one that fails the
//!  proof is handed to the service for deletion and the next candidate is tried.
//! A source with uncommitted changes is not served this way at all:
//! its snapshot's own index is one instant with its
//!  files,
//! so that snapshot has to be taken on the add's own path.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const look = lookForASnapshotAhead({ source, plan, stage, environment, service });
//! // 'claimed' with the proof, 'none' (take one here), or 'timedOut' with the words for the person
//! ```
//!
//! This file is the module's index:
//!  the answer one add's look for a snapshot came to,
//!  and the hand-over of the copies it proved stale or keeps no longer.
//!  The names a copy carries and the pool it stands in are in `worktree_snapshot_ahead_names.rs`,
//!  the records that ask the service for one are in `worktree_snapshot_ahead_asks.rs`,
//!  and both were split out because this module was over the line limit,
//!  with every name they defined re-exported here so no caller changes.

use crate::diagnostic_log::debug;
use crate::fs_beneath::ObjectId;
use crate::hexadecimal_of_bytes::hexadecimal_of_bytes;
use crate::iso_time::now_iso;
use crate::path_bytes::path_bytes_of;
use crate::service_work::work_directory;
use crate::subvolume_sha::sha256;
use crate::transaction_git::GitContext;
use crate::work_item::{
    Enqueued,
    WorkItem,
    WorkKind,
    enqueue,
    requeue_after_giving_up,
};
// What:     `use crate::worktree_removal_leaves_a_subvolume_for_the_service::DELETE_REFERENCE;`
//           imports the reference a record of a subvolume delete carries.
// Why:      The record this file enqueues is written by the file that owns it, and
//           naming that file keeps the snapshots off the hand-over record.
//
// In TS you'd write (pseudocode):
// ```ts
// import { DELETE_REFERENCE } from './worktree_removal_leaves_a_subvolume_for_the_service';
// ```
use crate::worktree_removal_leaves_a_subvolume_for_the_service::DELETE_REFERENCE;
use crate::worktree_snapshot_ahead_asks::TAG;
use crate::worktree_snapshot_ahead_asks::wake_the_service;
use crate::worktree_snapshot_index_step::Proof;
use crate::worktree_snapshot_source::SnapshotSource;
use std::ffi::{
    OsStr,
    OsString,
};
use std::path::{
    Path,
    PathBuf,
};

/// The seams and the records that ask the commit service for a snapshot ahead.
pub use crate::worktree_snapshot_ahead_asks::{
    AHEAD_REFERENCE,
    AheadService,
    ask_with,
    request_item,
};
/// The names a snapshot ahead carries,
/// the pool it stands in,
/// and which of its entries a source keeps.
pub use crate::worktree_snapshot_ahead_names::{
    AHEAD_KEPT,
    AHEAD_OLDEST,
    AHEAD_PREFIX,
    ahead_name,
    kept_in,
};
/// The same names for the parts of this crate that read a pool's entries.
pub(crate) use crate::worktree_snapshot_ahead_names::{
    POLL,
    ahead_key,
    candidates,
    made_at,
    over_kept,
};

/// The name of the folder a pool of snapshots ahead stands in,
/// inside the data folder of a Git directory:
/// the one path a fill reads and a removal takes.
pub const POOL_FOLDER: &str = "ahead";

/// What:
///  The folder the snapshots the wrapper makes ahead of an add stand in:
/// `<git-dir>/monochromatic-git-policy-cli/ahead`,
/// beside the work
///  queue of the same repository.
/// Why:
///   Two things must hold of a snapshot that waits for an add,
/// and this folder is what makes both true:
/// the
///  destination's parent is inside the source worktree and may be anything Git does not ignore,
/// so a copy of the whole
///  source standing there would list as untracked until an add took it;
/// and a name inside the Git directory is never untracked,
///  in any worktree of the repository,
/// whatever the person's ignore rules say.
/// It is also one btrfs filesystem with the
///  source root,
/// because it is inside it,
/// so the ioctl that makes an entry there can share every data block with the source.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pool = (gitDir: string) => join(gitDir, 'monochromatic-git-policy-cli', 'ahead');
/// ```
pub fn pool_folder(git_dir: &Path) -> PathBuf {
    return git_dir
        .join("monochromatic-git-policy-cli")
        .join(POOL_FOLDER);
}

/// What one add's look for a snapshot made ahead came to.
/// Why:
///   The three answers decide three different things:
/// the proof travels on to the index step,
/// no answer means the
///  add takes its own snapshot,
/// and a ceiling ends the snapshot rung and lets the rung after it make the worktree.
#[derive(Debug)]
pub enum AheadLook {
    /// A snapshot was claimed,
    /// with the proof that its files equal the source's index.
    Claimed {
        /// The identity of the object now at the stage name.
        made: ObjectId,
        /// The walk of the claimed snapshot against the source's index.
        proof: Proof,
        /// The millisecond its name says the copy was made in,
        /// which is the age of the ignored state the new worktree receives.
        copied_at_millis: u128,
    },
    /// There was nothing to claim and nothing was waited for:
    /// the add takes its own snapshot.
    None,
    /// Nothing came within the ceiling;
    /// the words say why,
    /// and this add takes no snapshot of its own.
    TimedOut(String),
}

/// What:
///  The item that has the service delete one snapshot ahead this add proved stale,
/// or has kept too long.
/// Why:
///   The destroy is a btrfs call that takes the filesystem's
/// `subvol_sem` write lock against every snapshot in flight,
/// so it is the service's work and never the person's,
/// as a removed worktree's subvolume already is
/// (`worktree_remove_hand_over.rs`).
/// The name is the identity,
/// so two adds that proved one entry stale write one record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const deletion = (commonDir, pool, name) => ({ kind: 'subvolume-delete', repository: pool, gitDir: commonDir, ref: DELETE_REFERENCE, oid: sha256(name), parent: join(pool, name), created: now, tries: 0 });
/// ```
pub fn deletion_item(
    git_dir: &Path,
    pool: &Path,
    name: &OsStr,
) -> WorkItem {
    let entry: PathBuf = pool.join(name);
    return WorkItem {
        kind: WorkKind::SubvolumeDelete,
        repository: pool.to_path_buf(),
        git_dir: git_dir.to_path_buf(),
        branch_ref: String::from(DELETE_REFERENCE),
        oid: hexadecimal_of_bytes(&sha256(path_bytes_of(entry.as_path()))),
        parent: entry.display().to_string(),
        created: now_iso(),
        tries: 0,
        transport: false,
    };
}

/// What:
///  Hand this add's stale names,
/// and the names it keeps no longer,
/// to the commit service of the repository,
/// and wake it.
/// Why:
///   The destroy each name needs is a btrfs call that takes the filesystem's write lock against every
///  snapshot in flight,
/// so it is the service's work and never the person's,
/// as a removed worktree's subvolume already is.
/// A
///  reason here is never the add's failure:
/// the names stay in the folder and a later add hands them over again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// handOverToTheService({ context, source, stale }) // enqueue(deletion) for each; wake(commonDir)
/// ```
pub fn hand_over_to_the_service(
    context: &GitContext,
    source: &SnapshotSource,
    stale: &[OsString],
) -> Result<(), String> {
    let pool: PathBuf = pool_folder(source.git_dir.as_path());
    hand_over_with(source.git_dir.as_path(), pool.as_path(), stale)?;
    return wake_the_service(context, source.git_dir.as_path());
}

/// What:
///  One record per stale name,
/// and nothing else.
/// Why:
///   The records are written before the service is told,
/// so a control asserts them without a service,
/// and one
///  record that cannot be written is said and does not stop the others.
/// A record the service already gave up on is queued again rather than left in `failed`:
/// the id of the
///  delete of one path is that path,
/// so every later add names the same record,
/// and a queue that only answers
///  "already there" would never delete that copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// handOverWith(gitDir, pool, stale) // the records only; the caller wakes the service
/// ```
pub fn hand_over_with(
    git_dir: &Path,
    pool: &Path,
    stale: &[OsString],
) -> Result<(), String> {
    let work: PathBuf = work_directory(git_dir);
    for name in stale {
        let record: WorkItem = deletion_item(git_dir, pool, name.as_os_str());
        match enqueue(work.as_path(), &record) {
            Ok(Enqueued::Added) => {}
            Ok(Enqueued::Known) => {
                if requeue_after_giving_up(work.as_path(), record.id().as_str()) {
                    debug(
                        TAG,
                        format!(
                            "the delete of {} was given up on and is queued again",
                            pool.join(name).display()
                        )
                        .as_str(),
                    );
                }
            }
            Err(reason) => debug(
                TAG,
                format!(
                    "the delete of {} could not be recorded: {reason}",
                    pool.join(name).display()
                )
                .as_str(),
            ),
        }
    }
    debug(
        TAG,
        format!(
            "{} name(s) of snapshots ahead are handed to the service in {}",
            stale.len(),
            pool.display()
        )
        .as_str(),
    );
    return Ok(());
}

/// Controls of the snapshots made ahead stay out of the release executable.
#[cfg(test)]
#[path = "worktree_snapshot_ahead_tests.rs"]
mod tests;
