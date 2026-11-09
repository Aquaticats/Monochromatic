//! What:
//!  The stat data a landing may record for a path whose worktree file provably holds the bytes the commit stores,
//! so the entry the in-process reset writes describes the file instead of carrying all-zero stat data.
//! Why:
//!  The landing's in-process index reset writes the commit's paths as `git reset --no-refresh` does,
//! with all-zero stat data (`index_reset.rs`,
//! `StatData::default()`),
//! where native `git commit` leaves the worktree's.
//!  The next commit therefore finds those paths stale and refreshes them once,
//! through one Git start and one hash per path the
//!  previous commit landed and this one does not name,
//! which native Git pays nothing for
//! (`doc/handover/cli-git-rust-implementation.md`,
//! "Still to do").
//!  An entry may carry the file's stat data only when nothing the file holds is hidden by it,
//! because a reader that trusts the stat data
//!  (this wrapper's own staleness question,
//! and Git's own checks)
//! would otherwise report a changed file as unchanged,
//! and the person's
//!  edit would reach neither the next commit nor `git status`.
//!  So the stat data here is recorded only when all three hold:
//!
//! - The file's raw bytes hash to the entry's object name,
//! read as a blob.
//!   A file whose bytes Git converts on the way into the index
//!  (a clean filter,
//! `ident`,
//! `working-tree-encoding`,
//! a line-ending conversion)
//! hashes to something else and keeps the
//!  all-zero stat data,
//! so a conversion can only cost a refresh,
//! never a wrong record.
//! - The state of the one inode that was read is the same before,
//! during and after the read,
//! and the path still names it,
//!  so the bytes that were hashed are the bytes the recorded stat data describes.
//! - The file is older than the index file this entry is written into,
//! compared to the nanosecond as Git's `is_racy_stat` compares them.
//!   An
//!  entry that is not is what Git calls racily clean:
//! Git hashes such a file because its stat data proves nothing,
//! and the next
//!  commit's staleness question asks for a refresh for it as well,
//! so recording it would buy nothing and could hide a same-size edit made
//!  in that same instant on a file system whose times are coarse.
//!   Such an entry keeps the all-zero stat data,
//! and both readers
//!  hash the file as they do today.
//! A file written earlier in the index file's own second is older than the index on a file system that stores
//!  nanoseconds,
//! so the entries a person's edit just made record their stat data and the next commit starts no Git for them.
//!
//! A path that is not a regular file or a symbolic link,
//! a file larger than `CONTENT_LIMIT`,
//! and a file that cannot
//!  be read keep the all-zero stat data too,
//! and Git's own refresh decides their stat data as it does today.
//! One landing hashes at most `CONTENT_LIMIT` bytes in all,
//! so a commit that lands a whole worktree never reads it again inside the landing's index lock;
//! the 4,000 newest
//!  non-merge commits of this repository change a median of 27 KB
//! (p99 3.3 MB,
//! largest 23.1 MB,
//! `package/git-policy/cli/e2e/commit-shape-trace.json`),
//!  so no commit there spends the bound.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const stat: StatData | undefined = recordedStat({ worktreeRoot, path, blob, mode, format, stamp });
//! ```

use crate::index_reset_worktree_stat_source::WorktreeStatSource;
use crate::index_stat::{IndexStamp, StatData};
use crate::repo_read_layout::ObjectFormat;
use crate::subvolume_sha::{sha1, sha256};
use crate::worktree_clean_content::{CONTENT_LIMIT, read_at_most};
use std::os::unix::ffi::{OsStrExt, OsStringExt as _};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

/// The file-type bits of an entry's mode.
const TYPE_BITS: u32 = 0o170_000;

/// The file-type bits of a regular file.
const REGULAR_FILE: u32 = 0o100_000;

/// The file-type bits of a symbolic link.
const SYMBOLIC_LINK: u32 = 0o120_000;

/// The one permission bit of a file's mode that an index entry records.
const EXECUTABLE_BIT: u32 = 0o100;

/// What:
///  One path whose entry a reset writes,
/// and what the entry says the file holds.
/// Why:
///   The proof reads the file and needs the value it must hash to,
/// the kind of file the entry names,
/// the repository's object hash and
///  the second the index file is dated with.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WorktreeStatRequest = { worktreeRoot: string; path: Uint8Array; blob: Uint8Array; mode: number; format: ObjectFormat; stamp: IndexStamp; hashed: { bytes: number } };
/// ```
pub struct WorktreeStatRequest<'a> {
    /// The worktree's top level,
    /// which index paths are relative to.
    pub worktree_root: &'a Path,
    /// The path as the index spells it,
    /// `/` separated.
    pub path: &'a [u8],
    /// The object name of the content the entry records.
    pub blob: &'a [u8],
    /// The mode the entry records,
    /// as `create_ce_mode` makes it.
    pub mode: u32,
    /// The object hash of the repository.
    pub format: ObjectFormat,
    /// The date of the index file this entry is written into.
    pub stamp: IndexStamp,
    /// The bytes this landing has hashed so far,
    /// which bound what one landing reads (`CONTENT_LIMIT`).
    pub hashed: &'a std::cell::Cell<u64>,
}

