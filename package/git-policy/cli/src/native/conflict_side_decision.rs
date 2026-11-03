//! What:
//!  Whether every token of a command line names a conflicted file, read by its literal spelling.
//! Why:
//!  A path wrongly judged conflicted lets a clean file be overwritten, so only a token that names exactly one file, below the
//!  working directory, with no wildcard or magic, can be allowed; every other token is refused and named.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // decideSides(['a.rs'], '', root, new Set(['a.rs'])) // { kind: 'allow', paths: ['a.rs'] }
//! ```

use std::collections::BTreeSet;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

/// What:
///  The repository-relative path a pathspec token names, when it names exactly one file by its literal spelling.
/// Why:
///   Git resolves a pathspec below the directory the command runs in. Magic (`:(glob)`, `:/`), wildcards, a trailing
///  slash and a directory are not a single conflicted file, so they resolve to nothing and the usual rejection stays.
///  `.` and `..` segments are folded; a path that leaves the worktree resolves to nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function literalPath(prefix: string, root: string, token: string): string | undefined;
/// ```
pub fn literal_path(prefix: &[u8], root: &Path, token: &OsStr) -> Option<Vec<u8>> {
    let bytes: &[u8] = token.as_bytes();
    if bytes.is_empty() || bytes[0] == b':' || bytes.ends_with(b"/") || bytes.iter().any(|byte| { return matches!(byte, b'*' | b'?' | b'[' | b'\\'); }) {
        return None;
    }
    let joined: Vec<u8> = if bytes[0] == b'/' {
        let below: &Path = Path::new(token).strip_prefix(root).ok()?;
        below.as_os_str().as_bytes().to_vec()
    } else {
        [prefix, bytes].concat()
    };
    let mut kept: Vec<&[u8]> = Vec::new();
    for segment in joined.split(|byte| { return *byte == b'/'; }) {
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

/// What a command line that names paths came to, against the conflicted set.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SideDecision {
    /// Every path is a conflicted file: the repository-relative paths, in the order named, each once.
    Allow(Vec<Vec<u8>>),
    /// At least one token is not a conflicted file; these are the tokens, as typed.
    Refuse(Vec<OsString>),
}

/// What:
///  Whether every token names a conflicted file.
/// Why:
///   A pathspec that mixes conflicted and clean paths is refused whole, and the refusal names the clean ones, so a
///  person sees exactly what to leave out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function decideSides(tokens: string[], prefix: string, root: string, conflicted: Set<string>): SideDecision;
/// ```
pub fn decide_sides(tokens: &[OsString], prefix: &[u8], root: &Path, conflicted: &BTreeSet<Vec<u8>>) -> SideDecision {
    let mut allowed: Vec<Vec<u8>> = Vec::new();
    let mut others: Vec<OsString> = Vec::new();
    for token in tokens {
        match literal_path(prefix, root, token.as_os_str()) {
            Some(path) if conflicted.contains(&path) => {
                if !allowed.contains(&path) {
                    allowed.push(path);
                }
            }
            _ => others.push(token.clone()),
        }
    }
    if others.is_empty() {
        return SideDecision::Allow(allowed);
    }
    return SideDecision::Refuse(others);
}
