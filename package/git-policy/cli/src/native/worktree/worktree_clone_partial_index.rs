//! What:
//!  The index a partial clone installs:
//! one entry for every path of the target tree,
//! with the stat fields of the files the
//!  clones made and none for the paths Git writes.
//! Why:
//!  A source with uncommitted changes has no index equal to the tree of `HEAD`:
//! a staged path records
//!  another blob,
//! a staged deletion is missing,
//! and a staged addition names a path no tree has.
//! The new worktree must hold what Git's own
//!  checkout holds,
//! so its index is built from the tree and never copied from the source.
//! An entry with zero stat fields is
//!  one Git checks by content before it trusts the file,
//! which is what a path Git is about to write needs;
//! an entry the clones made records the fields of the object they made,
//! so Git's first status is cheap.
//! A link's
//!  entry keeps the fields the source's own index recorded,
//! as the whole-rung clone leaves them,
//! so no name another
//!  process may have replaced is certified.
//! Git's writer demotes a version 3 file whose entries need no extended flags to
//!  version 2 and recomputes the checksum,
//! so what is written here is a file Git itself could have written.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const bytes = indexOfTheTree({ tree, stats, hashLength }); // throws the reason a tree path cannot be an entry
//! ```

use crate::index_full::{FullEntry, FullIndex};
use crate::index_full_write::{Trailer, write_full_index};
use crate::index_stat::StatData;
use crate::repo_read_tree::FlatEntry;
use crate::subvolume_index::{StatFields, StoredStat};
use crate::subvolume_sha::{sha1, sha256};
use std::collections::HashMap;

/// The index version this writes:
/// Git's own default,
/// which it rewrites when a configuration asks for another.
const INDEX_VERSION: u32 = 2;

/// What:
///  The stat fields `found` as the index stores them.
/// Why:
///   The clone records the object it made in `StatFields`,
/// and an entry of an index holds the same nine numbers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stat = { ctimeSec: found.ctimeSeconds, ... };
/// ```
fn stat_data_of(found: StatFields) -> StatData {
    return StatData {
        ctime_sec: found.ctime_seconds,
        ctime_nsec: found.ctime_nanos,
        mtime_sec: found.mtime_seconds,
        mtime_nsec: found.mtime_nanos,
        dev: found.dev,
        ino: found.ino,
        uid: found.uid,
        gid: found.gid,
        size: found.size,
    };
}

/// What:
///  One tree path as the index entry of a worktree at `HEAD`,
/// with the stat fields of its file when one was made here.
/// Why:
///   The entry names the tree's mode and object,
/// and the flags carry nothing:
/// no path here is
///  assumed unchanged,
/// unmerged,
/// staged or outside the checkout.
/// No stat fields means Git checks the file by content before it
///  trusts it,
/// which is what a path Git is about to write needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function entryOf(committed: FlatEntry, stats: StatFields | undefined): FullEntry;
/// ```
fn entry_of(committed: &FlatEntry, stats: Option<StatFields>) -> FullEntry {
    let stat: StatData = match stats {
        Some(found) => stat_data_of(found),
        None => StatData::default(),
    };
    return FullEntry { stat, mode: committed.mode, id: committed.id.clone(), flags: 0, extended: 0, path: committed.path.clone(), monitor_valid: false };
}

/// What:
///  The stat fields of a file as the source's own index recorded them.
/// Why:
///   A link is cloned by making another link with the same target,
/// and the object made is a different inode
///  with a new change time,
/// so fields taken from it would certify a name another process may have replaced since.
/// The
///  source's fields name the target's length and nothing of the new link,
/// which is what Git compares a link by,
/// and what the
///  whole-rung clone leaves in the index it installs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fields = fieldsTheSourceRecorded(stored);
/// ```
pub fn fields_the_source_recorded(stored: StoredStat) -> StatFields {
    return StatFields {
        ctime_seconds: stored.ctime_seconds,
        ctime_nanos: stored.ctime_nanos,
        mtime_seconds: stored.mtime_seconds,
        mtime_nanos: stored.mtime_nanos,
        dev: 0,
        ino: stored.ino,
        uid: stored.uid,
        gid: stored.gid,
        size: stored.size,
    };
}

/// What:
///  The checksum of the repository's object format,
/// as the function the index writer appends.
/// Why:
///   A SHA-256 repository's index ends with a SHA-256 checksum,
/// and a reader of it trusts nothing else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const hash = hashLength === 32 ? sha256 : sha1;
/// ```
fn checksum_of(hash_length: usize) -> fn(&[u8]) -> Vec<u8> {
    if hash_length == 32 {
        return |data: &[u8]| { return sha256(data).to_vec(); };
    }
    return |data: &[u8]| { return sha1(data).to_vec(); };
}

/// What:
///  The bytes of the index a new worktree at `tree` takes,
/// with `stats` for the paths the clones made.
/// Why:
///   See the module description.
/// A path whose object name is not the repository's hash length,
/// or one with no name,
/// is a tree
///  this rung never read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bytes = indexOfTheTree({ tree, stats, hashLength }); // throws the reason
/// ```
pub fn index_of_the_tree(tree: &[FlatEntry], stats: &HashMap<Vec<u8>, StatFields>, hash_length: usize) -> Result<Vec<u8>, String> {
    let mut entries: Vec<FullEntry> = Vec::with_capacity(tree.len());
    for committed in tree {
        if committed.path.is_empty() {
            return Err(String::from("a tree entry has no name"));
        }
        if committed.id.len() != hash_length {
            return Err(format!("the object name of {} is not the repository's hash length", String::from_utf8_lossy(committed.path.as_slice())));
        }
        entries.push(entry_of(committed, stats.get(committed.path.as_slice()).copied()));
    }
    let index: FullIndex = FullIndex { version: INDEX_VERSION, entries, cache_tree: None, resolve_undo: None, untracked: None, fsmonitor: None };
    return write_full_index(&index, &checksum_of(hash_length), Trailer::Hashed).ok_or_else(|| { return String::from("the index of the tree is larger than its format counts"); });
}

/// Partial-clone index controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_clone_partial_index_tests.rs"]
mod tests;
