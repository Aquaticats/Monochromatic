//! What:
//!  The stat rewrite of a Git index:
//! the bytes of an index file in,
//! the bytes of the same index out,
//! with the stat
//!  fields of each entry replaced by what the file on disk reports now.
//! Why:
//!  A reflink copy gives every file a new inode number and change time,
//! so the first `git status` in the copied tree
//!  finds every entry stat-dirty and hashes every file (22.1 s measured against 0.17 s with the rewrite,
//! and the clean
//!  filter of Git LFS runs on LFS files).
//! The rewrite touches an entry only when the file's size and modification
//!  time equal the entry's,
//! so a file that really changed stays dirty.
//! Entry lengths never change,
//! so the entries are
//!  edited in place.
//! The accelerator extensions that record stat-derived state (`UNTR` untracked cache,
//! `FSMN`
//!  fsmonitor token) or offsets into the entries (`EOIE`,
//! `IEOT`) are dropped,
//! because Git rebuilds them;
//! a split
//!  index (`link`) or a sparse index (`sdir`) is left untouched.
//! The checksum is recomputed with the hash that
//!  verifies the old one (SHA-1,
//! else SHA-256);
//! an index written with `index.skipHash` has an all-zero checksum and
//!  keeps one.
//! Versions 2,
//! 3 and 4 are read (`Documentation/gitformat-index.adoc`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = rewriteIndexStat(bytes, (path) => lstatFields(join(tree, path))); // { bytes, updated, total } | { skipped }
//! ```
//!
//! The code of this module is split by role,
//! and this file re-exports every name those files hold,
//! so that a caller writes `subvolume_index::rewrite_index_stat` as it did before:
//! `subvolume_index_rewrite_contract` holds what a caller supplies and reads back,
//! and `subvolume_index_bytes` reads an index file's header,
//! entries and extensions.

use crate::subvolume_index_bytes::{
    DECLINED,
    DROPPED,
    is_listed,
};
use crate::subvolume_sha::{
    sha1,
    sha256,
};

pub(crate) use crate::subvolume_index_bytes::{
    HEADER,
    Located,
    STAT,
    be32,
    checksum_kind,
    walk_entries,
};

pub use crate::subvolume_index_rewrite_contract::{
    AcceptEveryStat,
    IndexOutcome,
    RewriteContext,
    StatFields,
    StatLookup,
    StatTrust,
    StoredStat,
};

/// What:
///  Rewrite the stat fields of an index file;
/// see the module description for the rules.
///  `lookup` is called with an entry's path and returns what `lstat` reports now,
/// or nothing for a missing file.
///  `&mut dyn StatLookup` is a borrowed lookup that may keep state (a table,
/// a cursor over a directory).
/// Why:
///   Keeping the file system out of this function makes every branch testable with built bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rewriteIndexStat(bytes: Uint8Array, lookup: (path: Uint8Array) => StatFields | undefined): IndexOutcome;
/// ```
pub fn rewrite_index_stat(
    bytes: &[u8],
    lookup: &mut dyn StatLookup,
) -> IndexOutcome {
    return rewrite_index_stat_with(
        bytes,
        &RewriteContext::default(),
        lookup,
        &mut AcceptEveryStat,
    );
}

