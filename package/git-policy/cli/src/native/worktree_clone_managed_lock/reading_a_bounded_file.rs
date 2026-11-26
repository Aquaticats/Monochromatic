//! What:
//!  One bounded,
//!  no-follow read of a file this route does not own,
//!  as its bytes,
//!  as absence,
//!  or as the reason the read refused it.
//! Why:
//!  A lock's bytes and a slot's `.git` are both untrusted small files:
//!  a link planted at the name must be refused rather than followed,
//!  a named pipe must never be waited on,
//!  a runaway file must not fill memory,
//!  and the two readers name the same causes,
//!  so the cause is worded once
//!  (`doc/planning/cli-git-zfs-add-time-git-lock-design.md`,
//!  "How doctor and removal tell the states apart",
//!  and its limits `L5` and `L7`).
//!  The absence of a name is its own answer,
//!  because a missing lock and a lock the read refuses are different states of the add.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const reading = readABoundedFile(path, bound); // { kind: 'absent' } | { kind: 'bytes', bytes } | { kind: 'refused', cause }
//! ```

use crate::private_storage::{
    ReadRefusal,
    read_regular_file,
};
use std::os::unix::fs::FileTypeExt as _;
use std::path::Path;

/// What:
///  Why one file could not be read as the bytes it was expected to hold.
/// Why:
///  Each cause is a different sentence for a person,
///  and a removal must not release a lock it cannot attribute,
///  so nothing here is folded into one refusal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Cause = { kind: 'a-link' } | { kind: 'a-directory' } | { kind: 'a-named-pipe' } | { kind: 'another-kind' } | { kind: 'too-many-bytes', found, bound } | { kind: 'unreadable', reason };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ARefusedReading {
    /// A symbolic link stands at the name.
    ALink,
    /// A directory stands at the name.
    ADirectory,
    /// A named pipe stands at the name,
    /// which would wait inside the open for a writer that may never come.
    ANamedPipe,
    /// A socket,
    /// a device or another kind of entry stands at the name.
    AFileOfAnotherKind,
    /// The file holds more bytes than the reader reads.
    TooManyBytes {
        /// The bytes the file holds.
        found: u64,
        /// The bound the reader is given.
        bound: u64,
    },
    /// The system refused the read for another reason.
    Unreadable {
        /// What the call answered.
        reason: String,
    },
}

impl ARefusedReading {
    /// The words a person reads for one cause,
    /// as a clause a larger sentence holds.
    pub fn words(&self) -> String {
        return match self {
            ARefusedReading::ALink => String::from(
                "a symbolic link stands at the name and the file this route reads is a regular file",
            ),
            ARefusedReading::ADirectory => String::from(
                "a directory stands at the name and the file this route reads is a regular file",
            ),
            ARefusedReading::ANamedPipe => {
                String::from("a named pipe stands at the name, and this route never waits on one")
            }
            ARefusedReading::AFileOfAnotherKind => {
                String::from("a socket, a device or another non-regular entry stands at the name")
            }
            ARefusedReading::TooManyBytes {
                found,
                bound,
            } => {
                format!(
                    "it holds {found} bytes and this route reads at most {bound} bytes there, so the bytes it holds cannot be the ones this route wrote"
                )
            }
            ARefusedReading::Unreadable {
                reason,
            } => format!("the system refused the read ({reason})"),
        };
    }
}

/// What:
///  What one bounded read answered.
/// Why:
///  Absence,
///  bytes and a refusal are three different states of the record read,
///  and a caller that folded absence into a refusal would report a missing lock as a broken one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Reading = { kind: 'absent' } | { kind: 'bytes', bytes } | { kind: 'refused', cause: Cause };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Reading {
    /// Nothing stands at the name.
    Absent,
    /// The file holds these bytes.
    Bytes(Vec<u8>),
    /// The read refused the name.
    Refused(ARefusedReading),
}

/// What:
///  Read one regular file without following a link,
///  waiting on a writer or reading past `bound` bytes.
/// Why:
///  The metadata is read first so the cause names what stands at the name,
///  and the bytes are then read through the same rule every record of the wrapper is read
///  (`private_storage::read_regular_file`),
///  which caps a file that grew after its size was read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const metadata = await lstat(path); // ENOENT and ENOTDIR are 'absent'
/// if (metadata.isSymbolicLink()) return refused('a-link');
/// await readRegularFile(path, bound);
/// ```
pub fn read_a_bounded_file(
    path: &Path,
    bound: u64,
) -> Reading {
    let metadata: std::fs::Metadata = match std::fs::symlink_metadata(path) {
        Ok(found) => found,
        Err(error)
            if error.kind() == std::io::ErrorKind::NotFound
                || error.kind() == std::io::ErrorKind::NotADirectory =>
        {
            return Reading::Absent;
        }
        Err(error) => {
            return Reading::Refused(ARefusedReading::Unreadable {
                reason: error.to_string(),
            });
        }
    };
    let kind: std::fs::FileType = metadata.file_type();
    if kind.is_symlink() {
        return Reading::Refused(ARefusedReading::ALink);
    }
    if kind.is_dir() {
        return Reading::Refused(ARefusedReading::ADirectory);
    }
    if kind.is_fifo() {
        return Reading::Refused(ARefusedReading::ANamedPipe);
    }
    if !kind.is_file() {
        return Reading::Refused(ARefusedReading::AFileOfAnotherKind);
    }
    if metadata.len() > bound {
        return Reading::Refused(ARefusedReading::TooManyBytes {
            found: metadata.len(),
            bound,
        });
    }
    return match read_regular_file(path, bound) {
        Ok(bytes) => Reading::Bytes(bytes),
        Err(ReadRefusal::Missing) => Reading::Absent,
        Err(ReadRefusal::NotRegular) => Reading::Refused(ARefusedReading::AFileOfAnotherKind),
        Err(ReadRefusal::TooLarge) => Reading::Refused(ARefusedReading::TooManyBytes {
            found: metadata.len(),
            bound,
        }),
        Err(ReadRefusal::Io(error)) => Reading::Refused(ARefusedReading::Unreadable {
            reason: error.to_string(),
        }),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "reading_a_bounded_file_tests.rs"]
mod tests;
