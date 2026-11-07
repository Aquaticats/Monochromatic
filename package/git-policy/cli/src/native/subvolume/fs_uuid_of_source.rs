//! What:
//!  The filesystem UUID of the block device a mount names as its source,
//! read from udev's `/dev/disk/by-uuid` links.
//! Why:
//!  `fs_mount_options.rs` says which mount holds a path and what its source is;
//! the repository link check needs the UUID of that filesystem,
//! and `findmnt` (a process start) is banned on the path of every Git command.
//! This is the one reader of that fact in the crate.
//! The links are the device's,
//! so a source that is not a device with a link has no UUID here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const uuid = (await readdir(byUuid)).find(async (name) => (await realpath(join(byUuid, name))) === (await realpath(source)));
//! ```

use std::path::Path;

/// Where udev keeps one link per filesystem UUID,
/// each to the device that holds the filesystem.
pub const BY_UUID_DIRECTORY: &str = "/dev/disk/by-uuid";

/// What:
///  The UUID of the filesystem on the device `source` names,
/// nothing when the source is not an absolute path,
/// does not resolve,
/// or no link in `by_uuid_directory` leads to the same device.
/// Why:
///  A relative or non-path source (`tmpfs`,
/// `overlay`) must never resolve against the working directory.
/// The directory is a parameter so a test uses a folder of links it made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await uuidOfSource(source)
/// ```
pub fn filesystem_uuid_of_source(source: &str, by_uuid_directory: &Path) -> Option<String> {
    todo!("{source} {}", by_uuid_directory.display())
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "fs_uuid_of_source_tests.rs"]
mod tests;
