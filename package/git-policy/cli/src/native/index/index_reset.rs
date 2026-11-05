//! What:
//!  `git reset <tree> -- <paths>` on an index held in memory: for every path the literal pathspecs name, the entries of the
//!  index become the entries of the tree, as `update_index_from_diff` in `builtin/reset.c` leaves them.
//! Why:
//!  A mixed reset with paths compares the tree with the index (`diff_cache` in `diff-lib.c`) and, for each difference,
//!  removes the index entry (`remove_file_from_index`) or adds an entry made from the tree's mode and object name with
//!  all-zero stat data (`make_cache_entry`, `add_index_entry` in `read-cache.c`), so the next `git status` hashes the file.
//!  Every extension a change touches follows Git: the cache-tree is invalidated, the untracked cache is cleared for the
//!  path, removed unmerged entries go to resolve-undo, and the fsmonitor bitmap lists the new entries as dirty. A form whose
//!  result depends on more than that (a file-directory conflict between the tree and the index, a name Git refuses, a racily
//!  clean entry that Git would hash from the worktree) returns `None`, so Git answers.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const done: boolean = resetPaths(index, names, treeEntries, settings, timestampSeconds); // false: ask Git
//! ```

use crate::index_byte_search::is_slash;
use crate::index_full::{FullEntry, FullIndex, SKIP_WORKTREE};
use crate::index_settings::{IndexSettings, UntrackedMode};
use crate::index_stat::StatData;
use crate::repo_read_tree::{FlatEntry, MODE_GITLINK};

/// The mode a regular file with an executable bit gets in the index.
const MODE_EXECUTABLE: u32 = 0o100_755;

/// The mode of a regular file in the index.
const MODE_FILE: u32 = 0o100_644;

/// The mode of a symbolic link.
const MODE_LINK: u32 = 0o120_000;

/// What `create_ce_mode` makes of a tree entry's mode.
fn index_mode(mode: u32) -> u32 {
    if mode & 0o170_000 == MODE_LINK {
        return MODE_LINK;
    }
    if mode & 0o170_000 == 0o040_000 || mode & 0o170_000 == MODE_GITLINK {
        return MODE_GITLINK;
    }
    return if mode & 0o100 != 0 { MODE_EXECUTABLE } else { MODE_FILE };
}

/// Whether `byte` is a space or a dot, the two characters NTFS drops from the end of a name.
fn is_space_or_dot(byte: &u8) -> bool {
    return *byte == b' ' || *byte == b'.';
}

/// Whether `rest` is only the spaces and dots NTFS drops from the end of a name.
fn only_spaces_and_dots(rest: &[u8]) -> bool {
    return rest.iter().all(is_space_or_dot);
}

/// The byte of `name` at `index`; a position past the end reads as the terminating NUL, as Git's string would.
fn byte_at(name: &[u8], index: usize) -> u8 {
    return name.get(index).copied().unwrap_or(0);
}

/// Whether `name` from `start` on is only spaces and periods up to its end or a `:` that opens a stream.
fn spaces_and_periods(name: &[u8], start: usize) -> bool {
    for index in start.. {
        let byte: u8 = byte_at(name, index);
        if byte == 0 || byte == b':' {
            return true;
        }
        if byte != b' ' && byte != b'.' {
            return false;
        }
    }
    return false;
}

/// Whether the two bytes are the start of a UTF-8 sequence for one of the ignorable characters U+2000 to U+206F.
fn is_ignorable_pair(pair: &[u8]) -> bool {
    return pair == [0xe2, 0x80] || pair == [0xe2, 0x81];
}

/// Whether the three bytes are the UTF-8 form of the ignorable character U+FEFF.
fn is_ignorable_triple(triple: &[u8]) -> bool {
    return triple == [0xef, 0xbb, 0xbf];
}

