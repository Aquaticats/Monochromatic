//! What:
//!  The clone's live inventory:
//!  every snapshot and bookmark of the clone's dataset,
//!  by kind,
//!  name,
//!  GUID,
//!  owning dataset and `createtxg`,
//!  and the snapshots the removal records as its own.
//! Why:
//!  The proof's independent check reads all of them,
//!  because a snapshot-only inventory misses the transferred bookmark
//!  `[VM item 13]`
//!  measured,
//!  and a bookmark carries the GUID and birth transaction group of the snapshot it was made from
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Removal and recovery",
//! step 1).
//!  The per-name record is one line per attempt with no bookmark entry,
//!  so the own set is every live snapshot the proof accepted and never a bookmark.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type LiveObject = { kind: 'snapshot' | 'bookmark'; name; guid; createtxg; owningDataset; owningDatasetGuid };
//! const own = inventory.snapshots().filter(s => s.createtxg > cloneCreatetxg);
//! ```

use crate::worktree_clone_binding_record::binding_inventory::EntryKind;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_removal_proof_and_plan::plan_record::PlanSnapshot;

/// What:
///  One live object of a dataset,
///  by kind,
///  name,
///  GUID,
///  owning dataset and creation transaction group.
/// Why:
///  A bookmark lives under the dataset it belongs to and carries another snapshot's tuple,
///  so the owning dataset has to be part of what the proof reads:
///  without it two clones' bookmarks with one tuple would be one fact.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LiveObject = { kind: EntryKind; name: string; guid: DatasetGuid; createtxg: number; owningDataset: string; owningDatasetGuid: DatasetGuid };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveObject {
    /// Whether this object is a snapshot or a bookmark.
    pub kind: EntryKind,
    /// The object's own name,
    /// without the dataset prefix.
    pub name: String,
    /// The GUID the kernel reports,
    /// which a bookmark takes from the snapshot it was made from.
    pub guid: DatasetGuid,
    /// The transaction group the object's snapshot was created in,
    /// which a bookmark also takes from its snapshot.
    pub createtxg: u64,
    /// The dataset the object is listed under.
    pub owning_dataset: String,
    /// That dataset's GUID.
    pub owning_dataset_guid: DatasetGuid,
}

impl LiveObject {
    /// The object's full name,
    /// as a destroy request names it.
    pub fn full_name(&self) -> String {
        let separator: char = match self.kind {
            EntryKind::Snapshot => '@',
            EntryKind::Bookmark => '#',
        };
        return format!("{}{separator}{}", self.owning_dataset, self.name);
    }
}

/// What:
///  One dataset's live snapshot and bookmark inventory.
/// Why:
///  The proof reads one list and judges it whole,
///  so a list that could not be read completely is one refusal and never a short history.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LiveInventory = { dataset: string; datasetGuid: DatasetGuid; objects: LiveObject[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveInventory {
    /// The dataset every object is listed under.
    pub dataset: String,
    /// That dataset's GUID.
    pub dataset_guid: DatasetGuid,
    /// Every object the pool listed.
    pub objects: Vec<LiveObject>,
}

impl LiveInventory {
    /// Every snapshot of the list,
    /// in the order the pool listed them.
    pub fn snapshots(&self) -> Vec<LiveObject> {
        return self.objects.iter().filter(|object| return object.kind == EntryKind::Snapshot).cloned().collect();
    }

    /// Every bookmark of the list,
    /// in the order the pool listed them.
    pub fn bookmarks(&self) -> Vec<LiveObject> {
        return self.objects.iter().filter(|object| return object.kind == EntryKind::Bookmark).cloned().collect();
    }

    /// What:
    ///  The snapshots this clone made itself:
    ///  those whose creation transaction group is strictly above the clone's own.
    /// Why:
    ///  A transferred snapshot keeps the origin's transaction group,
    ///  which is at or below the origin snapshot's and therefore below the clone's,
    ///  so the comparison is what separates the clone's own history from the history it inherits.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const own = inventory.snapshots().filter(s => s.createtxg > cloneCreatetxg).map(toPlanSnapshot);
    /// ```
    pub fn own_snapshots(&self, clone_createtxg: u64) -> Vec<PlanSnapshot> {
        let mut own: Vec<PlanSnapshot> = self
            .snapshots()
            .into_iter()
            .filter(|snapshot| return snapshot.createtxg > clone_createtxg)
            .map(|snapshot| {
                return PlanSnapshot { name: snapshot.full_name(), guid: snapshot.guid, createtxg: snapshot.createtxg };
            })
            .collect();
        own.sort_by(|left, right| return left.name.cmp(&right.name));
        return own;
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "removal_inventory_tests.rs"]
mod tests;
