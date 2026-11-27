//! What:
//!  The bounds one path's proof reads under,
//!  the moments of the proof a control changes the file at,
//!  and what one path asks the proof for.
//! Why:
//!  The proof never waits and never passes a bound,
//!  so the file-type bits,
//!  the chunk one read takes,
//!  the tries an interrupted read gets and the bytes one landing has hashed so
//!  far are named once here,
//!  and every step a control changes the file at is a seam rather than a
//!  build-time branch in the release executable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type ProofSeams = { beforeOpen(): void; afterRead(): void; read(file, buffer): number };
//! type WorktreeStatRequest = { worktreeRoot; gitDir; commonDir; environment; path;
//!   blob; mode; format; stamp; hashed; indexedAttributes };
//! ```

/// The date of the index file this entry is written into.
use crate::index_stat::IndexStamp;
/// The object hash the entry's name is compared in.
use crate::repo_read_layout::ObjectFormat;
/// The bytes one landing has read so far,
/// which bound what one landing reads.
use std::cell::Cell;
/// The worktree's top level,
/// which index paths are relative to.
use std::path::Path;
/// The file-type bits of an entry's mode.
pub(super) const TYPE_BITS: u32 = 0o170_000;

/// The file-type bits of a regular file.
pub(super) const REGULAR_FILE: u32 = 0o100_000;

/// The one permission bit of a file's mode that an index entry records.
pub(super) const EXECUTABLE_BIT: u32 = 0o100;

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
pub(super) fn read_some(
    file: &std::fs::File,
    buffer: &mut [u8],
) -> std::io::Result<usize> {
    use std::io::Read as _;
    return (&*file).read(buffer);
}

/// The bytes one read takes at most.
pub(super) const READ_CHUNK: usize = 64 * 1024;

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
pub(super) const INTERRUPTED_TRIES: usize = 3;

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
pub(super) fn do_nothing() {}

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
