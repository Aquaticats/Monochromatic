//! What:
//!  One mount namespace that holds a clone:
//!  the kernel's own namespace identity,
//!  the task whose table was read,
//!  and every mount of the clone in it.
//! Why:
//!  A holder is reported by namespace rather than by task,
//!  because the mounts are the namespace's and the tasks that share it come
//!  and go
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Mounts held by other namespaces").
//!  The task is kept as evidence:
//!  it says a live task was there to read,
//!  and it is what a person inspects when they have to know which process kept
//!  the clone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const holder = { namespace: 'mnt:[4026532001]', task: '/proc/1234', mounts: [{ mountId: 41, root: '/', mountPoint: '/srv/holder' }] };
//! ```

use crate::clone_holder_scan::held_mounts::HeldMount;
use std::path::PathBuf;

/// What:
///  One namespace and the mounts of the dataset it holds.
/// Why:
///  The report's order is the scan's,
///  so two scans of one machine list the same namespaces in the same order and
///  a reader can compare them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NamespaceHolder = { namespace: string; task: string; mounts: HeldMount[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamespaceHolder {
    /// The namespace's own identity,
    /// such as `mnt:[4026532001]`.
    pub namespace: String,
    /// The live task whose mount table of that namespace was read.
    pub task: PathBuf,
    /// Every mount of the reported dataset in that namespace.
    pub mounts: Vec<HeldMount>,
}

impl NamespaceHolder {
    /// The namespace's identity as the report names it.
    pub fn namespace(&self) -> &str {
        return self.namespace.as_str();
    }

    /// How many mounts of the dataset this namespace holds.
    pub fn mount_count(&self) -> usize {
        return self.mounts.len();
    }
}
