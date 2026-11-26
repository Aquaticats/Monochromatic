//! What:
//!  Where one clone stands,
//!  read from this process's mount table:
//!  at its slot,
//!  nowhere,
//!  elsewhere,
//!  or behind something else at the slot.
//! Why:
//!  The command writes no second request where the clone already stands at its slot with a
//!  `mount` request standing,
//!  and it reports that the removal was already cancelled
//!  (`doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`,
//!  its "The act" section).
//!  The judgement is the repository's own reader of `/proc/self/mountinfo` and its own comparison
//!  of a row with a slot,
//!  so a control hands in the text of a table it wrote and the live answer reads this machine's,
//!  and a table that could not be read is reported rather than read as any of the four states.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const where = slotMount.whereTheCloneStands(cloneDataset, slotPath);
//! ```

use crate::worktree_clone_slot_state::slot_state_mounts::{
    CloneMount,
    clone_mount_of,
    read_mount_rows,
    slot_mount_of,
};
use crate::zfs_mounted_dataset::mounted_dataset_instance::{
    LiveMountedDatasets,
    MountedDatasets,
};
use std::path::Path;

/// What:
///  The mount state one clone is in,
///  as the command reads it.
/// Why:
///  A control answers this seam with a table it wrote,
///  and the release answers it with this process's own table,
///  so no control on a person's machine reads that machine's mounts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface TheSlotMount { whereTheCloneStands(cloneDataset, slotPath): CloneMount }
/// ```
pub trait TheSlotMount {
    /// Where this clone stands,
    /// as one mount table proves it.
    fn where_the_clone_stands(
        &self,
        clone_dataset: &str,
        slot_path: &Path,
    ) -> CloneMount;
}

/// What:
///  The release answer:
///  this process's mount table.
/// Why:
///  The wrapper and the handler both act in the host mount namespace,
///  so this process's table is the one the slot's mount appears in,
///  and a mount made inside another namespace is not read as the clone's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new LiveTheSlotMount().whereTheCloneStands('pool/clones/one', slotPath);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct LiveTheSlotMount;

impl TheSlotMount for LiveTheSlotMount {
    fn where_the_clone_stands(
        &self,
        clone_dataset: &str,
        slot_path: &Path,
    ) -> CloneMount {
        return of_a_table(&LiveMountedDatasets, clone_dataset, slot_path);
    }
}

/// What:
///  The same judgement over a table a caller hands in.
/// Why:
///  A control's table and this machine's go through one comparison,
///  so a planted table's arm and the live arm cannot disagree about which state a row is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const where = ofATable(scripted, 'pool/clones/one', slotPath);
/// ```
pub fn of_a_table(
    mounts: &dyn MountedDatasets,
    clone_dataset: &str,
    slot_path: &Path,
) -> CloneMount {
    return clone_mount_of(
        &read_mount_rows(mounts),
        &slot_mount_of(mounts, slot_path),
        clone_dataset,
        slot_path,
    );
}

/// What:
///  Whether the state is the one the answer must have produced.
/// Why:
///  A `done` answer whose mount table read says the clone stands anywhere else is a state the
///  command reports and refuses rather than acting on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ok = countsAsAtItsSlot(where);
/// ```
pub fn counts_as_at_its_slot(where_the_clone_stands: &CloneMount) -> bool {
    return matches!(where_the_clone_stands, CloneMount::AtItsSlot);
}

/// One mount state in words,
/// which the report prints.
pub fn word(state: &CloneMount) -> String {
    return match state {
        CloneMount::AtItsSlot => String::from("mounted at its slot"),
        CloneMount::Nowhere => String::from("mounted nowhere"),
        CloneMount::Elsewhere {
            described,
        } => format!("mounted elsewhere ({described})"),
        CloneMount::SomethingElseAtTheSlot {
            described,
        } => format!("with `{described}` at its slot"),
        CloneMount::Unreadable {
            reason,
        } => format!("with a mount table that could not be read ({reason})"),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_slot_mount_tests.rs"]
mod tests;
