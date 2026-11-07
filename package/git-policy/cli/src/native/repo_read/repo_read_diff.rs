//! What:
//!  The difference between two sorted lists of tree-like entries (a flattened tree,
//! an index),
//! in Git's raw and
//!  name-only spellings,
//! and the pathspec forms and filters the wrapper's own queries use.
//! Why:
//!  `diff-index --cached`,
//! `diff --cached` and `diff-tree` print exactly these lines for the plain cases;
//!  every case where Git's output depends on more than the two lists (a file that became a directory,
//! a gitlink,
//!  a possible rename) returns `None`,
//! so Git answers it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const changes = diffSorted(oldEntries, newEntries); // undefined: ask Git
//! const text = rawLines(changes) // ":100644 100644 <old> <new> M\0path\0"
//! ```

#![cfg(unix)]

use crate::repo_read_tree::{FlatEntry, MODE_GITLINK};

/// One changed path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Change {
    /// The path.
    pub path: Vec<u8>,
    /// The old mode,
    /// 0 for an added path.
    pub old_mode: u32,
    /// The new mode,
    /// 0 for a deleted path.
    pub new_mode: u32,
    /// The old object name,
    /// empty for an added path.
    pub old_id: Vec<u8>,
    /// The new object name,
    /// empty for a deleted path.
    pub new_id: Vec<u8>,
    /// The status letter:
    /// `A`,
    /// `D`,
    /// `M` or `T`.
    pub status: u8,
}

/// The file type bits of a mode.
fn type_of(mode: u32) -> u32 {
    return mode & 0o170_000;
}

/// Whether some entry of the sorted `list` lies below the directory `path/`.
fn has_entry_below(list: &[FlatEntry], path: &[u8]) -> bool {
    let mut prefix: Vec<u8> = path.to_vec();
    prefix.push(b'/');
    // A binary search over the sorted list, the same split `partition_point` finds.
    let (mut low, mut high): (usize, usize) = (0, list.len());
    while low < high {
        let middle: usize = low + (high - low) / 2;
        if list[middle].path.as_slice() < prefix.as_slice() {
            low = middle + 1;
        } else {
            high = middle;
        }
    }
    return match list.get(low) {
        Some(entry) => entry.path.starts_with(prefix.as_slice()),
        None => false,
    };
}

/// What:
///  The changes from `old` to `new`,
/// both sorted by path;
///  `None` when a path is a file on one side and a directory on the other or when a gitlink is involved.
/// Why:
///   Those are the cases whose ordering or spelling Git decides by more than the lists.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function diffSorted(oldList: FlatEntry[], newList: FlatEntry[]): Change[] | undefined;
/// ```
pub fn diff_sorted(old: &[FlatEntry], new: &[FlatEntry]) -> Option<Vec<Change>> {
    let mut changes: Vec<Change> = Vec::new();
    let (mut left, mut right): (usize, usize) = (0, 0);
    while left < old.len() || right < new.len() {
        let order = match (old.get(left), new.get(right)) {
            (Some(before), Some(after)) => before.path.cmp(&after.path),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, _) => std::cmp::Ordering::Greater,
        };
        match order {
            std::cmp::Ordering::Less => {
                let gone: &FlatEntry = &old[left];
                if has_entry_below(new, gone.path.as_slice()) {
                    return None;
                }
                changes.push(Change { path: gone.path.clone(), old_mode: gone.mode, new_mode: 0, old_id: gone.id.clone(), new_id: Vec::new(), status: b'D' });
                left += 1;
            }
            std::cmp::Ordering::Greater => {
                let added: &FlatEntry = &new[right];
                if has_entry_below(old, added.path.as_slice()) {
                    return None;
                }
                changes.push(Change { path: added.path.clone(), old_mode: 0, new_mode: added.mode, old_id: Vec::new(), new_id: added.id.clone(), status: b'A' });
                right += 1;
            }
            std::cmp::Ordering::Equal => {
                let (before, after): (&FlatEntry, &FlatEntry) = (&old[left], &new[right]);
                if before.mode != after.mode || before.id != after.id {
                    let status: u8 = if type_of(before.mode) == type_of(after.mode) { b'M' } else { b'T' };
                    changes.push(Change { path: before.path.clone(), old_mode: before.mode, new_mode: after.mode, old_id: before.id.clone(), new_id: after.id.clone(), status });
                }
                left += 1;
                right += 1;
            }
        }
    }
    for change in &changes {
        if type_of(change.old_mode) == MODE_GITLINK || type_of(change.new_mode) == MODE_GITLINK {
            return None;
        }
    }
    return Some(changes);
}

