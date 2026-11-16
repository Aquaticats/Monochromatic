//! What:
//!  The reading of every entry of the old tree that the migration's last pass over it took,
//!  kept beside the checkout,
//!  and the question the later deletion asks of it:
//!  does this entry still read as it did then?
//! Why:
//!  The old tree is the tree the checkout was before the migration,
//!  and the migration carried all of it into the checkout.
//!  When the person works in the checkout afterwards,
//!  the checkout moves on and the old tree does not,
//!  so comparing the two trees no longer answers the question the deletion has to answer.
//!  That question is:  has anything been written into the old tree since this program last read it?
//!  A write moves the change time, which no writer can set,
//!  so a reading of the device, the inode, the size, the kind, the modification time and the change time answers it.
//!  An entry that still reads the same was carried when the migration read it and nothing has touched it since;
//!  its bytes are in the checkout's history or in the person's later work,
//!  and deleting it discards nothing.
//!  An entry that reads differently was written into the old tree after the migration,
//!  and its bytes have to be kept or named.
//!  A modification time alone is not that reading:
//!  `touch`, `cp -p`, `tar -x` and `rsync -a` set one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const record = OldTreeRecord.read(layout.oldTreeRecordPath); // Map<path, Reading>
//! const same = record?.unchanged(relative, statSync(oldTree + '/' + relative));
//! ```

use crate::subvolume_error::MigrateError;
use crate::subvolume_file_write_steps::write_beside_then_rename;
use std::collections::BTreeMap;
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::path::{Path, PathBuf};

/// The kind of entry a reading describes,
/// one letter each in the record.
const KIND_OF_FILE: char = 'f';

/// A symbolic link.
const KIND_OF_SYMLINK: char = 'l';

/// A named pipe.
const KIND_OF_FIFO: char = 'p';

/// A socket.
const KIND_OF_SOCKET: char = 's';

/// A device node or anything else.
const KIND_OF_OTHER: char = 'o';

/// What:
///  Everything about one entry that a writer cannot change without changing the reading.
/// Why:
///   The change time is the field no writer can set,
///  so a write in place moves this reading even when it keeps the size and puts the modification time back.
///  The device and the inode tell a replaced entry from a written one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Reading = { kind: string; dev: bigint; ino: bigint; size: bigint; mtime: [number, number]; ctime: [number, number] };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OldTreeReading {
    /// The kind of entry, as the record spells it.
    pub(crate) kind: char,
    /// The device.
    pub(crate) dev: u64,
    /// The inode.
    pub(crate) ino: u64,
    /// The size in bytes.
    pub(crate) size: u64,
    /// The modification time, seconds and nanoseconds.
    pub(crate) mtime: (i64, i64),
    /// The change time, seconds and nanoseconds.
    pub(crate) ctime: (i64, i64),
}

impl OldTreeReading {
    /// The reading of an entry,
    /// or none for a directory,
    /// which is judged by its children and never by its own times.
    pub fn of(metadata: &std::fs::Metadata) -> Option<OldTreeReading> {
        let kind: std::fs::FileType = metadata.file_type();
        let spelled: char = if kind.is_file() {
            KIND_OF_FILE
        } else if kind.is_symlink() {
            KIND_OF_SYMLINK
        } else if kind.is_fifo() {
            KIND_OF_FIFO
        } else if kind.is_socket() {
            KIND_OF_SOCKET
        } else if kind.is_dir() {
            return None;
        } else {
            KIND_OF_OTHER
        };
        return Some(OldTreeReading {
            kind: spelled,
            dev: metadata.dev(),
            ino: metadata.ino(),
            size: metadata.len(),
            mtime: (metadata.mtime(), metadata.mtime_nsec()),
            ctime: (metadata.ctime(), metadata.ctime_nsec()),
        });
    }

    /// Whether an entry read now is the entry this reading describes,
    /// unchanged.
    pub fn still_reads(&self, now: &std::fs::Metadata) -> bool {
        return OldTreeReading::of(now).is_some_and(|found| return found == *self);
    }