/// What:
///  The object name of `data` read as a blob.
/// Why:
///   An entry's object name is the hash of the blob header and the bytes,
/// so comparing with it proves the bytes are the committed ones.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const name = hash(`${'blob '}${data.length}\0`, data); // sha1 or sha256
/// ```
fn blob_name(data: &[u8], format: ObjectFormat) -> Vec<u8> {
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
fn same_state(first: &std::fs::Metadata, second: &std::fs::Metadata) -> bool {
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
///  The stat data an entry records for a file whose metadata is `metadata`.
/// Why:
///   The index stores each field cut to 32 bits,
/// so the comparison the next reader makes is against these cuts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stat = { ctimeSec: m.ctime, ctimeNsec: m.ctimeNsec, mtimeSec: m.mtime, mtimeNsec: m.mtimeNsec, dev: m.dev, ino: m.ino, uid: m.uid, gid: m.gid, size: m.size };
/// ```
fn stat_of(metadata: &std::fs::Metadata) -> StatData {
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
///  Whether the metadata describes the kind of file and the permission bit the entry records.
/// Why:
///   A path that became a link,
/// a file or an executable of what the entry says has changed whatever its bytes are,
/// and recording
///  its stat data would describe a file the entry no longer names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const agrees = entry.mode === 0o120000 ? m.isSymbolicLink() : m.isFile() && (entry.mode & 0o100) === (m.mode & 0o100);
/// ```
fn mode_agrees(entry_mode: u32, metadata: &std::fs::Metadata) -> bool {
    return match entry_mode & TYPE_BITS {
        SYMBOLIC_LINK => metadata.file_type().is_symlink(),
        REGULAR_FILE => metadata.file_type().is_file() && (entry_mode & EXECUTABLE_BIT != 0) == (metadata.mode() & EXECUTABLE_BIT != 0),
        _ => false,
    };
}

/// What:
///  The bytes of the symbolic link at `path`,
/// with the link's state the same before and after.
/// Why:
///   A link's content is the bytes of its target,
/// which is what its blob holds,
/// and a link replaced while it is read proves nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bytes = readlink(path); // with the same lstat before and after
/// ```
fn link_bytes(path: &Path, before: &std::fs::Metadata) -> Option<Vec<u8>> {
    let target: PathBuf = std::fs::read_link(path).ok()?;
    let after: std::fs::Metadata = std::fs::symlink_metadata(path).ok()?;
    if !same_state(before, &after) {
        return None;
    }
    return Some(target.into_os_string().into_vec());
}

/// The moments of the proof at which a control changes the file under it.
pub struct ProofSeams<'a> {
    /// Called after the file's state was first read and before it is opened.
    pub before_open: &'a mut dyn FnMut(),
    /// Called after the bytes were read and before the state is read again.
    pub after_read: &'a mut dyn FnMut(),
}

/// What:
///  The bytes of the regular file at `path`,
/// or nothing when the file is larger than `CONTENT_LIMIT`,
/// cannot be opened without following a link,
/// or changed while it
///  was read.
/// Why:
///   The read is bounded and holds the one inode that is judged,
/// so a swap or a write during it refuses instead of hashing bytes
///  no stat data describes;
/// `O_NOFOLLOW` keeps a path swapped for a link from being read through it.
///  `after_read` runs between the read and the second look at the file's state,
/// which is where a control writes to it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bytes = readStable(path, before, afterRead); // open O_NOFOLLOW, read at most CONTENT_LIMIT, compare state before and after
/// ```
fn file_bytes(path: &Path, before: &std::fs::Metadata, seams: &mut ProofSeams<'_>) -> Option<(Vec<u8>, std::fs::Metadata)> {
    (seams.before_open)();
    if before.len() > CONTENT_LIMIT {
        return None;
    }
    let file: std::fs::File = std::fs::OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC).open(path).ok()?;
    let held: std::fs::Metadata = file.metadata().ok()?;
    if held.dev() != before.dev() || held.ino() != before.ino() {
        return None;
    }
    let data: Vec<u8> = read_at_most(&file, CONTENT_LIMIT).ok()??;
    (seams.after_read)();
    let after: std::fs::Metadata = file.metadata().ok()?;
    let by_name: std::fs::Metadata = std::fs::symlink_metadata(path).ok()?;
    if !same_state(&held, &after) || !same_state(&held, &by_name) {
        return None;
    }
    return Some((data, held));
}

/// What:
///  Do nothing;
/// the seam of the read runs this unless a control puts work there.
/// Why:
///   The release executable has nothing to do between the read and the second look at the file's state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const doNothing = () => {};
/// ```
fn do_nothing() {}

/// What:
///  The source a landing names:
/// the worktree whose files are read,
/// the object hash of the repository and the date of the index file the entry goes into.
/// Why:
///   Every `recorded` call needs those three and nothing else,
/// so the landing builds this once and the reset asks it per path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const source = new LandingStat({ worktreeRoot, format, stamp });
/// ```
pub struct LandingStat<'a> {
    /// The worktree's top level,
    /// which index paths are relative to.
    worktree_root: &'a Path,
    /// The object hash of the repository.
    format: ObjectFormat,
    /// The date of the index file the entry is written into.
    stamp: IndexStamp,
    /// The bytes this landing has hashed so far.
    hashed: std::cell::Cell<u64>,
}