/// What:
///  Git's `is_ntfs_dotgitmodules` (`path.c:1451-1522`, `is_ntfs_dot_generic` with the name `gitmodules` and the hashed short-name
///  prefix `gi7eba`) for one path component: `.gitmodules`, its short form `gitmod~1` to `gitmod~4`, and the fall-back short name
///  NTFS makes from the hash (`GI7EBA~1`), each followed only by spaces and dots or a stream.
/// Why:
///   A link with any of these names is refused by Git with the NTFS protection on, so the update must decline it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refused: boolean = isNtfsDotGitmodules(component);
/// ```
fn is_ntfs_dotgitmodules(name: &[u8]) -> bool {
    if byte_at(name, 0) == b'.' && name.len() > 10 && name[1..11].eq_ignore_ascii_case(b"gitmodules") {
        return spaces_and_periods(name, 11);
    }
    if name.len() > 5 && name[..6].eq_ignore_ascii_case(b"gitmod") && byte_at(name, 6) == b'~' && (b'1'..=b'4').contains(&byte_at(name, 7)) {
        return spaces_and_periods(name, 8);
    }
    let (mut index, mut saw_tilde): (usize, bool) = (0, false);
    while index < 8 {
        let byte: u8 = byte_at(name, index);
        if byte == 0 {
            return false;
        } else if saw_tilde {
            if !byte.is_ascii_digit() {
                return false;
            }
        } else if byte == b'~' {
            index += 1;
            if !(b'1'..=b'9').contains(&byte_at(name, index)) {
                return false;
            }
            saw_tilde = true;
        } else if index >= 6 || byte & 0x80 != 0 || byte.to_ascii_lowercase() != b"gi7eba"[index] {
            return false;
        }
        index += 1;
    }
    return spaces_and_periods(name, 8);
}

/// What:
///  Whether one path component is one Git's name checks (`verify_path_internal` in `read-cache.c` with `core.protectNTFS`,
///  which is on by default, and `core.protectHFS`) cannot refuse or alias to a refused name.
/// Why:
///   Git refuses `.`, `..` and `.git` in any case; with the protections on it also refuses the spellings a file system folds
///  to `.git` (trailing spaces and dots, the `::$INDEX_ALLOCATION` stream, `git~1`, the ignorable characters of a case-folding
///  disk) and, for a link, the same spellings of `.gitmodules` (`is_ntfs_dotgit`, `is_ntfs_dotgitmodules`, `is_hfs_dotgit`,
///  `is_hfs_dotgitmodules` in `path.c` and `utf8.c`). A backslash and a colon separate or open a stream on those disks. This
///  declines every such spelling whatever the settings say, which is a superset of what Git refuses with them on, so the
///  update never writes a name Git would not and Git answers the rest.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const plain = (component, isLink) => !/[\\:]/.test(component) && !aliasesDotGit(component) && !(isLink && aliasesDotGitmodules(component));
/// ```
fn component_is_plain(component: &[u8], is_link: bool) -> bool {
    let lowered: Vec<u8> = component.to_ascii_lowercase();
    let dot_git_like: bool = lowered.strip_prefix(b".git").is_some_and(only_spaces_and_dots);
    let dot_gitmodules_like: bool = is_link && (is_ntfs_dotgitmodules(component) || lowered.strip_prefix(b".gitmodules").is_some_and(only_spaces_and_dots) || lowered.starts_with(b"gitmod~"));
    // The ignorable characters of a case-folding disk: U+2000 to U+206F begin with E2 80 or E2 81, and U+FEFF is EF BB BF.
    let ignorable: bool = component.windows(2).any(is_ignorable_pair) || component.windows(3).any(is_ignorable_triple);
    let separators: bool = component.contains(&b'\\') || component.contains(&b':');
    return !component.is_empty() && component != b"." && component != b".." && !separators && !dot_git_like && !dot_gitmodules_like && !lowered.starts_with(b"git~") && !ignorable;
}

/// Whether Git could refuse the path or treat it specially on a file system that folds names; those paths are left to Git.
fn path_is_plain(path: &[u8], mode: u32) -> bool {
    if path.is_empty() {
        return false;
    }
    let is_link: bool = mode & 0o170_000 == MODE_LINK;
    for component in path.split(is_slash) {
        if !component_is_plain(component, is_link) {
            return false;
        }
    }
    return true;
}

/// Whether the path is one a literal pathspec `name` selects: the name itself or anything below it.
fn selected(path: &[u8], names: &[Vec<u8>]) -> bool {
    for name in names {
        if path == name.as_slice() || (path.starts_with(name.as_slice()) && path.get(name.len()) == Some(&b'/')) {
            return true;
        }
    }
    return false;
}

