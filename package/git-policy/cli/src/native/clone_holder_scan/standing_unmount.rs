//! What:
//!  The one managed attach a holder scan sets aside:
//!  the clone's own mount at its slot while the clone carries a standing
//!  `unmount` intent.
//! Why:
//!  "Before reporting removal-ready unmount,
//!  exclude new managed attaches with the standing `unmount` intent"
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Mounts held by other namespaces").
//!  The exclusion is narrow on purpose,
//!  because the scan's other errors are the cheap kind:
//!  a managed attach reported as a holder costs a removal that refuses and a
//!  person reading the report,
//!  while a holder set aside lets a removal go on around it.
//!  So a mount is set aside only when all of these hold:
//!  the clone's durable intent is a local `unmount` request,
//!  the caller knew the slot the managed attach stands at,
//!  the mount shows the dataset's own root and not a folder inside it,
//!  and its mount point is exactly that slot.
//!  A bind of a folder inside the clone is never set aside,
//!  and neither is any mount at another path,
//!  which is what keeps "a bind of the clone elsewhere is a holder" true while
//!  a removal is in progress.
//!  What the exclusion cannot tell apart,
//!  and therefore accepts:
//!  a bind of the clone's own root at the slot's path,
//!  which only root can make and which this route never makes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const standing = StandingUnmount.of(intent, '/home/user/Mono/.claude/worktrees/one'); // undefined without an unmount intent
//! ```

use crate::clone_holder_scan::held_mounts::HeldMount;
use crate::worktree_clone_mount_protocol::clone_mount_intent::{MountIntent, MountOperation};
use std::path::{Path, PathBuf};

/// What:
///  The slot a standing `unmount` intent is taking down,
///  which the scan may set aside when it matches.
/// Why:
///   A value of this type exists only when the intent read as `unmount`,
///  so the scan cannot set anything aside from a `mount` request,
///  an inherited value or a malformed one:
///  each of those answers `None` here and every mount stays a holder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class StandingUnmount { private constructor(readonly slot: string) {} }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StandingUnmount {
    /// The path the managed attach stands at.
    slot: PathBuf,
}

impl StandingUnmount {
    /// What the clone's intent and the caller's slot make of one scan.
    ///
    /// The intent is the clone's durable one as its reader answered it:
    /// `None` is a clone that carries none,
    /// an inherited value or a malformed one,
    /// and each of those sets nothing aside.
    /// The slot is the path the managed attach of this clone stands at,
    /// or `None` when the caller does not know it.
    pub fn of(intent: Option<&MountIntent>, slot: Option<&Path>) -> Option<Self> {
        let request: &MountIntent = intent?;
        if request.operation != MountOperation::Unmount {
            return None;
        }
        let slot: &Path = slot?;
        if slot.as_os_str().is_empty() || !slot.is_absolute() {
            return None;
        }
        return Some(StandingUnmount { slot: slot.to_path_buf() });
    }

    /// The path the managed attach stands at.
    pub fn slot(&self) -> &Path {
        return self.slot.as_path();
    }

    /// Whether one holder's mount is the managed attach this intent is taking
    /// down.
    pub fn sets_aside(&self, mount: &HeldMount) -> bool {
        return mount.is_the_dataset_root() && mount.mount_point == self.slot;
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "standing_unmount_tests.rs"]
mod tests;
