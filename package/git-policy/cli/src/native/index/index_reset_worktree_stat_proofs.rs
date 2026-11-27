//! What:
//!  The proofs a landing makes about one worktree file:
//!  its bytes hash to the entry's object name,
//!  its state did not move while it was read,
//!  its change time can tell two rewrites in one second apart,
//!  and its kind is the one the entry's mode records.
//! Why:
//!  A reader that trusts the stat data would report a changed file as
//!  unchanged,
//!  so the person's edit would reach neither the next commit nor
//!  `git status`,
//!  and each proof below refuses the stat data a file cannot stand behind,
//!  which costs one refresh and hides nothing.
//!  The read is bounded by the bytes the landing may take and by the tries an
//!  interrupted read gets,
//!  so a source that interrupts every read cannot hold the landing's index
//!  lock.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const name = hash(`blob ${data.length}\0`, data);
//! const same = (a, b) => a.ino === b.ino && a.mtimeNs === b.mtimeNs && ...;
//! const agrees = mode === 0o100644 || mode === 0o100755 ? held.isFile() && bitsAgree : false;
//! ```

/// The file-type bits and the chunk a read takes,
/// named beside the proofs that use them.
use super::index_reset_worktree_stat_request::{
    EXECUTABLE_BIT,
    INTERRUPTED_TRIES,
    ProofSeams,
    READ_CHUNK,
    REGULAR_FILE,
    TYPE_BITS,
};
/// The stat fields an index entry records.
use crate::index_stat::StatData;
/// The object hash the entry's name is compared in.
use crate::repo_read_layout::ObjectFormat;
/// The SHA-1 and SHA-256 the object name is computed with.
use crate::subvolume_sha::{
    sha1,
    sha256,
};
/// The bytes this landing has read so far,
/// which the bounded read charges every chunk to.
use std::cell::Cell;
/// The change and modification times,
/// and the file-type bits an entry's mode is read with.
use std::os::unix::fs::MetadataExt;
/// What:
///  The object name of `data` read as a blob.
/// Why:
///   An entry's object name is the hash of the blob header and the bytes,
/// so comparing with it proves the bytes are the committed ones.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const name = hash(`blob ${data.length}\0`, data); // sha1 or sha256
/// ```
pub(super) fn blob_name(
    data: &[u8],
    format: ObjectFormat,
) -> Vec<u8> {
    let mut framed: Vec<u8> = format!("blob {}\0", data.len()).into_bytes();
    framed.extend_from_slice(data);
    return match format {
        ObjectFormat::Sha1 => sha1(framed.as_slice()).to_vec(),
        ObjectFormat::Sha256 => sha256(framed.as_slice()).to_vec(),
    };
}

/// What:
///  Whether two examinations describe one unchanged file:
/// the same object,
/// size,
/// mode,
/// owner,
/// group and times.
/// Why:
///   A write,
/// a `chmod` or a swap while the file is read must leave the entry unstatted instead of passing,
/// and these are the
///  fields an index entry records,
/// so a change in any of them is one a reader would see.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const same = (a, b) => a.dev === b.dev && a.ino === b.ino && a.size === b.size && a.mode === b.mode && a.uid === b.uid
///   && a.gid === b.gid && a.mtimeNs === b.mtimeNs && a.ctimeNs === b.ctimeNs;
/// ```
pub(super) fn same_state(
    first: &std::fs::Metadata,
    second: &std::fs::Metadata,
) -> bool {
    return first.dev() == second.dev()
        && first.ino() == second.ino()
        && first.size() == second.size()
        && first.mode() == second.mode()
        && first.uid() == second.uid()
        && first.gid() == second.gid()
        && first.mtime() == second.mtime()
        && first.mtime_nsec() == second.mtime_nsec()
        && first.ctime() == second.ctime()
        && first.ctime_nsec() == second.ctime_nsec();
}

/// What:
///  Whether a file's change time can tell two rewrites in one second apart.
/// Why:
///   A file system that stores whole seconds reports a change time with no nanoseconds,
/// and a same-size rewrite whose modification time is put back inside that second leaves every field the proof here
///  compares unchanged,
/// so the bytes that were read cannot be proven to be the ones the path holds now.
/// Such a file keeps the zero stat data,
/// which costs a refresh and hides nothing,
/// as the attributes reader refuses such a file for the same reason
///  (`worktree_clean_content_attributes.rs`,
/// `attribute_bytes`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stampProvesOneVersion = ([, nanos]) => nanos !== 0;
/// ```
pub(super) fn stamp_proves_one_version(changed: (i64, i64)) -> bool {
    return changed.1 != 0;
}

