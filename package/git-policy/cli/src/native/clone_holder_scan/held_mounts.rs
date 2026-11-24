//! What:
//!  One mount of a clone,
//!  as a holder's report names it:
//!  the kernel's mount id,
//!  the root within the dataset the mount shows,
//!  and the mount point.
//! Why:
//!  A namespace that holds a clone can hold it more than once,
//!  at a slot and bound somewhere else,
//!  and the three fields are what tells those mounts apart:
//!  a bind of the clone's own root and one of a folder inside it name the same
//!  dataset and the same mount id family,
//!  and only the root and the mount point say which is which
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Mounts held by other namespaces").
//!  The source is not kept per mount:
//!  the scan matched every mount of a report against one source,
//!  and the report names that source once.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const held = { mountId: 41, root: '/', mountPoint: '/home/user/Monochromatic/.claude/worktrees/one' };
//! ```

use crate::fs_mount_options::MountFacts;
use std::path::{Path, PathBuf};

/// What:
///  One mount of the dataset a scan reports on.
/// Why:
///  The identity is the kernel's own mount number,
///  so a reader can confirm the mount it sees is the one reported,
///  and the two paths say what the namespace shows where.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HeldMount = { mountId: number; root: string; mountPoint: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HeldMount {
    /// The kernel's mount number.
    pub mount_id: u64,
    /// The root within the dataset this mount shows:
    /// `/` for the dataset's own mount and a path inside it for a bind.
    pub root: PathBuf,
    /// The path the namespace shows the mount at.
    pub mount_point: PathBuf,
}

impl HeldMount {
    /// One table row as the mount a report names.
    pub fn of(facts: &MountFacts) -> Self {
        return HeldMount { mount_id: facts.mount_id, root: facts.root.clone(), mount_point: facts.mount_point.clone() };
    }

    /// Whether this mount shows the dataset's own root.
    /// Why:
    ///  A managed attach mounts the dataset itself,
    ///  while a bind of a folder inside the clone shows that folder,
    ///  so the exclusion asks this beside the mount point.
    pub fn is_the_dataset_root(&self) -> bool {
        return self.root == Path::new("/");
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "held_mounts_tests.rs"]
mod tests;
