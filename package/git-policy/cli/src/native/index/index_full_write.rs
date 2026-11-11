//! What:
//!  Writes a `FullIndex` as Git's `do_write_index` does for the plain case:
//! the version Git picks,
//! the entries,
//! the kept
//!  extensions in Git's order,
//! and the trailing checksum (`read-cache.c`:
//! `do_write_index`,
//! `ce_write_entry`).
//! Why:
//!  The in-process index update must produce the bytes Git's own write produces.
//! A version 3 file with no extended flag is
//!  written as version 2;
//! a version 4 file prefix-compresses names;
//! the cache-tree,
//! resolve-undo,
//! untracked-cache and
//!  fsmonitor extensions follow in that order;
//! the offset-table extensions are not written because the configuration that
//!  asks for them is declined before this runs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const bytes = writeFullIndex(index, (data) => sha1(data), { kind: 'hashed' }); // undefined: a name or size outside what the format holds
//! ```

use crate::index_byte_search::common_prefix_length;
use crate::index_fsmonitor::write_fsmonitor;
use crate::index_full::{EXTENDED, FullEntry, FullIndex, INTENT_TO_ADD, SKIP_WORKTREE};
use crate::index_stat::encode_varint;
use crate::index_untracked_io::write_untracked;

/// What:
///  What the file ends with,
///  which the configuration decides.
/// Why:
///   `do_write_index` writes the checksum of everything before it,
///  or,
///  under `index.skipHash`,
///  a run of zero bytes saying the computation was skipped (`git-config`,
///  Git 2.56.0).
pub enum Trailer {
    /// The checksum over the bytes before it.
    Hashed,
    /// The zeros `index.skipHash` writes instead,
    /// `checksum_length` of them,
    /// which the object format decides.
    Skipped { checksum_length: usize },
}

/// The largest name length the flags hold;
/// a longer name stores this value.
const NAME_MASK: usize = 0x0fff;

/// Whether the entry has an extended flag Git keeps.
fn has_extended(entry: &FullEntry) -> bool {
    return entry.extended & (INTENT_TO_ADD | SKIP_WORKTREE) != 0;
}

/// Append one entry (`ce_write_entry`);
/// `previous` is the path before it when the file is version 4.
fn write_entry(out: &mut Vec<u8>, entry: &FullEntry, earlier: Option<&mut Vec<u8>>) {
    let extended: bool = has_extended(entry);
    for value in [entry.stat.ctime_sec, entry.stat.ctime_nsec, entry.stat.mtime_sec, entry.stat.mtime_nsec, entry.stat.dev, entry.stat.ino, entry.mode, entry.stat.uid, entry.stat.gid, entry.stat.size] {
        out.extend_from_slice(&value.to_be_bytes());
    }
    out.extend_from_slice(entry.id.as_slice());
    let flags: u16 = entry.flags | if extended { EXTENDED } else { 0 } | entry.path.len().min(NAME_MASK) as u16;
    out.extend_from_slice(&flags.to_be_bytes());
    if extended {
        out.extend_from_slice(&(entry.extended & (INTENT_TO_ADD | SKIP_WORKTREE)).to_be_bytes());
    }
    let Some(previous) = earlier else {
        // The entry, its name and padding fill a multiple of eight bytes, with at least one NUL after the name.
        let fixed: usize = 40 + entry.id.len() + if extended { 4 } else { 2 };
        out.extend_from_slice(entry.path.as_slice());
        let padding: usize = ((fixed + entry.path.len() + 8) & !7) - (fixed + entry.path.len());
        out.resize(out.len() + padding, 0);
        return;
    };
    let common: usize = common_prefix_length(previous.as_slice(), entry.path.as_slice());
    encode_varint((previous.len() - common) as u64, out);
    out.extend_from_slice(&entry.path[common..]);
    out.push(0);
    previous.clear();
    previous.extend_from_slice(entry.path.as_slice());
}

/// Append one extension:
/// its signature,
/// size and content.
fn write_extension(out: &mut Vec<u8>, signature: &[u8; 4], content: &[u8]) {
    out.extend_from_slice(signature);
    out.extend_from_slice(&(content.len() as u32).to_be_bytes());
    out.extend_from_slice(content);
}

/// The file's bytes,
/// with the checksum `hash` makes of everything before it;
/// `None` for more entries than the format counts.
pub fn write_full_index(index: &FullIndex, hash: &dyn Fn(&[u8]) -> Vec<u8>, trailer: Trailer) -> Option<Vec<u8>> {
    let extended: bool = index.entries.iter().any(has_extended);
    // Git demotes a version 3 file to version 2 when no entry needs the extended flags, and promotes a version 2 file with one.
    let version: u32 = if index.version == 4 { 4 } else if extended { 3 } else { 2 };
    let mut out: Vec<u8> = Vec::new();
    out.extend_from_slice(b"DIRC");
    out.extend_from_slice(&version.to_be_bytes());
    out.extend_from_slice(&u32::try_from(index.entries.len()).ok()?.to_be_bytes());
    let mut previous: Vec<u8> = Vec::new();
    for entry in &index.entries {
        write_entry(&mut out, entry, if version == 4 { Some(&mut previous) } else { None });
    }
    if let Some(tree) = &index.cache_tree {
        write_extension(&mut out, b"TREE", tree.write().as_slice());
    }
    if let Some(undo) = &index.resolve_undo {
        write_extension(&mut out, b"REUC", undo.write().as_slice());
    }
    if let Some(cache) = &index.untracked {
        write_extension(&mut out, b"UNTR", write_untracked(cache).as_slice());
    }
    if let Some(monitor) = &index.fsmonitor {
        // The bitmap lists, by position among the entries written, those the monitor does not vouch for.
        let mut dirty: Vec<usize> = Vec::new();
        for (position, entry) in index.entries.iter().enumerate() {
            if !entry.monitor_valid {
                dirty.push(position);
            }
        }
        write_extension(&mut out, b"FSMN", write_fsmonitor(monitor.token.as_slice(), dirty.as_slice()).as_slice());
    }
    match trailer {
        Trailer::Hashed => {
            let checksum: Vec<u8> = hash(out.as_slice());
            out.extend_from_slice(checksum.as_slice());
        }
        // Zeros of the checksum's own length: Git says with them that the computation was skipped.
        Trailer::Skipped { checksum_length } => out.resize(out.len() + checksum_length, 0),
    }
    return Some(out);
}

#[cfg(test)]
#[path = "index_full_write_tests.rs"]
mod tests;
