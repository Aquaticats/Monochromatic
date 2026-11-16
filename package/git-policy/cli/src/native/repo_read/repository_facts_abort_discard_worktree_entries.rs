//! What:
//!  Whether the worktree still holds an entry at a path a listing named.
//! Why:
//!  A path the listing names because the worktree file is gone holds nothing of the person's:
//!  the reset writes the tree back over that path and puts the file there again,
//!  so an abort undoes such a removal and loses no content.
//!  Moving a file out of the way is the way out a refusal of this family gives,
//!  and a reading that counted the removal as work would refuse the command that advice promised.
//!  It lives beside the reading that asks for it,
//!  so the provider's own file stays within its line bound.

use crate::git_metadata::path_from_git_bytes;
use crate::repository_facts::{GitFacts, RepositoryFacts};
use crate::worktree_identity::worktree_root;
use std::path::{Path, PathBuf};

/// What:
///  Whether the worktree holds any entry at a repository-relative path.
///  `Option<&Path>` is the top of the worktree,
/// when Git named one.
/// Why:
///   A path the listing names because the worktree file is gone holds nothing of the person's:
///  the reset writes the tree back over that path and puts the file there again,
///  so an abort undoes such a removal and loses no content.
///  Moving a file out of the way is exactly the way out this refusal's own advice names,
///  and a refusal that read the removal as work would refuse the command the advice promised.
///  A directory,
/// a link or a pipe standing there does hold something the reset would replace,
///  so only a path with no entry at all is left out.
///  A worktree Git did not name,
/// and a path that is not one path below it,
/// are read as holding an entry:
///  a reading that cannot tell refuses rather than lets a loss through.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// holdsAnEntry({ root, relative: 'a.txt' }); // false when nothing stands there
/// ```
pub(super) fn holds_an_entry(root: Option<&Path>, relative: &[u8]) -> bool {
    let Some(top) = root else {
        return true;
    };
    let Some(native) = path_from_git_bytes(relative) else {
        return true;
    };
    if native.is_absolute() {
        return true;
    }
    return std::fs::symlink_metadata(top.join(native)).is_ok();
}

/// What:
///  The top of the worktree this command runs in,
///  when Git named one.
/// Why:
///   The presence of the worktree's own entries is read from the filesystem,
///  and the answer Git gave for the repository location is already remembered by the provider,
///  so asking for it again costs no start of Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const top = worktreeTop(facts); // '/repo' | undefined
/// ```
pub(super) fn worktree_top(facts: &mut GitFacts) -> Option<PathBuf> {
    let location = facts.location().ok()?;
    return worktree_root(&location.identity).map(Path::to_path_buf);
}
