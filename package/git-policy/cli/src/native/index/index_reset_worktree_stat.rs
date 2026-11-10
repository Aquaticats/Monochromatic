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
//!  So the stat data here is recorded only when all five hold:
//!
//! - The path is a regular file,
//! and every directory on the way is a real one on the worktree's device,
//! reached below a held root without following a link.
//!   A symbolic link is left to Git:
//! it is rare,
//! the
//!  zero value only costs a refresh,
//! and a link read across a swap of its parent directory cannot be proven to be the one examined.
//! - The file's raw bytes hash to the entry's object name,
//! read as a blob.
//! - No attribute Git applies converts these bytes,
//! and the bytes hold no carriage return.
//!   A clean filter,
//! `ident`
//!  or a `working-tree-encoding` can leave raw bytes equal to the blob while Git's view of the file is another blob,
//! and a
//!  line-ending rule may strip a carriage return on the way into the index,
//! so such a path keeps the zero stat data and costs a refresh.
//!   The attributes are read in
//!  process (`worktree_clean_content.rs`,
//! `conversion_is_the_identity`),
//! from the files Git itself would read,
//! and where their location cannot be established the
//!  path keeps the zero value.
//! - The state of the one inode that was read is the same before,
//! during and after the read and the path still names it,
//! and the
//!  file is older than the index file this entry is written into,
//! compared to the nanosecond as Git's `is_racy_stat` compares them.
//!   An entry as
//!  new as the index is what Git calls racily clean,
//! so it keeps the zero value and both readers hash the file;
//! a file written
//!  earlier in the index file's own second is older than the index on a file system that stores nanoseconds,
//! so the entries a
//!  person's edit just made record their stat data and the next commit starts no Git for them.
//! - The change time the file had when it was read has nanoseconds.
//!   A file system that stores whole seconds reports a change time with none,
//! and a same-size rewrite whose modification time is put back inside that second leaves every field this proof compares
//!  unchanged,
//! so such a file keeps the zero stat data and costs a refresh.
//!   The attributes reader refuses such a file for
//!  the same reason (`worktree_clean_content_attributes.rs`,
//! `attribute_bytes`).
//!
//! The read never waits and never passes a bound:
//! a path that is not a regular file is refused by its type,
//! the open does
//!  not block (a fifo renamed over the path is opened without waiting for a writer and refused by its type),
//! and the bytes are
//!  read at most up to what is left of `CONTENT_LIMIT` for the whole landing.
//! A read the system interrupts is retried at most `INTERRUPTED_TRIES` times,
//! so a source that interrupts every read cannot hold the landing's index lock:
//!  the path then keeps the zero stat data,
//! and the next command that needs it hashes the file through Git.
//! Every byte that reaches the disk is charged,
//! including a
//!  read the stability checks later refuse,
//! so a landing that lands a whole worktree reads at most `CONTENT_LIMIT` bytes of it inside the
//!  landing's index lock.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const stat: StatData | undefined = recordedStat({ worktreeRoot, path, blob, mode, format, stamp, hashed });
//! ```

use crate::fs_beneath::{entry_metadata_at, open_directory_beneath, open_file_at, plain_components};
use crate::index_reset_worktree_stat_source::WorktreeStatSource;
use crate::index_stat::{IndexStamp, StatData};
use crate::repo_read_layout::ObjectFormat;
use crate::subvolume_sha::{sha1, sha256};
use crate::worktree_clean_content::{CONTENT_LIMIT, ContentJudge};
use crate::worktree_clean_tree_walk::ContentTarget;
use std::cell::{Cell, RefCell};
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

/// The file-type bits of an entry's mode.
const TYPE_BITS: u32 = 0o170_000;

/// The file-type bits of a regular file.
const REGULAR_FILE: u32 = 0o100_000;

/// The one permission bit of a file's mode that an index entry records.
const EXECUTABLE_BIT: u32 = 0o100;

/// The moments of the proof at which a control changes the file under it,
/// and the read itself.
pub struct ProofSeams<'a> {
    /// Called after the file's state was first read and before it is opened.
    pub before_open: &'a mut dyn FnMut(),
    /// Called after the bytes were read and before the state is read again.
    pub after_read: &'a mut dyn FnMut(),
    /// One read of the file,
    /// so a control can count what the file handed over or fail after some of it.
    pub read: &'a mut dyn FnMut(&std::fs::File, &mut [u8]) -> std::io::Result<usize>,
}

