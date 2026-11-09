//! What:
//!  The names a tree entry may carry into the index:
//! one path component Git's name checks with `core.protectNTFS` and `core.protectHFS` on
//!  cannot refuse or alias to a refused name,
//! and a whole path whose every component is such a name.
//! Why:
//!   The in-process reset (`index_reset.rs`) writes entries Git would refuse otherwise,
//! and the spellings a file
//!  system folds to
//! `.git` or to `.gitmodules` are refused there while Git answers every other refusal.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const plain = pathIsPlain(path, mode); // false: leave the reset to Git
//! ```

use crate::index_byte_search::is_slash;

/// The mode of a symbolic link.
const MODE_LINK: u32 = 0o120_000;

/// Whether `byte` is a space or a dot,
/// the two characters NTFS drops from the end of a name.
fn is_space_or_dot(byte: &u8) -> bool {
    return *byte == b' ' || *byte == b'.';
}

/// Whether `rest` is only the spaces and dots NTFS drops from the end of a name.
fn only_spaces_and_dots(rest: &[u8]) -> bool {
    return rest.iter().all(is_space_or_dot);
}

/// The byte of `name` at `index`;
/// a position past the end reads as the terminating NUL,
/// as Git's string would.
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
///  Git's `is_ntfs_dotgitmodules` (`path.c:1451-1522`,
/// `is_ntfs_dot_generic` with the name `gitmodules` and the hashed short-name
///  prefix `gi7eba`) for one path component:
/// `.gitmodules`,
/// its short form `gitmod~1` to `gitmod~4`,
/// and the fall-back short name
///  NTFS makes from the hash (`GI7EBA~1`),
/// each followed only by spaces and dots or a stream.
/// Why:
///   A link with any of these names is refused by Git with the NTFS protection on,
/// so the update must decline it.
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
///  which is on by default,
/// and `core.protectHFS`) cannot refuse or alias to a refused name.
/// Why:
///   Git refuses `.`,
/// `..` and `.git` in any case;
/// with the protections on it also refuses the spellings a file system folds
///  to `.git` (trailing spaces and dots,
/// the `::$INDEX_ALLOCATION` stream,
/// `git~1`,
/// the ignorable characters of a case-folding
///  disk) and,
/// for a link,
/// the same spellings of `.gitmodules` (`is_ntfs_dotgit`,
/// `is_ntfs_dotgitmodules`,
/// `is_hfs_dotgit`,
///  `is_hfs_dotgitmodules` in `path.c` and `utf8.c`).
/// A backslash and a colon separate or open a stream on those disks.
/// This
///  declines every such spelling whatever the settings say,
/// which is a superset of what Git refuses with them on,
/// so the
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

/// Whether Git could refuse the path or treat it specially on a file system that folds names;
/// those paths are left to Git.
pub fn path_is_plain(path: &[u8], mode: u32) -> bool {
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

/// Controls for the name rules stay out of the release executable.
#[cfg(test)]
#[path = "index_reset_path_names_tests.rs"]
mod tests;
