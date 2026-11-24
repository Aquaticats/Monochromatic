//! What:
//!  The three checks a removal makes before its first delete:
//!  the live pool,
//!  clone and origin dataset GUIDs against the add's record,
//!  the clone's live origin and both `createtxg` values,
//!  the clone's whole inventory by the transaction group that separates its own history from the
//!  history it inherited,
//!  the origin-side comparison,
//!  and the look at the origin's deferred-destroy mark.
//! Why:
//!  Item 11 showed why the lineage check must precede any snapshot delete:
//!  an explicit delete erased the relocated older checkout snapshot before the kernel refused the
//!  origin,
//!  and promoting back did not restore it
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`, "Removal and recovery", step 1;
//!  `doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md`, item 11).
//!  Item 13 measured the separation this proof rests on:
//!  transferred snapshots keep the origin's GUID and transaction group,
//!  the clone's own snapshot is newer than the clone,
//!  and a bookmark carries its snapshot's tuple.
//!  The three checks are layered,
//!  and the last of them refuses any object the origin side recorded,
//!  so a snapshot list alone cannot pass a clone that inherited history.
//!  The early mark look is the deferred-destroy decision's:
//!  a marked origin is refused here with the worktree still mounted and nothing written.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const proven = proveTheLineage(reads, recorded);
//! ```

use crate::worktree_clone_binding_record::binding_inventory::{EntryKind, InventoryRecord};
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_removal_proof_and_plan::plan_record::PlanSnapshot;
use crate::worktree_clone_removal_proof_and_plan::removal_identities::RecordedBinding;
use crate::worktree_clone_removal_proof_and_plan::removal_inventory::{LiveInventory, LiveObject};
use crate::worktree_clone_removal_proof_and_plan::removal_reads::RemovalPoolReads;
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use crate::worktree_removal_deferred_origin_check::origin_mark::{OriginMark, mark_of_reply};
use crate::worktree_removal_deferred_origin_check::origin_verdict::{Verdict, judge_the_look};
use crate::worktree_removal_deferred_origin_check::removal_history::RemovalHistory;
use monochromatic_zfs_request_to_openzfs_kernel_device::DatasetIdentity;

/// What:
///  One object's live identity:
///  its GUID and the transaction group that created it.
/// Why:
///  A name alone proves nothing about which object the pool holds,
///  and every recorded identity a removal compares is these two values.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ObjectIdentity = { guid: DatasetGuid; createtxg: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ObjectIdentity {
    /// The object's GUID.
    pub guid: DatasetGuid,
    /// The transaction group that created it.
    pub createtxg: u64,
}

/// What:
///  The identities one proof read live and matched against the add's record.
/// Why:
///  The plan copies exactly these,
///  and a reader that had only the record could not tell a replacement from the clone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ProvenIdentities = { pool; clone; originDataset; originSnapshot };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenIdentities {
    /// The pool's GUID.
    pub pool_guid: DatasetGuid,
    /// The clone's identity.
    pub clone: ObjectIdentity,
    /// The origin dataset's identity.
    pub origin_dataset: ObjectIdentity,
    /// The origin snapshot's identity.
    pub origin_snapshot: ObjectIdentity,
}

/// What:
///  What the lineage proof leaves a removal holding:
///  the identities,
///  the whole live inventory,
///  and the snapshots the removal may record and submit.
/// Why:
///  The plan records the clone's own snapshots and no bookmark,
///  and the bookmark rule reads the same inventory,
///  so both come out of one proof of one moment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ProvenRemoval = { identities: ProvenIdentities; inventory: LiveInventory; ownSnapshots: PlanSnapshot[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenRemoval {
    /// The identities the proof matched.
    pub identities: ProvenIdentities,
    /// Every live snapshot and bookmark of the clone.
    pub inventory: LiveInventory,
    /// The clone's own snapshots,
    /// which are the plan's tuples.
    pub own_snapshots: Vec<PlanSnapshot>,
}

