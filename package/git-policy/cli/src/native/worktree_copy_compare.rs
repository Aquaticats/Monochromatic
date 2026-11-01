//! What:
//!  Exact comparison of one manifest entry against the filesystem:
//!  its kind, its retained mode, its symbolic-link target text or its regular-file bytes.
//!  Also the final check that the source still matches the stage after staging.
//! Why:
//!  A destination must receive exactly what was staged from the source,
//!  and a source that changed during the transfer must block the destination.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await assertFinalSourceEquivalence({ sourceRoot, stageRoot, selectedRoots, excludedSourceRoots, stagedEntries });
//! ```

use super::worktree_copy_error::{WorktreeCopyError, failure_with_cause, raw_failure};
use super::worktree_copy_manifest::{
    EntryKind, WorktreeCopyEntry, collect_entry_manifest, retained_mode,
};
use super::worktree_copy_selection::filesystem_path;
use crate::json_record::quote;
use std::io::Read;
use std::path::{Path, PathBuf};

/// What:
///  The size of one comparison read.
/// Why:
///   Large files are compared in bounded chunks so memory does not grow with file size.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const COMPARE_BUFFER_BYTES = 64 * 1_024;
/// ```
const COMPARE_BUFFER_BYTES: usize = 64 * 1_024;

/// What:
///  The metadata of one no-follow entry, or its absence.
/// Why:
///   A missing destination entry is a normal answer for the comparison, not a failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await lstatOrAbsent(path)
/// ```
pub fn lstat_or_absent(path: &Path) -> Result<Option<std::fs::Metadata>, WorktreeCopyError> {
    // `match` separates "no entry" (not an error) from every other failure.
    match std::fs::symlink_metadata(path) {
        Ok(metadata) => return Ok(Some(metadata)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(raw_failure(&error)),
    }
}

/// What:
///  Whether two regular files hold the same bytes, compared in bounded chunks.
/// Why:
///   Sizes are compared first, and a short read means the file changed under the comparison.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function regularFileBytesEqual({ leftPath, rightPath }): Promise<boolean>;
/// ```
fn regular_file_bytes_equal(
    left_path: &Path,
    right_path: &Path,
) -> Result<bool, WorktreeCopyError> {
    let mut left: std::fs::File =
        std::fs::File::open(left_path).map_err(|error| return raw_failure(&error))?;
    let mut right: std::fs::File =
        std::fs::File::open(right_path).map_err(|error| return raw_failure(&error))?;
    let left_size: u64 = left
        .metadata()
        .map_err(|error| return raw_failure(&error))?
        .len();
    let right_size: u64 = right
        .metadata()
        .map_err(|error| return raw_failure(&error))?
        .len();
    if left_size != right_size {
        return Ok(false);
    }
    let mut left_buffer: Vec<u8> = vec![0; COMPARE_BUFFER_BYTES];
    let mut right_buffer: Vec<u8> = vec![0; COMPARE_BUFFER_BYTES];
    let mut remaining: u64 = left_size;
    while remaining > 0 {
        let length: usize = usize::try_from(remaining).map_or(COMPARE_BUFFER_BYTES, |size| {
            return size.min(COMPARE_BUFFER_BYTES);
        });
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
///   The target is compared as exact text, never resolved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readlink(path)
/// ```
fn link_target(path: &Path) -> Result<PathBuf, WorktreeCopyError> {
    return std::fs::read_link(path).map_err(|error| return raw_failure(&error));
}

/// What:
///  Whether one actual entry matches an expected entry exactly.
/// Why:
///   Kind, mode and content must all match, and an absent entry never matches.
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
    let expected_path: PathBuf = filesystem_path(expected_root, entry.relative_path.as_str())?;
    let actual_path: PathBuf = filesystem_path(actual_root, entry.relative_path.as_str())?;
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
            return regular_file_bytes_equal(expected_path.as_path(), actual_path.as_path());
        }
        EntryKind::Symlink => {
            if !actual.file_type().is_symlink() {
                return Ok(false);
            }
            return Ok(link_target(expected_path.as_path())? == link_target(actual_path.as_path())?);
        }
    }
}

/// What:
///  Checks that the source still matches the initial manifest after staging,
///  and that the stage holds exactly that manifest.
/// Why:
///   Any difference means the source changed during the copy, so the destination must not receive the stage.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await assertFinalSourceEquivalence({ sourceRoot, stageRoot, selectedRoots, excludedSourceRoots, stagedEntries });
/// ```
pub fn assert_final_source_equivalence(
    source_root: &Path,
    stage_root: &Path,
    selected_roots: &[String],
    excluded_source_roots: &[PathBuf],
    staged_entries: &[WorktreeCopyEntry],
) -> Result<(), WorktreeCopyError> {
    let final_source = collect_entry_manifest(source_root, selected_roots, excluded_source_roots)?;
    let final_stage = collect_entry_manifest(stage_root, selected_roots, &[])?;
    if final_source.len() != staged_entries.len() || final_stage.len() != staged_entries.len() {
        return Err(failure_with_cause(
            "cli-git: ignored source paths changed while snapshot was staged.",
            None,
        ));
    }
    for (index, staged) in staged_entries.iter().enumerate() {
        let source_entry: &WorktreeCopyEntry = &final_source[index];
        let stage_entry: &WorktreeCopyEntry = &final_stage[index];
        let same_shape: bool = source_entry.kind == staged.kind
            && source_entry.mode == staged.mode
            && source_entry.relative_path == staged.relative_path
            && stage_entry.kind == staged.kind
            && stage_entry.mode == staged.mode
            && stage_entry.relative_path == staged.relative_path;
        // The content check runs last so the cheap shape checks fail fast without reading large files.
        if !same_shape || !entry_matches(stage_root, source_root, staged)? {
            let message: String = format!(
                "cli-git: ignored source entry changed while copying: {}.",
                quote(staged.relative_path.as_str())
            );
            return Err(failure_with_cause(message.as_str(), None));
        }
    }
    return Ok(());
}

/// Compare controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_compare_tests.rs"]
mod tests;