/// What:
///  One read of `file` into `buffer`,
/// as the release executable makes it.
/// Why:
///  The seams carry every step a control changes,
/// and the operating system's read is the one the release executable takes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const readSome = (file, buffer) => readSync(file, buffer);
/// ```
fn read_some(file: &std::fs::File, buffer: &mut [u8]) -> std::io::Result<usize> {
    use std::io::Read as _;
    return (&*file).read(buffer);
}

/// The bytes one read takes at most.
const READ_CHUNK: usize = 64 * 1024;

/// How many times one file-content read is tried again after the system interrupts it.
/// Why:
///  An interruption is not a fact about the file,
/// so one is retried;
/// a source that interrupts every attempt for ever would otherwise hold the landing's index lock
///  without a bound,
/// which the owner's rule forbids ("nothing should ever wait forever or indefinitely").
/// Three tries is the same count the attributes reader takes
/// (`worktree_clean_content_attributes.rs`,
/// `INTERRUPTED_TRIES`):
/// a single
///  signal is the interruption the retry exists for,
/// and a source that interrupts three reads in a row is one this reader refuses rather than waits on.
const INTERRUPTED_TRIES: usize = 3;

/// What:
///  Do nothing;
/// the seams of the proof run this unless a control puts work there.
/// Why:
///   The release executable has nothing to do between those steps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const doNothing = () => {};
/// ```
fn do_nothing() {}