/// What:
///  Prove one removal's lineage against the add's record and the pool.
/// Why:
///  Every check reads the pool,
///  and the first one that cannot be proved refuses before anything is written or deleted.
///  The origin's deferred-destroy mark is read here rather than after the unmount,
///  because the decision places the look inside this proof.
///
/// # Throws
/// The check that refused,
/// the object it named,
/// and what the pool answered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const proven = proveTheLineage(reads, recorded);
/// ```
pub fn prove_the_lineage(reads: &dyn RemovalPoolReads, recorded: &RecordedBinding, origin_inventory: &InventoryRecord, history: &RemovalHistory) -> Result<ProvenRemoval, RemovalStop> {
    let pool: ObjectIdentity = identity_of(reads, recorded.pool.as_str())?;
    if pool.guid != recorded.pool_guid {
        return Err(RemovalStop::IdentityDiffers {
            object: recorded.pool.clone(),
            field: "poolGuid",
            recorded: recorded.pool_guid.file_name(),
            live: pool.guid.file_name(),
        });
    }
    let clone_identity: DatasetIdentity = reads.identity(recorded.clone_dataset.as_str()).map_err(|reason| {
        return RemovalStop::DoesNotAnswer { object: recorded.clone_dataset.clone(), reason };
    })?;
    let clone: ObjectIdentity = identity_from(&clone_identity, recorded.clone_dataset.as_str())?;
    if clone.guid != recorded.clone_guid {
        return Err(RemovalStop::IdentityDiffers {
            object: recorded.clone_dataset.clone(),
            field: "cloneGuid",
            recorded: recorded.clone_guid.file_name(),
            live: clone.guid.file_name(),
        });
    }
    if clone.createtxg != recorded.clone_createtxg {
        return Err(RemovalStop::IdentityDiffers {
            object: recorded.clone_dataset.clone(),
            field: "cloneCreatetxg",
            recorded: recorded.clone_createtxg.to_string(),
            live: clone.createtxg.to_string(),
        });
    }
    let live_origin: Option<String> = Some(clone_identity.stats.origin()).filter(|origin| return !origin.is_empty());
    if live_origin.as_deref() != Some(recorded.origin_snapshot.as_str()) {
        return Err(RemovalStop::OriginIsNotTheRecordedOne {
            clone: recorded.clone_dataset.clone(),
            recorded: recorded.origin_snapshot.clone(),
            live: live_origin,
        });
    }
    let origin_snapshot_identity: DatasetIdentity = reads.identity(recorded.origin_snapshot.as_str()).map_err(|reason| {
        return RemovalStop::DoesNotAnswer { object: recorded.origin_snapshot.clone(), reason };
    })?;
    let origin_snapshot: ObjectIdentity = identity_from(&origin_snapshot_identity, recorded.origin_snapshot.as_str())?;
    if origin_snapshot.guid != recorded.origin_snapshot_guid {
        return Err(RemovalStop::IdentityDiffers {
            object: recorded.origin_snapshot.clone(),
            field: "originSnapshotGuid",
            recorded: recorded.origin_snapshot_guid.file_name(),
            live: origin_snapshot.guid.file_name(),
        });
    }
    if origin_snapshot.createtxg != recorded.origin_snapshot_createtxg {
        return Err(RemovalStop::IdentityDiffers {
            object: recorded.origin_snapshot.clone(),
            field: "originSnapshotCreatetxg",
            recorded: recorded.origin_snapshot_createtxg.to_string(),
            live: origin_snapshot.createtxg.to_string(),
        });
    }
    let origin_dataset: ObjectIdentity = identity_of(reads, recorded.origin_dataset.as_str())?;
    if origin_dataset.guid != recorded.origin_dataset_guid {
        return Err(RemovalStop::IdentityDiffers {
            object: recorded.origin_dataset.clone(),
            field: "originDatasetGuid",
            recorded: recorded.origin_dataset_guid.file_name(),
            live: origin_dataset.guid.file_name(),
        });
    }
    let mark: OriginMark = mark_of_reply(&origin_snapshot_identity);
    if let Verdict::Refused(refusal) = judge_the_look(recorded.clone_dataset.as_str(), recorded.slot.as_str(), recorded.origin_snapshot.as_str(), &mark) {
        return Err(RemovalStop::OriginMarked { refusal: Box::new(refusal), history: history.clone() });
    }
    let inventory: LiveInventory = crate::worktree_clone_removal_proof_and_plan::removal_reads::whole_inventory(reads, recorded.clone_dataset.as_str(), clone.guid)
        .map_err(|reason| return RemovalStop::InventoryUnreadable { clone: recorded.clone_dataset.clone(), reason })?;
    prove_every_object_belongs(&inventory, recorded, origin_inventory)?;
    let own_snapshots: Vec<PlanSnapshot> = inventory.own_snapshots(recorded.clone_createtxg);
    return Ok(ProvenRemoval { identities: ProvenIdentities { pool_guid: pool.guid, clone, origin_dataset, origin_snapshot }, inventory, own_snapshots });
}

