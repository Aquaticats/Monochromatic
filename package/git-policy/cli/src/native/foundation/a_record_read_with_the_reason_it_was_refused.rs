//! What:
//!  Reading a record the wrapper wrote:
//!  a regular file,
//!  never through a final link,
//!  never waiting for a writer when the name holds a named pipe,
//!  never past the caller's limit,
//!  and the reason a refusal gives.
//! Why:
//!  Recovery reads untrusted state,
//!  so a link,
//!  a directory or a file that is too large is named as what it is and never followed.
//!  Some readers answer a waiting client,
//!  so an open that waited for a writer would hold back the proof of life and no timer could end it
//!  (`SPEC.md`,
//!  "Every wait has a bound").
//!  The type of the open handle's own metadata is the only answer a swap between the open and the check cannot change.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const bytes = readRegularFile(path, limit); // throws ReadRefusal
//! ```

/// What:
///  `File` is an open file handle;
///  `OpenOptions` configures how one is opened.
/// Why:
///  A no-follow open needs the configurable form.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { open, constants } from 'node:fs/promises';
/// ```
use std::fs::{
    File,
    OpenOptions,
};
/// The trait that gives files `.read_to_end` and `.take`.
use std::io::Read;
/// `Path` is a borrowed filesystem path of raw bytes.
use std::path::Path;

/// What:
///  Why a no-follow read refused a path.
///  An `enum` is a closed set of alternatives.
/// Why:
///   A missing file is an expected answer for optional records;
///  a link,
///  a directory or an
///       over-long file is unsafe or corrupt state that recovery must name,
///  never follow.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReadRefusal = { kind: 'missing' } | { kind: 'not-regular' } | { kind: 'too-large' } | { kind: 'io'; error: Error };
/// ```
#[derive(Debug)]
pub enum ReadRefusal {
    /// Nothing exists at the path.
    Missing,
    /// The path names a link,
    ///  directory or other non-regular file.
    NotRegular,
    /// The file holds more bytes than the caller allows.
    TooLarge,
    /// The operating system refused the read for another reason.
    Io(
        /// The operating-system error that refused the read.
        std::io::Error,
    ),
}

/// What:
///  Open a file for reading without following a final link,
///  and without waiting for a writer when the name holds a named pipe.
/// Why:
///   Records are always files the wrappers created;
///  a link planted at a record path must
///       be refused,
///  never read through.
///  `O_NONBLOCK` makes the open answer whatever the name holds,
///  so the type check that follows refuses a pipe at once instead of waiting inside `open(2)`
///  for a writer that may never come
///  (`SPEC.md`,
///  "Every wait has a bound";
///  a trial reviewer's finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await open(path, O_RDONLY | O_NOFOLLOW | O_NONBLOCK);
/// ```
fn open_no_follow(path: &Path) -> std::io::Result<File> {
    // `mut` allows the builder to be configured step by step.
    let mut options: OpenOptions = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        // The trait adds `.custom_flags` to open options on Unix.
        /// The Unix open-option extensions,
        ///  for the no-follow flag and the non-blocking open.
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    return options.open(path);
}

/// What:
///  Open a record for reading:
///  a regular file,
///  never through a final link,
///  and without waiting for a writer when the name holds a named pipe.
/// Why:
///   Some readers answer a waiting client,
/// so an open that waited for a writer would hold back the proof of life
///  and no timer could end it;
///  a link planted at a record path must be refused,
///  never read through;
///  and the type of the handle's own metadata is the only answer a swap between the open and the check cannot
///  change.
///  A caller that also bounds how much it reads is `read_regular_file`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const file = await open(path, O_RDONLY | O_NOFOLLOW | O_NONBLOCK); if (!(await file.stat()).isFile()) throw 'not regular';
/// ```
pub fn open_regular_file(path: &Path) -> Result<File, ReadRefusal> {
    let file: File = match open_no_follow(path) {
        Ok(opened) => opened,
        Err(error) => return Err(open_refusal(error)),
    };
    // The handle's own metadata, so a swap between the check and the read is impossible.
    let metadata: std::fs::Metadata = file.metadata().map_err(ReadRefusal::Io)?;
    if !metadata.is_file() {
        return Err(ReadRefusal::NotRegular);
    }
    return Ok(file);
}

/// What:
///  Read a whole regular file without following links,
///  up to `limit` bytes.
///       `u64` is an unsigned 64-bit size;
///  `Result<Vec<u8>, ReadRefusal>` is "the bytes,
///  or why
///       not".
/// Why:
///   Recovery reads untrusted state:
///  a link or a directory must be named as unsafe,
///  and a
///       runaway file must not be read into memory.
///  The open is `open_regular_file`'s,
/// so a named pipe at the record's name is refused instead of waited on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readPrivateFile(path: string, limit: number): Promise<Uint8Array>; // throws ReadRefusal
/// ```
pub fn read_regular_file(
    path: &Path,
    limit: u64,
) -> Result<Vec<u8>, ReadRefusal> {
    let file: File = open_regular_file(path)?;
    // The handle's own metadata, so a swap between the check and the read is impossible.
    let metadata: std::fs::Metadata = file.metadata().map_err(ReadRefusal::Io)?;
    if metadata.len() > limit {
        return Err(ReadRefusal::TooLarge);
    }
    // `.take(limit + 1)` caps the read even if the file grows after the size check.
    let mut bytes: Vec<u8> = Vec::new();
    file.take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(ReadRefusal::Io)?;
    if bytes.len() as u64 > limit {
        return Err(ReadRefusal::TooLarge);
    }
    return Ok(bytes);
}

/// What:
///  Classify an open failure as missing,
///  a refused link,
///  or another error.
/// Why:
///   `O_NOFOLLOW` on a link fails with `ELOOP` on Linux and macOS;
///  that is the unsafe-link
///       answer,
///  not an input-output failure.
///  A path through a non-directory (`ENOTDIR`) names
///       no file,
///  which the incumbent treats like a missing one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function openRefusal(error: NodeJS.ErrnoException): ReadRefusal;
/// ```
fn open_refusal(error: std::io::Error) -> ReadRefusal {
    if error.kind() == std::io::ErrorKind::NotFound {
        return ReadRefusal::Missing;
    }
    #[cfg(unix)]
    {
        if error.raw_os_error() == Some(libc::ENOTDIR) {
            return ReadRefusal::Missing;
        }
        if error.raw_os_error() == Some(libc::ELOOP) {
            return ReadRefusal::NotRegular;
        }
    }
    return ReadRefusal::Io(error);
}
