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
//!  Every read of a reflog is `plain_file.rs`'s:
//!  the name sits beside a checkout any process may write,
//!  so it may hold a named pipe,
//!  and an open that waited for a writer there would hold the person's `git merge`,
//!  which reads it,
//!  with nothing said and no end
//!  (a cross-vendor review of the landed sweep,
//!  round 208).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! branchMadeAt(commonDir, 'refs/heads/feature'); // 'dfc6c4e...' | undefined
//! branchMovedOnto(commonDir, 'refs/heads/main', 'dfc6c4e...'); // false for a branch created there
//! ```

use crate::plain_file::read_plain_file;
use std::path::{Path, PathBuf};

/// What:
///  The one move a merge makes in the branch it ran in,
///  which is no witness that the commit it moved onto came from another branch.
/// Why:
///  A fast-forward leaves the branch the merge ran in on the tip the landed branch has,
///  and Git records that move in its reflog exactly as it records one the person made:
///  a merge's own move is therefore skipped by the pair of commits it wrote,
///  while any other move of the same branch,
///  made before or after it,
///  still counts.
///  Measured with Git 2.56 on fixtures on 2026-11-11:
///  a fast-forward,
///  a `--no-ff` merge and an octopus merge each write one line in the branch they ran in,
///  whose old value is the head before the merge and whose new value is the head after it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mergeMove = { from: 'a1b2c3d...', to: 'dfc6c4e...' };
/// ```
#[derive(Clone, Copy, Debug)]
pub(crate) struct MergeMove<'a> {
    /// The commit the branch held before the merge.
    pub(crate) from: &'a str,
    /// The commit it holds now.
    pub(crate) to: &'a str,
}

/// How many bytes of one branch's reflog this reads.
/// Why:
///   A reflog grows with every move of its branch,
///  and the sweep reads one for the candidate and for every landed branch it asks about,
///  so a file that grows without bound would decide how much memory a merge reads.
///  Measured on this repository on 2026-11-11:
///  its `logs/HEAD` holds 8,267 lines in 1,844,737 bytes,
///  which is 224 bytes a line,
///  so this bound is about 75,000 moves of one branch.
///  A reflog past it answers nothing,
///  and the caller then treats the branch as it does one whose reflog Git no longer keeps.
const REFLOG_LIMIT: u64 = 16 << 20;

/// What:
///  Whether one reflog field is the name a reference had before it existed,
///  which is the null object name of the repository's own hash.
/// Why:
///  Measured with Git 2.56 on a fixture:
///  `git worktree add -b task-z` writes `0000...0000 <commit> ... branch: Created from HEAD`,
///  and no branch ever pointed anywhere before that line.
///  A reset onto a commit and a commit of its own both record the commit the branch held before them,
///  so a line whose old value is this name is the one line that says nothing about who made a commit.
///  Git writes the name with as many zeros as the repository's hash is wide,
///  and comparing it against the forty zeros of SHA-1 read a branch created in a SHA-256 repository as one
///  that moved onto the commit it was created at:
///  the sweep then kept every landed worktree whose tip another branch was just made at,
///  which is the state the wrapper's own `git worktree add -b` makes on every task
///  (a cross-vendor review of the landed sweep,
///  round 208).
///  Measured with Git 2.56 on a `git init --object-format=sha256` fixture on 2026-11-11:
///  it writes sixty-four zeros before a branch's first reflog line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isNoCommit('0000000000000000000000000000000000000000000000000000000000000000') // true
/// ```
fn is_no_commit(field: &[u8]) -> bool {
    return !field.is_empty() && field.iter().all(is_zero);
}

/// Whether the byte is the zero a null object name is made of.
/// Why:
///  Named apart from the test that uses it,
///  because a closure that captures nothing is what this codebase keeps out of its calls.
fn is_zero(byte: &u8) -> bool {
    return *byte == b'0';
}

/// The bytes of the reflog of `reference`,
/// read without waiting for a writer,
/// without following a link to it,
/// and no further than `REFLOG_LIMIT`,
/// or nothing when this repository keeps no reflog that can be read for it.
/// Why:
///  A reflog's name sits beside a checkout,
///  and any process that may write that directory may plant a named pipe or a link at it
///  (`plain_file.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readReflog(commonDir, 'refs/heads/feature') // Buffer | undefined
/// ```
fn read_reflog(common_dir: &Path, reference: &str) -> Option<Vec<u8>> {
    let path: PathBuf = common_dir.join("logs").join(reference);
    return read_plain_file(path.as_path(), REFLOG_LIMIT).ok();
}

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
    let text: Vec<u8> = read_reflog(common_dir, reference)?;
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
///  so a branch created at a commit proves nothing about who made that commit,
///  whatever width that repository's null object name has (`is_no_commit`).
///  A repository that keeps no reflog for the branch answers nothing,
///  and the caller then treats the branch as it did before this question existed.
///  The merge's own move is removed from the answer
///  because a fast-forward writes one line in the branch it ran in,
///  which would otherwise read as a witness against the branch it landed
///  (`MergeMove`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// branchMovedOnto(commonDir, 'refs/heads/main', 'dfc6c4e...', undefined) // true when main's own reflog moved there
/// ```
pub(crate) fn branch_moved_onto(common_dir: &Path, reference: &str, commit: &str, merge_move: Option<MergeMove<'_>>) -> bool {
    let Some(text) = read_reflog(common_dir, reference) else { return false };
    let wanted: &[u8] = commit.as_bytes();
    for line in text.split(is_line_end) {
        let Some((old, next)) = moved_between(line) else { continue };
        if next != wanted || old == next || is_no_commit(old) {
            continue;
        }
        let the_merges_own_move: bool = match merge_move {
            Some(skipped) => old == skipped.from.as_bytes() && next == skipped.to.as_bytes(),
            None => false,
        };
        if the_merges_own_move {
            continue;
        }
        return true;
    }
    return false;
}

/// The controls of this reader stay out of the release executable.
#[cfg(test)]
#[path = "worktree_branch_reflog_tests.rs"]
mod tests;
