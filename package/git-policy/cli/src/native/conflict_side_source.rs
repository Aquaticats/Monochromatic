//! What:
//!  Finding the worktree file a conflict-side command is about to replace, and copying it.
//! Why:
//!  The copy is the owner's "save, then allow": whatever Git is about to overwrite must be on disk first. This file is
//!  where a file is found and read, so it is where a stale, torn or wrongly found copy would begin.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const located = locate(root, 'dir/a.rs'); // undefined when nothing is there to save
//! if (located) copyLocated(located, destination, () => {});
//! ```

use std::io::{Read, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

/// Why a copy could not be written.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SaveFailure {
    /// The repository-relative path being copied, or the place the copies go when none was reached.
    pub subject: String,
    /// The operating system's words.
    pub reason: String,
}

/// A worktree entry that exists and is to be copied.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Located {
    /// The path the entry has now.
    source: PathBuf,
}

/// What:
///  Find the entry at `relative` below `root`, or nothing when no entry is there.
/// Why:
///   A path with no file in the worktree has nothing to copy, and Git then creates it; every other lookup failure
///  stops the save, because Git would still write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function locate(root: string, relative: Uint8Array): Located | undefined;
/// ```
pub fn locate(root: &Path, relative: &[u8]) -> Result<Option<Located>, SaveFailure> {
    use std::os::unix::ffi::OsStringExt;
    let source: PathBuf = root.join(std::ffi::OsString::from_vec(relative.to_vec()));
    return match std::fs::symlink_metadata(source.as_path()) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(SaveFailure { subject: String::from_utf8_lossy(relative).into_owned(), reason: error.to_string() }),
        Ok(_) => Ok(Some(Located { source })),
    };
}

/// Copy one file's content, mode and kind to `destination`, which must not exist, and sync the file.
///  `between` runs after each chunk is written, so a control can change the source mid-copy.
pub fn copy_located(located: &Located, destination: &Path, between: &mut dyn FnMut()) -> std::io::Result<()> {
    let source: &Path = located.source.as_path();
    let metadata: std::fs::Metadata = std::fs::symlink_metadata(source)?;
    if metadata.file_type().is_symlink() {
        std::os::unix::fs::symlink(std::fs::read_link(source)?, destination)?;
        return Ok(());
    }
    if !metadata.file_type().is_file() {
        return Err(std::io::Error::other("it is neither a file nor a symbolic link"));
    }
    let mut from: std::fs::File = std::fs::OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW).open(source)?;
    let mut to: std::fs::File = std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(destination)?;
    let written: std::io::Result<()> = (|| -> std::io::Result<()> {
        let mut buffer: Vec<u8> = vec![0; 1 << 16];
        loop {
            let count: usize = from.read(buffer.as_mut_slice())?;
            if count == 0 {
                break;
            }
            to.write_all(&buffer[..count])?;
            between();
        }
        to.set_permissions(std::fs::Permissions::from_mode_bits(metadata.mode() & 0o777))?;
        return to.sync_all();
    })();
    if let Err(error) = written {
        // A cut-short file named like a copy would pass for the whole file; the command is rejected, so none is left.
        let _removed = std::fs::remove_file(destination);
        return Err(error);
    }
    return Ok(());
}

/// `Permissions` from raw mode bits, named for what the call does.
trait FromModeBits {
    /// The permissions with exactly these bits.
    fn from_mode_bits(bits: u32) -> Self;
}

impl FromModeBits for std::fs::Permissions {
    fn from_mode_bits(bits: u32) -> Self {
        return std::os::unix::fs::PermissionsExt::from_mode(bits);
    }
}

/// Where a file is found and how it is copied, including a file that changes while it is read.
#[cfg(test)]
#[path = "conflict_side_source_tests.rs"]
mod tests;