/// What:
///  The pathspecs a query names:
/// the plain relative form (at the top of the worktree only) or `:(top,literal)path`,
///  or `:/` for everything.
/// Why:
///   A glob or any other magic changes what matches,
/// so those return `None`;
/// matching a plain name is
///  "the path,
/// or anything below it".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PathFilter = { all: true } | { names: Uint8Array[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PathFilter {
    /// The names;
    /// empty means every path.
    names: Vec<Vec<u8>>,
}

impl PathFilter {
    /// Read the pathspec arguments;
    /// `at_top` says whether the command runs at the top of the worktree.
    pub fn parse(specs: &[&[u8]], at_top: bool) -> Option<PathFilter> {
        let mut names: Vec<Vec<u8>> = Vec::new();
        let mut everything: bool = false;
        for spec in specs {
            // `:/` selects everything, but every other spec is still checked: a later exclusion or error is Git's.
            if *spec == b":/" {
                everything = true;
                continue;
            }
            let (name, top_ok): (&[u8], bool) = match spec.strip_prefix(b":(top,literal)") {
                Some(rest) => (rest, true),
                None => (spec, at_top),
            };
            let plain: bool = !name.is_empty()
                && !name.starts_with(b":")
                && !name.starts_with(b"/")
                && !name.ends_with(b"/")
                && !name.windows(2).any(is_double_slash)
                && !name.split(is_slash).any(is_dot_part)
                && !name.iter().any(is_pattern_byte);
            if !plain || !top_ok {
                return None;
            }
            names.push(name.to_vec());
        }
        if everything {
            names.clear();
        }
        return Some(PathFilter { names });
    }

    /// Whether `path` is named by the filter.
    pub fn matches(&self, path: &[u8]) -> bool {
        if self.names.is_empty() {
            return true;
        }
        for name in &self.names {
            if path == name.as_slice() || (path.starts_with(name.as_slice()) && path.get(name.len()) == Some(&b'/')) {
                return true;
            }
        }
        return false;
    }
}

/// Whether the two bytes are `//`.
fn is_double_slash(pair: &[u8]) -> bool {
    return pair == b"//";
}

/// Whether `byte` is the path separator `/`.
fn is_slash(byte: &u8) -> bool {
    return *byte == b'/';
}

/// Whether a path component is `.` or `..`.
fn is_dot_part(part: &[u8]) -> bool {
    return part == b"." || part == b"..";
}

/// Whether `byte` is a wildcard or escape byte (`*`,
/// `?`,
/// `[`,
/// `\` or NUL) that a literal pathspec cannot hold.
fn is_pattern_byte(byte: &u8) -> bool {
    return matches!(byte, b'*' | b'?' | b'[' | b'\\' | 0);
}

/// Whether `byte` is one of the status letters `A`,
/// `D`,
/// `M`,
/// `T`.
fn is_status_letter(byte: u8) -> bool {
    return matches!(byte, b'A' | b'D' | b'M' | b'T');
}

/// The statuses `--diff-filter` keeps;
/// `None` keeps all.
/// Only the upper-case letters `A D M T` are understood.
pub fn parse_diff_filter(text: &str) -> Option<Vec<u8>> {
    if text.is_empty() || !text.bytes().all(is_status_letter) {
        return None;
    }
    return Some(text.bytes().collect());
}

/// Keep the changes the filter and statuses name.
pub fn select(changes: Vec<Change>, filter: &PathFilter, statuses: Option<&[u8]>) -> Vec<Change> {
    let mut selected: Vec<Change> = Vec::new();
    for change in changes {
        if !filter.matches(change.path.as_slice()) {
            continue;
        }
        let status_kept: bool = match statuses {
            Some(kept) => kept.contains(&change.status),
            None => true,
        };
        if status_kept {
            selected.push(change);
        }
    }
    return selected;
}

/// The `-z` raw lines:
/// `:<old mode> <new mode> <old id> <new id> <status>\0<path>\0`.
pub fn raw_lines(changes: &[Change], zero_id: &str) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    for change in changes {
        let old_id: String = if change.old_id.is_empty() { zero_id.to_string() } else { crate::repo_read_object::to_hex(change.old_id.as_slice()) };
        let new_id: String = if change.new_id.is_empty() { zero_id.to_string() } else { crate::repo_read_object::to_hex(change.new_id.as_slice()) };
        out.extend_from_slice(format!(":{:06o} {:06o} {old_id} {new_id} {}", change.old_mode, change.new_mode, char::from(change.status)).as_bytes());
        out.push(0);
        out.extend_from_slice(change.path.as_slice());
        out.push(0);
    }
    return out;
}

/// The `-z` name-only lines:
/// `<path>\0`.
pub fn name_lines(changes: &[Change]) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    for change in changes {
        out.extend_from_slice(change.path.as_slice());
        out.push(0);
    }
    return out;
}

#[cfg(test)]
#[path = "repo_read_diff_tests.rs"]
mod tests;
