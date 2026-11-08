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

#[cfg(target_os = "linux")]
use crate::fs_mount_options::{MOUNTINFO_PATH, MountFacts, mount_holding, parse_mount_table};
#[cfg(target_os = "linux")]
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

impl EntryKind {
    /// What:
    ///  The word the plain messages use for what a path is.
    /// Why:
    ///  The TypeScript check prints the same words.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// entry.kind // 'file'
    /// ```
    pub fn name(self) -> &'static str {
        return match self {
            Self::Directory => "directory",
            Self::File => "file",
            Self::Link => "link",
            Self::Missing => "missing",
            Self::Other => "other",
        };
    }
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
/// It reads the mount table once and looks each device's UUID up once,
/// because a check asks for the holder of three paths on one filesystem
/// and the table of this host has hundreds of lines:
/// the first version read it three times and cost about a millisecond more than the commands it gates.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const realProbe: Probe = { ... };
/// ```
#[derive(Debug, Default)]
pub struct RealLinkProbe {
    /// The mount table,
    /// parsed on first use.
    #[cfg(target_os = "linux")]
    table: std::sync::OnceLock<Vec<MountFacts>>,
    /// The filesystem UUID found for each device,
    /// by the device's name in the table.
    #[cfg(target_os = "linux")]
    uuids: std::sync::Mutex<Vec<(String, String)>>,
    /// The table text to use instead of `/proc/self/mountinfo`,
    /// for the controls.
    #[cfg(target_os = "linux")]
    table_text: Option<Vec<u8>>,
    /// The folder of UUID links to use instead of `/dev/disk/by-uuid`,
    /// for the controls.
    #[cfg(target_os = "linux")]
    by_uuid_directory: Option<PathBuf>,
}

#[cfg(target_os = "linux")]
impl RealLinkProbe {
    /// A probe over a table text and a folder of UUID links the caller made,
    /// so a control needs no real mount.
    #[cfg(test)]
    pub(crate) fn over(table_text: Vec<u8>, by_uuid_directory: PathBuf) -> RealLinkProbe {
        return RealLinkProbe { table_text: Some(table_text), by_uuid_directory: Some(by_uuid_directory), ..RealLinkProbe::default() };
    }

    /// The mount table:
    /// the one of this process,
    /// read and parsed once.
    fn table(&self) -> &[MountFacts] {
        if self.table.get().is_none() {
            let text: Vec<u8> = match &self.table_text {
                Some(given) => given.clone(),
                None => std::fs::read(MOUNTINFO_PATH).unwrap_or_default(),
            };
            let _first: Result<(), Vec<MountFacts>> = self.table.set(parse_mount_table(text.as_slice()));
        }
        return self.table.get().map_or(&[], Vec::as_slice);
    }

    /// The mount that holds a path,
    /// nothing when the path does not resolve.
    fn mount_of(&self, path: &Path) -> Option<MountFacts> {
        let real: PathBuf = std::fs::canonicalize(path).ok()?;
        return mount_holding(self.table(), real.as_path()).cloned();
    }

    /// The filesystem UUID of the device a mount names,
    /// looked up once per device.
    fn uuid_of_mount(&self, mount: &MountFacts) -> String {
        if let Ok(known) = self.uuids.lock() {
            for (source, uuid) in known.iter() {
                if *source == mount.source {
                    return uuid.clone();
                }
            }
        }
        let folder: &Path = self.by_uuid_directory.as_deref().unwrap_or(Path::new(BY_UUID_DIRECTORY));
        let found: String = filesystem_uuid_of_source(mount.source.as_str(), folder).unwrap_or_default();
        if let Ok(mut known) = self.uuids.lock() {
            known.push((mount.source.clone(), found.clone()));
        }
        return found;
    }
}

/// Without the Linux mount table there is no mount to name:
/// the check then reports an unknown filesystem and a drive that is not mounted.
#[cfg(not(target_os = "linux"))]
impl RealLinkProbe {
    /// No mount.
    fn mount_of(&self, _path: &Path) -> Option<NoMount> {
        return None;
    }

    /// Never reached without the Linux mount table.
    fn uuid_of_mount(&self, _mount: &NoMount) -> String {
        return String::new();
    }
}

/// A mount that cannot exist on a system without the Linux mount table.
#[cfg(not(target_os = "linux"))]
struct NoMount {
    /// Never read.
    mount_point: PathBuf,
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
        let Some(mount) = self.mount_of(path) else {
            return String::new();
        };
        return self.uuid_of_mount(&mount);
    }

    fn is_mount_point(&self, path: &Path) -> bool {
        let Ok(resolved) = std::fs::canonicalize(path) else {
            return false;
        };
        return match self.mount_of(resolved.as_path()) {
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

/// Probe controls stay out of the release executable.
#[cfg(all(test, target_os = "linux"))]
#[path = "repository_link_probe_tests.rs"]
mod tests;
