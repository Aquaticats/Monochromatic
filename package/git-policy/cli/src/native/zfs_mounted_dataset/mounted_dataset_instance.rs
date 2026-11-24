//! What:
//!  One row of this process's mount table that is a ZFS dataset's mount,
//!  and the reader that answers every such row.
//! Why:
//!  A dataset count is decided by rows,
//! not by names:
//!  two pools that hold a leaf name,
//!  one dataset mounted at two paths,
//!  and a bind mount of a dataset's own root are each several rows,
//!  and a row of a mount namespace this process is not in is no row at all
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 8,
//!  "Where the checkout and the datasets are found").
//!  The table is read through the repository's own parser of
//!  `/proc/self/mountinfo` rather than by a second reader of the same file,
//!  and a control hands in the text of a table it wrote,
//! so nothing on a person's
//!  machine is read by a control.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const rows = new LiveMountedDatasets().mountedDatasets(); // MountedDatasetInstance[]
//! ```

use crate::fs_mount_options::{parse_mount_table, read_mount_facts, MountFacts, MOUNTINFO_PATH};
use crate::plain_file::{read_plain_file, MOUNT_TABLE_LIMIT};
use crate::worktree_clone_mount::attach_the_clone::ZFS_FILE_SYSTEM_TYPE;
use std::path::{Path, PathBuf};

/// The root a mount of the dataset's own root shows,
/// which a bind mount of a subtree shows deeper.
pub const THE_DATASETS_OWN_ROOT: &str = "/";

/// What:
///  The mount one path is on,
///  as far as the pool a design name lives in needs it.
/// Why:
///  The probe dataset's pool is a component of the checkout dataset's name,
///  so a checkout the configuration names without naming its dataset leaves the
///  pool to the mount the path is on;
///  that mount is not always a ZFS dataset,
/// and saying which mount it is is what
///  lets a refusal name the filesystem it found.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MountHoldingAPath = { dataset?: string; filesystemType: string; mountPoint: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MountHoldingAPath {
    /// The dataset the mount is of,
    /// or nothing when the mount is no ZFS dataset.
    pub dataset: Option<String>,
    /// The mount's filesystem type.
    pub filesystem_type: String,
    /// Where the mount stands.
    pub mount_point: PathBuf,
}

/// What:
///  One ZFS dataset's mount:
///  the dataset the kernel names as the source,
///  the path the mount stands at,
///  and the root within the dataset it shows.
/// Why:
///  The three fields decide whether a row is an instance of the dataset a caller
///  looks for:
///  the source's last name component,
///  the mount point a person is told,
///  and the root that tells a bind mount of a subtree from a mount of the
///  dataset itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MountedDatasetInstance = { dataset: string; mountPoint: string; root: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MountedDatasetInstance {
    /// The dataset's name,
    /// which is the mount's source because the ZFS mount prints its dataset.
    pub dataset: String,
    /// Where the mount stands.
    pub mount_point: PathBuf,
    /// The root within the dataset the mount shows.
    pub root: PathBuf,
}

impl MountedDatasetInstance {
    /// Whether the mount shows the dataset's own root,
    /// which a bind mount of a subtree does not.
    pub fn shows_the_datasets_own_root(&self) -> bool {
        return self.root == Path::new(THE_DATASETS_OWN_ROOT);
    }

    /// The last `/`-separated component of the dataset's name,
    /// which is the leaf name the design fixes.
    pub fn last_name_component(&self) -> &str {
        return self.dataset.rsplit_once('/').map_or(self.dataset.as_str(), |(_parent, leaf)| return leaf);
    }

    /// The instance in words,
    /// naming the dataset and the path so a person can go and look.
    pub fn described(&self) -> String {
        return if self.shows_the_datasets_own_root() {
            format!("`{}` at {}", self.dataset, self.mount_point.display())
        } else {
            format!("`{}` at {} with root {}", self.dataset, self.mount_point.display(), self.root.display())
        };
    }
}

/// What:
///  Where the rows of the mount table come from.
/// Why:
///  The release path reads the kernel's table,
///  and a control hands in the table it wrote,
///  so no control reads this machine's mounts and the rule that reads rows is
///  the same for both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface MountedDatasets { mountedDatasets(): MountedDatasetInstance[] }
/// ```
pub trait MountedDatasets {
    /// Every ZFS dataset mount this process's table lists,
    /// in file order,
    /// which is mount order.
    ///
    /// # Errors
    /// The table could not be read.
    fn mounted_datasets(&self) -> Result<Vec<MountedDatasetInstance>, String>;

    /// The mount one path is on,
    /// found the way path resolution reaches it,
    /// or nothing when no mount reaches it.
    ///
    /// # Errors
    /// The table or the path could not be read.
    fn mount_holding(&self, path: &Path) -> Result<Option<MountHoldingAPath>, String>;
}

/// What:
///  The release reader:
///  this process's own mount table.
/// Why:
///  The handler and the probe both run in the host mount namespace,
///  so this process's table is the one the checkout and its slots live in,
///  and a mount made inside another mount namespace is not in it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const rows = new LiveMountedDatasets().mountedDatasets();
/// ```
#[derive(Clone, Copy, Debug)]
pub struct LiveMountedDatasets;

impl MountedDatasets for LiveMountedDatasets {
    fn mounted_datasets(&self) -> Result<Vec<MountedDatasetInstance>, String> {
        let text: Vec<u8> = read_plain_file(Path::new(MOUNTINFO_PATH), MOUNT_TABLE_LIMIT).map_err(|error| return format!("cannot read {MOUNTINFO_PATH}: {error}"))?;
        return Ok(dataset_mounts_of(text.as_slice()));
    }

    fn mount_holding(&self, path: &Path) -> Result<Option<MountHoldingAPath>, String> {
        let facts: Option<MountFacts> = read_mount_facts(path)?;
        return Ok(facts.map(|mount| return holding_of(&mount)));
    }
}

/// What:
///  The mount a path is on,
///  in the shape the design-name lookups read.
/// Why:
///  The table's reader answers a mount's whole row;
///  the lookups need only the dataset,
///  the type and the path,
///  and a mount whose type is not `zfs` names no dataset.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holding = { dataset: facts.filesystemType === 'zfs' ? facts.source : undefined, filesystemType: facts.filesystemType, mountPoint: facts.mountPoint };
/// ```
pub fn holding_of(facts: &MountFacts) -> MountHoldingAPath {
    let dataset: Option<String> = if facts.filesystem_type == ZFS_FILE_SYSTEM_TYPE { Some(facts.source.clone()) } else { None };
    return MountHoldingAPath { dataset, filesystem_type: facts.filesystem_type.clone(), mount_point: facts.mount_point.clone() };
}

/// What:
///  Every ZFS dataset mount in one mount table's text.
/// Why:
///  A row is a dataset mount when its filesystem type is exactly `zfs`,
///  and every other row is left out here so no caller decides that again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const rows = parseMountTable(text).filter((row) => row.filesystemType === 'zfs');
/// ```
pub fn dataset_mounts_of(text: &[u8]) -> Vec<MountedDatasetInstance> {
    return parse_mount_table(text)
        .into_iter()
        .filter(|facts| return facts.filesystem_type == ZFS_FILE_SYSTEM_TYPE)
        .map(|facts| {
            return MountedDatasetInstance { dataset: facts.source, mount_point: facts.mount_point, root: facts.root };
        })
        .collect();
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "mounted_dataset_instance_tests.rs"]
mod tests;
