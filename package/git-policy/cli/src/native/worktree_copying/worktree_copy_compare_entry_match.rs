//! What:
//!  The comparison of one manifest entry against the filesystem:
//!  its kind,
//!  its retained mode,
//!  its symbolic-link target text or its regular-file bytes,
//!  and the bounded read a byte comparison makes.
//! Why:
//!  A destination must receive exactly what was staged from the source,
//!  so one entry is compared whole,
//!  and an entry a build replaced with something a regular file cannot equal answers as a difference,
//!  never as a wait and never as a failure of the add.
//!  The final check that walks both trees is in `worktree_copy_compare.rs`,
//!  which re-exports what a caller read here before the split.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await entryMatchesStamped({ expectedRoot, actualRoot, entry, stamp });
//! ```

use crate::worktree_copy_error::{
    WorktreeCopyError,
    raw_failure,
    raw_failure_of,
};
use crate::worktree_copy_manifest::{
    EntryKind,
    FileStamp,
    WorktreeCopyEntry,
    retained_mode,
    stamp_of,
};
use crate::worktree_copy_selection::filesystem_path;
use crate::worktree_copy_symlink_target::names_the_same_place;
use std::io::Read;
use std::path::{
    Path,
    PathBuf,
};

// The metadata question a comparison asks of both sides is the one every walk of a live checkout asks, so it has one
//  home (`worktree_copy_vanished_entry.rs`): a missing entry is an answer, and every other failure is not.
use crate::worktree_copy_vanished_entry::lstat_or_absent;

/// What:
///  The size of one comparison read.
/// Why:
///   Large files are compared in bounded chunks so memory does not grow with file size.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const COMPARE_BUFFER_BYTES = 64 * 1_024;
/// ```
///  Exempt from a citation under `LQM` (class:
///  buffer size):
///  it sizes one read buffer,
///  and the result of the comparison
///  does not depend on it.
///  Classed once in `doc/planning/cli-git-shipped-limits-audit.md`,
///  "Exempt limits";
///  the
///  coordinator's pick under the owner's delegation,
///  open to the owner's veto.
pub(crate) const COMPARE_BUFFER_BYTES: usize = 64 * 1_024;

/// What:
///  One side of a byte comparison,
///  opened without waiting and without following a link,
/// or nothing when the path holds
///  something a regular file cannot equal.
/// Why:
///   The comparison reads the source's entry after the walk recorded it,
/// and a name a build replaced
///  with a named pipe in between would hold the comparison on an open that waits for a writer that may never
///  come ("nothing should ever wait forever or indefinitely").
/// The kind is taken from the handle this opens,
///  so a pipe,
/// a device or a directory answers as no regular file at all,
/// which the caller reads as the
///  difference a settling pass retries and then drops,
/// never as a wait and never as a failure of the add.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const openedSide = (path) => { try { return openPlainFile(path); } catch (e) { if (e.code === 'EINVAL') return undefined; throw e; } };
/// ```
fn opened_regular_file(path: &Path) -> Result<Option<std::fs::File>, WorktreeCopyError> {
    return match crate::plain_file::open_plain_file(path) {
        Ok(file) => Ok(Some(file)),
        // The plain-file reader refuses a path that is not a plain file with `InvalidInput` and the words the person reads,
        //  and a link it may not follow with `ELOOP`; neither holds a regular file's bytes.
        Err(error)
            if error.kind() == std::io::ErrorKind::InvalidInput
                || error.raw_os_error() == Some(libc::ELOOP) =>
        {
            Ok(None)
        }
        Err(error) => Err(raw_failure(&error)),
    };
}

