//! What:
//!  A path resolved the way Git's own `realpath` resolves it (`abspath.c`, Git 2.56.0),
//!  and two paths compared the way Git compares them (`fspathcmp` in `dir.c`).
//! Why:
//!  `git worktree remove` picks the worktree its operand names by resolving the operand to a real path
//!  and comparing that with every worktree of the repository's list resolved the same way
//!  (`find_worktree_by_path` in `worktree.c`),
//!  so a guard that resolves the operand another way names nothing where Git names a worktree
//!  and stands down on a removal Git then makes
//!  (a review of `528279d099`, of `9e6c8619a9` and of `e615fb599b`, each naming this).
//!  Four rules of that resolution are where a plain `realpath(3)` differs, and each one decided a shape a review
//!  found the guard standing down on:
//!  a last component that is not there is kept as it was spelled instead of failing the resolution;
//!  a `..` component removes the last component resolved so far instead of being resolved by the kernel,
//!  so it never enters a symbolic link it names;
//!  a separator after a last component that is not there fails the resolution;
//!  and more than thirty-two links in one path fail it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const real = gitRealPath(p);           // the path Git would name, or undefined where Git's own fails
//! const same = theSamePath(one, other, foldCase); // Git's fspathcmp under core.ignorecase
//! ```

use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};

/// How many symbolic links one path may name before Git refuses it (`MAXSYMLINKS` in `abspath.c`).
const MAX_SYMLINKS: usize = 32;

/// What:
///  Where a path's root ends:
///  one byte for an absolute path, whose first byte is its root,
///  and none for a relative one (`offset_1st_component` in Git's `path.c` on a system with one root).
/// Why:
///   Git never removes the root itself,
///  so `..` at `/` leaves `/`,
///  and both loops of `strip_last_component` stop at this byte.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const rootLength = (path: Uint8Array) => path[0] === 0x2f ? 1 : 0;
/// ```
fn root_length(path: &[u8]) -> usize {
    return usize::from(path.starts_with(b"/"));
}

/// What:
///  `path` without its last component,
///  or unchanged when it is only a root.
/// Why:
///   A `..` component and a relative symbolic link both take the path back one component
///  (`strip_last_component` in Git's `abspath.c`),
///  and Git does that on the bytes it has resolved so far rather than by asking the kernel,
///  which is why a `..` after a symbolic link leaves the link's own directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stripLastComponent(path: Uint8Array): void;
/// ```
fn strip_last_component(path: &mut Vec<u8>) {
    let bound: usize = root_length(path);
    let mut length: usize = path.len();
    while length > bound && path[length - 1] != b'/' {
        length -= 1;
    }
    // Git skips a run of separators before the component, so a path ending in one still loses its component.
    while length > bound && path[length - 1] == b'/' {
        length -= 1;
    }
    path.truncate(length);
}

/// What:
///  `path` with `component` appended, a separator added first unless the path already ends in one.
/// Why:
///   Git appends each component to the path it has resolved so far the same way
///  (`strbuf_realpath_1` in `abspath.c`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pushComponent(path: Uint8Array, component: Uint8Array): void;
/// ```
fn push_component(path: &mut Vec<u8>, component: &[u8]) {
    if !path.ends_with(b"/") {
        path.push(b'/');
    }
    path.extend_from_slice(component);
}

