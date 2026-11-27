//! What:
//!  The landing's private index file read into entries,
//!  its modification time,
//!  and the directories whose `.gitattributes` the index supplies.
//! Why:
//!  Git refuses a file whose checksum is wrong,
//!  and an all-zero checksum is what a skipped hash looks like,
//!  so the file is compared against the checksum before anything is read from
//!  it;
//!  the entries' modification time decides which entries would be racily
//!  clean;
//!  and a directory whose worktree copy of `.gitattributes` is missing has its
//!  attributes read from the index,
//!  which the proof of a file's bytes must know about.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const read = readIndex(file, format); // { index, stamp } | undefined
//! const directories = indexedAttributeDirectories(index, tree); // Uint8Array[]
//! ```

/// The checksum and the read bound the index file is judged by.
use super::index_reset_file_names_and_checksums::{
    INDEX_LIMIT,
    checksum,
};
/// The first non-zero byte of a stretch,
/// which tells a checksum that was skipped from one that was wrong.
use crate::index_byte_search::has_nonzero_byte;
/// A whole index file read into entries,
/// and the parse that reads it.
use crate::index_full::{
    FullIndex,
    parse_full_index,
};
/// The date of the index file,
/// which decides whether an entry is racily clean.
use crate::index_stat::IndexStamp;
/// The object hash the file's own checksum is computed in.
use crate::repo_read_layout::ObjectFormat;
/// The length of one object name in that hash.
use crate::repo_read_object::name_length;
/// One entry of the commit's tree,
/// whose `.gitattributes` the index may hold.
use crate::repo_read_tree::FlatEntry;
/// The index file's path.
use std::path::Path;
/// The index of the file,
/// its modification time to the nanosecond,
/// or `None` when the checksum or the layout is not plain.
pub(super) fn read_index(
    file: &Path,
    format: ObjectFormat,
) -> Option<(FullIndex, IndexStamp)> {
    let bytes: Vec<u8> = crate::private_storage::read_regular_file(file, INDEX_LIMIT).ok()?;
    let hash_length: usize = name_length(format);
    let (body, stored) = bytes.split_at(bytes.len().checked_sub(hash_length)?);
    // Git refuses a file whose checksum is wrong; an all-zero checksum is what a skipped hash looks like.
    if stored != checksum(format, body).as_slice() && has_nonzero_byte(stored) {
        return None;
    }
    let modified: std::time::SystemTime = std::fs::metadata(file).ok()?.modified().ok()?;
    let age: std::time::Duration = modified.duration_since(std::time::UNIX_EPOCH).ok()?;
    let stamp: IndexStamp = IndexStamp {
        seconds: age.as_secs() as u32,
        nanoseconds: age.subsec_nanos(),
    };
    return Some((parse_full_index(bytes.as_slice(), hash_length)?, stamp));
}

/// What:
///  The directory one path is the `.gitattributes` of,
/// or nothing when it is another path.
/// Why:
///   The directories whose attributes the index can supply are named by the `.gitattributes` entries themselves,
/// at the root or below one directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const attributeDirectory = (path) => path === '.gitattributes' ? '' : path.endsWith('/.gitattributes') ? path.slice(0, -'/'.length - '.gitattributes'.length) : undefined;
/// ```
fn attribute_directory(path: &[u8]) -> Option<Vec<u8>> {
    if path == b".gitattributes" {
        return Some(Vec::new());
    }
    let suffix: &[u8] = b"/.gitattributes";
    return path.strip_suffix(suffix).map(<[u8]>::to_vec);
}

/// What:
///  The directories whose `.gitattributes` the index this update writes holds,
/// in path order.
/// Why:
///   Git reads the index's copy of an attributes file where the worktree's copy is missing
/// (`attr.c`,
/// measured on Git 2.56.0:
/// `git check-attr` still answers from the index after the worktree's `.gitattributes` is removed),
/// and a
///  sparse checkout leaves it outside its cone the same way,
/// so the proof must know which directories could supply one through the index.
///  The entries before the reset and the tree's entries are both taken,
/// so a path this reset installs is counted too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const directories = [...index.entries, ...tree].map((entry) => entry.path === '.gitattributes' ? '' : entry.path.replace(/\/\.gitattributes$/, '')).filter(defined);
/// ```
pub(super) fn indexed_attribute_directories(
    index: &FullIndex,
    tree: &[FlatEntry],
) -> Vec<Vec<u8>> {
    let mut directories: Vec<Vec<u8>> = Vec::new();
    for entry in index.entries.iter() {
        if entry.stage() == 0
            && let Some(directory) = attribute_directory(entry.path.as_slice())
        {
            directories.push(directory);
        }
    }
    for entry in tree {
        if let Some(directory) = attribute_directory(entry.path.as_slice()) {
            directories.push(directory);
        }
    }
    directories.sort();
    directories.dedup();
    return directories;
}
