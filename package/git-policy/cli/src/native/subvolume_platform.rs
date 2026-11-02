//! What:
//!  The operating-system operations the subvolume migration needs, behind one trait, and the Linux implementation
//!  that calls the primitives (`fs_kind`, `fs_btrfs_ioctl`, `fs_clone_exchange`, `fs_mount_options`).
//! Why:
//!  The migration's journal, resume, rollback and refusal logic is the same on any filesystem; only five operations
//!  need btrfs. With the operations behind a trait, the controls run the whole logic on a plain directory tree with a
//!  stand-in platform where there is no btrfs (the container gate), and run it against the real platform on btrfs
//!  (the host task). `doc/planning/cli-git-btrfs-worktree.md` is the design the primitives come from.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! interface Platform { filesystemKind(path): Kind; isSubvolumeRoot(path): boolean; createSubvolume(path): void; ... }
//! ```

use super::fs_btrfs_ioctl::{subvolume_create_at, subvolume_destroy_at};
use super::fs_clone_exchange::{clone_file, exchange_paths};
use super::fs_kind::{FilesystemKind, filesystem_facts, is_subvolume_root};
use super::fs_mount_options::read_mount_facts;
use super::fs_primitive_error::PrimitiveError;
use std::fs::File;
use std::path::Path;

/// What:
///  The operations of the migration that depend on the kind of filesystem.
///  `&self` borrows the platform read-only so one platform serves a whole run; `dyn Platform` is how a caller holds
///  "any implementation" (TS interface).
/// Why:
///   See the module description.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface Platform { ... }
/// ```
pub trait Platform {
    /// The filesystem kind holding `path` and the bytes free there.
    fn filesystem_facts(&self, path: &Path) -> Result<(FilesystemKind, u64), PrimitiveError>;
    /// Whether `path` is the root of a subvolume.
    fn is_subvolume_root(&self, path: &Path) -> Result<bool, PrimitiveError>;
    /// Create an empty subvolume at `path`.
    fn create_subvolume(&self, path: &Path) -> Result<(), PrimitiveError>;
    /// Destroy the subvolume at `path` with the kernel call (the caller falls back to emptying and `rmdir`).
    fn destroy_subvolume(&self, path: &Path) -> Result<(), PrimitiveError>;
    /// Whether an unprivileged subvolume destroy is allowed on the mount holding `path`.
    fn removal_allowed(&self, path: &Path) -> bool;
    /// Whether `path` is itself a mount point (a mount point cannot be exchanged).
    fn is_mount_point(&self, path: &Path) -> bool;
    /// Clone a whole file's data blocks.
    fn clone_file(&self, source: &File, destination: &File, path: &Path) -> Result<(), PrimitiveError>;
    /// Swap two paths atomically.
    fn exchange(&self, first: &Path, second: &Path) -> Result<(), PrimitiveError>;
    /// Make `new` another name of the file `existing`; fails with `EXDEV` where the two sit in different subvolumes.
    fn hard_link(&self, existing: &Path, new: &Path) -> std::io::Result<()> {
        return std::fs::hard_link(existing, new);
    }
}

/// What:
///  The real platform: every operation is the primitive of the same name.
/// Why:
///   Production code and the host controls use it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const linuxPlatform: Platform = { filesystemKind, isSubvolumeRoot, ... };
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct LinuxPlatform;

/// The primitives, called as they are.
impl Platform for LinuxPlatform {
    fn filesystem_facts(&self, path: &Path) -> Result<(FilesystemKind, u64), PrimitiveError> {
        return filesystem_facts(path);
    }

    fn is_subvolume_root(&self, path: &Path) -> Result<bool, PrimitiveError> {
        return is_subvolume_root(path);
    }

    fn create_subvolume(&self, path: &Path) -> Result<(), PrimitiveError> {
        return subvolume_create_at(path);
    }

    fn destroy_subvolume(&self, path: &Path) -> Result<(), PrimitiveError> {
        return subvolume_destroy_at(path);
    }

    fn removal_allowed(&self, path: &Path) -> bool {
        return read_mount_facts(path)
            .ok()
            .flatten()
            .is_some_and(|facts| { return facts.has_option("user_subvol_rm_allowed"); });
    }

    fn is_mount_point(&self, path: &Path) -> bool {
        let Ok(real) = std::fs::canonicalize(path) else { return false };
        return read_mount_facts(real.as_path())
            .ok()
            .flatten()
            .is_some_and(|facts| { return facts.mount_point == real; });
    }

    fn clone_file(&self, source: &File, destination: &File, path: &Path) -> Result<(), PrimitiveError> {
        return clone_file(source, destination, path);
    }

    fn exchange(&self, first: &Path, second: &Path) -> Result<(), PrimitiveError> {
        return exchange_paths(first, second);
    }
}
