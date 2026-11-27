//! What:
//!  Which `git worktree add` the ZFS add route serves,
//! and the facts the route
//!  then uses:
//! the checkout's dataset and pool,
//! the clone container,
//! the slot
//!  and its name.
//! Why:
//!  The route is wired into the wrapper's one add path,
//! and it must serve
//!  nothing else:
//! an add on another filesystem,
//! or in a checkout that is not
//!  the dataset the design pins,
//! takes the route it took before,
//! exactly as
//!  today (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Dataset and path layout").
//!  The checkout's dataset is the gate the plan names without a word of its
//!  own:
//! the route needs `monochromatic-git-checkout` mounted at the source,
//!  which only plan step 19's one-time migration makes,
//! so no person meets the
//!  route before that step
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//! step 19).
//!  A checkout on a ZFS mount of another leaf name,
//! which is this machine
//!  before the migration,
//! keeps Git's own checkout.
//!  The decision takes the two facts a host cannot fake (the filesystem kind
//!  and the dataset lookup) as a value,
//! so every branch is a control on any
//!  machine.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const facts = theRoutesAdd(identity, destination, mounts); // null: the ordinary add
//! ```

use crate::fs_kind::{
    FilesystemKind,
    filesystem_kind,
};
use crate::worktree_clone_binding_record::slot_of;
use crate::worktree_identity::WorktreeIdentity;
use crate::zfs_mounted_dataset::mounted_dataset_instance::MountedDatasets;
use crate::zfs_mounted_dataset::the_designs_datasets_in_the_mount_table::{
    the_checkout_dataset_at,
    the_designs_container_beside,
};
use std::path::{
    Path,
    PathBuf,
};

/// The folder,
/// below the checkout,
/// that holds every worktree the route makes.
pub const WORKTREES_FOLDER: &str = "worktrees";

/// The wrapper's own folder below the checkout,
/// which holds the worktrees folder.
pub const WRAPPER_FOLDER_BELOW_THE_CHECKOUT: &str = ".claude";

/// What:
///  What a checkout's mount says about the design's dataset.
/// Why:
///  The decision below is a function of this answer,
/// so the mount read and the
///  rule stay apart and every rule's branch is a host control.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CheckoutMount = { kind: 'not-zfs' } | { kind: 'other-dataset' } | { kind: 'the-designs'; dataset: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckoutMount {
    /// The source is not on ZFS at all.
    NotZfs,
    /// The source is on a ZFS dataset of another leaf name,
    /// or on no dataset.
    OtherDataset,
    /// The source is the design's checkout dataset.
    TheDesigns {
        /// The dataset's name.
        dataset: String,
    },
}

/// What:
///  Everything the route reads once and passes to every step after it.
/// Why:
///  A step that recomputed one of these could disagree with the step before
///  it,
/// and the record names all of them,
/// so one value carries the facts the
///  gate proved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RouteFacts = { sourceRoot; commonDir; gitDir; destination; slot; checkoutDataset; containerDataset; pool };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouteFacts {
    /// The main checkout's root,
    /// where the dataset is mounted.
    pub source_root: PathBuf,
    /// The repository's common Git directory.
    pub common_dir: PathBuf,
    /// The main checkout's own Git directory,
    /// which is the common one.
    pub git_dir: PathBuf,
    /// The destination the person's command names.
    pub destination: PathBuf,
    /// The destination's last name,
    /// which keys every record.
    pub slot: String,
    /// The dataset mounted at the source root.
    pub checkout_dataset: String,
    /// The sibling container the clones are made in.
    pub container_dataset: String,
    /// The pool that holds the checkout,
    /// which a snapshot set names.
    pub pool: String,
}

impl RouteFacts {
    /// The path of the `.claude/worktrees` folder the destination lies in.
    pub fn worktrees_parent(&self) -> PathBuf {
        return self
            .source_root
            .join(WRAPPER_FOLDER_BELOW_THE_CHECKOUT)
            .join(WORKTREES_FOLDER);
    }

    /// The dataset name of the clone this add will make for this slot.
    pub fn clone_dataset(&self) -> String {
        return format!("{}/{slot}", self.container_dataset, slot = self.slot);
    }

    /// Whether the destination rule holds for the design's route.
    ///
    /// # Errors
    /// The destination is not a direct child of the worktrees folder.
    pub fn proves_it_can_serve(&self) -> Result<(), String> {
        if self.destination.parent() != Some(self.worktrees_parent().as_path()) {
            return Err(format!(
                "the destination {} is not a direct child of {}, so the ZFS clone route cannot key its record by the slot",
                self.destination.display(),
                self.worktrees_parent().display()
            ));
        }
        return Ok(());
    }
}

