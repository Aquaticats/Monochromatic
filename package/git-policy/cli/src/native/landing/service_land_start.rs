//! What:
//!  The witness of the live index file as the landing found it under the index lock,
//! recorded in the journal,
//! and the comparison
//!  a replay makes with it.
//! Why:
//!  An index update that never installed is replayed by a restart after the lock was given up,
//! and another session may have
//!  staged into the index since.
//! What the index holds at a path cannot tell a path nobody staged from a path staged back to the
//!  same entry (stage another version,
//! then the old one again):
//! both read the same.
//! The file itself can tell.
//! Every program
//!  that changes an index writes a new file and renames it over the old one,
//! so a file that is still the one the landing saw
//!  (same inode,
//! size,
//! change time,
//! modification time and checksum) has been written by nobody since,
//! and a replay over it
//!  discards nothing.
//! Any other file might hold staging,
//! so the replay is not made over it
//!  (`doc/planning/cli-git-main-commit-within-native-plus-five.md`,
//! "A replay only over the index the landing saw").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const same = sameFile(await witnessOf(index), recordedWitness) // true: nobody wrote the index since the landing saw it
//! ```

use std::io::{Read as _, Seek as _, SeekFrom};
use std::os::unix::fs::MetadataExt as _;
use std::path::Path;

/// How many bytes at the end of the index file the witness keeps:
/// the checksum of either object format (SHA-1 and SHA-256),
///  which the file ends with.
const TRAILER_BYTES: u64 = 32;

/// What the landing saw of the live index file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IndexWitness {
    /// There was no index file.
    Absent,
    /// There was this file.
    File {
        /// The inode number.
        inode: u64,
        /// The size in bytes.
        size: u64,
        /// The modification time,
        /// seconds and nanoseconds.
        modified: (i64, i64),
        /// The time of the last change of the inode,
        /// seconds and nanoseconds;
        /// a rename onto the path moves it.
        changed: (i64, i64),
        /// The last bytes of the file,
        /// in hexadecimal.
        trailer: String,
    },
}

/// The hexadecimal text of `bytes`,
/// from the crate's one encoder of bytes as hexadecimal digits (`hexadecimal_of_bytes.rs`).
fn hex(bytes: &[u8]) -> String {
    return crate::hexadecimal_of_bytes::hexadecimal_of_bytes(bytes);
}

/// What:
///  The witness of the index file at `index`.
/// Why:
///   The landing records it while it holds the index lock and a replay reads it again under the lock it takes,
/// so the two are
///  compared by one function.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const witness = await witnessOf(indexPath) // 'absent' when there is no file
/// ```
///
/// # Errors
///
/// The error of the open,
/// the metadata or the read;
/// a file that is not there is `Absent`,
/// not an error.
pub fn witness_of(index: &Path) -> std::io::Result<IndexWitness> {
    let mut file: std::fs::File = match std::fs::File::open(index) {
        Ok(opened) => opened,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(IndexWitness::Absent),
        Err(error) => return Err(error),
    };
    // The metadata of the open file, so the inode and the end of the file are of one file even if the path is replaced meanwhile.
    let metadata: std::fs::Metadata = file.metadata()?;
    let size: u64 = metadata.len();
    let kept: u64 = size.min(TRAILER_BYTES);
    file.seek(SeekFrom::Start(size - kept))?;
    let mut trailer: Vec<u8> = vec![0; kept as usize];
    file.read_exact(trailer.as_mut_slice())?;
    return Ok(IndexWitness::File {
        inode: metadata.ino(),
        size,
        modified: (metadata.mtime(), metadata.mtime_nsec()),
        changed: (metadata.ctime(), metadata.ctime_nsec()),
        trailer: hex(trailer.as_slice()),
    });
}

/// The text of a witness:
/// `absent`,
/// or `file` and its fields separated by colons.
pub fn encode(witness: &IndexWitness) -> String {
    return match witness {
        IndexWitness::Absent => String::from("absent"),
        IndexWitness::File { inode, size, modified, changed, trailer } => {
            format!("file:{inode}:{size}:{}:{}:{}:{}:{trailer}", modified.0, modified.1, changed.0, changed.1)
        }
    };
}

/// The witness a text spells,
/// or nothing when it is damaged.
pub fn decode(text: &str) -> Option<IndexWitness> {
    if text == "absent" {
        return Some(IndexWitness::Absent);
    }
    let mut fields = text.split(':');
    if fields.next()? != "file" {
        return None;
    }
    let inode: u64 = fields.next()?.parse().ok()?;
    let size: u64 = fields.next()?.parse().ok()?;
    let modified: (i64, i64) = (fields.next()?.parse().ok()?, fields.next()?.parse().ok()?);
    let changed: (i64, i64) = (fields.next()?.parse().ok()?, fields.next()?.parse().ok()?);
    let trailer: String = fields.next()?.to_string();
    if fields.next().is_some() || !trailer.as_bytes().iter().all(u8::is_ascii_hexdigit) {
        return None;
    }
    return Some(IndexWitness::File { inode, size, modified, changed, trailer });
}

#[cfg(test)]
#[path = "service_land_start_tests.rs"]
mod tests;
