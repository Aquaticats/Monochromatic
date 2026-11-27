//! What:
//!  A stand-in `Platform` on a plain directory tree:
//!  a "subvolume" is a directory registered by its device and inode,
//!  an identity that survives the exchange and a rename as a real subvolume root's does,
//!  file clones are refused so the byte-copy path runs,
//!  and the exchange is the real `renameat2`.
//! Why:
//!  The migration's journal,
//!  resume,
//!  rollback,
//!  refusal and index logic needs no btrfs,
//!  only the operations of `Platform` do,
//!  so these controls run on the container gate's tmpfs fixture root too.
//!  The fields let a control make the filesystem look unsuitable,
//!  decide what `removal_allowed` answers,
//!  and forward clones to the kernel on a reflink filesystem.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class PlainPlatform implements Platform { subvolumes = new Set<string>(); ... }
//! ```

use crate::fs_clone_exchange::exchange_paths;
use crate::fs_kind::FilesystemKind;
use crate::fs_primitive_error::PrimitiveError;
use crate::subvolume_platform::Platform;
use std::collections::HashSet;
use std::fs::File;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::sync::Mutex;

/// What:
///  A platform on plain directories.
/// Why:
///   See the module description.
/// `kind` and `free` let a control make the filesystem look unsuitable;
/// `removal`
///  decides what `removal_allowed` answers and whether `destroy_subvolume` works;
/// `real_clone` forwards clones to the
///  kernel (true on a reflink filesystem only).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class PlainPlatform implements Platform { subvolumes = new Set<string>(); ... }
/// ```
pub(crate) struct PlainPlatform {
    /// Device and inode of every directory made by `create_subvolume`.
    pub subvolumes: Mutex<HashSet<(u64, u64)>>,
    /// What `filesystem_facts` reports as the kind.
    pub kind: FilesystemKind,
    /// What `filesystem_facts` reports as free bytes;
    /// `None` asks the real filesystem.
    pub free: Option<u64>,
    /// Whether subvolume destroy by the kernel call is allowed.
    pub removal: bool,
    /// Whether file clones go to the kernel.
    pub real_clone: bool,
    /// Whether `is_mount_point` answers true.
    pub mount_point: bool,
}

impl PlainPlatform {
    /// A platform that looks like btrfs with plenty of room,
    /// with removal allowed and clones refused.
    pub(crate) fn new() -> PlainPlatform {
        return PlainPlatform {
            subvolumes: Mutex::new(HashSet::new()),
            kind: FilesystemKind::Btrfs,
            free: Some(u64::MAX / 2),
            removal: true,
            real_clone: false,
            mount_point: false,
        };
    }

    /// Register a directory as a subvolume root.
    pub(crate) fn register(
        &self,
        path: &Path,
    ) {
        let metadata: std::fs::Metadata = std::fs::metadata(path).expect("registered path exists");
        self.subvolumes
            .lock()
            .expect("the registry of subvolumes")
            .insert((metadata.dev(), metadata.ino()));
    }
}

/// The five operations on plain directories.
impl Platform for PlainPlatform {
    fn conversion_environment(&self) -> crate::worktree_clean_content::ConversionEnvironment {
        return crate::worktree_clean_content::ConversionEnvironment {
            scope: crate::repo_read_config_scan::ScopeEnvironment {
                no_system: true,
                ..crate::repo_read_config_scan::ScopeEnvironment::default()
            },
            variables: Vec::new(),
            system_attributes: crate::worktree_clean_content::SystemAttributes::Disabled,
            alternate_attribute_source: None,
        };
    }

    fn filesystem_facts(
        &self,
        path: &Path,
    ) -> Result<(FilesystemKind, u64), PrimitiveError> {
        let (_, real_free) = crate::fs_kind::filesystem_facts(path)?;
        return Ok((self.kind, self.free.unwrap_or(real_free)));
    }

    fn is_subvolume_root(
        &self,
        path: &Path,
    ) -> Result<bool, PrimitiveError> {
        let Ok(metadata) = std::fs::metadata(path) else {
            return Ok(false);
        };
        return Ok(self
            .subvolumes
            .lock()
            .expect("the registry of subvolumes")
            .contains(&(metadata.dev(), metadata.ino())));
    }

    fn create_subvolume(
        &self,
        path: &Path,
    ) -> Result<(), PrimitiveError> {
        std::fs::create_dir(path).map_err(|error: std::io::Error| {
            return PrimitiveError {
                operation: "BTRFS_IOC_SUBVOL_CREATE",
                path: path.to_path_buf(),
                errno: error.raw_os_error().unwrap_or(0),
            };
        })?;
        self.register(path);
        return Ok(());
    }

    fn destroy_subvolume(
        &self,
        path: &Path,
    ) -> Result<(), PrimitiveError> {
        if !self.removal {
            return Err(PrimitiveError {
                operation: "BTRFS_IOC_SNAP_DESTROY",
                path: path.to_path_buf(),
                errno: libc::EPERM,
            });
        }
        let metadata: std::fs::Metadata =
            std::fs::metadata(path).map_err(|error: std::io::Error| {
                return PrimitiveError {
                    operation: "BTRFS_IOC_SNAP_DESTROY",
                    path: path.to_path_buf(),
                    errno: error.raw_os_error().unwrap_or(0),
                };
            })?;
        std::fs::remove_dir_all(path).map_err(|error: std::io::Error| {
            return PrimitiveError {
                operation: "BTRFS_IOC_SNAP_DESTROY",
                path: path.to_path_buf(),
                errno: error.raw_os_error().unwrap_or(0),
            };
        })?;
        self.subvolumes
            .lock()
            .expect("the registry of subvolumes")
            .remove(&(metadata.dev(), metadata.ino()));
        return Ok(());
    }

    fn removal_allowed(
        &self,
        _path: &Path,
    ) -> bool {
        return self.removal;
    }

    fn is_mount_point(
        &self,
        _path: &Path,
    ) -> bool {
        return self.mount_point;
    }

    fn clone_file(
        &self,
        source: &File,
        destination: &File,
        path: &Path,
    ) -> Result<(), PrimitiveError> {
        if self.real_clone {
            return crate::fs_clone_exchange::clone_file(source, destination, path);
        }
        return Err(PrimitiveError {
            operation: "FICLONE",
            path: path.to_path_buf(),
            errno: libc::EOPNOTSUPP,
        });
    }

    fn exchange(
        &self,
        first: &Path,
        second: &Path,
    ) -> Result<(), PrimitiveError> {
        return exchange_paths(first, second);
    }

    /// A control that uses this platform simulates a kill in the process,
    /// which loses no page cache,
    /// so the journal's flushes are left out.
    fn flushes_to_disk(&self) -> bool {
        return false;
    }
}
