//! What:
//!  Small facts about the disk and the directories a layout read needs:
//! a small file's text,
//! whether a directory holds the
//!  entries of a Git directory,
//! who owns a path,
//! which device a directory is on,
//! and where a `-C` option leads.
//! Why:
//!  `repo_read_layout.rs` finds and checks the repository layout;
//! these facts read the file system and the command line only,
//!  so they sit in a file of their own that does not use it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function startDirectory(globalPrefix: string[]): string | undefined; // the process directory moved by leading -C options
//! function deviceOf(path: string): bigint | undefined;
//! ```

use crate::private_storage::read_regular_file;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The longest `HEAD` or `commondir` or gitfile this reader looks at.
///  Exempt from a citation under `LQM` (class:
/// reader fallback):
/// a file,
/// chain or count past this bound makes the reader
///  decline and real Git answer,
/// so a Git start is added and no result changes.
/// Classed once in
///  `doc/planning/cli-git-shipped-limits-audit.md`,
/// "Exempt limits";
/// the coordinator's pick under the owner's delegation,
///  open to the owner's veto.
pub(crate) const SMALL_FILE_LIMIT: u64 = 4096;

/// The directory a command starts in:
/// the process directory moved by leading `-C <path>` options.
///  Any other global option returns `None`.
pub(crate) fn start_directory(global_prefix: &[OsString]) -> Option<PathBuf> {
    let mut directory: PathBuf = std::env::current_dir().ok()?;
    let mut options = global_prefix.iter();
    while let Some(option) = options.next() {
        if option != "-C" {
            return None;
        }
        let path: &OsString = options.next()?;
        if !path.is_empty() {
            directory = directory.join(path);
        }
        // Git changes directory at each `-C` in turn and fails at the first one that is not a directory.
        if !directory.is_dir() {
            return None;
        }
    }
    return std::fs::canonicalize(directory).ok();
}

/// A small regular file's text,
/// refusing links.
/// Why:
///   Text is what a caller that compares the file's spelling needs;
///  a caller that reads a pathname out of the file needs the bytes (`small_file_bytes`).
pub(crate) fn small_file(path: &Path) -> Option<String> {
    return String::from_utf8(small_file_bytes(path)?).ok();
}

/// What:
///  A small regular file's bytes,
/// refusing links.
/// Why:
///   A file that names a pathname names it as bytes,
/// and only a decoding would lose a name that is not UTF-8.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readFile(path) // a link or a file of another type is refused
/// ```
pub(crate) fn small_file_bytes(path: &Path) -> Option<Vec<u8>> {
    return read_regular_file(path, SMALL_FILE_LIMIT).ok();
}

/// Whether `directory` holds the three entries Git takes for a Git directory.
pub(crate) fn has_git_directory_entries(directory: &Path) -> bool {
    return directory.join("HEAD").exists() && directory.join("objects").is_dir() && directory.join("refs").is_dir();
}

/// Whether `byte` is an ASCII hexadecimal digit.
pub(crate) fn is_hex_byte(byte: u8) -> bool {
    return byte.is_ascii_hexdigit();
}

/// Whether `path` is owned by the effective user,
/// as Git's ownership check requires.
#[cfg(unix)]
pub(crate) fn owned_by_current_user(path: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;
    let Ok(metadata) = std::fs::symlink_metadata(path) else {
        return false;
    };
    // SAFETY: `geteuid` takes no argument, touches no memory and cannot fail.
    return metadata.uid() == unsafe { libc::geteuid() };
}

/// Without user ids,
/// every repository is left to Git.
#[cfg(not(unix))]
pub(crate) fn owned_by_current_user(_path: &Path) -> bool {
    return false;
}

/// The device a directory lives on,
/// so a walk can stop where Git stops.
#[cfg(unix)]
pub(crate) fn device_of(path: &Path) -> Option<u64> {
    use std::os::unix::fs::MetadataExt;
    let metadata = std::fs::metadata(path).ok()?;
    return Some(metadata.dev());
}

/// Without device numbers,
/// every walk is left to Git.
#[cfg(not(unix))]
pub(crate) fn device_of(_path: &Path) -> Option<u64> {
    return None;
}

/// The three-entry check,
/// with `HEAD` in the worktree's directory and the rest in the common directory.
pub(crate) fn has_git_directory_entries_split(git_dir: &Path, common_dir: &Path) -> bool {
    return git_dir.join("HEAD").exists() && common_dir.join("objects").is_dir() && common_dir.join("refs").is_dir();
}
