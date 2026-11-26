//! What:
//!  The source's index,
//! the tree of its `HEAD`,
//! the paths Git writes,
//! and the stat fields the new index records,
//! which a partial
//!  clone reads and decides before it copies anything.
//! Why:
//!  The split compares three things (the index entry,
//! the tree entry and the file),
//! and the index build takes the
//!  tree,
//! so one reading answers both;
//! the written paths and the recorded stats are what that reading decides,
//! and they
//!  live here so `worktree_clone_partial.rs` stays under the file-size limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const tracked = readTracked(source); // throws the reason
//! const written = theWrittenPaths(split, decided);
//! const recorded = theRecordedStats(made, split, entries, written);
//! ```

use crate::repo_read_index::parse_index;
use crate::repo_read_layout::ObjectFormat;
use crate::repo_read_odb::ObjectDatabase;
use crate::repo_read_tree::{
    FlatEntry,
    flatten_tree,
};
use crate::subvolume_index::StatFields;
use crate::worktree_clean::stored_stats;
use crate::worktree_clone_conversion_fence::Fence;
use crate::worktree_clone_partial_index::fields_the_source_recorded;
use crate::worktree_clone_partial_split::Split;
use crate::worktree_snapshot_source::SnapshotSource;
use std::collections::{
    HashMap,
    HashSet,
};

/// The mode bits that say an entry is a link rather than a file.
const MODE_SYMLINK: u32 = 0o120_000;

/// The type bits of an entry's mode.
const MODE_TYPE: u32 = 0o170_000;

/// What:
///  The tracked paths the source's index and the tree of the source's `HEAD` name,
/// and the tree itself.
/// Why:
///   The split compares three things (the index entry,
/// the tree entry and the file),
/// and the index build
///  takes the tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Tracked = { entries: StatEntry[]; indexed: IndexEntry[]; tree: FlatEntry[] };
/// ```
pub(crate) struct Tracked {
    /// The index's entries with their recorded stat data.
    pub entries: Vec<crate::worktree_clean::StatEntry>,
    /// The same entries as the plain index reader gives them,
    /// with their stage and extended flags.
    pub indexed: Vec<crate::repo_read_index::IndexEntry>,
    /// The flattened tree of the source's `HEAD`.
    pub tree: Vec<FlatEntry>,
}

/// What:
///  Read the source's index and the tree of its `HEAD`.
/// Why:
///   The split judges every tree path against the index entry and the file,
/// and the index build takes
///  the tree,
/// so one reading answers both,
/// and the tree is the source's `HEAD`'s because that is what the split and the index name;
/// the ladder's `read-tree -m -u` carries a differing new `HEAD` in afterwards.
///
/// The index entries the ladder already read are handed in rather than parsed again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const tracked = readTracked(source, entries); // throws the reason
/// ```
pub(crate) fn read_tracked(
    source: &SnapshotSource,
    known: Option<Vec<crate::worktree_clean::StatEntry>>,
) -> Result<Tracked, String> {
    let hash_length: usize = source.clean.hash_length;
    let format: ObjectFormat = if hash_length == 32 {
        ObjectFormat::Sha256
    } else {
        ObjectFormat::Sha1
    };
    let bytes: &[u8] = source.clean.index_bytes.as_slice();
    let Some(entries) = known.or_else(|| {
        return stored_stats(bytes, hash_length);
    }) else {
        return Err(String::from("the source's index changed shape"));
    };
    let Some(indexed) = parse_index(bytes, format) else {
        return Err(String::from(
            "the source's index has a layout the reader does not know",
        ));
    };
    let Some(database) = ObjectDatabase::open(source.common_dir.as_path(), None, format) else {
        return Err(String::from(
            "the object database has a layout the reader does not know",
        ));
    };
    let Some(tree) = flatten_tree(&database, source.clean.head_tree.as_slice(), format) else {
        return Err(String::from("the tree of HEAD is unreadable"));
    };
    return Ok(Tracked {
        entries,
        indexed,
        tree,
    });
}

/// What:
///  The paths Git writes:
/// the paths a change reaches,
/// and the paths the conversion fence takes because a clone
///  would keep bytes Git's checkout would write others for,
/// in path order.
/// Why:
///  Both lists are sorted by path,
/// and the index,
/// the tree and both steps read them in that order;
/// a path on both lists is written once,
/// which is what the fence's own answer for a path a change also reaches needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const written = [...new Set([...split.writes, ...fence.gitWrites])].sort(compareBytes);
/// ```
pub(crate) fn the_written_paths(
    split: &Split,
    fence: &Fence,
) -> Vec<Vec<u8>> {
    let mut written: HashSet<Vec<u8>> = split.writes.iter().cloned().collect();
    for path in &fence.git_writes {
        written.insert(path.clone());
    }
    let mut ordered: Vec<Vec<u8>> = written.into_iter().collect();
    ordered.sort();
    return ordered;
}

/// What:
///  The stat fields the new index records:
/// those of the files the clones made,
/// with a link's entry keeping the
///  fields its source recorded,
/// and none at all for a path Git writes,
/// so nothing records a path Git has not written.
/// Why:
///   An entry whose stat data matches the file is exactly what makes Git skip it,
/// so a path the
///  index may not vouch for yet keeps the zero stat data the tree's own entry holds;
/// the index then takes the
///  worktree's name only after Git wrote those paths (`worktree_clone_index`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const recorded = withoutTheWritten(statsOfTheClones({ made, split, entries }), written);
/// ```
pub(crate) fn the_recorded_stats(
    made: HashMap<Vec<u8>, StatFields>,
    split: &Split,
    entries: &[crate::worktree_clean::StatEntry],
    written: &HashSet<Vec<u8>>,
) -> HashMap<Vec<u8>, StatFields> {
    let mut stats: HashMap<Vec<u8>, StatFields> = made;
    for at in &split.clones {
        let Some(entry) = entries.get(*at) else {
            continue;
        };
        if entry.mode & MODE_TYPE == MODE_SYMLINK {
            stats.insert(entry.path.clone(), fields_the_source_recorded(entry.stored));
        }
    }
    stats.retain(|path: &Vec<u8>, _fields: &mut StatFields| {
        return !written.contains(path);
    });
    return stats;
}
