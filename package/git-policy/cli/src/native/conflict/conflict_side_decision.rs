//! What:
//!  Whether every token of a command line names a conflicted file,
//! read by its literal spelling.
//! Why:
//!  A path wrongly judged conflicted lets a clean file be overwritten,
//! so only a token that names exactly one file,
//! below the
//!  working directory,
//! with no wildcard or magic,
//! can be allowed;
//! every other token is refused and named.
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
///  The repository-relative path a pathspec token names,
/// when it names exactly one file by its literal spelling.
/// Why:
///   Git resolves a pathspec below the directory the command runs in.
/// Magic (`:(glob)`,
/// `:/`),
/// wildcards,
/// a trailing
///  slash and a directory are not a single conflicted file,
/// so they resolve to nothing and the usual rejection stays.
///  `.` and `..` segments are folded;
/// a path that leaves the worktree resolves to nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function literalPath(prefix: string, root: string, token: string): string | undefined;
/// ```
pub fn literal_path(prefix: &[u8], root: &Path, token: &OsStr) -> Option<Vec<u8>> {
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

/// Whether a byte closes the long form of pathspec magic.
fn is_close_paren(byte: &u8) -> bool {
    return *byte == b')';
}

/// Whether a byte separates the words of long-form pathspec magic.
fn is_comma(byte: &u8) -> bool {
    return *byte == b',';
}

/// What:
///  The magic of a pathspec token and the repository-relative path it names,
/// when the token is a plain file name under
///  magic that does not change which file is named (`:(icase)`,
/// `:(glob)`,
/// `:(literal)`,
/// `:(top)`,
/// `:(exclude)`,
/// `:!`,
/// `:^`,
///  `:/`).
/// Why:
///   Such a token is not a conflicted file as typed,
/// so the line is rejected;
/// naming the file itself is allowed,
/// and the
///  rejection can say so.
/// Any other magic (`:(attr:...)`,
/// `:(prefix:...)`,
/// an unknown word) and any wildcard stay unreadable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function magicName(prefix: string, root: string, token: string): { magic: string; path: string } | undefined;
/// ```
pub fn magic_name(prefix: &[u8], root: &Path, token: &OsStr) -> Option<(String, Vec<u8>)> {
    let bytes: &[u8] = token.as_bytes();
    if bytes.first() != Some(&b':') {
        return None;
    }
    let mut top: bool = false;
    let length: usize = if bytes.get(1) == Some(&b'(') {
        let close: usize = bytes.iter().position(is_close_paren)?;
        for word in bytes[2..close].split(is_comma) {
            match word {
                b"top" => top = true,
                b"literal" | b"icase" | b"glob" | b"exclude" => {}
                _ => return None,
            }
        }
        close + 1
    } else {
        let mut at: usize = 1;
        while at < bytes.len() && matches!(bytes[at], b'!' | b'^' | b'/') {
            top |= bytes[at] == b'/';
            at += 1;
        }
        if at == 1 {
            return None;
        }
        if bytes.get(at) == Some(&b':') {
            at += 1;
        }
        at
    };
    let path: Vec<u8> = literal_path(if top { b"" } else { prefix }, root, OsStr::from_bytes(&bytes[length..]))?;
    return Some((String::from_utf8_lossy(&bytes[..length]).into_owned(), path));
}

/// What:
///  The magic and the file name of each token,
/// when every one of `others` is pathspec magic on a conflicted file's name.
/// Why:
///   Only then is magic the single reason a line was refused;
/// a clean path,
/// a wildcard or other magic beside it means the
///  carve-out would refuse the line anyway,
/// and no sentence promises otherwise.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function magicOnConflicted(others: string[], prefix: string, root: string, conflicted: Set<string>): [string, string][] | undefined;
/// ```
pub fn magic_on_conflicted(others: &[OsString], prefix: &[u8], root: &Path, conflicted: &BTreeSet<Vec<u8>>) -> Option<Vec<(String, String)>> {
    let mut found: Vec<(String, String)> = Vec::new();
    for token in others {
        let (magic, path) = magic_name(prefix, root, token.as_os_str())?;
        if !conflicted.contains(&path) {
            return None;
        }
        found.push((magic, String::from_utf8_lossy(&path).into_owned()));
    }
    return if found.is_empty() { None } else { Some(found) };
}

/// What a command line that names paths came to,
/// against the conflicted set.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SideDecision {
    /// Every path is a conflicted file:
    /// the repository-relative paths,
    /// in the order named,
    /// each once.
    Allow(Vec<Vec<u8>>),
    /// At least one token is not a conflicted file;
    /// these are the tokens,
    /// as typed.
    Refuse(Vec<OsString>),
}

/// What:
///  Whether every token names a conflicted file.
/// Why:
///   A pathspec that mixes conflicted and clean paths is refused whole,
/// and the refusal names the clean ones,
/// so a
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
