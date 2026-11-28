//! What:
//!  The mounts a shutdown pass may look at:
//!  every standing mount whose source is an immediate child of this boot's
//!  clone container and which stands at a slot path under the checkout this
//!  boot verified.
//! Why:
//!  The pass unmounts "the mounts this boot verified"
//!  (`crate::worktree_clone_mount_handler::shutdown_unmount_mode`),
//!  and a mount it may look at is one the route could have made this boot:
//!  the boot unit's own `ExecStart` reconciles the immediate children of the
//!  configured container at the slots of the configured checkout
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "ZED reconciliation and mount safety";
//!  `doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 19).
//!  Everything else the mount table holds is left alone and is not even
//!  looked at:
//!  the checkout's own mount,
//!  a snapshot the kernel mounted below `.zfs`,
//!  a clone mounted somewhere nobody asked for,
//!  and a child of the container mounted outside the checkout.
//!  The rows are values,
//!  so this decision is a value test and the live table is read once by the
//!  pass
//!  (`crate::worktree_clone_slot_state::slot_state_mounts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const candidates = mountsAtSlots(rows, 'zpcachyos/nosnap/git-worktree-clones', '/home/user/Mono');
//! candidates; // [{ clone: '…/clones/one', slot: 'one', slotPath: '/home/user/Mono/.claude/worktrees/one' }]
//! ```

use crate::worktree_clone_mount::slot_name_refusal;
use crate::zfs_mounted_dataset::mounted_dataset_instance::MountedDatasetInstance;
use std::path::{
    Path,
    PathBuf,
};

/// Where the slots hang under a checkout.
const SLOT_FOLDER_NAMES: [&str; 2] = [".claude", "worktrees"];

/// What:
///  One mount the pass may look at:
///  the clone its source names,
///  the slot's own name and the whole path the mount stands at.
/// Why:
///   The slot's name is the key the route's own records and claims are filed
///  under,
/// and the path is what the row,
///  the walk and the descriptor are compared on,
///  so both travel and neither is derived twice
///  (`crate::worktree_clone_mount_handler::per_slot_root_claim`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CandidateMount = { clone: string; slot: string; slotPath: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CandidateMount {
    /// The clone dataset the mount's source named.
    pub(crate) clone: String,
    /// The slot's own name,
    /// which is one folder name under `.claude/worktrees`.
    pub(crate) slot: String,
    /// The whole path the mount stands at.
    pub(crate) slot_path: PathBuf,
}

/// What:
///  The standing mounts of the container's clones that stand at a slot of the
///  checkout,
///  in mount order.
/// Why:
///   The order is the kernel's,
///  so two passes over one machine read the same mounts in the same order and
///  a person comparing two journals reads them position by position
///  (`crate::zfs_mounted_dataset::mounted_dataset_instance`:
///  the rows are in mount order).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mountsAtSlots(rows, container, checkout).map((row) => row.slot);
/// ```
pub(crate) fn mounts_at_slots(
    rows: &[MountedDatasetInstance],
    container: &str,
    checkout: &Path,
) -> Vec<CandidateMount> {
    let mut found: Vec<CandidateMount> = Vec::new();
    for row in rows {
        // A mount of the container itself, of another container's child, or of
        // the checkout is not a managed clone, whatever its path.
        let Some(leaf) = clone_leaf(row.dataset.as_str(), container) else {
            continue;
        };
        if slot_name_refusal(leaf).is_some() {
            continue;
        }
        // A bind of a folder inside a clone shows a deeper root, and unmounting
        // the row the pass read would take down a different mount.
        if !row.shows_the_datasets_own_root() {
            continue;
        }
        let slot_path: PathBuf = slot_path_of(checkout, leaf);
        if row.mount_point != slot_path {
            continue;
        }
        found.push(CandidateMount {
            clone: row.dataset.clone(),
            slot: String::from(leaf),
            slot_path,
        });
    }
    return found;
}

/// What:
///  The one child name of the container a dataset is,
///  or nothing when it is the container itself,
///  another container's child or another pool's dataset.
/// Why:
///  The sweep enumerates exactly these children and no others,
///  and a dataset one level deeper is a snapshot's clone,
///  never a slot the route made
///  (`doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
///  "The datasets the sweep acts on").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// cloneLeaf('tank/clones/one', 'tank/clones'); // 'one'
/// cloneLeaf('tank/clones/one/inner', 'tank/clones'); // null
/// ```
fn clone_leaf<'a>(
    dataset: &'a str,
    container: &str,
) -> Option<&'a str> {
    let rest: &str = dataset.strip_prefix(container)?.strip_prefix('/')?;
    if rest.is_empty() || rest.contains('/') {
        return None;
    }
    return Some(rest);
}

/// What:
///  The path a slot's mount stands at under one checkout.
/// Why:
///  The route fixes the folder,
///  so the pass derives the path from the checkout and the slot rather than
///  believing the row about where the mount is
///  (`crate::worktree_clone_mount::clone_slot_walk`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// slotPathOf('/home/user/Mono', 'one'); // '/home/user/Mono/.claude/worktrees/one'
/// ```
pub(crate) fn slot_path_of(
    checkout: &Path,
    slot: &str,
) -> PathBuf {
    let mut path: PathBuf = checkout.to_path_buf();
    for name in SLOT_FOLDER_NAMES {
        path.push(name);
    }
    path.push(slot);
    return path;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "unmountable_mounts_tests.rs"]
mod tests;
