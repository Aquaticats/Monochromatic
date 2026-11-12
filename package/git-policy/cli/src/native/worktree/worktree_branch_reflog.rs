//! What:
//!  The commits a branch's own reflog says it moved between.
//! Why:
//!  A branch's reflog is the only record of how it came to the commit it points at:
//!  every line names the commit the branch held before it and the one it held after,
//!  and its first line names no commit as the value before it,
//!  because no branch pointed anywhere before that line.
//!  The landed sweep asks those two questions to decide whether a branch that a merge brought in holds a commit of its own
//!  (`worktree_landed_removal.rs`),
//!  which is what keeps a worktree a person has not started work in out of the sweep
//!  and what keeps one whose branch a person moved onto another branch's commit.
//!  The reader is its own file because Git's reflog format is:
//!  bytes rather than text,
//!  because a branch name's bytes are the person's,
//!  and only the two commit fields of a line are read at all,
//!  which are hexadecimal whatever they hold.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! branchMadeAt(commonDir, 'refs/heads/feature'); // 'dfc6c4e...' | undefined
//! branchMovedOnto(commonDir, 'refs/heads/main', 'dfc6c4e...'); // false for a branch created there
//! ```

use std::path::{Path, PathBuf};

/// The commit a reflog line names as the value a reference had before it existed,
/// which is what a branch created at a commit records.
/// Why:
///  Measured with Git 2.56 on a fixture:
///  `git worktree add -b task-z` writes `0000...0000 <commit> ... branch: Created from HEAD`,
///  and no branch ever pointed anywhere before that line.
///  A reset onto a commit and a commit of its own both record the commit the branch held before them,
///  so a line that names this value is the one line that says nothing about who made a commit.
const NO_COMMIT: &str = "0000000000000000000000000000000000000000";

/// The commit the branch named by `reference` was made at,
/// read from the oldest entry of its own reflog,
/// or nothing when this repository keeps no reflog for it.
/// Why:
///  Every reflog line begins with the commit the reference named before the entry and the one it names after it,
///  and the oldest line of a branch's reflog is the entry the branch was made in:
///  `git worktree add -b` writes it as surely as `git branch` does.
///  The line is read as bytes because a branch name's bytes are the person's,
///  and its two commit fields are hexadecimal whatever they hold.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// branchMadeAt(commonDir, 'refs/heads/feature') // 'dfc6c4e...' | undefined
/// ```
pub(crate) fn branch_made_at(common_dir: &Path, reference: &str) -> Option<String> {
    let path: PathBuf = common_dir.join("logs").join(reference);
    let text: Vec<u8> = std::fs::read(path.as_path()).ok()?;
    let oldest: &[u8] = text.split(is_line_end).next()?;
    let made: &[u8] = oldest.split(is_space).nth(1)?;
    return std::str::from_utf8(made).ok().map(str::to_owned);
}

/// Whether the byte ends a line of a reflog.
/// Why:
///  Named apart from the split that uses it,
///  because a closure that captures nothing is what this codebase keeps out of its calls.
fn is_line_end(byte: &u8) -> bool {
    return *byte == b'\n';
}

/// Whether the byte separates two fields of a reflog line.
fn is_space(byte: &u8) -> bool {
    return *byte == b' ';
}

/// The two commits one reflog line moves between,
/// the one it moved from and the one it moved to,
/// or nothing when the line holds no such pair.
/// Why:
///  Every line begins with those two fields as hexadecimal,
///  and the rest of it is the person's name and words,
///  which this reads no part of.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// movedBetween(line) // [old, next] | undefined
/// ```
fn moved_between(line: &[u8]) -> Option<(&[u8], &[u8])> {
    let after_old: usize = line.iter().position(is_space)?;
    let rest: &[u8] = line.get(after_old + 1..)?;
    let after_next: usize = rest.iter().position(is_space)?;
    return Some((line.get(..after_old)?, rest.get(..after_next)?));
}

/// Whether a branch's own reflog records a move onto `commit` from another commit,
/// which is a commit some branch already held when this one took it.
/// Why:
///  A commit a branch creates is the new value of its own reflog line,
///  and a commit it merely moved onto is the new value of a line the same way:
///  Git records what a reference held,
///  never where a commit came from.
///  The one difference it does record is a branch's first line,
///  which names no commit as the value before it,
///  so a branch created at a commit proves nothing about who made that commit.
///  A repository that keeps no reflog for the branch answers nothing,
///  and the caller then treats the branch as it did before this question existed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// branchMovedOnto(commonDir, 'refs/heads/main', 'dfc6c4e...') // true when main's own reflog moved there
/// ```
pub(crate) fn branch_moved_onto(common_dir: &Path, reference: &str, commit: &str) -> bool {
    let path: PathBuf = common_dir.join("logs").join(reference);
    let Ok(text) = std::fs::read(path.as_path()) else { return false };
    let wanted: &[u8] = commit.as_bytes();
    for line in text.split(is_line_end) {
        let Some((old, next)) = moved_between(line) else { continue };
        if next == wanted && old != next && old != NO_COMMIT.as_bytes() {
            return true;
        }
    }
    return false;
}

/// The controls of this reader stay out of the release executable.
#[cfg(test)]
#[path = "worktree_branch_reflog_tests.rs"]
mod tests;
