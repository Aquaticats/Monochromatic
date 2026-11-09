//! What:
//!  The one read of an index file that every private copy of a commit is written from:
//!  the bytes it returned,
//!  the mode and times each copy takes,
//!  and the statement of that read and of each copy.
//! Why:
//!  A commit writes two private copies of the real index,
//!  the captured one the landing reads and the commit's own,
//!  and each of those copies used to read its source again,
//!  so one preparation read an index of a large worktree over and over.
//!  The source of the second copy is a file this process wrote a moment earlier,
//!  and copying that file waited 989 to 3,349 ms inside the kernel in nine of twenty measured commits
//!  while reading the same three megabytes took 4 to 182 ms
//! (`doc/planning/cli-git-commit-latency-under-many-agents.md`).
//!  Reading once and writing every copy from those bytes is what the commit's work needs:
//!  both copies hold the same bytes by construction,
//!  so a staging that lands in the real index between the read and the writes reaches neither,
//!  and no staged work can be lost or recorded under the wrong commit.
//!  The whole file is held in memory once,
//!  which the flags reader and the stat check of the same file already do on this path.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const index = await readIndexOnce(realIndex); await index.writeCopy(capturedIndex); await index.writeCopy(commitIndex);
//! ```

/// The failure a step of the commit reports.
use crate::transaction_git::GitFailure;
/// The names a read of an index and a copy of it are stated under.
use crate::prepare_work_spans::INDEX_COPY;
use crate::prepare_work_spans::INDEX_READ;
use crate::prepare_work_spans::note_span;
/// `FileTimes` carries the copied access and modification times.
use std::fs::FileTimes;
/// `Permissions` carries the copied mode.
use std::fs::Permissions;
/// `Read` fills the buffer the read returns.
use std::io::Read;
/// `Write` writes a copy from that buffer.
use std::io::Write;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// What:
///  The failure that reports an index file which could not be read or copied.
/// Why:
///   The first preparation reads the real index and writes two copies of it,
///  and every one of those steps asks the same question of the operating system,
///  so they report the same failure instead of four messages that name one step each.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const copyIndexFailure = (error: Error) => new GitFailure(`reading or copying the index failed: ${error}`);
/// ```
pub fn copy_index_failure(error: std::io::Error) -> GitFailure {
    return GitFailure(format!("reading or copying the index failed: {error}"));
}

/// What:
///  Reads a whole index file into memory,
///  with the mode and the times its copies take.
/// Why:
///   The times come from the open file and not from its path,
///  so a rename of another index over that path between the read and the stat cannot give this content that file's
///  times.
///  A read that fails is reported to the caller:
/// a commit whose real index cannot be read cannot know what it stages.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const opened = await open(source); const metadata = await opened.stat(); const bytes = await opened.readAll();
/// ```
fn read_index_bytes(source: &Path) -> std::io::Result<IndexBytes> {
    let mut opened: std::fs::File = std::fs::File::open(source)?;
    let metadata: std::fs::Metadata = opened.metadata()?;
    let times: FileTimes = FileTimes::new()
        .set_accessed(metadata.accessed()?)
        .set_modified(metadata.modified()?);
    let mut bytes: Vec<u8> = Vec::new();
    opened.read_to_end(&mut bytes)?;
    return Ok(IndexBytes { bytes, permissions: metadata.permissions(), times });
}

/// What:
///  Writes index bytes as one private copy,
///  with the read source's mode and times.
/// Why:
///   The bytes are already in memory,
///  so the copy is written from them rather than by asking the kernel to copy a file,
///  which is what waited seconds on a source this process had just written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const copy = await create(destination); await copy.write(bytes); await copy.chmod(mode); await copy.utimes(times);
/// ```
fn write_index_bytes(bytes: &IndexBytes, destination: &Path) -> std::io::Result<()> {
    let mut copy: std::fs::File = std::fs::File::create(destination)?;
    copy.write_all(bytes.bytes.as_slice())?;
    copy.set_permissions(bytes.permissions.clone())?;
    return copy.set_times(bytes.times);
}

/// What:
///  The bytes of one index file,
/// read one time,
///  with the mode and the times every copy of them is given.
/// Why:
///   Every private copy of one commit is written from these bytes,
///  so no copy can hold a state the real index reached after this read,
///  and a writer that stages in the real index while the commit prepares changes neither copy.
///  A copy keeps the source's times because Git trusts stat data captured in the same second as a later same-size
///  edit otherwise (issue #544).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type IndexBytes = { bytes: Uint8Array; permissions: Permissions; times: FileTimes };
/// ```
pub struct IndexBytes {
    /// The bytes the one read returned.
    bytes: Vec<u8>,
    /// The mode each copy is given,
    /// read from the open source.
    permissions: Permissions,
    /// The access and modification times each copy is given.
    times: FileTimes,
}

impl IndexBytes {
    /// What:
    ///  Reads a whole index file one time,
    ///  stating how long the read took.
    /// Why:
    ///   This is the only read of that file a preparation makes,
    ///  and a cold read of it is one of the steps a slow preparation must be able to name.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const began = performance.now(); const read = readIndexBytes(source); noteSpan('index read', performance.now() - began); return read;
    /// ```
    pub fn read(source: &Path) -> std::io::Result<IndexBytes> {
        let began: std::time::Instant = std::time::Instant::now();
        let read: std::io::Result<IndexBytes> = read_index_bytes(source);
        note_span(INDEX_READ, began.elapsed());
        return read;
    }

    /// What:
    ///  Writes one private copy of these bytes,
    ///  stating how long the copy took.
    /// Why:
    ///   A copy of an index of a large worktree is megabytes,
    ///  and the copies are where the measured tail of a preparation sits.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const began = performance.now(); const written = writeIndexBytes(this, destination); noteSpan('index copy', performance.now() - began); return written;
    /// ```
    pub fn write_copy(&self, destination: &Path) -> std::io::Result<()> {
        let began: std::time::Instant = std::time::Instant::now();
        let written: std::io::Result<()> = write_index_bytes(self, destination);
        note_span(INDEX_COPY, began.elapsed());
        return written;
    }

    /// What:
    ///  The bytes themselves,
    ///  which the seed decision is made from.
    /// Why:
    ///   The marks that decide how a private index may be seeded are read from these bytes,
    ///  never from the copy on disk,
    ///  so the answer describes the state the copies hold.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// this.bytes
    /// ```
    pub fn bytes(&self) -> &[u8] {
        return self.bytes.as_slice();
    }
}

/// What:
///  Copies an index file to one destination,
///  reading the source one time.
/// Why:
///   A step that needs exactly one copy,
///  such as a rebuild of the private index after the branch moved,
///  takes the same read and the same copy as a preparation that writes two.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await copyIndexFile({ sourcePath, destinationPath });
/// ```
pub fn copy_index_with_timestamps(source: &Path, destination: &Path) -> std::io::Result<()> {
    return IndexBytes::read(source)?.write_copy(destination);
}
