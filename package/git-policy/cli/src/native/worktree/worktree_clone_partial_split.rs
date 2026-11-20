//! What:
//!  Which tracked paths of a source with uncommitted changes a clone may still clone,
//! and which of them
//!  the new worktree's Git writes.
//! Why:
//!  The clone rung used to need a source whose files all equal its index and whose index equals the tree of
//!  `HEAD`;
//! a source with changes was left to Git's own checkout.
//! A path whose file its own index entry still
//!  proves is the same bytes Git's checkout would write,
//! so it can be cloned,
//! and only the paths a change reaches need Git.
//! The proof is Git's own fast path
//!  taken in process:
//! the entry's recorded size,
//! type,
//! mode,
//! inode and times are compared with the file,
//! and an entry Git
//!  would call racy,
//! or one whose recorded nanoseconds are zero,
//! proves nothing and is written instead,
//! which costs one
//!  write and never a wrong byte.
//! A path staged differently from `HEAD` carries another blob than the tree names,
//! so it is
//!  written too;
//! a path only the index knows is in no tree,
//!  and reaches no new worktree,
//! whether the index holds it as a staged addition or as an intent-to-add entry;
//! and a state a clone cannot be proven right for ends the rung with its reason.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const split = splitTheTree({ entries, tree, root, indexSeconds }); // throws the reason a clone cannot serve
//! ```

use crate::fs_beneath::{entry_metadata_at, open_root, plain_components};
use crate::fs_beneath_directory_chain::DirectoryChain;
use crate::repo_read_tree::{FlatEntry, MODE_GITLINK};
use crate::worktree_clean::{CleanRefusal, CleanRefusalKind, StatEntry, StatVerdict, metadata_verdict};
use crate::worktree_clean_stat_verdicts::ASSUME_VALID;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

/// The mode bits that say an entry is a link rather than a file.
const MODE_SYMLINK: u32 = 0o120_000;

/// The mode bits that say an entry is a file.
const MODE_REGULAR: u32 = 0o100_000;

/// The type bits of an entry's mode.
const MODE_TYPE: u32 = 0o170_000;

/// What:
///  The two sets one source with changes splits into.
/// Why:
///   The clones take the entries the proof settles,
/// and Git writes the rest,
/// so each path is made by
///  exactly one of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Split = { clones: number[]; writes: Uint8Array[] };
/// ```
#[derive(Debug, Eq, PartialEq)]
pub struct Split {
    /// The positions in the index of the entries a clone may take,
    /// in the index's order.
    pub clones: Vec<usize>,
    /// The paths of the tree nothing proves,
    /// which the new worktree's Git writes,
    /// in the tree's order.
    pub writes: Vec<Vec<u8>>,
}

/// What:
///  The refusal for a tree or index entry a clone cannot be proven right for,
/// in words.
/// Why:
///   A submodule,
/// a skip-worktree entry and an assume-unchanged entry are states the proof above never
///  judges,
/// and the rung ends with the reason instead of guessing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new Refusal('notPlain', `${shown} is a submodule`)
/// ```
fn not_plain(shown: &str, reason: &str) -> CleanRefusal {
    return CleanRefusal::new(CleanRefusalKind::NotPlain, format!("{shown} {reason}"));
}

/// What:
///  The entry's kind,
/// or the refusal when it is not a file or a link.
/// Why:
///   A clone makes a file or a link;
/// anything else in the tree is a kind this rung never writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function kindOf(mode: number, shown: string): 'file' | 'link'; // throws
/// ```
fn kind_of(mode: u32, shown: &str) -> Result<(), CleanRefusal> {
    return match mode & MODE_TYPE {
        MODE_REGULAR | MODE_SYMLINK => Ok(()),
        MODE_GITLINK => Err(not_plain(shown, "is a submodule")),
        _type => Err(not_plain(shown, "is neither a file nor a link")),
    };
}

/// What:
///  The refusal for an index entry whose flags say Git does not judge the file itself.
/// Why:
///   An unmerged entry names no bytes of its own,
/// an intent-to-add entry names none either,
/// and
///  skip-worktree and assume-unchanged tell Git to trust the entry without looking;
/// a clone of such a file could carry bytes the
///  tree does not name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function flagsRefusal(entry: StatEntry, shown: string): CleanRefusal | undefined;
/// ```
fn flags_refusal(entry: &StatEntry, shown: &str) -> Option<CleanRefusal> {
    if entry.flags & ASSUME_VALID != 0 {
        return Some(not_plain(shown, "is marked assume-unchanged"));
    }
    return None;
}

