//! What:
//!  The linked-worktree policy's reading of a `git checkout` or `git restore` command line
//!  that would replace a conflicted file while a merge,
//!  a rebase,
//!  a cherry-pick or a revert waits,
//!  and the refusal that names every file it would replace.
//! Why:
//!  A conflicted path is where the person's own resolution of it lives,
//!  and nothing else holds it:
//!  the index holds the three sides,
//!  and this program keeps no copy.
//!  The policy already rejects these forms,
//!  so the reading is here to name what the rejection would destroy:
//!  the guard finds the operation and the unmerged entries in process,
//!  starts no Git,
//!  and answers with the repository-relative paths the line names that the index marks as unmerged.
//!  A line whose paths cannot be read as single files (`--patch`,
//!  a wildcard,
//!  pathspec magic,
//!  a directory)
//!  names nothing,
//!  and the usual rejection stands without an extra sentence.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const verdict = judgeConflictSides({ context, arguments }); // 'not-applicable' | { kind: 'would-replace', paths }
//! ```

use crate::command_conflict_side::{SideRequest, conflict_side_request};
use crate::conflict_side_state::{Operation, conflicted_paths, operation_in_progress};
use crate::diagnostic_log::debug;
use crate::global_arguments::command_tokens;
use crate::private_storage::read_regular_file;
use crate::repo_read_index::parse_index;
use crate::repo_read_layout::{RepoLayout, read_layout};
use crate::transaction_git::GitContext;
use std::collections::BTreeSet;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

/// Module tag for diagnostics.
const TAG: &str = "judgeConflictSides";

/// The largest index file this reads.
const INDEX_LIMIT_BYTES: u64 = 1 << 30;

/// What the reading of one command line came to.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SideVerdict {
    /// Not a form this reads,
    /// no operation in progress,
    /// or no path of the line is unmerged:
    /// the usual rejection stands as it is.
    NotApplicable,
    /// The command would replace these repository-relative paths,
    /// each of which the index marks as unmerged,
    /// in the order named,
    /// while this operation waits for its conflicts.
    WouldReplace {
        /// The operation whose resolution the command would replace.
        operation: Operation,
        /// The repository-relative paths it would replace.
        paths: Vec<String>,
    },
}

/// What:
///  The operation that waits and the paths it left unmerged,
///  or why the line cannot be judged.
/// Why:
///   Both are read in process from the Git directory,
///  so the refusal and the command it refuses see the same moment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function waiting(layout: RepoLayout): { operation: Operation; conflicted: Set<string> } | undefined;
/// ```
fn waiting(layout: &RepoLayout) -> Option<(Operation, BTreeSet<Vec<u8>>)> {
    let operation: Operation = operation_in_progress(layout.git_dir.as_path())?;
    if std::env::var_os("GIT_INDEX_FILE").is_some() {
        debug(TAG, "declined: GIT_INDEX_FILE names another index, so no path can be shown to be unmerged");
        return None;
    }
    let Ok(bytes) = read_regular_file(layout.git_dir.join("index").as_path(), INDEX_LIMIT_BYTES) else {
        debug(TAG, "declined: the index cannot be read in process, so no path can be shown to be unmerged");
        return None;
    };
    let Some(entries) = parse_index(bytes.as_slice(), layout.object_format) else {
        debug(TAG, "declined: the index is split, sparse or carries an extension the reader cannot vouch for");
        return None;
    };
    return Some((operation, conflicted_paths(entries.as_slice())));
}

