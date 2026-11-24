//! What:
//!  The checkout's own mount,
//! as the handler reads it from the kernel's table:
//!  the dataset the checkout is on,
//!  its filesystem type,
//!  its mount point,
//!  and whether it is a shared mount.
//! Why:
//!  Two of the sweep's decisions rest on the checkout's mount rather than on a
//!  written name:
//!  the origin a clone may name is a snapshot of the checkout's own dataset
//!  (`doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
//!  "The datasets the sweep acts on"),
//!  and the design requires a verified,
//! not-shared immediate parent before the
//!  route acts on a clone
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Mounts held by other namespaces",
//!  which measured the move refusing `EINVAL` under a shared checkout and
//!  passing once the checkout was private).
//!  A mount whose `shared:` tag is present delivers mounts made on it to another
//!  namespace,
//! so the checkout is not ready there;
//!  other non-shared propagation was not measured and is left as it is.
//!  The table is read through the repository's own mountinfo parser rather than
//!  by a second reader of the same file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const mount = liveCheckoutMounts.mountHolding('/home/user/Monochromatic'); // { source, filesystemType, shared }
//! ```

use crate::fs_mount_options::{read_mount_facts, MountFacts};
use std::path::{Path, PathBuf};

/// The leaf name the design gives the checkout dataset.
pub const DESIGN_CHECKOUT_DATASET_NAME: &str = crate::zfs_mounted_dataset::the_designs_datasets_in_the_mount_table::DESIGN_CHECKOUT_DATASET_NAME;

/// The leaf name the design gives the container of managed clones.
pub const DESIGN_CONTAINER_DATASET_NAME: &str = crate::zfs_mounted_dataset::the_designs_datasets_in_the_mount_table::DESIGN_CONTAINER_DATASET_NAME;

/// The prefix the kernel prints on a mount that delivers its mounts elsewhere.
pub const SHARED_FIELD_PREFIX: &str = "shared:";

/// The filesystem type a dataset is attached as,
/// which is the type the fallback reads a dataset name from.
pub const ZFS_FILE_SYSTEM_TYPE: &str = crate::worktree_clone_mount::attach_the_clone::ZFS_FILE_SYSTEM_TYPE;

/// What:
///  The mount the checkout is on.
/// Why:
///  The dataset,
/// the type and the propagation travel together,
///  so a caller asking about the origin reads the same mount the propagation
///  check judged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CheckoutMount = { source: string; filesystemType: string; mountPoint: string; shared?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckoutMount {
    /// The mount's source,
    /// which for a dataset mount is the dataset's name.
    pub source: String,
    /// The mount's filesystem type.
    pub filesystem_type: String,
    /// The mount point the kernel lists.
    pub mount_point: PathBuf,
    /// The `shared:` field,
    /// where the mount delivers its mounts to another namespace.
    pub shared: Option<String>,
}

impl CheckoutMount {
    /// Whether the mount is shared,
    /// which makes the checkout an unverified immediate parent.
    pub fn is_shared(&self) -> bool {
        return self.shared.is_some();
    }

    /// The dataset the checkout is on,
    /// where it is a dataset mount rather than a folder on another filesystem.
    pub fn dataset(&self) -> Option<&str> {
        return if self.filesystem_type == ZFS_FILE_SYSTEM_TYPE { Some(self.source.as_str()) } else { None };
    }
}

/// What:
///  Why the checkout's mount could not be read.
/// Why:
///  Neither decision may be taken on a mount nobody read:
///  an unread mount is not a private parent and not a dataset name,
///  and each refusal names what a person checks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CheckoutMountRefusal = { kind: 'unreadable' | 'not-mounted'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckoutMountRefusal {
    /// The mount table or the checkout's own path could not be read.
    Unreadable {
        /// The checkout whose mount was asked for.
        checkout: PathBuf,
        /// What the read answered.
        reason: String,
    },
    /// The table lists no mount holding the checkout.
    NotMounted {
        /// The checkout whose mount was asked for.
        checkout: PathBuf,
    },
}

impl CheckoutMountRefusal {
    /// The refusal in words,
    /// naming the table and the way forward.
    pub fn message(&self) -> String {
        return match self {
            CheckoutMountRefusal::Unreadable { checkout, reason } => format!(
                "the mount {} is on could not be read from `{}` ({reason}), so nothing here says which dataset the checkout is on or whether it is shared. Nothing was enumerated, attached or recorded. Ways forward: check that this process can read the kernel's mount table, then run the sweep again.",
                checkout.display(),
                crate::fs_mount_options::MOUNTINFO_PATH
            ),
            CheckoutMountRefusal::NotMounted { checkout } => format!(
                "`{}` lists no mount holding {}, so the checkout is not the mount the route expects. Nothing was enumerated, attached or recorded. Ways forward: check that the checkout dataset is mounted at that path, then run the sweep again.",
                crate::fs_mount_options::MOUNTINFO_PATH,
                checkout.display()
            ),
        };
    }
}

/// What:
///  Where the checkout's mount is read from.
/// Why:
///  The release path reads the kernel's table,
///  and a control hands in the mount it wants judged,
///  so no control reads this machine's mounts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface CheckoutMounts { mountHolding(checkout: string): CheckoutMount }
/// ```
pub trait CheckoutMounts {
    /// The mount holding the given checkout.
    fn mount_holding(&self, checkout: &Path) -> Result<CheckoutMount, CheckoutMountRefusal>;
}

/// What:
///  The release reader:
///  the kernel's own mount table.
/// Why:
///  The sweep runs in the host mount namespace,
///  so this process's table is the one the checkout and its slots live in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mount = new LiveCheckoutMounts().mountHolding(checkout);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct LiveCheckoutMounts;

impl CheckoutMounts for LiveCheckoutMounts {
    fn mount_holding(&self, checkout: &Path) -> Result<CheckoutMount, CheckoutMountRefusal> {
        let read: Option<MountFacts> =
            read_mount_facts(checkout).map_err(|reason| return CheckoutMountRefusal::Unreadable { checkout: checkout.to_path_buf(), reason })?;
        let Some(facts) = read else {
            return Err(CheckoutMountRefusal::NotMounted { checkout: checkout.to_path_buf() });
        };
        let shared: Option<String> = facts.optional_field(SHARED_FIELD_PREFIX).map(String::from);
        return Ok(CheckoutMount { source: facts.source, filesystem_type: facts.filesystem_type, mount_point: facts.mount_point, shared });
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "sweep_checkout_mount_tests.rs"]
mod tests;
