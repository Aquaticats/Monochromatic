//! What:
//!  Whether the worktree an interrupted add copies its ignored state from is still somewhere a worktree could
//!  stand,
//! and the words for the one it is not.
//! Why:
//!  The listing that decides whether a source is registered runs inside the source's own directory,
//! so a
//!  source that was removed or replaced by a file cannot even be asked.
//! That answer is about the source and not about the copy,
//! so the pass that finishes the copy reads it from
//!  here,
//! and a refusal that could change still leaves the records for the next command.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const gone = !couldBeAWorktree(source) ? 'the worktree it copies from, ..., is no longer a worktree of this repository' : undefined;
//! ```

use crate::json_record::quote;
use std::path::Path;

/// What:
///  The words for a copy whose source is no worktree of this repository any more.
/// Why:
///   A source that was removed,
/// moved or replaced cannot be staged from,
/// so the worktree it was to fill is named
///  with one reason whether the listing answered that or the source's directory could not be entered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reason = `the worktree it copies from, ${quote(source)}, is no longer a worktree of this repository`;
/// ```
pub(crate) fn source_is_no_worktree_reason(source: &Path) -> String {
    return format!(
        "the worktree it copies from, {}, is no longer a worktree of this repository",
        quote(source.to_string_lossy().as_ref())
    );
}

/// What:
///  Whether anything that could be a worktree stands at the source's path.
/// Why:
///   A source that is gone,
/// or one replaced by a file,
/// is provably no worktree and no copy can ever be made from
///  it,
/// while a directory that cannot be read proves nothing and is asked again by the next command.
/// A source that
///  is gone needs this answer because the registered-root listing cannot start in it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const couldBeAWorktree = (source) => { try { return await lstat(source).isDirectory() } catch (e) { return e.code !== 'ENOENT' && e.code !== 'ENOTDIR' } };
/// ```
pub(crate) fn source_could_be_a_worktree(source: &Path) -> bool {
    // `match` on the source itself: absence and a file where a directory belongs are answers, and every refusal that could change keeps the question open.
    match std::fs::metadata(source) {
        Ok(found) => return found.is_dir(),
        Err(error) => return error.kind() != std::io::ErrorKind::NotFound && error.kind() != std::io::ErrorKind::NotADirectory,
    }
}

/// Source-directory controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_owed_copy_source_tests.rs"]
mod tests;