/// What:
///  The repository-relative path a pathspec token names,
///  when it names exactly one file by its literal spelling.
/// Why:
///   Git resolves a pathspec below the directory the command runs in.
///  Magic (`:(glob)`,
///  `:/`),
///  wildcards,
///  a trailing slash and a directory are not a single file,
///  so they resolve to nothing and add no name to the refusal.
///  `.` and `..` segments are folded;
///  a path that leaves the worktree resolves to nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function literalPath(prefix: string, root: string, token: string): string | undefined;
/// ```
fn literal_path(prefix: &[u8], root: &Path, token: &OsStr) -> Option<Vec<u8>> {
    let bytes: &[u8] = token.as_bytes();
    if bytes.is_empty() || bytes[0] == b':' || bytes.ends_with(b"/") || bytes.iter().any(is_wildcard) {
        return None;
    }
    let joined: Vec<u8> = if bytes[0] == b'/' {
        let below: &Path = Path::new(token).strip_prefix(root).ok()?;
        below.as_os_str().as_bytes().to_vec()
    } else {
        [prefix, bytes].concat()
    };
    let mut kept: Vec<&[u8]> = Vec::new();
    for segment in joined.split(is_slash) {
        match segment {
            b"" | b"." => {}
            b".." => {
                kept.pop()?;
            }
            other => kept.push(other),
        }
    }
    if kept.is_empty() {
        return None;
    }
    return Some(kept.join(&b'/'));
}

/// Whether a byte makes a pathspec token more than a literal name.
fn is_wildcard(byte: &u8) -> bool {
    return matches!(byte, b'*' | b'?' | b'[' | b'\\');
}

/// Whether a byte separates the segments of a path.
fn is_slash(byte: &u8) -> bool {
    return *byte == b'/';
}

/// What:
///  The paths of `tokens` that the index marks as unmerged,
///  each once and in the order named.
/// Why:
///   Only a token that names exactly one file below the working directory can be shown to be one of them,
///  so every other token adds no name:
///  the rejection stays either way,
///  and it never claims a path the command would not replace.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unmergedAmong(tokens, prefix, root, conflicted): string[];
/// ```
fn unmerged_among(tokens: &[OsString], prefix: &[u8], root: &Path, conflicted: &BTreeSet<Vec<u8>>) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for token in tokens {
        let Some(path) = literal_path(prefix, root, token.as_os_str()) else {
            continue;
        };
        if !conflicted.contains(&path) {
            continue;
        }
        let shown: String = String::from_utf8_lossy(path.as_slice()).into_owned();
        if !found.contains(&shown) {
            found.push(shown);
        }
    }
    return found;
}

/// What:
///  Judge a command line against the operation that waits and the unmerged entries of the index.
/// Why:
///   The form must be one of the `checkout` and `restore` shapes that write a worktree file
///  (`command_conflict_side.rs`),
///  an operation must be waiting,
///  the index must be readable in process,
///  and at least one named path must be unmerged.
///  An answer this reader cannot vouch for names nothing,
///  so the refusal never claims a file it cannot prove.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function judgeConflictSides(context, arguments): SideVerdict;
/// ```
pub fn judge_conflict_sides(context: &GitContext, arguments: &[OsString]) -> SideVerdict {
    let Some((word, region)) = command_tokens(arguments) else {
        return SideVerdict::NotApplicable;
    };
    let Some(request): Option<SideRequest> = conflict_side_request(word.as_encoded_bytes(), region) else {
        return SideVerdict::NotApplicable;
    };
    let Some(layout) = read_layout(context.global_prefix.as_slice(), context.overlay.as_slice()) else {
        // A global option other than `-C` or a `GIT_` variable the layout reader does not tolerate lands here:
        //  a pathspec mode such as `--icase-pathspecs` among them, whose paths cannot be read as named.
        debug(TAG, "declined: the layout reader does not read this invocation, so its paths cannot be named");
        return SideVerdict::NotApplicable;
    };
    let Some((operation, conflicted)) = waiting(&layout) else {
        return SideVerdict::NotApplicable;
    };
    let paths: Vec<String> = unmerged_among(request.paths.as_slice(), layout.prefix.as_slice(), layout.worktree_root.as_path(), &conflicted);
    if paths.is_empty() {
        debug(TAG, &format!("{operation:?} waits, but no path this line names is unmerged among {} conflicted", conflicted.len()));
        return SideVerdict::NotApplicable;
    }
    debug(TAG, &format!("{operation:?} waits and this line would replace {paths:?}"));
    return SideVerdict::WouldReplace { operation, paths };
}

/// The reading and the paths, over a directory of fixtures and a real index.
#[cfg(test)]
#[path = "conflict_side_guard_tests.rs"]
mod tests;