/// What:
///  Whether the destination is the shape the route serves:
/// a direct child of
///  `<source>/.claude/worktrees`.
/// Why:
///  The design puts every managed clone at that path,
/// and a record is keyed by
///  the slot's last name,
/// so an add anywhere else is not this route's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mine = destination.parent === join(sourceRoot, '.claude', 'worktrees');
/// ```
pub fn destination_is_the_routes_slot(
    source_root: &Path,
    destination: &Path,
) -> bool {
    let parent: PathBuf = source_root
        .join(WRAPPER_FOLDER_BELOW_THE_CHECKOUT)
        .join(WORKTREES_FOLDER);
    return destination.parent() == Some(parent.as_path()) && destination.file_name().is_some();
}

/// What:
///  The route's facts from the facts the caller read,
/// or nothing when the add
///  is not the route's,
/// or the refusal that stops the add.
/// Why:
///  The caller makes the two host reads (the filesystem kind and the mount
///  lookup) and this rule decides,
/// so a control supplies both answers and no
///  control needs a pool.
///
/// # Errors
/// The checkout is the design's dataset but the destination breaks the route's
/// shape,
/// or its name is no slot.
pub fn decide_the_routes_add(
    identity: &WorktreeIdentity,
    destination: &Path,
    checkout: &CheckoutMount,
) -> Result<Option<RouteFacts>, String> {
    let WorktreeIdentity::MainWorktree {
        common_dir,
        git_dir,
        worktree_root,
    } = identity
    else {
        return Ok(None);
    };
    let CheckoutMount::TheDesigns {
        dataset,
    } = checkout
    else {
        return Ok(None);
    };
    if !destination_is_the_routes_slot(worktree_root.as_path(), destination) {
        return Ok(None);
    }
    let container_dataset: String =
        the_designs_container_beside(dataset.as_str()).map_err(|refusal| return format!("the checkout dataset `{dataset}` names no sibling clone container ({refusal:?}), so the ZFS clone route cannot name where the clone goes"))?;
    let pool: String = dataset.split_once('/').map_or_else(
        || return dataset.clone(),
        |(pool, _rest)| return String::from(pool),
    );
    let slot: String = match slot_of(destination) {
        Ok(found) => found,
        Err(reason) => {
            return Err(format!(
                "the ZFS clone route cannot key a record for this add: {reason}. Ways forward: name the worktree with a plain folder name below {}, or remove the record that blocks the name.",
                worktree_root
                    .join(WRAPPER_FOLDER_BELOW_THE_CHECKOUT)
                    .join(WORKTREES_FOLDER)
                    .display()
            ));
        }
    };
    let facts: RouteFacts = RouteFacts {
        source_root: worktree_root.clone(),
        common_dir: common_dir.clone(),
        git_dir: git_dir.clone(),
        destination: destination.to_path_buf(),
        slot,
        checkout_dataset: dataset.clone(),
        container_dataset,
        pool,
    };
    facts.proves_it_can_serve()?;
    return Ok(Some(facts));
}

/// What:
///  The route's facts for one add,
/// reading the two host facts first.
/// Why:
///  The wrapper's add path asks this once.
/// `Ok(None)` keeps the ladder and
///  the copy exactly as they were.
///
/// # Errors
/// The rule refused the add.
pub fn the_routes_add(
    identity: &WorktreeIdentity,
    destination: &Path,
    mounts: &dyn MountedDatasets,
) -> Result<Option<RouteFacts>, String> {
    let WorktreeIdentity::MainWorktree {
        worktree_root,
        ..
    } = identity
    else {
        return Ok(None);
    };
    // The destination's shape decides first, so an add anywhere else pays neither the `statfs` below nor the mount-table read.
    if !destination_is_the_routes_slot(worktree_root.as_path(), destination) {
        return Ok(None);
    }
    // One `statfs` keeps every add on another filesystem off the mount-table read.
    let checkout: CheckoutMount = match filesystem_kind(worktree_root.as_path()) {
        Ok(FilesystemKind::Zfs) => match the_checkout_dataset_at(worktree_root.as_path(), mounts) {
            Ok(dataset) => CheckoutMount::TheDesigns {
                dataset,
            },
            // A ZFS mount of another leaf name is this machine before the
            // migration, and it keeps Git's own checkout.
            Err(_another_dataset) => CheckoutMount::OtherDataset,
        },
        _ => CheckoutMount::NotZfs,
    };
    return decide_the_routes_add(identity, destination, &checkout);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "add_route_gate_tests.rs"]
mod tests;