/// What:
///  One path whose entry a reset writes,
/// and what the entry says the file holds.
/// Why:
///   The proof reads the file and needs the value it must hash to,
/// the kind of file the entry names,
/// the repository's object hash,
///  the date of the index file and the bytes this landing has hashed so far.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WorktreeStatRequest = { worktreeRoot: string; gitDir: string; commonDir: string; environment: ConversionEnvironment; path: Uint8Array; blob: Uint8Array; mode: number; format: ObjectFormat; stamp: IndexStamp; hashed: { bytes: number }; indexedAttributes: Uint8Array[] };
/// ```
pub struct WorktreeStatRequest<'a> {
    /// The worktree's top level,
    /// which index paths are relative to.
    pub worktree_root: &'a Path,
    /// The worktree's own Git directory.
    pub git_dir: &'a Path,
    /// The shared Git directory.
    pub common_dir: &'a Path,
    /// The attribute files Git would read.
    pub environment: &'a crate::worktree_clean_content_attribute_sources::ConversionEnvironment,
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
    /// The bytes this landing has read so far,
    /// which bound what one landing reads (`CONTENT_LIMIT`).
    pub hashed: &'a Cell<u64>,
    /// The directories whose `.gitattributes` the index holds,
    /// in path order,
    /// which Git reads where the worktree's copy is missing.
    pub indexed_attributes: &'a [Vec<u8>],
}

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
fn stamp_proves_one_version(changed: (i64, i64)) -> bool {
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
fn read_by_chunks(file: &std::fs::File, room: u64, spent: &Cell<u64>, seams: &mut ProofSeams<'_>) -> Option<Vec<u8>> {
    let mut data: Vec<u8> = Vec::new();
    let mut interrupted: usize = 0;
    let mut buffer: Vec<u8> = vec![0; READ_CHUNK];
    while (data.len() as u64) < room {
        let want: usize = usize::try_from(room - data.len() as u64).unwrap_or(usize::MAX).min(READ_CHUNK);
        match (seams.read)(file, &mut buffer[..want]) {
            Ok(0) => break,
            Ok(taken) => {
                let took: usize = taken.min(want);
                data.extend_from_slice(&buffer[..took]);
                spent.set(spent.get() + took as u64);
            }
            // An interruption is not a fact about the file, so it is tried again;
            // a source that interrupts every try is refused rather than waited on.
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted && interrupted < INTERRUPTED_TRIES => {
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
fn mode_agrees(entry_mode: u32, metadata: &std::fs::Metadata) -> bool {
    if entry_mode & TYPE_BITS != REGULAR_FILE {
        return false;
    }
    return metadata.file_type().is_file() && (entry_mode & EXECUTABLE_BIT != 0) == (metadata.mode() & EXECUTABLE_BIT != 0);
}

/// What:
///  The content decision's judge for a landing,
/// with its attribute reads charged to the landing's bound.
/// Why:
///   The attributes a path's bytes are judged against are read under the landing's index lock,
/// so those bytes are part of
///  what the landing bounds,
/// and a file that does not fit is refused (a refresh) rather than read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const judge = new ContentJudge({ root, gitDir, commonDir, environment, format }); judge.chargeAttributesTo(hashed);
/// ```
fn charged_judge<'a>(root: &'a std::fs::File, target: &ContentTarget<'a>, hashed: &'a Cell<u64>) -> ContentJudge<'a> {
    let mut judge: ContentJudge<'a> = ContentJudge::new(root, target.git_dir, target.common_dir, target.environment, target.format);
    judge.charge_attributes_to(hashed);
    return judge;
}

/// What:
///  The held directory out of the cached entry.
/// Why:
///   The cache holds the names it was opened for beside the handle,
/// and only the handle is read out of the entry.
/// It is duplicated for the caller there,
/// so no borrow of the cache outlives the look-up that asked for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theDirectory = (held) => held?.opened;
/// ```
fn the_directory(held: &Option<(Vec<OsString>, std::fs::File)>) -> Option<&std::fs::File> {
    let (_names, opened) = held.as_ref()?;
    return Some(opened);
}

/// What:
///  The source a landing names:
/// the worktree held open,
/// the content decision's own attributes,
/// the object hash and the date of the index file.
/// Why:
///   A landing proves many paths against one worktree,
/// so the held root,
/// the attribute files of each directory and the read
///  budget are kept here,
/// and no closure carries either side's state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const source = new LandingStat({ root, target, stamp });
/// ```
pub struct LandingStat<'a> {
    /// The worktree's top level,
    /// held open.
    root: &'a std::fs::File,
    /// The attributes Git would apply to a path,
    /// which decide whether a conversion may change its bytes.
    judge: RefCell<ContentJudge<'a>>,
    /// The directories opened so far:
    /// the names below the root and the held directory,
    /// because the entries come in path order and most share one.
    directory: RefCell<Option<(Vec<OsString>, std::fs::File)>>,
    /// The object hash of the repository.
    format: ObjectFormat,
    /// The date of the index file the entry is written into.
    stamp: IndexStamp,
    /// The bytes this landing has read so far.
    hashed: &'a Cell<u64>,
    /// The directories whose `.gitattributes` the index holds.
    indexed_attributes: &'a [Vec<u8>],
}

impl<'a> LandingStat<'a> {
    /// The source of the worktree held open as `root`,
    /// whose index file is dated `stamp`.
    pub fn new(root: &'a std::fs::File, target: &ContentTarget<'a>, stamp: IndexStamp, hashed: &'a Cell<u64>, indexed_attributes: &'a [Vec<u8>]) -> LandingStat<'a> {
        return LandingStat {
            root,
            judge: RefCell::new(charged_judge(root, target, hashed)),
            directory: RefCell::new(None),
            format: target.format,
            stamp,
            hashed,
            indexed_attributes,
        };
    }

    /// What:
    ///  The directory holding the entry at `parents`,
    /// opened below the held root without following a link and kept for the next entry,
    /// as a handle of the caller's own.
    /// Why:
    ///   A held directory makes every look-up below it immune to a swap of an ancestor,
    /// which a path built from the
    ///  worktree root is not.
    /// The handle is duplicated for the caller rather than lent out of the cache:
    /// this method replaces the cache whenever another directory is asked for,
    /// and a borrow of the cache alive across that call panics
    ///  (`RefCell` holds one mutable borrow,
    /// and the third review found exactly that with the root and an ancestor's `.gitattributes`).
    /// A duplicate costs one `dup` a path and cannot outlive the call it is made for.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const directory = (cache.get(parents) ?? openBelow(heldRoot, parents)).dup();
    /// ```
    fn directory(&self, parents: &[OsString]) -> Option<std::fs::File> {
        let missed: bool = match self.directory.borrow().as_ref() {
            Some((known, _opened)) => known.as_slice() != parents,
            None => true,
        };
        if missed {
            let opened: std::fs::File = open_directory_beneath(self.root, parents).ok()?;
            self.directory.replace(Some((parents.to_vec(), opened)));
        }
        let cache: std::cell::Ref<'_, Option<(Vec<OsString>, std::fs::File)>> = self.directory.borrow();
        let cached: &std::fs::File = the_directory(&cache)?;
        return cached.try_clone().ok();
    }

    /// What:
    ///  The bytes of the regular file the entry names,
    /// read at most up to `room`,
    /// with what was read charged to the landing whatever becomes of it.
    /// Why:
    ///   A read that is later refused still reached the disk,
    /// so the bound is on the reading;
    /// the file is opened without waiting (`O_NONBLOCK`,
    /// `fs_beneath::open_file_at`),
    /// so a fifo
    ///  renamed over the path cannot hold the landing's index lock;
    /// the handle is judged by its type and by the inode the first look saw,
    /// so a swap
    ///  between the two is refused.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function readCharged(directory, name, before, room) {
    ///   seams.beforeOpen(); const file = openFileAt(directory, name); const held = fstat(file);
    ///   if (!held.isFile() || held.ino !== before.ino) return; const data = readAtMost(file, room);
    ///   hashed += data === undefined ? room : data.length; if (data === undefined) return;
    ///   seams.afterRead(); ... stability checks ...; return data;
    /// }
    /// ```
    fn read_charged(&self, directory: &std::fs::File, name: &OsStr, before: &std::fs::Metadata, room: u64, seams: &mut ProofSeams<'_>) -> Option<(Vec<u8>, std::fs::Metadata)> {
        (seams.before_open)();
        let file: std::fs::File = open_file_at(directory, name).ok()?;
        let held: std::fs::Metadata = file.metadata().ok()?;
        if !held.file_type().is_file() || held.dev() != before.dev() || held.ino() != before.ino() {
            return None;
        }
        // A file whose change time has no nanoseconds is on a file system that cannot tell two rewrites in one second
        // apart, so no state it reports can prove the bytes were one version (a trial review's seventh finding).
        if !stamp_proves_one_version((held.ctime(), held.ctime_nsec())) {
            return None;
        }
        // A file whose handle already reports more than the room the landing has is not read at all:
        // the size the handle gives decides, not an earlier look at the path.
        if held.len() > room {
            return None;
        }
        let data: Vec<u8> = read_by_chunks(&file, room, self.hashed, seams)?;
        (seams.after_read)();
        let after: std::fs::Metadata = file.metadata().ok()?;
        let by_name: std::fs::Metadata = entry_metadata_at(directory, name).ok()?;
        if !same_state(&held, &after) || !same_state(&held, &by_name) {
            return None;
        }
        return Some((data, after));
    }

    /// What:
    ///  Whether a directory whose `.gitattributes` the index holds has the worktree's copy.
    /// Why:
    ///   Git reads the index's copy when the worktree's is missing (`attr.c`,
    /// `prepare_attr_stack` falls from the file to the index),
    /// measured on Git 2.56.0 on 2026-11-09:
    /// `git check-attr filter -- a.txt` still answers from the index after the worktree's `.gitattributes` is
    ///  removed,
    /// and a sparse checkout that leaves the file outside its cone reads the index the same way.
    /// A proof that reads only the worktree sees no attributes there,
    /// so the path keeps the zero stat data instead.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const attributesMissing = (parents) => !exists(join(heldDirectory(parents), '.gitattributes'));
    /// ```
    fn attributes_missing(&self, parents: &[OsString]) -> Option<bool> {
        let directory: std::fs::File = self.directory(parents)?;
        return Some(entry_metadata_at(&directory, OsStr::new(".gitattributes")).is_err());
    }

    /// What:
    ///  The refusal for a path whose attributes could come from the index because the worktree's copy is gone.
    /// Why:
    ///   Refusing costs a refresh and recording wrongly hides an edit,
    /// so a directory the index can supply attributes for is checked from the root
    ///  down before any of its files is trusted.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// for (const directory of ancestors(parents)) if (indexed.has(directory) && attributesMissing(directory)) return undefined;
    /// ```
    fn attributes_come_only_from_the_index(&self, parents: &[OsString]) -> bool {
        if self.indexed_attributes.is_empty() {
            return false;
        }
        if self.indexed_attributes.binary_search(&Vec::new()).is_ok() && matches!(self.attributes_missing(&[]), None | Some(true)) {
            return true;
        }
        for depth in 1..=parents.len() {
            let mut prefix: Vec<u8> = Vec::new();
            for name in &parents[..depth] {
                if !prefix.is_empty() {
                    prefix.push(b'/');
                }
                prefix.extend_from_slice(name.as_os_str().as_bytes());
            }
            if self.indexed_attributes.binary_search(&prefix).is_err() {
                continue;
            }
            if matches!(self.attributes_missing(&parents[..depth]), None | Some(true)) {
                return true;
            }
        }
        return false;
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
    /// const stat = source.recorded(path, blob, mode);
    /// ```
    fn record(&self, path: &[u8], blob: &[u8], mode: u32, seams: &mut ProofSeams<'_>) -> Option<StatData> {
        let names: Vec<OsString> = plain_components(Path::new(OsStr::from_bytes(path))).ok()?;
        let (name, parents): (&OsString, &[OsString]) = names.split_last()?;
        let directory: std::fs::File = self.directory(parents)?;
        let before: std::fs::Metadata = entry_metadata_at(&directory, name.as_os_str()).ok()?;
        if !mode_agrees(mode, &before) || self.stamp.is_racy(&stat_of(&before)) {
            return None;
        }
        // A conversion Git may apply can leave these bytes looking equal to the blob while Git's view of the file is not.
        if self.attributes_come_only_from_the_index(parents) {
            return None;
        }
        if self.judge.borrow_mut().conversion_is_the_identity(path, parents).is_err() {
            return None;
        }
        let room: u64 = CONTENT_LIMIT.saturating_sub(self.hashed.get());
        if before.len() > room {
            return None;
        }
        let (data, examined): (Vec<u8>, std::fs::Metadata) = self.read_charged(&directory, name.as_os_str(), &before, room, seams)?;
        // A line-ending rule Git applies whatever the attributes say may strip a carriage return on the way into the index.
        if data.contains(&b'\r') || blob_name(data.as_slice(), self.format) != blob {
            return None;
        }
        return Some(stat_of(&examined));
    }
}

impl WorktreeStatSource for LandingStat<'_> {
    fn recorded(&self, path: &[u8], blob: &[u8], mode: u32) -> Option<StatData> {
        return self.record(path, blob, mode, &mut ProofSeams { before_open: &mut do_nothing, after_read: &mut do_nothing, read: &mut read_some });
    }
}

/// What:
///  The stat data one path's file provably has,
/// asked with the seams a control changes the file at.
/// Why:
///   The source a landing uses keeps the held root and the attribute files across paths,
/// and a caller that proves one path has nothing to keep,
///  so this reads the one file from the worktree root itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stat = recordedStatSeamed(request, seams);
/// ```
pub fn recorded_stat_seamed(request: &WorktreeStatRequest<'_>, seams: &mut ProofSeams<'_>) -> Option<StatData> {
    let root: std::fs::File = crate::fs_beneath::open_root(request.worktree_root).ok()?;
    let target: ContentTarget<'_> = ContentTarget { root: request.worktree_root, git_dir: request.git_dir, common_dir: request.common_dir, environment: request.environment, format: request.format };
    let source: LandingStat<'_> = LandingStat::new(&root, &target, request.stamp, request.hashed, request.indexed_attributes);
    return source.record(request.path, request.blob, request.mode, seams);
}

/// What:
///  `recorded_stat_seamed` with the seams that change nothing.
/// Why:
///  The release executable has no control under the read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stat = recordedStat(request);
/// ```
pub fn recorded_stat(request: &WorktreeStatRequest<'_>) -> Option<StatData> {
    return recorded_stat_seamed(request, &mut ProofSeams { before_open: &mut do_nothing, after_read: &mut do_nothing, read: &mut read_some });
}

/// Controls for the recorded stat data stay out of the release executable.
#[cfg(test)]
#[path = "index_reset_worktree_stat_tests.rs"]
mod tests;