/// What:
///  Rewrite the stat fields of an index file;
/// see the module description for the rules.
///  `trusted` is asked,
/// with an entry's path and stored fields,
/// whether the file the entry was recorded for is still
///  the one the copy was made from.
///  An entry whose modification time is not earlier than the index file's own (racy in Git's terms) keeps its fields,
///  so Git hashes it.
/// Why:
///   Replacing the change time and inode of an entry that Git could not trust,
/// or of a file that changed after Git
///  looked,
/// would make Git believe the stored blob is current.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rewriteIndexStatWith(bytes: Uint8Array, context: RewriteContext, lookup: Lookup, trusted: (path: Uint8Array, stored: StoredStat) => boolean): IndexOutcome;
/// ```
pub fn rewrite_index_stat_with(
    bytes: &[u8],
    context: &RewriteContext,
    lookup: &mut dyn StatLookup,
    trusted: &mut dyn StatTrust,
) -> IndexOutcome {
    if bytes.len() < HEADER || &bytes[..4] != b"DIRC" {
        return IndexOutcome::Skipped(String::from("the file is not an index (no DIRC signature)"));
    }
    let version: u32 = be32(bytes, 4).unwrap_or(0);
    if !(2..=4).contains(&version) {
        return IndexOutcome::Skipped(format!("index version {version} is not 2, 3 or 4"));
    }
    let (hash_length, zero_checksum): (usize, bool) =
        match checksum_kind(bytes, context.hash_length) {
            Ok(found) => found,
            Err(reason) => return IndexOutcome::Skipped(reason),
        };
    let Some((entries, entries_end)) = walk_entries(bytes, hash_length) else {
        return IndexOutcome::Skipped(String::from(
            "the entries do not have the documented layout",
        ));
    };
    let limit: usize = bytes.len() - hash_length;
    // The extensions after the entries: keep each unless it is derived state; decline on one that changes the meaning.
    let mut kept: Vec<u8> = Vec::new();
    let mut at: usize = entries_end;
    while at < limit {
        let (Some(size), Some(signature)) = (be32(bytes, at + 4), bytes.get(at..at + 4)) else {
            return IndexOutcome::Skipped(String::from("an extension header is cut off"));
        };
        let end: usize = at + 8 + usize::try_from(size).unwrap_or(usize::MAX);
        if end > limit {
            return IndexOutcome::Skipped(String::from(
                "an extension runs past the end of the file",
            ));
        }
        if is_listed(DECLINED, signature) {
            return IndexOutcome::Skipped(format!(
                "the index uses the {} extension (split or sparse index)",
                String::from_utf8_lossy(signature)
            ));
        }
        if !is_listed(DROPPED, signature) {
            kept.extend_from_slice(&bytes[at..end]);
        }
        at = end;
    }
    let mut output: Vec<u8> = bytes[..entries_end].to_vec();
    let mut updated: usize = 0;
    for entry in &entries {
        let regular: bool = matches!(entry.mode & 0o170_000, 0o100_000 | 0o120_000);
        let resolved: bool = (entry.flags >> 12) & 3 == 0;
        let stored_size: u32 = be32(bytes, entry.start + 36).unwrap_or(0);
        if !regular || !resolved || stored_size == 0 {
            continue;
        }
        let stored_seconds: u32 = be32(bytes, entry.start + 8).unwrap_or(0);
        let stored_nanos: u32 = be32(bytes, entry.start + 12).unwrap_or(0);
        // Git's racy test: an entry modified in the same second as the index file, or later, cannot be trusted by its
        // stat fields alone. Its fields stay as recorded, which no longer match the copy, so Git hashes the file.
        if let Some((index_seconds, _nanos)) = context.index_mtime
            && stored_seconds >= index_seconds
        {
            continue;
        }
        let Some(now) = lookup.stat_now(entry.path.as_slice()) else {
            continue;
        };
        // An index written by a Git built without nanosecond support stores 0 there.
        let same_time: bool = now.mtime_seconds == stored_seconds
            && (stored_nanos == 0 || stored_nanos == now.mtime_nanos);
        if now.size != stored_size || !same_time {
            continue;
        }
        let recorded: StoredStat = StoredStat {
            ctime_seconds: be32(bytes, entry.start).unwrap_or(0),
            ctime_nanos: be32(bytes, entry.start + 4).unwrap_or(0),
            mtime_seconds: stored_seconds,
            mtime_nanos: stored_nanos,
            ino: be32(bytes, entry.start + 20).unwrap_or(0),
            uid: be32(bytes, entry.start + 28).unwrap_or(0),
            gid: be32(bytes, entry.start + 32).unwrap_or(0),
            size: stored_size,
        };
        if !trusted.is_trusted(entry.path.as_slice(), &recorded) {
            continue;
        }
        let fields: [(usize, u32); 6] = [
            (0, now.ctime_seconds),
            (4, now.ctime_nanos),
            (16, now.dev),
            (20, now.ino),
            (28, now.uid),
            (32, now.gid),
        ];
        for (offset, value) in fields {
            output[entry.start + offset..entry.start + offset + 4]
                .copy_from_slice(&value.to_be_bytes());
        }
        updated += 1;
    }
    output.extend_from_slice(kept.as_slice());
    if zero_checksum {
        output.extend(std::iter::repeat_n(0u8, hash_length));
    } else if hash_length == 20 {
        let digest: [u8; 20] = sha1(output.as_slice());
        output.extend_from_slice(&digest);
    } else {
        let digest: [u8; 32] = sha256(output.as_slice());
        output.extend_from_slice(&digest);
    }
    return IndexOutcome::Rewritten {
        bytes: output,
        updated,
        total: entries.len(),
    };
}

/// Index rewrite controls stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_index_tests.rs"]
mod tests;
