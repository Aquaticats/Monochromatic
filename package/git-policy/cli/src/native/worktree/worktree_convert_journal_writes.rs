//! What:
//!  Writing one conversion's journal into a file of its own,
//!  reading it back,
//!  and sweeping the temporary file a killed write leaves.
//! Why:
//!  The journal is written through exclusively created files and never replaces a file it did not make:
//!  a temporary name this process and the clock make,
//!  a link or a rename into place,
//!  and the directory made durable afterwards.
//!  A conversion killed between its write and its rename leaves that temporary file behind,
//!  so every write sweeps the leftovers first,
//!  and only those whose maker is proven gone.
//!  This file is one part of the module whose name it carries,
//!  split because that module was over the line limit:
//!  `worktree_convert_journal_text.rs` holds the journal's shape and its text,
//!  and `worktree_convert_journal.rs` holds the claim a conversion takes and the kernel lock an older
//!  release left.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await journal.save(admin);        // exclusive temporary, then rename
//! await journal.create_new(admin);  // exclusive temporary, then a link that fails when one exists
//! const loaded = Journal.load(admin);
//! ```

use crate::diagnostic_log::debug;
use crate::process_identity::process_birth_identity;
use crate::subvolume_error::MigrateError;
use crate::worktree_convert_journal_text::{
    JOURNAL_NAME,
    Journal,
};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{
    Path,
    PathBuf,
};

/// The prefix of the temporary file each journal write makes,
/// followed by the number of the process that made it and the clock reading.
pub const TEMPORARY_PREFIX: &str = "cli-git-convert.tmp.";

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeConvertJournal";

/// What:
///  Make the directory's own entry durable by syncing a handle on it.
/// Why:
///   A rename or a link is only as durable as the directory that holds it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await (await open(admin)).sync()
/// ```
fn sync_directory_of(admin: &Path) -> std::io::Result<()> {
    return crate::subvolume_file_write_steps::sync_directory_to_disk(admin);
}

/// What:
///  An input or output outcome with its failure turned into a conversion error that names the action and the path.
/// Why:
///   Many steps of the conversion report the same way,
/// and a named function replaces a closure at each.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ioStep<T>(outcome: () => T, action: string, path: string): T { try { return outcome(); } catch (error) { throw migrateIo(action, path, error); } }
/// ```
pub fn io_step<T>(
    outcome: std::io::Result<T>,
    action: &str,
    path: &Path,
) -> Result<T, MigrateError> {
    return match outcome {
        Ok(value) => Ok(value),
        Err(error) => Err(MigrateError::io(action, path, &error)),
    };
}

/// A name for a temporary file that no other file has:
/// this process and the clock make it,
/// and the creation is exclusive.
fn temporary_name() -> String {
    let nanos: u128 = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(elapsed) => elapsed.as_nanos(),
        Err(_before_epoch) => 0,
    };
    return format!("{TEMPORARY_PREFIX}{}.{nanos}", std::process::id());
}

/// What:
///  Whether one field of a temporary file's name is one or more ASCII digits and nothing else.
/// Why:
///  The process number and the clock reading are written as decimal text,
/// and a name with anything else there is no temporary file of this module.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isDecimalField(field: string): boolean;
/// ```
fn is_decimal_field(field: &str) -> bool {
    if field.is_empty() {
        return false;
    }
    for byte in field.bytes() {
        if !byte.is_ascii_digit() {
            return false;
        }
    }
    return true;
}

/// What:
///  The process number in the name of a temporary journal file,
///  nothing for every other name.
/// Why:
///  Only the files a journal write makes carry it,
/// and the sweep may remove nothing else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function makerOf(name: string): number | undefined;
/// ```
fn maker_of(name: &str) -> Option<u32> {
    // `?` returns `None` from this function for a name without the prefix.
    let rest: &str = name.strip_prefix(TEMPORARY_PREFIX)?;
    let (pid, nanos): (&str, &str) = rest.split_once('.')?;
    // Both fields are decimal digits and nothing else, as `temporary_name` writes them.
    if !is_decimal_field(pid) || !is_decimal_field(nanos) {
        return None;
    }
    return pid.parse().ok();
}

