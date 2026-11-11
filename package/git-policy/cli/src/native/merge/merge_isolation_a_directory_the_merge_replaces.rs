//! What:
//!  A directory standing where the merge puts a file,
//! and the words that stop the merge before anything is written.
//! Why:
//!   Git's own update of the checkout replaces such a directory with the merge's file,
//! and everything in it goes
//!  with it:
//! measured,
//! an untracked `a.txt/nested/file.txt` was gone after a merge that added `a.txt`,
//! and so was a
//!  *staged* `a.txt/nested/file.txt`,
//! with its index entry,
//! after a merge the wrapper reported as made.
//!  Nothing in the carry can hold a directory:
//! a capture refuses one,
//! and the merge's file and the person's directory
//!  cannot both stand at the same path.
//!  So the merge stops before the first byte is written,
//! and says which directory holds what for the
//!  person (`SPEC.md`,
//! "Merges and commits never fail",
//! "A merge whose paths hold the person's own work carries it forward",
//! and the owner's rule "never discard user data").
//!  That is the answer the recovery of a killed merge already gives such a directory,
//! so a directory the merge meets
//!  now is judged the same way before the merge runs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const inTheWay = aDirectoryTheMergeWouldReplace(context, root, head, target, touched);
//! if (inTheWay) return refused(1, theLine(reference, inTheWay));
//! ```

use crate::auto_push_dirty_paths::listed;
use crate::merge_isolation_carry_reads::changes_between;
use crate::merge_isolation_changes::{Change, the_directory_holds_only_what_the_merge_deletes, worktree_path};
use crate::transaction_git::GitContext;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};

/// A directory the merge would replace with a file,
/// and the entries below it,
/// as text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type InTheWay = { path: Uint8Array; entries: string[] };
/// ```
pub struct InTheWay {
    /// Path from the top of the worktree,
    /// exact bytes.
    pub path: Vec<u8>,
    /// Everything below it,
    /// named from the top of the worktree.
    pub entries: Vec<String>,
}

/// What:
///  The directory a merge would replace with a file,
/// when it holds anything the merge does not delete itself.
/// Why:
///   A path the merge changes is a path a merge of the two commits writes,
/// so a directory standing there is in the
///  merge's way;
/// it is the branch's own only when the merge deletes every file below it,
/// and otherwise it is
///  someone else's and nothing may be written over it.
///  A path the merge only deletes holds nothing to replace,
/// so a directory there is left where it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aDirectoryTheMergeWouldReplace(context, root, head, target, touched): InTheWay | null;
/// ```
pub fn a_directory_the_merge_would_replace(
    context: &GitContext,
    root: &Path,
    head: &str,
    target: &str,
    touched: &[Vec<u8>],
) -> Result<Option<InTheWay>, String> {
    let mut candidates: Vec<Vec<u8>> = Vec::new();
    for path in touched {
        let location: PathBuf = worktree_path(root, path.as_slice());
        if !is_a_directory(location.as_path()) {
            continue;
        }
        // An empty directory holds nothing of anyone's, and Git replaces it as it replaces a missing path.
        if the_entries_below(root, path.as_slice()).is_empty() {
            continue;
        }
        candidates.push(path.clone());
    }
    if candidates.is_empty() {
        return Ok(None);
    }
    let changes: Vec<Change> = changes_between(context, root, head, target)?;
    for path in candidates {
        let Some(change) = change_named(changes.as_slice(), path.as_slice()) else {
            continue;
        };
        // A path neither side holds after the merge is one the merge deletes, which leaves a directory standing.
        if !change.exists_after() {
            continue;
        }
        if the_directory_holds_only_what_the_merge_deletes(change, changes.as_slice(), root) {
            continue;
        }
        let entries: Vec<String> = the_entries_below(root, path.as_slice());
        return Ok(Some(InTheWay { path, entries }));
    }
    return Ok(None);
}

/// The change a merge makes to one path,
/// when the difference between its two commits names it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const changeNamed = (changes, path) => changes.find((one) => one.path === path);
/// ```
fn change_named<'a>(changes: &'a [Change], path: &[u8]) -> Option<&'a Change> {
    for change in changes {
        if change.path == path {
            return Some(change);
        }
    }
    return None;
}

/// Whether what stands at `location` is a directory.
///  A symbolic link to a directory is not one:
/// the link is a file of its own,
/// and the merge writes over it without losing anything in it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// symlinkMetadataSync(location).isDirectory();
/// ```
fn is_a_directory(location: &Path) -> bool {
    return std::fs::symlink_metadata(location).map(is_a_directory_entry).unwrap_or(false);
}

/// Whether one entry's own metadata says it is a directory.
///  A symbolic link is not one:
/// its metadata is the link's,
/// and a link to a directory is a file of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isADirectoryEntry = (found) => found.isDirectory();
/// ```
fn is_a_directory_entry(found: std::fs::Metadata) -> bool {
    return found.is_dir();
}

/// What:
///  Everything below one directory of the worktree,
/// named from the top of the worktree.
/// Why:
///   The line names what the merge would take,
/// and a person reads paths from the top of their checkout.
///  Every kind of entry is listed,
/// ignored files included,
/// because a merge that replaces the directory takes them all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theEntriesBelow(root, prefix): string[]; // walk every directory below, name each entry
/// ```
fn the_entries_below(root: &Path, prefix: &[u8]) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    let location: PathBuf = worktree_path(root, prefix);
    let listing: std::fs::ReadDir = match std::fs::read_dir(location.as_path()) {
        Ok(found) => found,
        Err(_) => return names,
    };
    for entry in listing.flatten() {
        let mut child: Vec<u8> = prefix.to_vec();
        child.push(b'/');
        child.extend_from_slice(entry.file_name().as_encoded_bytes());
        let below: PathBuf = location.join(entry.file_name());
        if is_a_directory(below.as_path()) {
            names.extend(the_entries_below(root, child.as_slice()));
            continue;
        }
        names.push(String::from_utf8_lossy(child.as_slice()).into_owned());
    }
    names.sort();
    return names;
}

/// The bytes of one entry's name,
/// which the list of names prints.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theBytesOf = (name) => new TextEncoder().encode(name);
/// ```
fn the_bytes_of(name: &String) -> Vec<u8> {
    return name.as_bytes().to_vec();
}

/// What:
///  The sentence a merge that would replace such a directory ends with.
/// Why:
///   The person must know which path is in the way,
/// what stands in it,
/// and that nothing of theirs was changed,
///  before they decide what to do with their files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `git merge ${reference}: ${path} stands as a directory that holds ${listed}, and the merge puts a file there, so the merge was not made...`;
/// ```
pub fn the_line(reference: &str, found: &InTheWay) -> String {
    let path: String = String::from_utf8_lossy(found.path.as_slice()).into_owned();
    let named: Vec<Vec<u8>> = found.entries.iter().map(the_bytes_of).collect();
    return format!(
        "monochromatic-git-policy-cli: git merge {reference}: {path} stands as a directory that holds {}, and this merge puts a file \
         at {path}, so the merge was not made and nothing in the worktree, the index or the branch changed, and no file of yours was \
         removed. Move those files somewhere else, or commit them, and run the merge again; they are not in the merge.",
        listed(named.as_slice())
    );
}
