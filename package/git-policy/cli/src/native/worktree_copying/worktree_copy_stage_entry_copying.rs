//! What:
//!  Copying one manifest entry into the stage:
//! a file's bytes,
//! a link's text written for its new place,
//! or the answer that the source no longer holds it.
//! Why:
//!  The walk and the copy are two passes over a live checkout,
//! so an entry a build removed or replaced between them holds none of what the listing promised,
//! and that answer is a skip rather than a failure of the copy.
//! A failure that is not that names the source path a person can act on,
//! never only the checkout it was copying from.
//! It lives beside the leaf copying because that file was over the line limit before this change
//! (`worktree_copy_stage_leaf_copying.rs`,
//! which re-exports it).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const leaf = copyOneLeaf({ sourceRoot, stagePath, entry }); // { kind: 'staged', stamp } | { kind: 'gone' }
//! ```

use crate::worktree_copy_error::{
    WorktreeCopyError,
    failure_with_cause,
    raw_failure_of,
};
use crate::worktree_copy_manifest::{
    EntryKind,
    FileStamp,
    WorktreeCopyEntry,
};
use crate::worktree_copy_selection::filesystem_path;
use crate::worktree_copy_stage_file_copying::copy_file_exclusive_with;
use crate::worktree_copy_symlink_target::target_for_the_new_place;
use crate::worktree_copy_vanished_entry::lstat_or_absent;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  The answer of copying one manifest entry into the stage.
/// Why:
///   A source entry that is no longer there has nothing to copy,
/// and a live checkout is constantly
///  removing and rewriting its ignored files,
/// so that answer is not a failure of the copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CopiedLeaf = { kind: 'staged'; stamp?: FileStamp } | { kind: 'gone' };
/// ```
pub enum CopiedLeaf {
    /// The entry is in the stage;
    /// the stamp proves the bytes when one was taken from a clone.
    Staged(Option<FileStamp>),
    /// The source no longer holds the entry.
    Gone,
}

/// What:
///  Copy one file or link of the manifest into `stage_path`,
/// and say whether the source still had it.
/// Why:
///   The walk and the copy are separate passes,
/// so a file a build removed between them is gone by now;
///  reading it is what discovers that,
/// and the answer is a skip rather than a failure.
/// An entry that changed kind in between is answered the same way:
/// it holds none of what the
///  listing promised,
/// so the entry the copy listed is no more part of this copy than one that went away.
/// A failure names the source path it
///  could not read,
/// never only the checkout it was copying from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function copyOneLeaf(sourceRoot, stagePath, entry): CopiedLeaf { try { return { kind: 'staged', stamp: copyFileExclusiveWith(...) }; } catch (e) { if (e.code === 'ENOENT' || stillHoldsTheListedKind(sourcePath, entry.kind) === false) return { kind: 'gone' }; throw `${sourcePath} cannot be copied (${e})`; } }
/// ```
pub fn copy_one_leaf(
    source_root: &Path,
    stage_path: &Path,
    entry: &WorktreeCopyEntry,
    clone: fn(&std::fs::File, &std::fs::File) -> bool,
) -> Result<CopiedLeaf, WorktreeCopyError> {
    let source_path: PathBuf = filesystem_path(source_root, entry.relative_path.as_slice())?;
    return match entry.kind {
        EntryKind::Directory => Ok(CopiedLeaf::Staged(None)),
        EntryKind::File => match copy_file_exclusive_with(source_path.as_path(), stage_path, clone)
        {
            Ok(stamp) => Ok(CopiedLeaf::Staged(stamp)),
            Err(_no_longer_the_listed_entry)
                if !still_holds_the_listed_kind(&source_path, EntryKind::File) =>
            {
                Ok(CopiedLeaf::Gone)
            }
            Err(error) => Err(named_source_failure(
                &source_path,
                "cannot be copied",
                &error,
            )),
        },
        EntryKind::Symlink => match std::fs::read_link(source_path.as_path()) {
            Ok(target) => {
                // The text is written for the place the link will occupy, never for the stage: a relative text counts levels
                //  from the directory the link sits in, and the stage stands elsewhere.
                let written: PathBuf =
                    target_for_the_new_place(target.as_path(), source_path.as_path(), source_root);
                symlink_at(written.as_path(), stage_path)?;
                Ok(CopiedLeaf::Staged(None))
            }
            Err(_no_longer_the_listed_entry)
                if !still_holds_the_listed_kind(&source_path, EntryKind::Symlink) =>
            {
                Ok(CopiedLeaf::Gone)
            }
            Err(error) => Err(named_source_failure(&source_path, "cannot be read", &error)),
        },
    };
}

/// What:
///  Whether the source still holds the entry the walk listed,
/// in the kind the walk recorded for it.
/// Why:
///   A file and the directory holding it can both go between the walk and the copy,
/// and only the
///  filesystem knows whether the failure was that,
/// so the question is asked again after the failure rather than guessed
///  from its text.
/// A kind that changed is the same answer as an entry that went away:
/// a named pipe where a
///  file was listed holds none of the bytes the listing promised,
/// and the copy stage's answer stays the same
///  whatever the entry became.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stillHoldsTheListedKind = (path, kind) => { try { const found = lstatSync(path); return kind === 'directory' ? found.isDirectory() : kind === 'file' ? found.isFile() : found.isSymbolicLink(); } catch { return false; } };
/// ```
fn still_holds_the_listed_kind(
    source_path: &Path,
    kind: EntryKind,
) -> bool {
    let Ok(Some(metadata)) = lstat_or_absent(source_path) else {
        return false;
    };
    return match kind {
        EntryKind::Directory => metadata.is_dir(),
        EntryKind::File => metadata.is_file(),
        EntryKind::Symlink => metadata.file_type().is_symlink(),
    };
}

/// What:
///  A failure of one source entry,
/// worded with the path the person can act on.
/// Why:
///   `No such file or directory` names no file,
/// and the checkout it came from holds thousands:
/// the line says which entry failed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${path} ${what} (${error})`
/// ```
fn named_source_failure(
    source_path: &Path,
    what: &str,
    error: &dyn std::fmt::Display,
) -> WorktreeCopyError {
    let message: String = format!("{} {what} ({error})", source_path.display());
    return failure_with_cause(message.as_str(), None);
}
/// What:
///  Creates a symbolic link with the exact target text.
/// Why:
///   The target is never resolved,
/// so a link to a missing or outside path is copied as-is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await symlink(target, stagePath);
/// ```
#[cfg(unix)]
fn symlink_at(
    target: &Path,
    link: &Path,
) -> Result<(), WorktreeCopyError> {
    return std::os::unix::fs::symlink(target, link).map_err(raw_failure_of);
}

/// What:
///  Creates a symbolic link on a platform where this lane does not stage links.
/// Why:
///   Links are a Unix feature in the copy contract;
/// other platforms refuse them with the generic sentence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await symlink(target, stagePath);
/// ```
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
fn symlink_at(
    _target: &Path,
    _link: &Path,
) -> Result<(), WorktreeCopyError> {
    return Err(crate::worktree_copy_error::raw_failure(
        &std::io::Error::other("symbolic links are not staged on this platform"),
    ));
}
