//! What:
//!  What one process's mount table says about a slot and about the clone
//!  dataset a binding record names:
//!  whether the clone's own root is mounted at its slot,
//!  somewhere else,
//!  or not at all,
//!  and whether something else stands at the slot instead.
//! Why:
//!  A known unmounted clone and an add that has not mounted yet look the same
//!  from the slot's directory alone,
//!  so the read that tells them apart is the mount table
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Unmounted,
//! pruned or moved worktrees").
//!  The rows come from the repository's own reader of
//!  `/proc/self/mountinfo`,
//!  and a control hands in the text of a table it wrote,
//!  so no control reads this machine's mounts and the rule that compares a row
//!  with a slot is the same code in both.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const mount = cloneMountOf(readMountRows(liveMounts), slotMountOf(liveMounts, slotPath), 'pool/clones/topic', slotPath);
//! ```

use crate::zfs_mounted_dataset::mounted_dataset_instance::{
    MountedDatasetInstance,
    MountedDatasets,
};
use std::path::Path;

/// What:
///  Every ZFS dataset mount one process's table listed,
///  or why the table could not be read.
/// Why:
///  A table that could not be read proves neither that the clone is mounted
///  nor that it is not,
///  and a reader that treated the refusal as absence would report an unmounted
///  clone over a mounted one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MountRows = { kind: 'rows'; rows: MountedDatasetInstance[] } | { kind: 'unreadable'; reason: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MountRows {
    /// The table's ZFS dataset mounts, in mount order.
    Rows(Vec<MountedDatasetInstance>),
    /// The table could not be read,
    /// and this is what the reader said.
    Unreadable {
        /// Why the table was not read.
        reason: String,
    },
}

/// Read one process's mount table through the caller's reader.
///
/// Why:
///  The live reader and a scripted control fill the same value,
///  so the comparison below has one input shape.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const rows = readMountRows(mounts);
/// ```
pub fn read_mount_rows(mounts: &dyn MountedDatasets) -> MountRows {
    return match mounts.mounted_datasets() {
        Ok(rows) => MountRows::Rows(rows),
        Err(reason) => MountRows::Unreadable {
            reason,
        },
    };
}

/// What:
///  The mount that holds one slot path,
///  which is what the slot's own directory shows.
/// Why:
///  A mount of another dataset,
///  a bind mount or a filesystem that is no ZFS dataset all stand at the slot
///  without being the clone,
///  and the slot's own row is what tells them from an empty underlay.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SlotMount = { kind: 'nothing' } | { kind: 'something'; described: string } | { kind: 'unreadable'; reason: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SlotMount {
    /// No mount stands at the slot's own path.
    Nothing,
    /// A mount stands at the slot's path,
    /// described with its filesystem and path.
    Something {
        /// The mount in words.
        described: String,
    },
    /// The slot's mount could not be read.
    Unreadable {
        /// Why the mount was not read.
        reason: String,
    },
}

/// What:
///  Where the binding record's clone dataset is mounted,
///  as far as one mount table proves it.
/// Why:
///  The five answers are the ones whose next safe action differs:
///  a clone at its slot is in use,
///  a clone nowhere is the known unmounted clone,
///  and every other shape is a state no reader may treat as either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CloneMount = { kind: 'at-its-slot' } | { kind: 'nowhere' } | { kind: 'elsewhere'; described: string } | { kind: 'something-else-at-the-slot'; described: string } | { kind: 'unreadable'; reason: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloneMount {
    /// The clone's own root is the mount standing at its slot.
    AtItsSlot,
    /// No row of the table is a mount of the clone dataset.
    Nowhere,
    /// The clone dataset is mounted, and not at its slot.
    Elsewhere {
        /// The clone's instance in words.
        described: String,
    },
    /// Something that is not the clone's own root stands at the slot.
    SomethingElseAtTheSlot {
        /// The mount in words.
        described: String,
    },
    /// The table or the slot's own mount could not be read.
    Unreadable {
        /// Why the mount was not read.
        reason: String,
    },
}

/// What:
///  Where the clone is mounted,
///  from the table's rows and the slot's own mount.
/// Why:
///  A row is an instance of the clone only when its source is the clone dataset
///  and its root is the dataset's own root,
///  which the proposal's mount rules already fix for every reader;
///  a bind mount of a subtree shows a deeper root and is something else at the
///  slot rather than the clone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mount = cloneMountOf(rows, slotMount, cloneDataset, slotPath);
/// ```
pub fn clone_mount_of(
    rows: &MountRows,
    slot_mount: &SlotMount,
    clone_dataset: &str,
    slot: &Path,
) -> CloneMount {
    let found: &[MountedDatasetInstance] = match rows {
        MountRows::Unreadable {
            reason,
        } => {
            return CloneMount::Unreadable {
                reason: reason.clone(),
            };
        }
        MountRows::Rows(found) => found.as_slice(),
    };
    let at_the_slot: Option<&MountedDatasetInstance> = found
        .iter()
        .find(|row| return row.dataset == clone_dataset && row.mount_point == slot);
    if let Some(row) = at_the_slot {
        if row.shows_the_datasets_own_root() {
            return CloneMount::AtItsSlot;
        }
        return CloneMount::SomethingElseAtTheSlot {
            described: row.described(),
        };
    }
    let elsewhere: Option<&MountedDatasetInstance> =
        found.iter().find(|row| return row.dataset == clone_dataset);
    if let Some(row) = elsewhere {
        return CloneMount::Elsewhere {
            described: row.described(),
        };
    }
    return match slot_mount {
        SlotMount::Something {
            described,
        } => CloneMount::SomethingElseAtTheSlot {
            described: described.clone(),
        },
        SlotMount::Unreadable {
            reason,
        } => CloneMount::Unreadable {
            reason: reason.clone(),
        },
        SlotMount::Nothing => CloneMount::Nowhere,
    };
}

/// What:
///  The slot's own mount,
///  read through the caller's table.
/// Why:
///  One call answers whether anything at all stands at the slot path,
///  including a mount whose filesystem is no ZFS dataset,
///  which the ZFS rows alone cannot show.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const slotMount = slotMountOf(mounts, slotPath);
/// ```
pub fn slot_mount_of(
    mounts: &dyn MountedDatasets,
    slot: &Path,
) -> SlotMount {
    let read = match mounts.mount_holding(slot) {
        Ok(read) => read,
        // A slot that does not exist holds no mount, so a failure to resolve it is the empty arm; any other
        // failure proves nothing about the slot and is reported as it stands.
        Err(reason) => {
            return match std::fs::symlink_metadata(slot) {
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => SlotMount::Nothing,
                _ => SlotMount::Unreadable {
                    reason,
                },
            };
        }
    };
    let Some(facts) = read else {
        return SlotMount::Nothing;
    };
    if facts.mount_point != slot {
        return SlotMount::Nothing;
    }
    let named: String = match facts.dataset {
        Some(dataset) => format!("`{dataset}`"),
        None => String::from("a mount that is no ZFS dataset"),
    };
    return SlotMount::Something {
        described: format!(
            "{named} at the slot, mounted as `{}`",
            facts.filesystem_type
        ),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "slot_state_mounts_tests.rs"]
mod tests;
