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

use crate::fs_btrfs_ioctl::{subvolume_create_at, subvolume_destroy_at};
use crate::fs_clone_exchange::{clone_file, exchange_paths};
use crate::fs_kind::{FilesystemKind, filesystem_facts, is_subvolume_root};
use crate::fs_mount_options::read_mount_facts;
use crate::fs_primitive_error::PrimitiveError;
use crate::git_system_config::with_real_git;
use crate::worktree_clean_content::ConversionEnvironment;
use std::ffi::OsString;
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
    /// Which configuration files and system attributes file Git reads here. The default names no Git and no variables, so the
    ///  system files are unknown and a content decision that needs them refuses; a stand-in overrides it with what a control
    ///  needs, and the real platform answers for the Git it forwards to.
    fn conversion_environment(&self) -> ConversionEnvironment {
        return ConversionEnvironment::from_variables(&[]);
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
#[derive(Clone, Debug, Default)]
pub struct LinuxPlatform {
    /// The environment of the Git this platform's command forwards to, with the forward-target variable naming that Git, so the
    ///  facts about Git's files are learned from the Git that reads them and never from the wrapper's own environment, which
    ///  does not carry the variable; empty where no Git is known.
    pub git_environment: Vec<(OsString, OsString)>,
}

impl LinuxPlatform {
    /// What:
    ///  The platform of a command that forwards to the real Git `real_git`, from the environment the command was started in.
    /// Why:
    ///   The wrapper puts the forward-target variable only into its children's environment, so the command names the Git it
    ///  resolved itself; without one (`None`) the environment is left as it is and the system files stay unknown.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const platform = linuxPlatform({ ...environment, [FORWARD_TARGET]: realGit });
    /// ```
    pub fn forwarding_to(environment: &[(OsString, OsString)], real_git: Option<&Path>) -> LinuxPlatform {
        let named: Vec<(OsString, OsString)> = match real_git {
            Some(found) => with_real_git(environment, found),
            None => environment.to_vec(),
        };
        return LinuxPlatform { git_environment: named };
    }
}

/// The primitives, called as they are.
impl Platform for LinuxPlatform {
    fn conversion_environment(&self) -> ConversionEnvironment {
        return ConversionEnvironment::from_variables(self.git_environment.as_slice());
    }

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
        let Ok(Some(facts)) = read_mount_facts(path) else { return false };
        return facts.has_option("user_subvol_rm_allowed");
    }

    fn is_mount_point(&self, path: &Path) -> bool {
        let Ok(real) = std::fs::canonicalize(path) else { return false };
        let Ok(Some(facts)) = read_mount_facts(real.as_path()) else { return false };
        return facts.mount_point == real;
    }

    fn clone_file(&self, source: &File, destination: &File, path: &Path) -> Result<(), PrimitiveError> {
        return clone_file(source, destination, path);
    }

    fn exchange(&self, first: &Path, second: &Path) -> Result<(), PrimitiveError> {
        return exchange_paths(first, second);
    }
}

/// Platform controls stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_platform_tests.rs"]
mod tests;
