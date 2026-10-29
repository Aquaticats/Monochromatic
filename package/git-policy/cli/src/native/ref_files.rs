//! What:
//!  Reads of the files ref storage that answer a question without a Git process,
//!  and refuse to answer anything they cannot prove equal to Git's answer.
//! Why:
//!  `git symbolic-ref --quiet HEAD` only reads the worktree's `HEAD` file and,
//!  for a branch, checks that the branch is not itself a symbolic ref.
//!  A strict reader of the plain spellings saves a process on every commit,
//!  and every other spelling (a different ref name shape, a link,
//!  a symbolic chain,
//!  reftable storage) returns `None`,
//!  which sends the caller to Git.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const head = readHeadFile(layout); // undefined means "ask Git"
//! ```

/// Bounded no-follow reading of a regular file.
use super::private_storage::read_regular_file;
/// The ref storage backend and the symbolic `HEAD` answer.
use super::transaction_journal::{RefFormat, SymbolicHead};
/// Borrowed filesystem paths.
use std::path::Path;

/// The longest `HEAD` file this reader looks at;
///  a real one is a short line.
const HEAD_LIMIT: u64 = 512;

/// The longest loose branch file this reader looks at;
///  a real one is an object name and a newline.
const LOOSE_REF_LIMIT: u64 = 256;

/// What:
///  Where one worktree's ref files live and what format they are in.
/// Why:
///   The per-worktree `HEAD` is in the Git directory,
///  the branches are in the common directory,
///  and the length of an object name depends on the object format.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RefFilesLayout = { gitDir: string; commonDir: string; refFormat: 'files' | 'reftable'; objectFormat: string };
/// ```
#[derive(Clone, Copy, Debug)]
pub struct RefFilesLayout<'a> {
    /// Absolute Git directory of the owning worktree.
    pub git_dir: &'a Path,
    /// Absolute common Git directory.
    pub common_dir: &'a Path,
    /// Ref storage backend.
    pub ref_format: RefFormat,
    /// Object hash algorithm name as `rev-parse --show-object-format` spells it.
    pub object_format: &'a str,
}

/// What:
///  The number of hexadecimal digits of an object name under an object format,
///  or `None` for a format this reader does not know.
/// Why:
///   Git accepts a detached `HEAD` only with the repository's own name length.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ({ sha1: 40, sha256: 64 } as const)[objectFormat]
/// ```
fn hex_length(object_format: &str) -> Option<usize> {
    return match object_format {
        "sha1" => Some(40),
        "sha256" => Some(64),
        _ => None,
    };
}

/// What:
///  Whether `text` is exactly one object name of `length` lowercase hexadecimal digits that is not the null name.
/// Why:
///   Git also accepts uppercase digits and trailing spaces;
///  refusing them only costs a fallback,
///  and the null name has a meaning of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new RegExp(`^[0-9a-f]{${length}}$`).test(text) && !/^0+$/.test(text)
/// ```
fn is_object_name(text: &str, length: usize) -> bool {
    if text.len() != length {
        return false;
    }
    let all_hex: bool = text.bytes().all(|byte| { return matches!(byte, b'0'..=b'9' | b'a'..=b'f'); });
    let all_zero: bool = text.bytes().all(|byte| { return byte == b'0'; });
    return all_hex && !all_zero;
}

/// What:
///  Whether `name` is a branch ref name of a plain, always valid shape:
///  `refs/heads/` and then components of ASCII letters,
///  digits, `.`, `_` and `-`.
/// Why:
///   Git refuses names with a component that starts with `.` or ends with `.lock`,
///  a `..`, an empty component or a trailing `.`;
///  the plain alphabet needs no other rule,
///  and any other spelling is left to Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// /^refs\/heads\/([A-Za-z0-9_-][A-Za-z0-9._-]*)(\/[A-Za-z0-9_-][A-Za-z0-9._-]*)*$/.test(name) && ...
/// ```
pub fn is_plain_branch_name(name: &str) -> bool {
    let Some(rest) = name.strip_prefix("refs/heads/") else {
        return false;
    };
    if rest.is_empty() || rest.ends_with('.') || rest.contains("..") {
        return false;
    }
    for component in rest.split('/') {
        if component.is_empty() || component.starts_with('.') || component.ends_with(".lock") {
            return false;
        }
        let plain: bool = component
            .bytes()
            .all(|byte| { return byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'); });
        if !plain {
            return false;
        }
    }
    return true;
}

/// What:
///  Whether the loose file of a branch, if there is one, is a plain object name.
/// Why:
///   `symbolic-ref` follows a branch that is itself a symbolic ref and fails on a broken one;
///  a missing loose file is an unborn or packed branch,
///  and packed refs cannot be symbolic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = await readFile(join(commonDir, name)).catch(notFound); return text === undefined || isObjectName(text.trimEnd('\n'));
/// ```
fn loose_branch_is_plain(layout: &RefFilesLayout<'_>, name: &str, length: usize) -> bool {
    let path = layout.common_dir.join(name);
    let metadata: std::fs::Metadata = match std::fs::symlink_metadata(&path) {
        Ok(found) => found,
        Err(error) => return error.kind() == std::io::ErrorKind::NotFound,
    };
    if !metadata.is_file() {
        return false;
    }
    let Ok(bytes) = read_regular_file(path.as_path(), LOOSE_REF_LIMIT) else {
        return false;
    };
    let Ok(text) = std::str::from_utf8(bytes.as_slice()) else {
        return false;
    };
    let Some(line) = text.strip_suffix('\n') else {
        return false;
    };
    return is_object_name(line, length);
}

/// What:
///  `git symbolic-ref --quiet HEAD` answered from the worktree's `HEAD` file,
///  or `None` when the file is not one of the two plain spellings.
/// Why:
///   `ref: <plain branch>` and a lone object name are the only shapes the commit path meets;
///  the file is read afresh on every call,
///  so a landing that calls this under its lock sees the file as it is then.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = await readFile(join(gitDir, 'HEAD'), 'utf8'); // 'ref: refs/heads/x\n' or '<oid>\n'
/// ```
pub fn read_symbolic_head(layout: &RefFilesLayout<'_>) -> Option<SymbolicHead> {
    if layout.ref_format != RefFormat::Files {
        return None;
    }
    let length: usize = hex_length(layout.object_format)?;
    let bytes: Vec<u8> = read_regular_file(layout.git_dir.join("HEAD").as_path(), HEAD_LIMIT).ok()?;
    let text: &str = std::str::from_utf8(bytes.as_slice()).ok()?;
    let line: &str = text.strip_suffix('\n')?;
    if let Some(name) = line.strip_prefix("ref: ") {
        if is_plain_branch_name(name) && loose_branch_is_plain(layout, name, length) {
            return Some(SymbolicHead::Branch(String::from(name)));
        }
        return None;
    }
    if is_object_name(line, length) {
        return Some(SymbolicHead::Detached);
    }
    return None;
}

/// Files ref reader controls stay out of the release executable.
#[cfg(test)]
#[path = "ref_files_tests.rs"]
pub(crate) mod tests;
