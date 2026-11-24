//! What:
//!  A mount table whose text a control writes,
//!  read by the release parser and the release walk.
//! Why:
//!  The rule that finds the checkout counts rows of a mount table,
//!  and a control has to prove that rule on a table it chose:
//!  two pools that hold one leaf name,
//!  one dataset mounted twice and a bind mount of a subtree are shapes no
//!  machine can be asked for
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 8,
//!  "Where the checkout and the datasets are found").
//!  Both answers come from the same text through the release parser and the
//!  release walk,
//! so a control proves the release rule rather than a second
//!  reading of it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const table = ScriptedMountTable.holding([row(40, 1, '/', '/home/user/Monochromatic', 'zfs', 'pool/home/monochromatic-git-checkout')]);
//! ```

use crate::fs_mount_options::{mount_holding, parse_mount_table};
use crate::zfs_mounted_dataset::mounted_dataset_instance::{MountHoldingAPath, MountedDatasetInstance, MountedDatasets, dataset_mounts_of, holding_of};
use std::path::Path;

/// One row of a mount table's text,
/// in the grammar `proc_pid_mountinfo(5)` fixes.
pub(crate) fn row(id: u32, parent: u32, root: &str, mount_point: &str, filesystem: &str, source: &str) -> String {
    return format!("{id} {parent} 0:42 {root} {mount_point} rw,relatime shared:1 - {filesystem} {source} rw,xattr,posixacl");
}

/// The table a machine with the design's checkout mounted holds:
/// the root filesystem on a dataset,
/// the checkout dataset at the checkout path,
/// and a tmpfs elsewhere.
pub(crate) fn the_designs_table(checkout: &str, checkout_dataset: &str) -> String {
    return [
        row(1, 0, "/", "/", "zfs", "zpcachyos/ROOT/cos/root"),
        row(40, 1, "/", checkout, "zfs", checkout_dataset),
        row(41, 1, "/", "/run", "tmpfs", "tmpfs"),
    ]
    .join("\n");
}

/// One scripted mount table.
pub(crate) struct ScriptedMountTable {
    /// The table's text,
    /// or the refusal a control hands in instead of one.
    text: Result<String, String>,
}

impl ScriptedMountTable {
    /// A table holding `rows`,
    /// in mount order.
    pub(crate) fn holding(rows: &[String]) -> Self {
        return ScriptedMountTable { text: Ok(rows.join("\n")) };
    }

    /// A table holding `text`,
    /// as a control writes it.
    pub(crate) fn text(text: &str) -> Self {
        return ScriptedMountTable { text: Ok(String::from(text)) };
    }

    /// A table no process can read,
    /// which a control hands in to prove the refusal that names it.
    pub(crate) fn unreadable(said: &str) -> Self {
        return ScriptedMountTable { text: Err(String::from(said)) };
    }
}

impl MountedDatasets for ScriptedMountTable {
    fn mounted_datasets(&self) -> Result<Vec<MountedDatasetInstance>, String> {
        return self.text.as_ref().map(|text| return dataset_mounts_of(text.as_bytes())).map_err(|said| return said.clone());
    }

    fn mount_holding(&self, path: &Path) -> Result<Option<MountHoldingAPath>, String> {
        let text: &String = self.text.as_ref().map_err(|said| return said.clone())?;
        let facts = parse_mount_table(text.as_bytes());
        return Ok(mount_holding(facts.as_slice(), path).map(holding_of));
    }
}