/// What:
///  The stat data an entry records for a file whose metadata is `metadata`.
/// Why:
///   The index stores each field cut to 32 bits,
/// so the comparison the next reader makes is against these cuts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stat = { ctimeSec: m.ctime, ctimeNsec: m.ctimeNsec, mtimeSec: m.mtime, mtimeNsec: m.mtimeNsec, dev: m.dev, ino: m.ino, uid: m.uid, gid: m.gid, size: m.size };
/// ```
pub(super) fn stat_of(metadata: &std::fs::Metadata) -> StatData {
    return StatData {
        ctime_sec: metadata.ctime() as u32,
        ctime_nsec: metadata.ctime_nsec() as u32,
        mtime_sec: metadata.mtime() as u32,
        mtime_nsec: metadata.mtime_nsec() as u32,
        dev: metadata.dev() as u32,
        ino: metadata.ino() as u32,
        uid: metadata.uid(),
        gid: metadata.gid(),
        size: metadata.size() as u32,
    };
}

/// What:
///  The bytes of `file` up to `room`,
/// each chunk charged to `spent` as it is taken.
/// Why:
///   The bound is on the reading and not on an earlier look at the size,
/// and every byte that reached the disk is charged,
/// including
///  what a read that then fails took:
/// a refused read still consumed it.
/// A read the system interrupts is tried again,
/// because an interruption is not a fact about the file;
/// the tries are counted (`INTERRUPTED_TRIES`),
/// and a source that interrupts every one of them ends in the same refusal a hard read error does,
///  so nothing waits for an interruption to stop.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readByChunks(file, room, spent, seams) {
///   const data = []; let interrupted = 0; while (data.length < room) { let taken; try { taken = seams.read(file, new Uint8Array(Math.min(64 << 10, room - data.length))); }
///     catch (error) { if (error.kind === 'Interrupted' && interrupted < INTERRUPTED_TRIES) { interrupted += 1; continue; } return undefined; } if (taken === 0) break; data.push(...taken); spent += taken; } return data;
/// }
/// ```
pub(super) fn read_by_chunks(
    file: &std::fs::File,
    room: u64,
    spent: &Cell<u64>,
    seams: &mut ProofSeams<'_>,
) -> Option<Vec<u8>> {
    let mut data: Vec<u8> = Vec::new();
    let mut interrupted: usize = 0;
    let mut buffer: Vec<u8> = vec![0; READ_CHUNK];
    while (data.len() as u64) < room {
        let want: usize = usize::try_from(room - data.len() as u64)
            .unwrap_or(usize::MAX)
            .min(READ_CHUNK);
        match (seams.read)(file, &mut buffer[..want]) {
            Ok(0) => break,
            Ok(taken) => {
                let took: usize = taken.min(want);
                data.extend_from_slice(&buffer[..took]);
                spent.set(spent.get() + took as u64);
            }
            // An interruption is not a fact about the file, so it is tried again;
            // a source that interrupts every try is refused rather than waited on.
            Err(error)
                if error.kind() == std::io::ErrorKind::Interrupted
                    && interrupted < INTERRUPTED_TRIES =>
            {
                interrupted += 1;
                continue;
            }
            // The bytes taken before the failure are charged already, and the read proves nothing.
            Err(_) => return None,
        }
    }
    return Some(data);
}

/// What:
///  Whether the metadata describes the one kind of file this records stat data for,
/// with the permission bit the entry records.
/// Why:
///   A symbolic link is left to Git (`a_symbolic_link_records_nothing`),
/// a gitlink holds no file,
/// and a path that became a
///  directory,
/// a fifo or a device has changed whatever its bytes are.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const agrees = entry.mode === 0o100644 || entry.mode === 0o100755
///   ? m.isFile() && (entry.mode & 0o100) === (m.mode & 0o100) : false;
/// ```
pub(super) fn mode_agrees(
    entry_mode: u32,
    metadata: &std::fs::Metadata,
) -> bool {
    if entry_mode & TYPE_BITS != REGULAR_FILE {
        return false;
    }
    return metadata.file_type().is_file()
        && (entry_mode & EXECUTABLE_BIT != 0) == (metadata.mode() & EXECUTABLE_BIT != 0);
}
