//! What:
//!  The entries and the paths the stat-data question leaves to Git:
//!  an entry Git's own refresh never re-reads,
//!  and a path this commit stages from the worktree afresh.
//! Why:
//!  An unmerged entry,
//!  one marked assume-unchanged or skip-worktree,
//!  and a gitlink have no file this reading could compare,
//!  and Git's `refresh_index` skips exactly those.
//!  A pathspec with magic,
//!  an escape or a wildcard names paths this reading cannot work out,
//!  and exempting the wrong entry would leave the scratch commit to read a file it need not.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const leftToGit = (entry) => entry.stage !== 0 || entry.assumeValid || entry.skipWorktree || entry.gitlink;
//! ```

/// What a commit stages from the worktree.
use crate::commit_index_stat_staleness_staging::WorktreeStaging;
/// An entry whose mode names another repository,
/// and the bits of a mode that name its type.
use crate::commit_index_stat_staleness_worktree_file::{
    GITLINK,
    TYPE_BITS,
};
/// An entry of the whole index,
///  and the flags that say Git's refresh skips it.
use crate::index_full::{
    ASSUME_VALID,
    FullEntry,
    SKIP_WORKTREE,
};
/// `OsStrExt` is the trait whose `as_bytes` spells one pathspec.
use std::os::unix::ffi::OsStrExt;

/// What:
///  Whether one entry is one Git's own refresh never re-reads.
/// Why:
///  An unmerged entry,
///  one marked assume-unchanged or skip-worktree,
///  and a gitlink have no file this reading could compare,
///  and Git's `refresh_index` skips exactly those.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const leftToGit = (entry) => entry.stage !== 0 || entry.assumeValid || entry.skipWorktree || entry.gitlink;
/// ```
#[cfg(unix)]
pub(crate) fn is_left_to_git(entry: &FullEntry) -> bool {
    if entry.stage() != 0 {
        return true;
    }
    if entry.flags & ASSUME_VALID != 0 {
        return true;
    }
    if entry.extended & SKIP_WORKTREE != 0 {
        return true;
    }
    return entry.mode & TYPE_BITS == GITLINK;
}

/// What:
///  Whether one byte of a pathspec is one of the characters that make a pathspec name more than the one path it spells.
/// Why:
///  A pathspec with magic,
///  an escape or a wildcard names paths this reading cannot work out,
///  and exempting the wrong entry would leave the scratch commit to read a file it need not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isNotMagicByte = (byte) => !':*?[]\\\0'.includes(String.fromCharCode(byte));
/// ```
#[cfg(unix)]
fn is_not_a_magic_byte(byte: &u8) -> bool {
    return !matches!(*byte, b':' | b'*' | b'?' | b'[' | b']' | b'\\' | 0);
}

/// What:
///  Whether a pathspec names exactly the one path it spells.
/// Why:
///  Only such a pathspec can be matched against an index path without guessing what a directory or a glob names,
/// and a pathspec this refuses costs one Git start,
/// never a wrong answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isLiteralPath = (spec) => spec.length > 0 && !spec.endsWith('/') && [...spec].every(isNotMagicByte);
/// ```
#[cfg(unix)]
fn is_literal_path(spec: &[u8]) -> bool {
    if spec.is_empty() || spec.ends_with(b"/") {
        return false;
    }
    return spec.iter().all(is_not_a_magic_byte);
}

/// What:
///  The top-relative paths the selection stages from the worktree.
/// Why:
///  `git add --all` writes their stat data into the private index afresh,
/// so what the real index records for them cannot cost the scratch commit a read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const restaged = staging.named.filter(isLiteralPath).map((spec) => prefix + spec);
/// ```
#[cfg(unix)]
pub(crate) fn restaged_paths(
    staging: &WorktreeStaging<'_>,
    prefix: &[u8],
) -> Vec<Vec<u8>> {
    let mut paths: Vec<Vec<u8>> = Vec::new();
    let WorktreeStaging::Named(named) = staging else {
        return paths;
    };
    for spec in named.iter() {
        let spelled: &[u8] = spec.as_bytes();
        if !is_literal_path(spelled) {
            continue;
        }
        let mut top_relative: Vec<u8> = Vec::with_capacity(prefix.len() + spelled.len());
        top_relative.extend_from_slice(prefix);
        top_relative.extend_from_slice(spelled);
        paths.push(top_relative);
    }
    return paths;
}

/// What:
///  Whether one index path is one the selection stages from the worktree.
/// Why:
///  The lookup says whether the entry's recorded stat data still matters.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isRestaged = (restaged, path) => restaged.some((named) => named === path);
/// ```
#[cfg(unix)]
pub(crate) fn is_restaged(
    restaged: &[Vec<u8>],
    path: &[u8],
) -> bool {
    for named in restaged.iter() {
        if named.as_slice() == path {
            return true;
        }
    }
    return false;
}