/// The position of the first entry whose path is not before `path`.
fn first_at(entries: &[FullEntry], path: &[u8]) -> usize {
    // A binary search over the sorted entries, the same split `partition_point` finds.
    let (mut low, mut high): (usize, usize) = (0, entries.len());
    while low < high {
        let middle: usize = low + (high - low) / 2;
        if entries[middle].path.as_slice() < path {
            low = middle + 1;
        } else {
            high = middle;
        }
    }
    return low;
}

/// Whether the entry at `position` exists and has exactly the path `path`.
fn has_path_at(entries: &[FullEntry], position: usize, path: &[u8]) -> bool {
    return match entries.get(position) {
        Some(entry) => entry.path.as_slice() == path,
        None => false,
    };
}

/// The entry of `tree` (sorted by path, one entry per path) with exactly the path `path`.
fn find_in_tree<'a>(tree: &'a [FlatEntry], path: &[u8]) -> Option<&'a FlatEntry> {
    let (mut low, mut high): (usize, usize) = (0, tree.len());
    while low < high {
        let middle: usize = low + (high - low) / 2;
        match tree[middle].path.as_slice().cmp(path) {
            std::cmp::Ordering::Less => low = middle + 1,
            std::cmp::Ordering::Greater => high = middle,
            std::cmp::Ordering::Equal => return Some(&tree[middle]),
        }
    }
    return None;
}

/// Whether the index holds an entry below the directory `path/` or at one of the directories above `path`.
fn has_directory_conflict(entries: &[FullEntry], path: &[u8]) -> bool {
    let mut below: Vec<u8> = path.to_vec();
    below.push(b'/');
    let start: usize = first_at(entries, below.as_slice());
    if let Some(entry) = entries.get(start)
        && entry.path.starts_with(below.as_slice())
    {
        return true;
    }
    for (slash, byte) in path.iter().enumerate() {
        if *byte != b'/' {
            continue;
        }
        let above: &[u8] = &path[..slash];
        let at: usize = first_at(entries, above);
        if has_path_at(entries, at, above) {
            return true;
        }
    }
    return false;
}

/// Everything the changes to one index need besides the index.
struct Edit<'a> {
    /// The index being changed.
    index: &'a mut FullIndex,
    /// What Git's configuration makes of the extensions.
    settings: IndexSettings,
}

impl Edit<'_> {
    /// Clear the untracked cache for `path`, if there is one (`untracked_cache_invalidate_path`).
    fn forget_untracked(&mut self, path: &[u8]) {
        if let Some(cache) = self.index.untracked.as_mut() {
            cache.invalidate_path(path);
        }
    }

    /// Take the entries named `path` out of the index, remembering unmerged ones for resolve-undo (`remove_index_entry_at`).
    fn take_entries(&mut self, path: &[u8]) {
        let start: usize = first_at(self.index.entries.as_slice(), path);
        while has_path_at(self.index.entries.as_slice(), start, path) {
            let gone: FullEntry = self.index.entries.remove(start);
            if gone.stage() != 0 {
                self.index.resolve_undo.get_or_insert_with(Default::default).record(gone.path.as_slice(), gone.stage(), gone.mode, gone.id.as_slice());
            }
        }
    }

    /// `remove_file_from_index`.
    fn remove(&mut self, path: &[u8]) {
        if let Some(tree) = self.index.cache_tree.as_mut() {
            tree.invalidate(path);
        }
        self.forget_untracked(path);
        self.take_entries(path);
    }

    /// `add_index_entry` with the add and replace options, for an entry made from the tree.
    fn put(&mut self, wanted: &FlatEntry) {
        let path: &[u8] = wanted.path.as_slice();
        if let Some(tree) = self.index.cache_tree.as_mut() {
            tree.invalidate(path);
        }
        let start: usize = first_at(self.index.entries.as_slice(), path);
        let mut existing: Option<usize> = None;
        if let Some(entry) = self.index.entries.get(start)
            && entry.path.as_slice() == path
            && entry.stage() == 0
        {
            existing = Some(start);
        }
        // An entry that was outside the checkout stays outside it.
        let skip: u16 = match existing {
            Some(at) => self.index.entries[at].extended & SKIP_WORKTREE,
            None => 0,
        };
        let made: FullEntry = FullEntry {
            stat: StatData::default(),
            mode: index_mode(wanted.mode),
            id: wanted.id.clone(),
            flags: 0,
            extended: skip,
            path: wanted.path.clone(),
            monitor_valid: false,
        };
        if let Some(at) = existing {
            self.index.entries[at] = made;
            // Replacing an entry clears the untracked cache only when fsmonitor is on (`mark_fsmonitor_invalid`).
            if self.settings.monitor {
                self.forget_untracked(path);
            }
            return;
        }
        self.forget_untracked(path);
        // A resolved entry replaces every unmerged entry of the path.
        self.take_entries(path);
        let at: usize = first_at(self.index.entries.as_slice(), path);
        self.index.entries.insert(at, made);
    }
}

