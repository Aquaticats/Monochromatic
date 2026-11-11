//! What:
//!  Ask real Git which worktree of a repository is the main checkout,
//!  in one query.
//! Why:
//!  A refusal in a linked worktree must name the checkout the shared settings belong to,
//!  so the
//!      person can change them on purpose there.
//!  `git worktree list --porcelain` prints the main worktree first
//!      (`git help worktree`),
//!  so the first stanza's path is that answer,
//!  and `-z` keeps a path that holds a
//!      line feed readable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const main = mainWorktreeFromAnswer(await git([...globalPrefix, 'worktree', 'list', '--porcelain', '-z']));
//! ```

/// The captured-query runner's output type and its byte helpers.
use crate::git_metadata::{MetadataOutput, path_from_git_bytes};
/// What:
///  `OsString` is owned operating-system text of raw bytes.
///  Sibling the reader might
///       expect:
///  `String`,
///  which must be valid UTF-8.
/// Why:
///   The caller's global options are replayed to Git unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // string[] argv, but byte-preserving.
/// ```
use std::ffi::OsString;
/// `PathBuf` is an owned filesystem path of raw bytes.
use std::path::PathBuf;

/// The line `git worktree list --porcelain` prints a worktree's path on.
const WORKTREE_PREFIX: &[u8] = b"worktree ";

/// What:
///  The complete argument list of the query.
///  `&[OsString]` borrows the caller's global options.
/// Why:
///   Replaying the global prefix lets Git apply `-C` chains and `--git-dir` exactly as it will
///  for the forwarded command,
///  and `-z` separates the lines with a NUL so a path holding a line
///       feed is still one value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const args = [...globalPrefix, 'worktree', 'list', '--porcelain', '-z'];
/// ```
pub fn main_worktree_query_arguments(global_prefix: &[OsString]) -> Vec<OsString> {
    // `.to_vec()` copies the borrowed prefix into an owned, growable argument list.
    let mut arguments: Vec<OsString> = global_prefix.to_vec();
    for part in ["worktree", "list", "--porcelain", "-z"] {
        arguments.push(OsString::from(part));
    }
    return arguments;
}

/// What:
///  Whether one byte ends a line of the NUL-separated listing.
/// Why:
///   `split` takes the predicate itself,
///  so the test is a named function instead of a closure that
///       captures nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isNul = (byte: number): boolean => byte === 0;
/// ```
fn is_nul(byte: &u8) -> bool {
    return *byte == b'\0';
}

/// What:
///  The main worktree's path from the query's answer.
///  `&MetadataOutput` borrows what Git printed and how it ended.
/// Why:
///   The first stanza of the listing is the main worktree;
///  a failure or a stanza without that
///       line is reported,
///  never guessed,
///  because a refusal that names the wrong directory sends the person
///       to the wrong place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mainWorktreeFromAnswer(output): string; // throws
/// ```
pub fn main_worktree_from_answer(output: &MetadataOutput) -> Result<PathBuf, String> {
    if !output.success {
        return Err(String::from(
            "monochromatic-git-policy-cli could not ask Git for the worktrees of this repository, so the main \
             checkout cannot be named.",
        ));
    }
    // The entries are NUL-separated lines; the first stanza starts with the main worktree's path.
    let recorded: &[u8] = output.stdout.as_slice();
    if !recorded.starts_with(WORKTREE_PREFIX) {
        return Err(String::from(
            "monochromatic-git-policy-cli could not read the worktrees Git printed, so the main checkout cannot be \
             named.",
        ));
    }
    // `.split(...)` cuts every line at a NUL; `.next()` takes the first one.
    let first_line: &[u8] = recorded.split(is_nul).next().unwrap_or(recorded);
    // `strip_prefix` is `Some(rest)` when the line starts with the prefix.
    let path_bytes: &[u8] = first_line
        .strip_prefix(WORKTREE_PREFIX)
        .unwrap_or(first_line);
    return match path_from_git_bytes(path_bytes) {
        Some(found) => Ok(found),
        None => Err(String::from(
            "monochromatic-git-policy-cli could not read the main checkout's path, which Git printed as an empty \
             line.",
        )),
    };
}
