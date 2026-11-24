//! What:
//!  One inventory of a dataset's snapshots and bookmarks:
//!  the grammar the two inventory files share,
//!  their bytes,
//!  and the reading that refuses a truncated or misnamed file.
//! Why:
//!  The origin-side set fixes what history the clone started from,
//!  and the clone's initial set cannot be rebuilt once the clone holds a snapshot of its own,
//!  so each set is a separate write-once file with its own count
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
//!  "The two inventories").
//!  A GUID and a `createtxg` are canonical decimal,
//!  a name is the ZFS name verbatim,
//!  and `entries=` is the completeness count:
//!  a reader counts the lines and refuses a file whose count disagrees.
//!  No cap is shipped on the count,
//!  because no kernel fact bounds an origin's history (rule `LQM`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Inventory = { kind: 'origin' | 'clone'; poolGuid: number; dataset: string; ...; entries: Entry[] };
//! const bytes = [HEADER, `kind=${kind}`, ..., `entries=${entries.length}`, ...entries.map(e => `entry=${e.kind} ${e.guid} ${e.createtxg} ${e.name}`)].join('\n') + '\n';
//! ```

use super::binding_bound::{INVENTORY_ENTRY_LINE_MAXIMUM_BYTES, INVENTORY_FIXED_BYTES};
use super::binding_words::INVENTORY_HEADER;
use crate::worktree_clone_mount_protocol::DatasetGuid;

///The inventory grammar's keys, in the order it writes them.
pub const INVENTORY_KEYS_IN_ORDER: [&str; 8] = ["kind", "poolGuid", "dataset", "datasetGuid", "bounds", "boundsGuid", "boundsCreatetxg", "entries"];

/// What:
///  Which of the two inventories a file is.
/// Why:
///  The same grammar serves both,
///  and the file's name says which one it holds,
///  so a copied file refuses under the wrong name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type InventoryKind = 'origin' | 'clone';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InventoryKind {
    /// The origin dataset's snapshots through the origin snapshot's transaction group,
    /// and its bookmarks.
    Origin,
    /// The clone's own history immediately after it was made.
    Clone,
}

impl InventoryKind {
    /// The word the file carries.
    pub fn word(self) -> &'static str {
        return match self {
            InventoryKind::Origin => "origin",
            InventoryKind::Clone => "clone",
        };
    }

    /// The kind a stored word names.
    pub fn parse(word: &str) -> Option<InventoryKind> {
        return match word {
            "origin" => Some(InventoryKind::Origin),
            "clone" => Some(InventoryKind::Clone),
            _ => None,
        };
    }
}

/// What:
///  Whether one entry is a snapshot or a bookmark.
/// Why:
///  A bookmark carries the GUID and `createtxg` of the snapshot it came from,
///  and a reader compares the two as the removal's proof expects.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type EntryKind = 'snapshot' | 'bookmark';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntryKind {
    /// A snapshot of the inventoried dataset.
    Snapshot,
    /// A bookmark of it.
    Bookmark,
}

impl EntryKind {
    /// The word the entry line carries.
    pub fn word(self) -> &'static str {
        return match self {
            EntryKind::Snapshot => "snapshot",
            EntryKind::Bookmark => "bookmark",
        };
    }

    /// The kind a stored word names.
    pub fn parse(word: &str) -> Option<EntryKind> {
        return match word {
            "snapshot" => Some(EntryKind::Snapshot),
            "bookmark" => Some(EntryKind::Bookmark),
            _ => None,
        };
    }
}

/// What:
///  One snapshot or bookmark of a dataset,
///  by kind,
///  GUID,
///  `createtxg` and name.
/// Why:
///  The removal's lineage proof compares exactly these four,
///  and an inventory without one of them cannot name what the clone started from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Entry = { kind: EntryKind; guid: number; createtxg: number; name: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InventoryEntry {
    /// Whether the entry is a snapshot or a bookmark.
    pub kind: EntryKind,
    /// The GUID the kernel reports.
    pub guid: DatasetGuid,
    /// The transaction group the object was created in.
    pub createtxg: u64,
    /// The object's name,
    /// which a ZFS name holds verbatim.
    pub name: String,
}

/// What:
///  One dataset's inventory at one moment.
/// Why:
///  The origin's and the clone's sets have the same shape and different lifetimes,
///  so one type carries both and the file's name says which.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type InventoryRecord = { kind: InventoryKind; poolGuid: number; dataset: string; datasetGuid: number; bounds: string; boundsGuid: number; boundsCreatetxg: number; entries: Entry[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InventoryRecord {
    /// Which inventory this is.
    pub kind: InventoryKind,
    /// The pool the dataset lives in.
    pub pool_guid: DatasetGuid,
    /// The inventoried dataset.
    pub dataset: String,
    /// That dataset's GUID.
    pub dataset_guid: DatasetGuid,
    /// The name whose transaction group bounds the set:
    /// the origin snapshot,
    /// or the clone dataset itself.
    pub bounds: String,
    /// The bound's GUID.
    pub bounds_guid: DatasetGuid,
    /// The bound's `createtxg`,
    /// which every snapshot entry's is at or below.
    pub bounds_createtxg: u64,
    /// The set itself.
    pub entries: Vec<InventoryEntry>,
}

impl InventoryRecord {
    /// The inventory's bytes,
    /// its count and then one line per entry.
    pub fn encode(&self) -> Vec<u8> {
        let mut text: String = String::from(INVENTORY_HEADER);
        text.push('\n');
        text.push_str(format!("kind={}\n", self.kind.word()).as_str());
        text.push_str(format!("poolGuid={}\n", self.pool_guid.file_name()).as_str());
        text.push_str(format!("dataset={}\n", self.dataset).as_str());
        text.push_str(format!("datasetGuid={}\n", self.dataset_guid.file_name()).as_str());
        text.push_str(format!("bounds={}\n", self.bounds).as_str());
        text.push_str(format!("boundsGuid={}\n", self.bounds_guid.file_name()).as_str());
        text.push_str(format!("boundsCreatetxg={}\n", self.bounds_createtxg).as_str());
        text.push_str(format!("entries={}\n", self.entries.len()).as_str());
        for entry in &self.entries {
            text.push_str(format!("entry={} {} {} {}\n", entry.kind.word(), entry.guid.file_name(), entry.createtxg, entry.name).as_str());
        }
        return text.into_bytes();
    }

    /// The widest bytes a file of this set may hold,
    /// which is its fixed lines and every entry line at its widest:
    /// no total cap is shipped,
    /// so this only describes the file the caller holds.
    pub fn widest_bytes_for_its_count(&self) -> u64 {
        return INVENTORY_FIXED_BYTES + self.entries.len() as u64 * INVENTORY_ENTRY_LINE_MAXIMUM_BYTES;
    }
}