/// What:
///  The path Git names when it resolves `spelled`,
///  or nothing where Git's own resolution fails.
/// Why:
///   This is Git's `real_pathdup(path, 0)`:
///  a relative path starts at the directory the command runs in,
///  `.` components are dropped,
///  `..` removes the last component,
///  every other component is looked at with `lstat` and followed when it is a link,
///  and only the last component may be missing, which is then kept as it was spelled.
///  A last component that is missing while a separator follows it fails,
///  and so does a component before the last that is missing.
///  Git's `find_worktree_by_path` compares this answer with every registered path,
///  and answers no worktree when this resolution gives nothing,
///  which is why a guard reproducing it names the same worktree Git names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function gitRealPath(spelled: string): string | undefined;
/// ```
pub fn git_real_path(spelled: &Path) -> Option<PathBuf> {
    let bytes: &[u8] = spelled.as_os_str().as_bytes();
    if bytes.is_empty() {
        return None;
    }
    // The root part: an absolute path starts at its own root, a relative one at this program's directory.
    let mut resolved: Vec<u8> = Vec::new();
    let mut remaining: Vec<u8> = Vec::new();
    if bytes.starts_with(b"/") {
        resolved.push(b'/');
        remaining.extend_from_slice(&bytes[1..]);
    } else {
        resolved = std::env::current_dir().ok()?.into_os_string().into_vec();
        remaining.extend_from_slice(bytes);
    }
    let mut links: usize = 0;
    // The byte of the component being read; `remaining` from here on is what Git calls `remaining`.
    let mut cursor: usize = 0;
    while cursor < remaining.len() {
        while cursor < remaining.len() && remaining[cursor] == b'/' {
            cursor += 1;
        }
        let start: usize = cursor;
        while cursor < remaining.len() && remaining[cursor] != b'/' {
            cursor += 1;
        }
        let component: Vec<u8> = remaining[start..cursor].to_vec();
        // A run of separators, a `.`, and an empty tail name no component of their own.
        if component.is_empty() || component == b"." {
            continue;
        }
        if component == b".." {
            strip_last_component(&mut resolved);
            continue;
        }
        push_component(&mut resolved, component.as_slice());
        let looked_at: &Path = Path::new(OsStr::from_bytes(resolved.as_slice()));
        match std::fs::symlink_metadata(looked_at) {
            Err(error) => {
                // Git keeps a last component that is not there and fails on any other, and fails on a separator after it.
                if error.kind() != std::io::ErrorKind::NotFound || cursor < remaining.len() {
                    return None;
                }
            }
            Ok(metadata) => {
                if !metadata.file_type().is_symlink() {
                    continue;
                }
                links += 1;
                if links > MAX_SYMLINKS {
                    return None;
                }
                let target: Vec<u8> = std::fs::read_link(looked_at).ok()?.into_os_string().into_vec();
                let tail: Vec<u8> = remaining[cursor..].to_vec();
                let mut next: Vec<u8> = Vec::with_capacity(target.len() + tail.len() + 1);
                if target.starts_with(b"/") {
                    // An absolute link starts again at the root it names.
                    resolved.clear();
                    resolved.push(b'/');
                    next.extend_from_slice(&target[1..]);
                } else {
                    // A relative link stands where the last component stood.
                    strip_last_component(&mut resolved);
                    next.extend_from_slice(target.as_slice());
                }
                if !tail.is_empty() {
                    next.push(b'/');
                    next.extend_from_slice(tail.as_slice());
                }
                remaining = next;
                cursor = 0;
            }
        }
    }
    return Some(PathBuf::from(OsString::from_vec(resolved)));
}

/// What:
///  Whether Git reads two paths as the same one,
///  given whether the repository folds case.
/// Why:
///   Git asks that question with `fspathcmp` in `dir.c`,
///  which is `strcasecmp` exactly when `core.ignorecase` is set and `strcmp` otherwise,
///  so the case-differing name a person types is the same worktree only when the repository says so.
///  The comparison is over ASCII letters alone,
///  which is what `strcasecmp` does in the locales Git runs in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theSamePath(one: string, other: string, foldCase: boolean): boolean;
/// ```
pub fn the_same_path(one: &Path, other: &Path, fold_case: bool) -> bool {
    let left: &[u8] = one.as_os_str().as_bytes();
    let right: &[u8] = other.as_os_str().as_bytes();
    return match fold_case {
        true => left.eq_ignore_ascii_case(right),
        false => left == right,
    };
}

/// The controls of this resolution stay out of the release executable.
#[cfg(test)]
#[path = "worktree_path_as_git_resolves_it_tests.rs"]
mod tests;