/// What the reader of the extensions must settle before the entries change: fsmonitor and the untracked cache as Git's read
/// leaves them. `None` for a combination this update does not reproduce.
fn prepare_extensions(index: &mut FullIndex, settings: IndexSettings) -> Option<()> {
    if settings.monitor {
        let dirty: Vec<usize> = index.fsmonitor.as_ref()?.dirty.clone();
        for position in &dirty {
            if *position >= index.entries.len() {
                return None;
            }
        }
        for entry in &mut index.entries {
            entry.monitor_valid = entry.mode != MODE_GITLINK;
        }
        for position in dirty {
            index.entries[position].monitor_valid = false;
        }
    } else {
        index.fsmonitor = None;
    }
    match settings.untracked {
        UntrackedMode::Keep => {}
        UntrackedMode::Write => {
            index.untracked.as_ref()?;
        }
        UntrackedMode::Remove => index.untracked = None,
    }
    return Some(());
}

/// What:
///  Make the entries of `names` (and below) equal to `tree`'s, as `git reset <tree> -- <names>` does; `tree` holds the
///  tree's entries under those names in path order. `None` when the result is not the plain case described above, with the
///  index then in an unspecified state.
/// Why:
///   `timestamp_seconds` is the index file's modification time: an entry that stays and was recorded in that second or later
///  is racily clean, and Git would compare it with the worktree while writing, which this update does not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resetPaths(index: FullIndex, names: Uint8Array[], tree: FlatEntry[], settings: IndexSettings, timestampSeconds: number): void | undefined;
/// ```
pub fn reset_paths(index: &mut FullIndex, names: &[Vec<u8>], tree: &[FlatEntry], settings: IndexSettings, timestamp_seconds: u32) -> Option<()> {
    for name in names {
        if !path_is_plain(name.as_slice(), 0) || name.last() == Some(&b'/') {
            return None;
        }
    }
    prepare_extensions(index, settings)?;
    // The paths that differ, in path order: each is in the tree, the index or both.
    let mut paths: Vec<Vec<u8>> = Vec::new();
    for entry in tree {
        paths.push(entry.path.clone());
    }
    for entry in &index.entries {
        if selected(entry.path.as_slice(), names) {
            paths.push(entry.path.clone());
        }
    }
    paths.sort();
    paths.dedup();
    let mut edit: Edit<'_> = Edit { index, settings };
    for path in paths {
        let wanted: Option<&FlatEntry> = find_in_tree(tree, path.as_slice());
        let start: usize = first_at(edit.index.entries.as_slice(), path.as_slice());
        let mut held: Vec<&FullEntry> = Vec::new();
        for entry in &edit.index.entries[start..] {
            if entry.path != path {
                break;
            }
            held.push(entry);
        }
        match wanted {
            None => edit.remove(path.as_slice()),
            Some(entry) => {
                // The same mode and object name is no difference, whatever the flags of the index entry say.
                let same: bool = matches!(held.as_slice(), [only] if only.stage() == 0 && only.mode == index_mode(entry.mode) && only.id == entry.id);
                if same {
                    continue;
                }
                if !path_is_plain(entry.path.as_slice(), index_mode(entry.mode)) || has_directory_conflict(edit.index.entries.as_slice(), entry.path.as_slice()) {
                    return None;
                }
                edit.put(entry);
            }
        }
    }
    // Git hashes a racily clean entry from the worktree when it writes the index; that is left to Git.
    for entry in &index.entries {
        if entry.mode != MODE_GITLINK && timestamp_seconds <= entry.stat.mtime_sec {
            return None;
        }
    }
    return Some(());
}

#[cfg(test)]
#[path = "index_reset_tests.rs"]
mod tests;
