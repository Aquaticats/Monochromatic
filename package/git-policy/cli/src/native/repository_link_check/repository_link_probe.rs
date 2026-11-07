//! What:
//!  The filesystem questions the repository link check asks,
//! and the probe that answers them from the running system.
//! Why:
//!  The questions equal the TypeScript `Probe` field for field,
//! so one table of fixture states drives both checks.
//! The real probe creates nothing and starts no process:
//! it reads the mount table in process (`subvolume/fs_mount_options.rs`)
//! and udev's `/dev/disk/by-uuid` links (`subvolume/fs_uuid_of_source.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const probe: Probe = { lstat, readlink, realpath, readText, uuidOf, isMountPoint };
//! ```

use crate::fs_mount_options::{MountFacts, read_mount_facts};
use crate::fs_uuid_of_source::{BY_UUID_DIRECTORY, filesystem_uuid_of_source};
use std::path::{Path, PathBuf};

/// What:
///  What a path is,
/// looked at without following a final link.
/// Why:
///  The check tells a link,
/// a folder and a file apart,
/// and a missing path from all three.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type EntryKind = 'directory' | 'file' | 'link' | 'missing' | 'other';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntryKind {
    /// A folder.
    Directory,
    /// A regular file.
    File,
    /// A symbolic link.
    Link,
    /// Nothing is there.
    Missing,
    /// Anything else,
    /// or something that cannot be looked at.
    Other,
}

/// What:
///  The questions the check asks of the filesystem.
/// Why:
///  The real probe answers from the system;
/// tests answer from fixtures,
/// and a test of the deadline answers none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Probe = { lstat; readlink; realpath; readText; uuidOf; isMountPoint };
/// ```
pub trait LinkProbe {
    /// Looks at a path without following a final link.
    fn lstat(&self, path: &Path) -> EntryKind;
    /// The text of a link;
    /// empty when it cannot be read.
    fn readlink(&self, path: &Path) -> String;
    /// Every link of a path resolved;
    /// nothing when the path does not resolve.
    fn realpath(&self, path: &Path) -> Option<PathBuf>;
    /// A small text file;
    /// empty when it cannot be read.
    fn read_text(&self, path: &Path) -> String;
    /// The filesystem UUID of the filesystem that holds a path;
    /// empty when it has none or cannot be found.
    fn uuid_of(&self, path: &Path) -> String;
    /// Whether a path is a mount point.
    fn is_mount_point(&self, path: &Path) -> bool;
}

/// What:
///  The probe that reads the running system.
/// Why:
///  The gate's only probe in production.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const realProbe: Probe = { ... };
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct RealLinkProbe;

/// The mount that holds a path,
/// from the table of this process,
/// nothing when the path does not resolve or the table cannot be read.
fn mount_of(path: &Path) -> Option<MountFacts> {
    return read_mount_facts(path).ok().flatten();
}

impl LinkProbe for RealLinkProbe {
    fn lstat(&self, path: &Path) -> EntryKind {
        let Ok(metadata) = std::fs::symlink_metadata(path) else {
            return if path_is_absent(path) { EntryKind::Missing } else { EntryKind::Other };
        };
        let kind = metadata.file_type();
        if kind.is_symlink() {
            return EntryKind::Link;
        }
        if kind.is_dir() {
            return EntryKind::Directory;
        }
        return if kind.is_file() { EntryKind::File } else { EntryKind::Other };
    }

    fn readlink(&self, path: &Path) -> String {
        return match std::fs::read_link(path) {
            Ok(text) => text.to_string_lossy().into_owned(),
            Err(_unreadable) => String::new(),
        };
    }

    fn realpath(&self, path: &Path) -> Option<PathBuf> {
        return std::fs::canonicalize(path).ok();
    }

    fn read_text(&self, path: &Path) -> String {
        return std::fs::read_to_string(path).unwrap_or_default();
    }

    fn uuid_of(&self, path: &Path) -> String {
        let Some(mount) = mount_of(path) else {
            return String::new();
        };
        return filesystem_uuid_of_source(mount.source.as_str(), Path::new(BY_UUID_DIRECTORY)).unwrap_or_default();
    }

    fn is_mount_point(&self, path: &Path) -> bool {
        let Ok(resolved) = std::fs::canonicalize(path) else {
            return false;
        };
        return match mount_of(resolved.as_path()) {
            Some(mount) => mount.mount_point == resolved,
            None => false,
        };
    }
}

/// Whether a path is absent or cannot be followed:
/// the errors that are an answer and not a failure.
fn path_is_absent(path: &Path) -> bool {
    return match std::fs::symlink_metadata(path) {
        Ok(_found) => false,
        Err(error) => {
            error.kind() == std::io::ErrorKind::NotFound || error.raw_os_error() == Some(libc::ENOTDIR) || error.raw_os_error() == Some(libc::ELOOP)
        }
    };
}
