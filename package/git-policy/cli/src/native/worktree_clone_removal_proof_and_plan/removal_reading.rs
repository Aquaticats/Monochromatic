//! What:
//!  One removal's proof,
//!  inventory and bookmark rule,
//!  run read-only,
//!  and the lines a command prints for them before anything is written.
//! Why:
//!  The design's first write is the destruction plan,
//!  and it sits after the removal intent and after the unmount,
//!  because a plan written earlier would name an unmount that never happened
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  section 3's list and "The removal intent").
//!  This release has no snapshot batch,
//!  head destroy, origin cleanup or Git settle,
//!  so a command that filed the intent,
//!  unmounted the clone and wrote the plan would leave a person with an unmounted clone and a plan
//!  no command of this release can finish or cancel
//!  (plan steps 15 to 17;
//!  `doc/decision/cli-git-zfs-dirty-main-restore-fallback-proof-window-and-slot.md`'s merge bar,
//!  "a branch may make nothing `main` does well worse").
//!  So the removal's command path runs everything here that writes nothing:
//!  the add's record,
//!  the registration's identity,
//!  the origin-side inventory,
//!  the lineage proof,
//!  the bookmark rule,
//!  and the list of names a removal would lose,
//!  and it refuses a promoted,
//!  mismatched or unexplained clone with the proof's own sentence.
//!  The first write,
//!  the intent and the unmount are switched on by plan step 16's wiring,
//!  which replaces this call in the removal's command path with
//!  `removal_entry::enter_the_removal`;
//!  until then no command of this release reaches that entry.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const reading = readTheRemoval(entry, reads);
//! console.log(reading.lines().join('\n'));
//! ```

use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_removal_proof_and_plan::loss_names::NamesToBeLost;
use crate::worktree_clone_removal_proof_and_plan::plan_record::PlanSnapshot;
use crate::worktree_clone_removal_proof_and_plan::removal_entry::{RemovalEntry, prepare_the_removal};
use crate::worktree_clone_removal_proof_and_plan::removal_reads::RemovalPoolReads;
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;

/// What:
///  What one removal would prove and lose,
///  read from the add's records and the live pool,
///  with nothing written and nothing unmounted.
/// Why:
///  A command that cannot finish a removal still owes the person what the removal it refused would
///  have done,
///  and every one of these values is a fact the proof read rather than a promise.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Reading = { clone; cloneGuid; cloneCreatetxg; originSnapshot; originSnapshotGuid; originSnapshotCreatetxg; ownSnapshots; lost };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemovalReading {
    /// The clone's full dataset name.
    pub clone: String,
    /// The clone's GUID as the proof read it.
    pub clone_guid: DatasetGuid,
    /// The transaction group that created the clone.
    pub clone_createtxg: u64,
    /// The origin snapshot the add recorded and the proof matched.
    pub origin_snapshot: String,
    /// That snapshot's GUID as the proof read it.
    pub origin_snapshot_guid: DatasetGuid,
    /// The transaction group that created the origin snapshot.
    pub origin_snapshot_createtxg: u64,
    /// Every live snapshot of the clone's own history,
    /// which is what a plan would record.
    pub own_snapshots: Vec<PlanSnapshot>,
    /// Every name a removal would lose.
    pub lost: NamesToBeLost,
}

impl RemovalReading {
    /// What:
    ///  The lines a person reads:
    ///  what the proof matched,
    ///  what a removal would lose,
    ///  and what this release does not do yet.
    /// Why:
    ///  The middle lines are the loss report the design requires before a destroy,
    ///  and the last line is the reason the command stopped where it did,
    ///  so a person learns both from one refusal.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const lines = reading.lines();
    /// ```
    pub fn lines(&self) -> Vec<String> {
        let mut lines: Vec<String> = Vec::with_capacity(self.lost.snapshots.len() + self.lost.bookmarks.len() + 3);
        lines.push(format!(
            "the lineage proof a removal makes passed without writing anything: the clone `{}` (GUID `{}`, creation transaction group {}) is the one the add recorded and descends from the origin snapshot `{}` (GUID `{}`, creation transaction group {}).",
            self.clone,
            self.clone_guid.file_name(),
            self.clone_createtxg,
            self.origin_snapshot,
            self.origin_snapshot_guid.file_name(),
            self.origin_snapshot_createtxg
        ));
        lines.extend(self.lost.lines());
        lines.push(String::from(
            "this release does not remove a clone yet: nothing was unmounted, no removal intent was filed and no destruction plan was written, so every snapshot, bookmark, record and registration still stands. Plan step 16 builds the snapshot batch, the head destroy, the origin cleanup and the Git settle, and its wiring is what switches the intent, the unmount and the plan on.",
        ));
        return lines;
    }
}

/// What:
///  Read one removal:
///  the add's record,
///  the registration's identity,
///  the origin-side inventory,
///  the lineage proof,
///  the bookmark rule,
///  and the names a removal would lose.
/// Why:
///  This is every step of the removal's order that changes nothing,
///  in the order the entry itself takes them,
///  so the reading and the entry cannot disagree about which check refuses first.
///
/// # Throws
/// The check that refused,
/// with the object it named and the sentence a person reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reading = readTheRemoval(entry, reads);
/// ```
pub fn read_the_removal(entry: &RemovalEntry<'_>, reads: &dyn RemovalPoolReads) -> Result<RemovalReading, RemovalStop> {
    let prepared = prepare_the_removal(entry, reads)?;
    let identities = &prepared.proven.identities;
    return Ok(RemovalReading {
        clone: prepared.recorded.clone_dataset.clone(),
        clone_guid: identities.clone.guid,
        clone_createtxg: identities.clone.createtxg,
        origin_snapshot: prepared.recorded.origin_snapshot.clone(),
        origin_snapshot_guid: identities.origin_snapshot.guid,
        origin_snapshot_createtxg: identities.origin_snapshot.createtxg,
        own_snapshots: prepared.proven.own_snapshots.clone(),
        lost: prepared.lost,
    });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "removal_reading_tests.rs"]
mod tests;
