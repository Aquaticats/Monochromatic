//! What:
//!  The checks that prove the worktree about to be emptied is the one whose administrative directory the record names:
//!  the worktree holds a `.git` file,
//!  that file names a directory the record's Git directory contains,
//!  and two paths that both resolve may name the same place.
//! Why:
//!  A trash's record is written under the repository's common directory,
//!  and the worktree that holds the trash is a linked one,
//!  so its pointer names `<common>/worktrees/<name>` and never the common directory itself.
//!  The proof is still a proof:
//!  the pointer names a directory the record's Git directory contains,
//!  and the emptying then opens every step below the worktree without following a link
//!  and compares the trash's identity.
//!  `worktree_trash_dispose_the_trash_held_open.rs` asks for this proof before it opens anything below the worktree.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const pointsInto = (gitDir, pointed) => pointed === gitDir || pointed.parent() === join(gitDir, 'worktrees');
//! const verifyAssociation = (root, repository, gitDir) => { /* throws the reason the trash is left alone */ };
//! ```

use crate::fs_beneath::open_file_at;
use std::fs::File;
use std::path::{
    Path,
    PathBuf,
};

/// The most bytes of a worktree's `.git` file that are read.
const POINTER_LIMIT: u64 = 64 * 1024;

/// Whether two paths that both resolve name the same place;
/// a path that cannot be resolved names none.
fn same_after_resolving(
    left: &Path,
    right: &Path,
) -> bool {
    return match (std::fs::canonicalize(left), std::fs::canonicalize(right)) {
        (Ok(resolved_left), Ok(resolved_right)) => resolved_left == resolved_right,
        _ => false,
    };
}

/// What:
///  Whether the administrative directory a worktree's `.git` file names belongs to the directory the record was read
///  from:
/// that directory itself,
/// or one of the per-worktree directories below it.
/// Why:
///   A trash's record is written under the repository's common directory,
/// because one repository's service
///  empties its trashes one at a time and a record is drained where it was read;
/// the worktree that holds the
///  trash is a linked one,
/// so its pointer names `<common>/worktrees/<name>` and never the common directory
///  itself.
/// The proof is still a proof:
/// the pointer names a directory the record's Git directory contains,
/// and the
///  emptying then opens every step below the worktree without following a link and compares the trash's identity.
/// A path
///  that resolves to the same place is accepted as well,
/// so a repository whose paths are reached through a link is not
///  refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pointsInto = (gitDir, pointed) => pointed === gitDir || pointed.parent() === join(gitDir, 'worktrees') || sameAfterResolving(pointed, gitDir) || sameAfterResolving(pointed.parent(), join(gitDir, 'worktrees'));
/// ```
fn points_into(
    git_dir: &Path,
    pointed: &Path,
) -> bool {
    if pointed == git_dir || same_after_resolving(pointed, git_dir) {
        return true;
    }
    // Every per-worktree directory of a repository sits below `worktrees` in its common directory; a record and a pointer
    //  that reach it through different names are the same place when their parents resolve to one.
    let below: PathBuf = git_dir.join("worktrees");
    let Some(parent) = pointed.parent() else {
        return false;
    };
    return parent == below || same_after_resolving(parent, below.as_path());
}

/// Check that the worktree at `repository` is the one whose administrative directory is `git_dir`:
/// its `.git` file,
/// read
///  below the held root,
/// points at that directory.
/// It is visible to this folder because the step that opens the trash
///  (`worktree_trash_dispose_the_trash_held_open.rs`) asks for it first.
pub(crate) fn verify_association(
    root: &File,
    repository: &Path,
    git_dir: &Path,
) -> Result<(), String> {
    let pointer: File = match open_file_at(root, std::ffi::OsStr::new(".git")) {
        Ok(opened) => opened,
        Err(error) => {
            return Err(format!(
                "{} has no readable .git file ({error}), so it is no linked worktree and its trash is left alone",
                repository.display()
            ));
        }
    };
    let is_file: bool = match pointer.metadata() {
        Ok(found) => found.is_file(),
        Err(_unreadable) => false,
    };
    if !is_file {
        return Err(format!(
            "{}/.git is not a file, so its trash is left alone",
            repository.display()
        ));
    }
    let mut text: String = String::new();
    if let Err(error) =
        std::io::Read::read_to_string(&mut std::io::Read::take(&pointer, POINTER_LIMIT), &mut text)
    {
        return Err(format!(
            "{}/.git cannot be read ({error})",
            repository.display()
        ));
    }
    let Some(after_prefix) = text.strip_prefix("gitdir:") else {
        return Err(format!(
            "{}/.git does not point at an administrative directory, so its trash is left alone",
            repository.display()
        ));
    };
    let recorded: &str = after_prefix.trim();
    let pointed: PathBuf = if Path::new(recorded).is_absolute() {
        PathBuf::from(recorded)
    } else {
        repository.join(recorded)
    };
    if !points_into(git_dir, pointed.as_path()) {
        return Err(format!(
            "{}/.git points at {} and not into {}, so its trash is left alone",
            repository.display(),
            pointed.display(),
            git_dir.display()
        ));
    }
    return Ok(());
}
