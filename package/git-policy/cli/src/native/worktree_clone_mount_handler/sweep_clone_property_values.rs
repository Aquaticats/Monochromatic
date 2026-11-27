//! What:
//!  The values the design gives a clone's own properties and its origin
//!  snapshot:
//!  the legacy mountpoint,
//!  the property value that says "not encrypted",
//!  the separator between a dataset and its snapshot,
//!  and the prefix the add route gives the snapshot it clones.
//! Why:
//!  The checks that read a child's reply and the words a refusal answers with
//!  both name these values,
//!  and a file that only words a refusal must be readable without the checks
//!  (rule `MXL`):
//!  `sweep_clone_refusals.rs` names the mountpoint here rather than reaching the
//!  file that refuses a clone
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Dataset and path layout").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const LEGACY_MOUNTPOINT = 'legacy';
//! const SNAPSHOT_SEPARATOR = '@';
//! const CLONE_ORIGIN_SNAPSHOT_PREFIX = 'git-worktree-clone-origin-';
//! ```

/// The mountpoint value that keeps `zfs mount -a` and the systemd mount
/// generator away from a clone.
pub const LEGACY_MOUNTPOINT: &str = "legacy";

/// The property value that says a dataset's data is not encrypted.
pub const ENCRYPTION_OFF: u64 = 0;

/// The separator between a dataset and the snapshot of it a clone names as its
/// origin.
pub const SNAPSHOT_SEPARATOR: char = '@';

/// The prefix the design gives the snapshot the add route clones a managed
/// clone from.
pub const CLONE_ORIGIN_SNAPSHOT_PREFIX: &str = "git-worktree-clone-origin-";