/// What:
///  Whether the entry's extended flags,
/// its stage and the kind of its mode let the proof judge the file at all.
/// Why:
///   `dirty_source` already refused every entry a reset cannot undo;
/// this rung judges index entries on its own,
/// so the same
///  shapes stop it here as well.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function judgeable(entry: StatEntry, shown: string): void; // throws
/// ```
fn judgeable(indexed: &crate::repo_read_index::IndexEntry, entry: &StatEntry, shown: &str) -> Result<(), CleanRefusal> {
    if indexed.stage != 0 {
        return Err(not_plain(shown, "is unmerged"));
    }
    if indexed.extended != 0 {
        return Err(not_plain(shown, "is an intent-to-add or skip-worktree entry"));
    }
    if let Some(refusal) = flags_refusal(entry, shown) {
        return Err(refusal);
    }
    return kind_of(entry.mode, shown);
}

/// What:
///  Whether the file at `path` below the held root is the one its entry recorded,
/// by the strict comparison the restore already trusts.
/// Why:
///   A file whose recorded stat data is its own to the nanosecond is the one instant the index was written
/// at,
/// so its bytes are the entry's,
/// whatever Git's own racy rule says about an index and a file written in the same
///  second;
/// every other verdict,
/// including a missing file and a path reached through a link,
/// hands the path to Git,
/// which
///  writes the tree's bytes there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theFileIsTheEntry(chain, names, entry, indexSeconds): boolean;
/// ```
fn the_file_is_the_entry(chain: &mut DirectoryChain<'_>, names: &[std::ffi::OsString], entry: &StatEntry, index_seconds: u32) -> bool {
    let Some((name, parents)) = names.split_last() else { return false };
    let Some(directory) = chain.directory(parents).ok() else { return false };
    let Ok(metadata) = entry_metadata_at(directory, name.as_os_str()) else { return false };
    return matches!(metadata_verdict(entry, &metadata, index_seconds), StatVerdict::Agrees);
}

/// What:
///  Split the tree of the new `HEAD` into the entries a clone may take and the paths Git writes.
/// Why:
///   See the module description.
/// A path the index lists without a tree entry is named here and reaches nothing:
/// the new worktree's index is built from the tree,
/// so a staged addition is not in it and no file of the source's reaches
///  the destination.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const split = splitTheTree({ entries, indexed, tree, root, indexSeconds });
/// ```
pub fn split_the_tree(entries: &[StatEntry], indexed: &[crate::repo_read_index::IndexEntry], tree: &[FlatEntry], root: &Path, index_seconds: u32) -> Result<Split, CleanRefusal> {
    if entries.len() != indexed.len() {
        return Err(CleanRefusal::unreadable(String::from("the index entries and their stat data are not the same files")));
    }
    let held: std::fs::File = match open_root(root) {
        Ok(opened) => opened,
        Err(error) => return Err(CleanRefusal::unreadable(format!("{} cannot be opened without following a link ({error})", root.display()))),
    };
    let Ok(mut chain) = DirectoryChain::new(&held) else { return Err(CleanRefusal::unreadable(format!("{} cannot be examined", root.display()))) };
    let mut clones: Vec<usize> = Vec::new();
    let mut writes: Vec<Vec<u8>> = Vec::new();
    let mut at: usize = 0;
    for committed in tree {
        let shown: String = String::from_utf8_lossy(committed.path.as_slice()).into_owned();
        kind_of(committed.mode, shown.as_str())?;
        // The index lists the entries of the tree in the same order, so the entries a tree path has already
        //  passed are the ones only the index names: a staged addition, or an entry of an earlier path.
        while at < entries.len() && entries[at].path.as_slice() < committed.path.as_slice() {
            at += 1;
        }
        let same_path = |found: &&StatEntry| { return found.path == committed.path; };
        let Some(entry) = entries.get(at).filter(same_path) else {
            // The tree names a path the index lacks: a deletion staged in the source, which Git's checkout writes.
            writes.push(committed.path.clone());
            continue;
        };
        judgeable(&indexed[at], entry, shown.as_str())?;
        if entry.mode != committed.mode || indexed[at].id != committed.id {
            // The index records another blob or mode than `HEAD` names: the path is staged with a change.
            writes.push(committed.path.clone());
            continue;
        }
        let Ok(names) = plain_components(Path::new(OsStr::from_bytes(committed.path.as_slice()))) else {
            return Err(CleanRefusal::unreadable(format!("{shown} is not a plain path")));
        };
        if the_file_is_the_entry(&mut chain, names.as_slice(), entry, index_seconds) {
            clones.push(at);
            continue;
        }
        writes.push(committed.path.clone());
    }
    return Ok(Split { clones, writes });
}

/// Where the entry of one path is.
impl Split {
    /// The index entries the clones take,
    /// in the index's order.
    pub fn cloned_entries<'entries>(&self, entries: &'entries [StatEntry]) -> Vec<StatEntry> {
        let mut taken: Vec<StatEntry> = Vec::with_capacity(self.clones.len());
        for at in &self.clones {
            if let Some(entry) = entries.get(*at) {
                taken.push(entry.clone());
            }
        }
        return taken;
    }
}

/// Partial-clone controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_clone_partial_split_tests.rs"]
mod tests;