/// What:
///  Removes the temporary files of journal writes whose maker is proven gone,
///  and says how many it removed.
/// Why:
///  Each write makes `cli-git-convert.tmp.<pid>.<nanos>` and removes it only when its own rename or link fails,
///  so a conversion killed in between leaves one that nothing else cleans.
///  A file stays unless its maker is gone:
/// the name carries only a process number,
/// so a number that still runs,
///  a record this host refuses to read,
/// or a number a later process got keeps the file,
///  which costs one
///  file rather than someone else's write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sweepTemporaryFiles(admin: string): number;
/// ```
pub fn sweep_temporary_files(admin: &Path) -> usize {
    // Nothing to sweep when the directory cannot be read; the journal write that follows reports the real failure.
    let Ok(entries) = std::fs::read_dir(admin) else {
        return 0;
    };
    let mut removed: usize = 0;
    for entry in entries.flatten() {
        // A name that is not text under a home this command wrote is no temporary file of its own.
        let Ok(name) = entry.file_name().into_string() else {
            continue;
        };
        let Some(pid) = maker_of(name.as_str()) else {
            continue;
        };
        // `Ok(None)` is the one answer that proves no process of this number runs; anything else proves nothing.
        if !matches!(process_birth_identity(i64::from(pid)), Ok(None)) {
            continue;
        }
        if std::fs::remove_file(entry.path()).is_ok() {
            removed += 1;
        }
    }
    return removed;
}

impl Journal {
    /// Write the journal into a file made for it alone:
    /// created exclusively,
    /// so nothing that is there is truncated or followed.
    fn write_temporary(
        &self,
        admin: &Path,
    ) -> std::io::Result<PathBuf> {
        // A conversion killed between its write and its rename leaves the temporary file behind; every write sweeps the
        // leftovers of its own worktree first, and only those whose maker is gone.
        let swept: usize = sweep_temporary_files(admin);
        if swept > 0 {
            debug(
                TAG,
                format!(
                    "removed {swept} temporary journal file(s) left by a conversion that is gone"
                )
                .as_str(),
            );
        }
        let temporary: PathBuf = admin.join(temporary_name());
        let mut file: std::fs::File = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&temporary)?;
        let written: std::io::Result<()> = crate::subvolume_file_write_steps::write_all_then_sync(
            &mut file,
            self.encode().as_bytes(),
        );
        if let Err(error) = written {
            let _removed: std::io::Result<()> = std::fs::remove_file(&temporary);
            return Err(error);
        }
        return Ok(temporary);
    }

    /// Replace the journal with this one:
    /// written beside it,
    /// synchronized,
    /// renamed into place.
    pub fn save(
        &self,
        admin: &Path,
    ) -> Result<(), MigrateError> {
        let target: PathBuf = admin.join(JOURNAL_NAME);
        return io_step(
            self.write_and_publish(admin, target.as_path()),
            "write the conversion journal",
            target.as_path(),
        );
    }

    /// Write the journal to a temporary file,
    /// rename it over `target` and make the directory durable.
    fn write_and_publish(
        &self,
        admin: &Path,
        target: &Path,
    ) -> std::io::Result<()> {
        let temporary: PathBuf = self.write_temporary(admin)?;
        if let Err(error) = std::fs::rename(&temporary, target) {
            let _removed: std::io::Result<()> = std::fs::remove_file(&temporary);
            return Err(error);
        }
        return sync_directory_of(admin);
    }

    /// Make this the journal of a conversion that has none:
    /// it fails when any journal exists,
    /// so a journal another command
    ///  made is never replaced.
    pub fn create_new(
        &self,
        admin: &Path,
    ) -> Result<(), MigrateError> {
        let target: PathBuf = admin.join(JOURNAL_NAME);
        return io_step(
            self.write_and_link(admin, target.as_path()),
            "create the conversion journal",
            target.as_path(),
        );
    }

    /// Write the journal to a temporary file,
    /// link it at `target` only if nothing is there and make the directory durable.
    fn write_and_link(
        &self,
        admin: &Path,
        target: &Path,
    ) -> std::io::Result<()> {
        let temporary: PathBuf = self.write_temporary(admin)?;
        let linked: std::io::Result<()> = std::fs::hard_link(&temporary, target);
        let _removed: std::io::Result<()> = std::fs::remove_file(&temporary);
        linked?;
        return sync_directory_of(admin);
    }

    /// The journal of an administrative directory,
    /// if one exists.
    pub fn load(admin: &Path) -> Result<Option<Journal>, MigrateError> {
        let path: PathBuf = admin.join(JOURNAL_NAME);
        match crate::plain_file::read_plain_file(
            path.as_path(),
            crate::plain_file::SMALL_FILE_LIMIT,
        ) {
            Ok(bytes) => {
                return Journal::decode(String::from_utf8_lossy(bytes.as_slice()).as_ref())
                    .map(Some);
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => {
                return Err(MigrateError::io(
                    "read the conversion journal",
                    path.as_path(),
                    &error,
                ));
            }
        }
    }
}
