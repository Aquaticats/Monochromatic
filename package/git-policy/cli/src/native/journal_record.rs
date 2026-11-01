//! What:
//!  One journal record written whole, followed by exactly the syncs its durability level names.
//! Why:
//!  Recovery reads each record of a transaction directory, and the tearing table in
//!  `doc/planning/cli-git-spawn-and-sync-reduction.md` shows which records a power loss may lose or tear without any
//!  change of outcome and which must reach the disk before the next step. Naming the level at the call site keeps
//!  every record's sync decision visible, and the budget tests pin the resulting counts.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await writeRecord({ directory, filename, bytes, sync: 'nothing' });
//! ```

/// `Path` is a borrowed filesystem path; `PathBuf` is the owned one.
use std::path::{Path, PathBuf};
/// Whole-or-absent record writes, with and without the sync.
use super::private_storage::{sync_directory, write_private_file, write_private_file_unsynced};

/// What:
///  Which syncs follow a journal record's write.
/// Why:
///   A record published by link is whole or absent after a kill whatever the level; the level only decides what a
///  power loss may leave.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RecordSync = 'file-and-directory' | 'file' | 'nothing';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecordSync {
    /// The file's bytes, then its directory entry: the record is a barrier that later steps rely on.
    FileAndDirectory,
    /// The file's bytes only: a later record's directory sync makes the entry durable before anything depends on it,
    ///  and a name a power loss took anyway leaves the state of a record that was never written.
    FileOnly,
    /// Neither: recovery completes or discards the transaction whatever a power loss left of this record.
    Nothing,
}

/// What:
///  Write `filename` in `directory` exclusively and whole, then sync as `sync` says.
///  Returns the failure as a sentence that names the path.
/// Why:
///   The callers report a failed write as a landing or transaction failure and differ only in the type that wraps
///  the sentence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeRecord({ directory, filename, bytes, sync });
/// ```
pub fn write_record(directory: &Path, filename: &str, bytes: &[u8], sync: RecordSync) -> Result<(), String> {
    let path: PathBuf = directory.join(filename);
    let written: std::io::Result<()> = match sync {
        RecordSync::FileAndDirectory | RecordSync::FileOnly => write_private_file(path.as_path(), bytes),
        RecordSync::Nothing => write_private_file_unsynced(path.as_path(), bytes),
    };
    if let Err(error) = written {
        return Err(format!("writing {} failed: {error}", path.display()));
    }
    if sync == RecordSync::FileAndDirectory
        && let Err(error) = sync_directory(directory)
    {
        return Err(format!("syncing {} failed: {error}", directory.display()));
    }
    return Ok(());
}

/// Journal record controls stay out of the release executable.
#[cfg(test)]
#[path = "journal_record_tests.rs"]
mod tests;
