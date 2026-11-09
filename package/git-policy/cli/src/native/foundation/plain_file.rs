//! What:
//!  Reading a file only when it is a plain one:
//! opened without blocking and without following a link,
//! refused when
//!  the open handle is a named pipe,
//! a device or a directory,
//! and never read past a limit.
//! Why:
//!  Opening a named pipe for reading waits for a writer that may never come,
//! and a command that waits while it
//!  owns a checkout or a worktree blocks every other command that wants it,
//! with nothing said and no end.
//! A
//!  file beside a checkout can be made by any process that may write that directory,
//! and a `.git`,
//! a
//!  registration's `gitdir`,
//! a journal and a list of files are all read this way,
//! so the type is taken from the open
//!  handle:
//! a name swapped for a link or a pipe between a check and the open cannot redirect the read,
//!  and the read is bounded by the caller's limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const bytes = readPlainFile(path, limit); // throws when the file is not a plain one
//! ```

use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

/// How many bytes of a small file this reads:
/// a journal,
/// a registration's `gitdir`,
/// a worktree's `.git` file,
/// a repository's
///  configuration or a list of paths.
///  Exempt from a citation under `LQM` (class:
/// display cap):
/// it bounds the record files this program
///  writes itself or Git writes,
/// and a file larger than it is refused with its path named rather than truncated.
/// Classed once in
///  `doc/planning/cli-git-shipped-limits-audit.md`.
pub const SMALL_FILE_LIMIT: u64 = 1 << 20;

/// What:
///  The bytes of the plain file at `path`,
/// opened without blocking and without following a link,
/// refused when it is
///  anything else or larger than `limit`.
/// Why:
///   See the module description.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readPlainFile(path: string, limit: number): Buffer; // throws
/// ```
pub fn read_plain_file(path: &Path, limit: u64) -> std::io::Result<Vec<u8>> {
    let file: std::fs::File = std::fs::OpenOptions::new().read(true).custom_flags(libc::O_NONBLOCK | libc::O_NOFOLLOW | libc::O_CLOEXEC).open(path)?;
    if !file.metadata()?.file_type().is_file() {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "it is not a plain file (a named pipe, a device or a directory)"));
    }
    let mut bytes: Vec<u8> = Vec::new();
    std::io::Read::read_to_end(&mut std::io::Read::take(&file, limit + 1), &mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "it is larger than this command reads"));
    }
    return Ok(bytes);
}

/// The text of the plain file at `path`,
/// read like `read_plain_file`.
pub fn read_plain_text(path: &Path) -> std::io::Result<String> {
    return match String::from_utf8(read_plain_file(path, SMALL_FILE_LIMIT)?) {
        Ok(text) => Ok(text),
        Err(_not_text) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "it is not text")),
    };
}

/// Plain-file controls stay out of the release executable.
#[cfg(test)]
#[path = "plain_file_tests.rs"]
mod tests;