/// What:
///  Whether two regular files hold the same bytes,
/// compared in bounded chunks.
/// Why:
///   Sizes are compared first,
/// and a short read means the file changed under the comparison.
/// A side that is no longer a regular file holds none of the other side's bytes,
/// so the two differ.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function regularFileBytesEqual({ leftPath, rightPath }): Promise<boolean>;
/// ```
fn regular_file_bytes_equal(
    left_path: &Path,
    right_path: &Path,
) -> Result<bool, WorktreeCopyError> {
    let (Some(mut left), Some(mut right)) = (
        opened_regular_file(left_path)?,
        opened_regular_file(right_path)?,
    ) else {
        return Ok(false);
    };
    let left_size: u64 = left.metadata().map_err(raw_failure_of)?.len();
    let right_size: u64 = right.metadata().map_err(raw_failure_of)?.len();
    if left_size != right_size {
        return Ok(false);
    }
    let mut left_buffer: Vec<u8> = vec![0; COMPARE_BUFFER_BYTES];
    let mut right_buffer: Vec<u8> = vec![0; COMPARE_BUFFER_BYTES];
    let mut remaining: u64 = left_size;
    while remaining > 0 {
        let length: usize = match usize::try_from(remaining) {
            Ok(size) => size.min(COMPARE_BUFFER_BYTES),
            Err(_too_large) => COMPARE_BUFFER_BYTES,
        };
        let left_chunk: &mut [u8] = &mut left_buffer[..length];
        let right_chunk: &mut [u8] = &mut right_buffer[..length];
        // `read_exact` reports a file that ended early as `UnexpectedEof`, which is a difference, not a failure.
        match (left.read_exact(left_chunk), right.read_exact(right_chunk)) {
            (Ok(()), Ok(())) => {}
            (Err(error), _) | (_, Err(error))
                if error.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                return Ok(false);
            }
            (Err(error), _) | (_, Err(error)) => {
                return Err(raw_failure(&error));
            }
        }
        if left_chunk != right_chunk {
            return Ok(false);
        }
        remaining -= u64::try_from(length).unwrap_or(0);
    }
    return Ok(true);
}

/// What:
///  The symbolic-link target text of one no-follow entry.
/// Why:
///   The target is read as the link holds it,
/// never resolved,
/// so a link whose target does not exist can still be compared.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readlink(path)
/// ```
fn link_target(path: &Path) -> Result<PathBuf, WorktreeCopyError> {
    return std::fs::read_link(path).map_err(raw_failure_of);
}

/// What:
///  Whether one actual entry matches an expected entry exactly.
/// Why:
///   Kind,
/// mode and content must all match,
/// and an absent entry never matches.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function entryMatches({ expectedRoot, actualRoot, entry }): Promise<boolean>;
/// ```
pub fn entry_matches(
    expected_root: &Path,
    actual_root: &Path,
    entry: &WorktreeCopyEntry,
) -> Result<bool, WorktreeCopyError> {
    return entry_matches_stamped(expected_root, actual_root, entry, None);
}

/// What:
///  Whether one actual entry matches an expected entry,
/// taking an unchanged stamp of the actual file as proof of its bytes.
/// Why:
///   A file the stage cloned holds the very blocks the source held at the clone;
/// when the source's stamp is still the one
///  recorded around the clone,
/// no write reached it since,
/// so reading both files again could only repeat the clone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function entryMatchesStamped({ expectedRoot, actualRoot, entry, stamp }): Promise<boolean>;
/// ```
pub fn entry_matches_stamped(
    expected_root: &Path,
    actual_root: &Path,
    entry: &WorktreeCopyEntry,
    stamp: Option<&FileStamp>,
) -> Result<bool, WorktreeCopyError> {
    let expected_path: PathBuf = filesystem_path(expected_root, entry.relative_path.as_slice())?;
    let actual_path: PathBuf = filesystem_path(actual_root, entry.relative_path.as_slice())?;
    let Some(actual) = lstat_or_absent(actual_path.as_path())? else {
        return Ok(false);
    };
    if retained_mode(&actual) != entry.mode {
        return Ok(false);
    }
    // `match` on the entry kind selects the one comparison that applies to it.
    match entry.kind {
        EntryKind::Directory => return Ok(actual.is_dir()),
        EntryKind::File => {
            if !actual.is_file() {
                return Ok(false);
            }
            if let Some(recorded) = stamp {
                return Ok(stamp_of(&actual).as_ref() == Some(recorded));
            }
            return regular_file_bytes_equal(expected_path.as_path(), actual_path.as_path());
        }
        EntryKind::Symlink => {
            if !actual.file_type().is_symlink() {
                return Ok(false);
            }
            // The two links stand in two trees at two depths, so their texts are compared by the place each names, never as
            //  text: a copy of a link that leaves its tree had to be written as an absolute path.
            return Ok(names_the_same_place(
                &link_target(expected_path.as_path())?,
                expected_path.as_path(),
                expected_root,
                &link_target(actual_path.as_path())?,
                actual_path.as_path(),
                actual_root,
            ));
        }
    }
}
