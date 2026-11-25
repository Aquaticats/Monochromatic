//! What:
//!  The healthy origin cleanup record and the helpers every control of this folder starts from.
//! Why:
//!  The record's bytes,
//! its identity and its own name are needed by the record's controls,
//!  the parse's,
//!  the attempt's and the look's,
//!  and one value means no control reads a record another file spelled differently.
//!  Never part of the release executable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const record = anOwedRecord();
//! ```

use crate::worktree_clone_binding_record::binding_inventory::EntryKind;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_origin_cleanup::origin_cleanup_record::OriginCleanupRecord;
use crate::worktree_clone_removal_proof_and_plan::removal_inventory::LiveObject;
use crate::worktree_clone_removal_proof_and_plan::removal_reads::RemovalPoolReads;
use crate::worktree_removal_deferred_origin_check::fixtures::origin_properties;
use crate::worktree_removal_deferred_origin_check::origin_mark::{MARK_OFF, MARK_ON};
use monochromatic_zfs_request_to_openzfs_kernel_device::{DatasetIdentity, NvList};
use std::cell::Cell;
use std::path::PathBuf;
use std::rc::Rc;

/// The origin snapshot's own name,
/// which is the record's file name.
pub const OWN: &str = "git-worktree-clone-origin-445566778899aabbccddeeff00112233";

/// The origin dataset's full name.
pub const ORIGIN_DATASET: &str = "tank/mono-checkout";

/// The origin dataset's GUID.
pub const ORIGIN_DATASET_GUID: u64 = 4_815_162_342_109_876_543;

/// The origin snapshot's GUID.
pub const ORIGIN_SNAPSHOT_GUID: u64 = 11_050_373_805_378_911_967;

/// The origin snapshot's creation transaction group.
pub const ORIGIN_TXG: u64 = 15;

/// The clone's full dataset name.
pub const CLONE: &str = "tank/wt/fix-flaky-test";

/// The clone's GUID.
pub const CLONE_GUID: u64 = 7_828_944_690_444_299_989;

/// The slot's path,
/// whose bytes the record carries as hexadecimal.
pub const SLOT_PATH: &str = "/home/user/Mono/.claude/worktrees/fix-flaky-test";

/// One GUID built from a decimal number.
pub fn guid(value: u64) -> DatasetGuid {
    return DatasetGuid::parse(value.to_string().as_str()).expect("a number is a canonical GUID");
}

/// The origin snapshot's full name.
pub fn the_full_name() -> String {
    return format!("{ORIGIN_DATASET}@{OWN}");
}

/// The healthy owed record every control starts from.
pub fn an_owed_record() -> OriginCleanupRecord {
    return OriginCleanupRecord::owed(
        ORIGIN_DATASET,
        guid(ORIGIN_DATASET_GUID),
        OWN,
        guid(ORIGIN_SNAPSHOT_GUID),
        ORIGIN_TXG,
        CLONE,
        guid(CLONE_GUID),
        PathBuf::from(SLOT_PATH).as_path(),
        "2026-11-25T09:12:44-05:00",
    );
}

/// The owed record's own bytes as text.
pub fn the_bytes() -> String {
    return String::from_utf8(an_owed_record().encode().expect("the record encodes")).expect("the record is text");
}

/// What:
///  The pool a look's controls answer:
///  the origin dataset and the recorded snapshot,
///  whose presence and clone count a control scripts.
/// Why:
///  A look's attempts destroy one snapshot and prove its absence,
///  so the pool must share its listed flag with the device that destroys it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pool = VisiblePool.free(); const device = OneDestroy.over(pool);
/// ```
pub struct VisiblePool {
    /// Whether the recorded snapshot is listed.
    pub present: Rc<Cell<bool>>,
    /// How many clones the snapshot reports.
    pub clones: u64,
    /// The mark the snapshot reports.
    pub mark: u64,
}

impl VisiblePool {
    /// The snapshot is there and free.
    pub fn free() -> VisiblePool {
        return VisiblePool { present: Rc::new(Cell::new(true)), clones: 0, mark: MARK_OFF };
    }

    /// The snapshot is pinned by one clone.
    pub fn pinned() -> VisiblePool {
        return VisiblePool { present: Rc::new(Cell::new(true)), clones: 1, mark: MARK_OFF };
    }

    /// The snapshot is pinned by two clones.
    pub fn pinned_by_two() -> VisiblePool {
        return VisiblePool { present: Rc::new(Cell::new(true)), clones: 2, mark: MARK_OFF };
    }

    /// The snapshot is there and held,
    /// which the counts also call a dependency.
    pub fn holding() -> VisiblePool {
        return VisiblePool { present: Rc::new(Cell::new(true)), clones: 0, mark: MARK_ON };
    }
}

impl RemovalPoolReads for VisiblePool {
    fn identity(&self, name: &str) -> Result<DatasetIdentity, String> {
        if name == ORIGIN_DATASET {
            return Ok(DatasetIdentity { stats: crate::worktree_clone_removal_proof_and_plan::removal_fixtures::the_stats(ORIGIN_DATASET_GUID, false, 2, 1, ""), properties: NvList::new() });
        }
        if name == the_full_name().as_str() {
            if !self.present.get() {
                return Err(String::from("the snapshot is gone"));
            }
            return Ok(DatasetIdentity {
                stats: crate::worktree_clone_removal_proof_and_plan::removal_fixtures::the_stats(ORIGIN_SNAPSHOT_GUID, true, ORIGIN_TXG, self.clones, ""),
                properties: origin_properties(Some(self.mark), Some(0)),
            });
        }
        return Err(format!("`{name}` does not answer this scripted pool"));
    }

    fn snapshots_of(&self, dataset: &str, _dataset_guid: DatasetGuid) -> Result<Vec<LiveObject>, String> {
        if dataset != ORIGIN_DATASET || !self.present.get() {
            return Ok(Vec::new());
        }
        return Ok(vec![LiveObject {
            kind: EntryKind::Snapshot,
            name: String::from(OWN),
            guid: guid(ORIGIN_SNAPSHOT_GUID),
            createtxg: ORIGIN_TXG,
            owning_dataset: String::from(ORIGIN_DATASET),
            owning_dataset_guid: guid(ORIGIN_DATASET_GUID),
        }]);
    }

    fn bookmarks_of(&self, _dataset: &str, _dataset_guid: DatasetGuid) -> Result<Vec<LiveObject>, String> {
        return Ok(Vec::new());
    }
}