impl<'a> LandingStat<'a> {
    /// The source of `worktree_root`'s files for an index file dated `stamp` in a repository of `format`.
    pub fn new(worktree_root: &'a Path, format: ObjectFormat, stamp: IndexStamp) -> LandingStat<'a> {
        return LandingStat { worktree_root, format, stamp, hashed: std::cell::Cell::new(0) };
    }
}

impl WorktreeStatSource for LandingStat<'_> {
    fn recorded(&self, path: &[u8], blob: &[u8], mode: u32) -> Option<StatData> {
        return recorded_stat(&WorktreeStatRequest { worktree_root: self.worktree_root, path, blob, mode, format: self.format, stamp: self.stamp, hashed: &self.hashed });
    }
}

/// What:
///  The stat data the landing may record for one path,
/// or nothing when the file cannot prove it holds the committed bytes.
/// Why:
///  `index_reset.rs` calls this for every entry it writes,
/// and the answer decides between the file's stat data and the all-zero
///  value `git reset --no-refresh` writes,
/// so every doubt keeps the zero value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stat = recordedStat({ worktreeRoot, path, blob, mode, format, stamp });
/// ```
pub fn recorded_stat(request: &WorktreeStatRequest<'_>) -> Option<StatData> {
    return recorded_stat_seamed(request, &mut ProofSeams { before_open: &mut do_nothing, after_read: &mut do_nothing });
}

/// What:
///  `recorded_stat` with the moment at which a control changes the file under the read.
/// Why:
///   A control can only show that a file written while it is read records nothing if it can write at that moment,
/// and the
///  release executable's seam does nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stat = recordedStatSeamed(request, afterRead);
/// ```
pub fn recorded_stat_seamed(request: &WorktreeStatRequest<'_>, seams: &mut ProofSeams<'_>) -> Option<StatData> {
    let path: PathBuf = request.worktree_root.join(std::ffi::OsStr::from_bytes(request.path));
    let before: std::fs::Metadata = std::fs::symlink_metadata(path.as_path()).ok()?;
    if !mode_agrees(request.mode, &before) || request.stamp.is_racy(&stat_of(&before)) {
        return None;
    }
    // What:
    //  The bytes one landing may still hash,
    //  the bound `CONTENT_LIMIT` set beside the file it was measured for.
    // Why:
    //  Without the bound a commit that lands a whole worktree would read every byte of it again inside the landing's
    // index
    //  lock,
    //  where an ordinary commit lands a median of 27 KB in the 4,000 newest commits of this repository.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const room = CONTENT_LIMIT - request.hashed.bytes;
    // if (before.size > room) return undefined;
    // ```
    let spent: u64 = request.hashed.get();
    if before.len() > CONTENT_LIMIT.saturating_sub(spent) {
        return None;
    }
    let (data, examined): (Vec<u8>, std::fs::Metadata) = match request.mode & TYPE_BITS {
        SYMBOLIC_LINK => (link_bytes(path.as_path(), &before)?, before),
        _ => file_bytes(path.as_path(), &before, seams)?,
    };
    request.hashed.set(spent + data.len() as u64);
    if blob_name(data.as_slice(), request.format) != request.blob {
        return None;
    }
    return Some(stat_of(&examined));
}

/// Controls for the recorded stat data stay out of the release executable.
#[cfg(test)]
#[path = "index_reset_worktree_stat_tests.rs"]
mod tests;
