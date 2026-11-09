//! What:
//!  The path vocabulary of the linked-worktree copy:
//!  the bytes of one repository-relative path,
//!  how its components are joined,
//!  split and compared,
//!  the validation every one passes before it touches the filesystem,
//! and the order two of them sort in.
//! Why:
//!   Every rule about a repository path reads its bytes,
//! and a module that keeps only these rules can be read and tested without the
//!  Git listing,
//! the walk or the records around it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // worktree-copy/repository-path.ts: the byte-level path rules of the copy.
//! ```

use crate::json_record::quote;
use crate::path_bytes::{os_from_path_bytes, path_message_text};
use crate::worktree_copy_error::WorktreeCopyError;
use std::cmp::Ordering;
use std::path::{Path, PathBuf};

///  Whether a byte separates the components of a repository path.
/// Why:
///   A repository path is the bytes Git reported,
/// so every rule about its components reads those bytes;
/// a name with any other byte,
/// and that includes a byte that is not
///  text,
/// stays one component.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const byte === 0x2f;
/// ```
pub fn is_separator_byte(byte: &u8) -> bool {
    return *byte == b'/';
}

/// What:
///  Whether a repository path equals or descends from another repository path.
/// Why:
///   Comparison is by whole components,
/// so `cache2` is not inside `cache`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (candidate === parent) || candidate.startsWith(`${parent}/`)
/// ```
pub fn repository_path_within(candidate: &[u8], parent: &[u8]) -> bool {
    // The exact path, or the parent followed by one separator, is the whole of the rule.
    if candidate == parent {
        return true;
    }
    return candidate.len() > parent.len()
        && candidate.starts_with(parent)
        && candidate[parent.len()] == b'/';
}

/// What:
///  The repository path of one entry below a directory's repository path.
/// Why:
///   The walk and the checks build every child path from its parent and the name the
/// filesystem listed,
/// and that name keeps every byte it has.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Buffer.concat([parent, Buffer.from('/'), name])
/// ```
pub fn join_repository_path(parent: &[u8], name: &[u8]) -> Vec<u8> {
    let mut joined: Vec<u8> = Vec::with_capacity(parent.len() + 1 + name.len());
    joined.extend_from_slice(parent);
    joined.push(b'/');
    joined.extend_from_slice(name);
    return joined;
}

/// What:
///  The repository path of one entry's parent directory,
/// or nothing for a path with no separator.
/// Why:
///   Installation creates the parents scaffolding beside the entries it writes,
/// and that scaffolding is named by the parent of an entry's own bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const at = path.lastIndexOf('/'); at < 0 ? undefined : path.subarray(0, at)
/// ```
pub fn parent_of_repository_path(path: &[u8]) -> Option<&[u8]> {
    let at: usize = path.iter().rposition(is_separator_byte)?;
    return Some(&path[..at]);
}

/// What:
///  The repository path as the text a person reads,
/// with each invalid sequence replaced by U+FFFD.
/// Why:
///   A diagnostic names the entry;
/// the exact bytes travel in the record and in the path used to act on it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// path.toString('utf8')
/// ```
pub fn shown_repository_path(path: &[u8]) -> String {
    return path_message_text(path);
}

/// What:
///  Whether a repository path component is empty,
/// `.` or `..`.
/// Why:
///   Such a component could leave the repository,
/// so a path with one is unsafe.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// component === '' || component === '.' || component === '..'
/// ```
fn is_unsafe_component(component: &[u8]) -> bool {
    return component.is_empty() || component == b"." || component == b"..";
}

/// What:
///  Whether a repository path is at or below any of the root paths.
/// Why:
///   A root already selected covers everything below it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// roots.some((root) => repositoryPathWithin(candidate, root))
/// ```
pub fn within_any_root(candidate: &[u8], roots: &[Vec<u8>]) -> bool {
    for root in roots {
        if repository_path_within(candidate, root.as_slice()) {
            return true;
        }
    }
    return false;
}

/// What:
///  Validates one slash-separated path that Git reported before it is joined to a filesystem root.
/// Why:
///   An empty,
/// absolute,
/// `.` or `..` component could select a path outside the worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function assertSafeRepositoryPath(repositoryPath: string): void;
/// ```
pub fn assert_safe_repository_path(repository_path: &[u8]) -> Result<(), WorktreeCopyError> {
    let unsafe_path: bool = repository_path.is_empty()
        || repository_path[0] == b'/'
        || repository_path
            .split(is_separator_byte)
            .any(is_unsafe_component);
    if unsafe_path {
        return Err(WorktreeCopyError(format!(
            "monochromatic-git-policy-cli: Git returned unsafe ignored path {}.",
            quote(shown_repository_path(repository_path).as_str())
        )));
    }
    return Ok(());
}

/// What:
///  The native path of one validated repository path under a filesystem root.
/// Why:
///   Every filesystem access of the copy goes through this join,
/// so the validation cannot be skipped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// join(root, ...repositoryPath.split('/'));
/// ```
pub fn filesystem_path(root: &Path, repository_path: &[u8]) -> Result<PathBuf, WorktreeCopyError> {
    assert_safe_repository_path(repository_path)?;
    let mut path: PathBuf = root.to_path_buf();
    for component in repository_path.split(is_separator_byte) {
        path.push(os_from_path_bytes(component));
    }
    return Ok(path);
}

/// What:
///  Sorts strings the way JavaScript's default `toSorted` does:
/// by UTF-16 code units.
/// Why:
///   Manifest and journal order must match the incumbent's,
/// which differs from byte order
///  only for supplementary-plane characters.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// [...values].toSorted()
/// ```
pub fn js_order(left: &str, right: &str) -> Ordering {
    return left.encode_utf16().cmp(right.encode_utf16());
}

/// What:
///  Order two names the way JavaScript's default sort does,
/// for a sort of owned names.
/// Why:
///   `sort_by` hands its function references to the items,
/// and a generic function over `AsRef<str>` takes them without a closure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// names.sort()
/// ```
pub fn by_js_order<Name: AsRef<str>>(left: &Name, right: &Name) -> std::cmp::Ordering {
    return js_order(left.as_ref(), right.as_ref());
}

/// What:
///  Orders two repository paths:
/// first by the UTF-16 code units of their text,
/// as JavaScript sorts names,
/// then by their bytes.
/// Why:
///   The stage and the source are walked twice and compared entry by entry,
/// so both walks must order the same names the same way.
/// A name that is text keeps the order the incumbent gave it;
/// a name that is not text sorts where its text does and
///  the bytes only decide between names whose text is equal,
/// which keeps the walk reproducible for names the incumbent never had.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// compareText(a) || compareBytes(a, b)
/// ```
/// What:
///  Orders two owned repository paths,
/// for a sort of names the walk collected.
/// Why:
///   `sort_by` hands its function references to the items,
/// so a named function takes them without a closure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// names.sort(compareRepositoryPaths)
/// ```
pub fn by_repository_path_order(left: &Vec<u8>, right: &Vec<u8>) -> Ordering {
    return repository_path_order(left.as_slice(), right.as_slice());
}

pub fn repository_path_order(left: &[u8], right: &[u8]) -> Ordering {
    let shown: Ordering = js_order(
        String::from_utf8_lossy(left).as_ref(),
        String::from_utf8_lossy(right).as_ref(),
    );
    if shown != Ordering::Equal {
        return shown;
    }
    return left.cmp(right);
}
