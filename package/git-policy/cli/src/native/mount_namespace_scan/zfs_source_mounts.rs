//! What:
//!  Which mounts of one table are mounts of one exact ZFS dataset:
//!  the filesystem type `zfs` and the source equal to the dataset's name,
//!  character for character.
//! Why:
//!  "Unmounted" means absent in every mount namespace holding the clone's ZFS
//!  source,
//!  and the check is the ZFS filesystem and the exact dataset source,
//!  not a slot's spelling and not a working directory
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Mounts held by other namespaces").
//!  Equal is the whole name:
//!  a dataset whose name begins with this one is another dataset,
//!  and a table that spelled it as a prefix would hide its own holder,
//!  so no prefix and no case folding is applied here.
//!  A bind of the clone carries the clone's own superblock,
//!  so it names the same filesystem type and source and is found too,
//!  whatever its root within the clone is.
//!  This is the source match the start probe's arm 7 and the root handler's
//!  holder scan both call,
//!  so the two cannot disagree about what "a mount of this dataset" means.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const held = mountsOfSource(table, 'zpcachyos/…/clones/one'); // MountFacts[]
//! ```

use crate::fs_mount_options::MountFacts;
use crate::worktree_clone_mount::attach_the_clone::ZFS_FILE_SYSTEM_TYPE;

/// What:
///  Every mount in one table that is a mount of the exact ZFS source.
/// Why:
///   One namespace can hold the clone more than once,
///  at its slot and bound somewhere else,
///  and every one of them is a holder,
///  so the whole list is returned rather than the first match.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = mountsOfSource(table, source);
/// ```
pub fn mounts_of_source<'a>(mounts: &'a [MountFacts], source: &str) -> Vec<&'a MountFacts> {
    return mounts.iter().filter(|facts| return is_of_source(facts, source)).collect();
}

/// Whether one mount is a mount of the exact ZFS source.
pub fn is_of_source(facts: &MountFacts, source: &str) -> bool {
    return facts.filesystem_type == ZFS_FILE_SYSTEM_TYPE && facts.source == source;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "zfs_source_mounts_tests.rs"]
mod tests;
