//! What:
//!  Publishing one record of the linked-worktree copy whole:
//!  the name of the temporary its writer fills,
//!  the write into that temporary,
//!  and the one rename that gives it its final name.
//! Why:
//!  A crash can happen at any instruction,
//!  so a reader of the journal root must see a complete record or none:
//!  every record is written under a temporary beside its final name and renamed into place once whole,
//!  and the writer's owner is part of the temporary's name,
//!  so a sweep removes only a dead writer's temporaries and can never delete one a live writer is about to rename.
//!  What one journal says, and the phase it names, are in `worktree_copy_journal.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await writeFileAtomically({ path, bytes, subject });
//! ```

use crate::json_record::quote;
use crate::private_storage::{
    sync_directory,
    write_exclusive_scratch,
    write_exclusive_synced,
};
use crate::random_id::random_uuid;
use crate::worktree_copy_error::{
    WorktreeCopyError,
    failure_with_cause,
};
use crate::worktree_copy_journal_encoding_and_validation::TEMPORARY_SUFFIX;
use crate::worktree_copy_owner::{
    CopyOwner,
    current_copy_owner,
};
use std::path::{
    Path,
    PathBuf,
};
/// What:
///  The name of the temporary file a write of `path` by `owner` uses:
/// `<path>.<owner>.<unique>.tmp`.
/// Why:
///   The writer's owner is part of the temporary's name,
/// so a sweep removes only a dead writer's temporaries and can never
///  delete one a live writer is about to rename.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// temporaryPath('/j/abc.stage', owner, 'u1') === `/j/abc.stage.${owner.token()}.u1.tmp`;
/// ```
pub fn temporary_path(
    path: &Path,
    owner: &CopyOwner,
    unique: &str,
) -> PathBuf {
    // The suffix is appended to the path's own bytes, so a name that is not text keeps every byte of itself.
    let mut temporary: std::ffi::OsString = path.as_os_str().to_owned();
    temporary.push(format!(".{}.{unique}{TEMPORARY_SUFFIX}", owner.token()));
    return PathBuf::from(temporary);
}

/// What:
///  The writer's owner token in a temporary file's name,
/// or nothing for a name of another shape.
/// Why:
///   It is the inverse of [`temporary_path`] that the sweep of dead writers' temporaries reads.
///  A name of that shape has a token field,
///  so a stranger's name whose token field is empty names no writer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writerTokenOf('abc.stage.<owner>.u1.tmp') === '<owner>';
/// ```
pub fn temporary_writer_token(file_name: &str) -> Option<&str> {
    let stem: &str = file_name.strip_suffix(TEMPORARY_SUFFIX)?;
    let (rest, _unique) = stem.rsplit_once('.')?;
    let (_item, token) = rest.rsplit_once('.')?;
    // The writer fills each field, so an empty token field is a stranger's file that must name no writer.
    if token.is_empty() {
        return None;
    }
    return Some(token);
}

/// What:
///  Writes one private file atomically beside its final name;
/// a failure names `subject` and leaves no temporary.
/// Why:
///   Journals and stage claims share one rule:
/// a reader sees a complete file or none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeFileAtomically({ path, bytes, subject });
/// ```
pub fn write_atomic(
    path: &Path,
    bytes: &[u8],
    subject: &str,
) -> Result<(), WorktreeCopyError> {
    return write_atomic_flushing(path, bytes, subject, Flush::ToDisk);
}

/// Whether a record written by [`write_atomic_flushing`] is flushed to disk before the call returns.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Flush {
    /// The file and its directory are flushed:
    /// a crash finds the record.
    ToDisk,
    /// Neither is flushed:
    /// the file is published whole by the rename,
    /// and the filesystem makes it durable with its next commit.
    Never,
}

/// What:
///  Writes one private file atomically beside its final name without flushing the file or its directory to disk;
/// a failure
///  names `subject` and leaves no temporary.
/// Why:
///   A flush after a subvolume was swapped into place in the same btrfs transaction is a commit of the whole filesystem (the
///  kernel sets `btrfs_set_log_full_commit` for the swap),
/// which waits for every other writer's data.
/// A record that only
///  silences a notice is published whole by the rename and made durable by the commit the filesystem makes anyway.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeFileAtomicallyUnflushed({ path, bytes, subject });
/// ```
pub fn write_atomic_unsynced(
    path: &Path,
    bytes: &[u8],
    subject: &str,
) -> Result<(), WorktreeCopyError> {
    return write_atomic_flushing(path, bytes, subject, Flush::Never);
}

/// `write_atomic` and `write_atomic_unsynced`:
/// one rename publishes the whole file;
/// `flush` says whether the file and its
///  directory are flushed.
fn write_atomic_flushing(
    path: &Path,
    bytes: &[u8],
    subject: &str,
    flush: Flush,
) -> Result<(), WorktreeCopyError> {
    let unique: String = match random_uuid() {
        Ok(made) => made,
        Err(error) => {
            return Err(failure_with_cause(
                format!(
                    "monochromatic-git-policy-cli: could not persist worktree-copy {subject} {}.",
                    quote(path.to_string_lossy().as_ref())
                )
                .as_str(),
                Some(&error.0),
            ));
        }
    };
    let temporary: PathBuf = temporary_path(path, &current_copy_owner()?, unique.as_str());
    let written: std::io::Result<()> = match flush {
        Flush::ToDisk => write_exclusive_synced(temporary.as_path(), bytes),
        Flush::Never => write_exclusive_scratch(temporary.as_path(), bytes),
    };
    let persisted: std::io::Result<()> = match written {
        Ok(()) => publish_written(temporary.as_path(), path, flush),
        Err(error) => Err(error),
    };
    // A failed write removes its own temporary, so the directory keeps no partial journal.
    if let Err(error) = persisted {
        let _ignored_removal: std::io::Result<()> = std::fs::remove_file(temporary.as_path());
        return Err(failure_with_cause(
            format!(
                "monochromatic-git-policy-cli: could not persist worktree-copy {subject} {}.",
                quote(path.to_string_lossy().as_ref())
            )
            .as_str(),
            Some(&error),
        ));
    }
    return Ok(());
}

/// What:
///  Rename a written temporary file over `path`,
/// and make the directory durable when the flush mode asks.
/// Why:
///   The rename publishes the journal in one step,
/// and the sync makes the publication survive a crash.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// renameSync(temporary, path); if (flush === 'toDisk') syncDirectory(dirname(path));
/// ```
fn publish_written(
    temporary: &Path,
    path: &Path,
    flush: Flush,
) -> std::io::Result<()> {
    std::fs::rename(temporary, path)?;
    return match (flush, path.parent()) {
        (Flush::ToDisk, Some(parent)) => sync_directory(parent),
        _ => Ok(()),
    };
}