/// What:
///  Whether every live object is the clone's own:
///  each snapshot newer than the clone,
///  and no object carrying a GUID the origin side recorded.
/// Why:
///  A promoted clone's snapshot list holds the transferred origin names at or below the origin
///  snapshot's group,
///  and item 13 saw a bookmark travel with an unchanged tuple,
///  so the transaction group and the origin-side GUIDs are the two facts that tell inherited history
///  from a clone's own.
///
/// # Throws
/// The first object that is not the clone's own,
/// which is the one a person needs to see.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// proveEveryObjectBelongsToTheClone(inventory, recorded, originInventory);
/// ```
fn prove_every_object_belongs(inventory: &LiveInventory, recorded: &RecordedBinding, origin_inventory: &InventoryRecord) -> Result<(), RemovalStop> {
    for object in &inventory.objects {
        if object.kind == EntryKind::Snapshot && object.createtxg <= recorded.clone_createtxg {
            return Err(RemovalStop::TransferredHistory {
                clone: recorded.clone_dataset.clone(),
                snapshot: object.full_name(),
                createtxg: object.createtxg,
                clone_createtxg: recorded.clone_createtxg,
            });
        }
        if let Some(held) = origin_side_hit(object, origin_inventory) {
            return Err(RemovalStop::OriginSideHit { kind: object.kind, name: object.full_name(), guid: held.guid });
        }
    }
    return Ok(());
}

/// What:
///  The origin-side inventory's entry one live object's GUID matches,
///  whatever its kind.
/// Why:
///  Item 13 saw the bookmark move with an unchanged tuple,
///  so a comparison that read only snapshots would miss the very object it measured.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const hit = originInventory.entries.find(e => e.guid === object.guid);
/// ```
fn origin_side_hit(object: &LiveObject, origin_inventory: &InventoryRecord) -> Option<crate::worktree_clone_binding_record::binding_inventory::InventoryEntry> {
    for entry in &origin_inventory.entries {
        if entry.guid == object.guid {
            return Some(entry.clone());
        }
    }
    return None;
}

/// One identity read,
/// as the pair a removal compares.
fn identity_of(reads: &dyn RemovalPoolReads, name: &str) -> Result<ObjectIdentity, RemovalStop> {
    let identity: DatasetIdentity = reads.identity(name).map_err(|reason| return RemovalStop::DoesNotAnswer { object: String::from(name), reason })?;
    return identity_from(&identity, name);
}

/// One live reply as the pair a removal compares.
fn identity_from(identity: &DatasetIdentity, name: &str) -> Result<ObjectIdentity, RemovalStop> {
    let guid: u64 = identity.stats.dds_guid;
    if guid == 0 {
        return Err(RemovalStop::DoesNotAnswer { object: String::from(name), reason: String::from("the pool reported GUID 0, and no live object has that identity") });
    }
    let parsed: DatasetGuid = DatasetGuid::parse(guid.to_string().as_str()).map_err(|_| {
        return RemovalStop::DoesNotAnswer { object: String::from(name), reason: String::from("the pool's GUID is not canonical") };
    })?;
    return Ok(ObjectIdentity { guid: parsed, createtxg: identity.stats.dds_creation_txg });
}
