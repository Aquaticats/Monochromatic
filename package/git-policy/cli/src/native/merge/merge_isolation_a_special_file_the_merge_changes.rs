//! What:
//!  A named pipe,
//! a socket,
//! or a device standing at a path a merge writes or deletes,
//! and the words that stop the merge before it
//!  waits on that entry or removes it.
//! Why:
//!   Such an entry holds no bytes,
//! so no layer of the carry can hold it,
//! and the merge's file and the entry cannot both
//!  stand at one path.
//!  Reading one is worse than losing it:
//! opening a named pipe for reading waits until a writer appears,
//! and
//!  nothing promises one,
//! so a merge that read the worktree at such a path would wait without an end while it holds
//!  the checkout ("nothing should ever wait forever or indefinitely").
//!  Git's own update of the checkout replaces or removes
//!  the entry without a copy of it,
//! so the merge would take away a pipe another process still writes into (the owner's rule
//!  "never discard user data").
//!  The merge therefore stops before the first byte is written,
//! and the line names the paths,
//!  says
//! what stands there,
//! that nothing was changed,
//! and that the entries are still in the person's checkout
//!  (`SPEC.md`,
//! "Merges and commits never fail",
//! beside the directory the merge would replace).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const found = aSpecialFileTheMergeChanges(root, incoming);
//! if (found.length) return refused(1, theLine(reference, found));
//! ```

use crate::auto_push_dirty_paths::listed;
use crate::merge_isolation_changes::worktree_path;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};

/// What:
///  Every path of `paths` whose worktree entry is a named pipe,
/// a socket or a device.
/// Why:
///   These are exactly the paths whose bytes no capture can hold and whose open no command may wait on,
/// so the merge
///  stops before Git's update of the checkout meets one.
///  A directory is judged by `merge_isolation_a_directory_the_merge_replaces`,
/// which holds files the
///  person may have written,
/// and a link is a file of its own with bytes to carry,
/// so neither is named here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aSpecialFileTheMergeChanges(root, paths) { return paths.filter((p) => holdsNoBytes(join(root, p))); }
/// ```
pub fn a_special_file_the_merge_changes(root: &Path, paths: &[Vec<u8>]) -> Vec<Vec<u8>> {
    let mut found: Vec<Vec<u8>> = Vec::new();
    for path in paths {
        let location: PathBuf = worktree_path(root, path.as_slice());
        if holds_no_bytes(location.as_path()) {
            found.push(path.clone());
        }
    }
    return found;
}

/// What:
///  Whether what stands at `location` is a named pipe,
/// a socket or a device.
/// Why:
///   The type is read without following a link and without opening the entry,
/// so a pipe is named
///  rather than waited on,
/// and a link to one is a file whose target text is what the carry holds.
///  Anything the metadata call cannot read is not named here:
/// the merge then meets it exactly as native Git does,
///  and never opens it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// symlinkMetadataSync(location).isFile() || isDirectory() || isSymbolicLink() ? false : true;
/// ```
fn holds_no_bytes(location: &Path) -> bool {
    return match std::fs::symlink_metadata(location) {
        Ok(found) => {
            let kind: std::fs::FileType = found.file_type();
            !kind.is_file() && !kind.is_dir() && !kind.is_symlink()
        }
        Err(_absent) => false,
    };
}

/// What:
///  The sentence a merge that met such an entry ends with.
/// Why:
///   The person must know which paths hold what,
/// that the merge was not made,
/// that nothing of theirs was removed,
/// and
///  what to do before running the merge again,
/// because the entry and the merge's path cannot both stand there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `git merge ${reference}: the worktree holds ${listed(found)} where this merge writes or deletes a path, ...`;
/// ```
pub fn the_line(reference: &str, found: &[Vec<u8>]) -> String {
    return format!(
        "monochromatic-git-policy-cli: git merge {reference}: the worktree holds {} where this merge writes or deletes a path, and a \
         named pipe, a socket or a device holds no bytes a merge can carry, so the merge was not made and nothing in the worktree, \
         the index or the branch changed. Move each of them out of the way with a name of your own and run the merge again; they are \
         still in your checkout as they were.\n",
        listed(found)
    );
}

/// The wording of the line and the entries it names.
#[cfg(test)]
#[path = "merge_isolation_a_special_file_the_merge_changes_tests.rs"]
mod tests;