    /// The reading as one line of the record.
    fn spelled(&self, relative: &Path) -> String {
        return format!(
            "{} {} {} {} {} {} {} {}\t{}\n",
            self.kind,
            self.dev,
            self.ino,
            self.size,
            self.mtime.0,
            self.mtime.1,
            self.ctime.0,
            self.ctime.1,
            relative.to_string_lossy()
        );
    }

    /// The reading a line of the record spells,
    /// and the path it names.
    fn from_spelled(line: &str) -> Option<(PathBuf, OldTreeReading)> {
        let (fields, relative) = line.split_once('\t')?;
        if relative.is_empty() {
            return None;
        }
        let mut parts = fields.split(' ');
        let kind: char = parts.next()?.chars().next()?;
        let dev: u64 = parts.next()?.parse().ok()?;
        let ino: u64 = parts.next()?.parse().ok()?;
        let size: u64 = parts.next()?.parse().ok()?;
        let mtime_seconds: i64 = parts.next()?.parse().ok()?;
        let mtime_nanos: i64 = parts.next()?.parse().ok()?;
        let ctime_seconds: i64 = parts.next()?.parse().ok()?;
        let ctime_nanos: i64 = parts.next()?.parse().ok()?;
        return Some((PathBuf::from(relative), OldTreeReading { kind, dev, ino, size, mtime: (mtime_seconds, mtime_nanos), ctime: (ctime_seconds, ctime_nanos) }));
    }
}

/// What:
///  What the old tree read like when the migration last read it,
///  by path.
/// Why:
///   See the module description.
///  A path the record does not name is an entry written into the old tree after the migration,
///  which is read the same way as one whose reading moved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class OldTreeRecord { readings: Map<string, Reading>; unchanged(relative, metadata): boolean }
/// ```
pub struct OldTreeRecord {
    /// The reading of every non-directory entry the migration read.
    readings: BTreeMap<PathBuf, OldTreeReading>,
}

impl OldTreeRecord {
    /// The record of the readings one pass of the migration took.
    pub fn of(readings: BTreeMap<PathBuf, OldTreeReading>) -> OldTreeRecord {
        return OldTreeRecord { readings };
    }

    /// What:
    ///  The record the file at `path` holds,
    ///  or none when there is no file there.
    /// Why:
    ///   A checkout migrated by an earlier version of this command has no record,
    ///  and the deletion has to say so rather than take a silence for a proof.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const record = await OldTreeRecord.read(path); // undefined when the file is absent
    /// ```
    pub fn read(path: &Path) -> std::io::Result<Option<OldTreeRecord>> {
        let file: std::fs::File = match crate::plain_file::open_plain_file(path) {
            Ok(opened) => opened,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error),
        };
        let mut reader: std::io::BufReader<std::fs::File> = std::io::BufReader::new(file);
        let mut line: Vec<u8> = Vec::new();
        let mut readings: BTreeMap<PathBuf, OldTreeReading> = BTreeMap::new();
        loop {
            line.clear();
            let read: usize = std::io::BufRead::read_until(&mut reader, b'\n', &mut line)?;
            if read == 0 {
                break;
            }
            if line.last() == Some(&b'\n') {
                line.pop();
            }
            let text: std::borrow::Cow<'_, str> = String::from_utf8_lossy(line.as_slice());
            if let Some((relative, reading)) = OldTreeReading::from_spelled(text.as_ref()) {
                readings.insert(relative, reading);
            }
        }
        return Ok(Some(OldTreeRecord { readings }));
    }

    /// Write the record at `path`,
    /// with one durability sync.
    /// It is written once,
    /// when the migration's last pass over the old tree has finished.
    pub fn write(&self, path: &Path) -> Result<(), MigrateError> {
        let mut text: String = String::new();
        for (relative, reading) in &self.readings {
            text.push_str(reading.spelled(relative).as_str());
        }
        return MigrateError::from_io_result(write_beside_then_rename(path, text.as_bytes()), "write the reading of the old tree", path);
    }

    /// Whether the entry at `relative` reads now as it did when the migration read it.
    pub fn unchanged(&self, relative: &Path, now: &std::fs::Metadata) -> bool {
        return match self.readings.get(relative) {
            Some(reading) => reading.still_reads(now),
            None => false,
        };
    }

}

/// The controls of the record stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_old_tree_record_tests.rs"]
mod tests;
