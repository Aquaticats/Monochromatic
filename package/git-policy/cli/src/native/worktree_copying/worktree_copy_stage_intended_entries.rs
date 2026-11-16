//! What:
//!  The record a copy writes into its stage before the first byte of the payload:
//! the repository paths the payload is about to hold,
//!  one per line,
//! and the reader that answers what a stage's payload may hold.
//! Why:
//!  A stage a copy left is removed only when nothing in it can be a person's,
//! and the wrapper cannot tell its own copy of a source's ignored entry from a file a person put at a path
//!  nobody copied:
//! the copied file and the person's file look alike,
//! and most of them are a copy of something the source holds.
//! A birth time does not separate them either:
//! a copy is killed at an arbitrary instruction,
//! so the instant after which nothing of its own was written is not on disk anywhere,
//! while a person's file is written after that instant and after the kill.
//! What does separate them is the copy's own statement of what it is writing,
//!  written before it writes anything:
//! a path the record names is one this copy meant to write,
//! and anything else inside the stage is somebody else's and keeps the folder
//!  (`worktree_copy_stage_claim_reaping.rs`).
//! The record is not installed:
//!  installation reads `payload/` alone, so a file of the container never reaches a worktree,
//! which `worktree_copy_stage_binding.rs` says of its own record too.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await recordTheEntriesThePayloadWillHold(container, entries);
//! const intended = entriesThePayloadMayHold(container); // undefined: no record, or one that does not read
//! ```

use crate::path_bytes_in_records::{bytes_of_hex, hex_of_bytes};
use crate::worktree_copy_error::WorktreeCopyError;
use crate::worktree_copy_journal::write_atomic_unsynced;
use crate::worktree_copy_manifest::WorktreeCopyEntry;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// What:
///  The name of the record inside a stage container.
/// Why:
///   It sits beside `payload/` and the registration,
/// so no entry of the payload can ever have this name,
/// and a reader of the stage's top level knows it is the wrapper's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const INTENDED_NAME = 'intended';
/// ```
pub const INTENDED_NAME: &str = "intended";

/// What:
///  The text of the record:
/// every repository path the payload will hold,
/// hex-encoded,
///  one per line,
/// in the order the manifest walked them.
/// Why:
///   A path is written as its bytes,
/// so a name that is not text is recorded whole and one line is always one path.
/// The list is the copy's statement of intent and not a promise of what stands there:
/// a kill leaves some of these paths missing,
/// which is why the reader adds the directories a path needs to the set it answers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// entries.map((entry) => hexOfBytes(entry.relativePath)).join('\n') + '\n'
/// ```
fn intended_text(entries: &[WorktreeCopyEntry]) -> String {
    let mut text: String = String::with_capacity(entries.len().saturating_mul(48));
    for entry in entries {
        text.push_str(hex_of_bytes(entry.relative_path.as_slice()).as_str());
        text.push('\n');
    }
    return text;
}

/// What:
///  Records the paths the payload of this stage is about to hold,
/// published whole by one rename and flushed by nobody.
/// Why:
///   It is written after the manifest is walked and before the first entry is copied,
/// so a kill at any later instruction leaves a stage that says what it was writing;
/// a kill before it leaves a stage holding no payload entry at all,
/// which loses nothing and is removed.
/// It flushes nothing for the reason `SPEC.md` gives under "The records an add writes for itself":
/// a kill keeps the write in the page cache,
/// and a power cut that loses it only leaves a stage the next command keeps and names,
/// which is the safe direction.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordTheEntriesThePayloadWillHold(container, entries);
/// ```
pub fn record_intended_entries(container: &Path, entries: &[WorktreeCopyEntry]) -> Result<(), WorktreeCopyError> {
    let path: PathBuf = container.join(INTENDED_NAME);
    return write_atomic_unsynced(path.as_path(), intended_text(entries).as_bytes(), "stage payload entries");
}

/// What:
///  The relative paths a stage's payload may hold,
/// with every directory one of them needs,
/// or nothing when the stage holds no record this release writes.
/// Why:
///   A directory is the wrapper's own when a recorded path lies below it,
/// so the set answers for the directories the copy made as well as for the entries themselves.
/// A record that does not read proves nothing,
/// and a stage holding one is kept rather than removed:
/// a person's file there exists nowhere else, and a wrong line here deletes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const intended = entriesThePayloadMayHold(container); // undefined: no record, or one that does not read
/// ```
pub fn entries_the_payload_may_hold(container: &Path) -> Option<HashSet<Vec<u8>>> {
    let path: PathBuf = container.join(INTENDED_NAME);
    // The record is read through the bounded plain-file reader every record of this package is read through: a named
    // pipe, a device or a link at its name is refused instead of waited on, and its size is bounded, so no name a person
    // or another program puts there can hold a command of the repository.
    let text: String = crate::plain_file::read_plain_text(path.as_path()).ok()?;
    let mut allowed: HashSet<Vec<u8>> = HashSet::new();
    for line in text.lines() {
        let path_bytes: Vec<u8> = bytes_of_hex(line)?;
        if path_bytes.is_empty() || path_bytes.contains(&0) {
            return None;
        }
        for relative in ancestors_and_self(path_bytes.as_slice()) {
            allowed.insert(relative);
        }
    }
    return Some(allowed);
}

/// What:
///  A path and every directory above it,
/// relative to the payload's root.
/// Why:
///   The copy creates the directory of every entry it writes,
/// so each of those directories is the wrapper's own although no recorded path names it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ['cache/npm/a.json', 'cache/npm', 'cache']
/// ```
fn ancestors_and_self(path: &[u8]) -> Vec<Vec<u8>> {
    let mut out: Vec<Vec<u8>> = Vec::new();
    let mut end: usize = path.len();
    loop {
        out.push(path[..end].to_vec());
        let Some(separator) = path[..end].iter().rposition(|byte: &u8| { return *byte == b'/'; }) else {
            return out;
        };
        if separator == 0 {
            return out;
        }
        end = separator;
    }
}

/// What:
///  The suffix of the record that says a stage was examined and kept,
/// beside the claim that names it.
/// Why:
///   The comparison against the record of what the copy was writing costs one lookup per payload entry,
/// and a kept stage is named on every command while its claim stands,
/// so the walk is made once for a folder that does not change and its answer is written down:
/// the person reads the same reason each time and no command pays the walk again.
/// The record can only ever save a keep and never a removal:
/// a removal is decided by a fresh walk every time,
/// so a record out of date keeps a folder longer and never deletes anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const KEPT_SUFFIX = '.kept';
/// ```
pub const KEPT_SUFFIX: &str = ".kept";

/// What:
///  The line that holds the reason the person reads.
/// Why:
///   One field per line, read by its own key, as every record of this package is.
const REASON_LINE_PREFIX: &str = "reason=";

/// What:
///  The line that holds one directory the walk read,
/// as its path below the annex and the four numbers of its change and modification times.
/// Why:
///   A directory's own times change when an entry is added to it or removed from it and by nothing else,
/// so they say whether the walk's answer can have changed without reading one entry.
/// There is one line per directory the walk read,
/// which is what makes a change below a subfolder of the payload stale:
/// removing `payload/ignored/pkg/notes.txt` changes the times of `payload/ignored/pkg/`,
/// and that directory is in the list.
const DIRECTORY_LINE_PREFIX: &str = "directory=";

/// What:
///  The times one directory carries,
/// as the kernel reports them.
/// Why:
///   Four numbers rather than two:
/// whole seconds would miss a change made in the same second as the record,
/// and a missed change would keep a folder forever with a reason that is no longer true.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DirectoryTimes = { changed: [number, number]; modified: [number, number] };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DirectoryTimes {
    /// `ctime` as seconds and nanoseconds.
    changed: (i64, i64),
    /// `mtime` as seconds and nanoseconds.
    modified: (i64, i64),
}

/// What:
///  The times of one directory of a stage,
/// or nothing when it cannot be examined.
/// Why:
///   A directory that is gone or unreadable is no state the record was written in,
/// so the answer is "the record does not apply" and the walk runs again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const timesOf = (directory) => { const found = lstatSync(directory); return { changed: [found.ctime, found.ctimeNs], modified: [found.mtime, found.mtimeNs] }; };
/// ```
fn times_of(directory: &Path) -> Option<DirectoryTimes> {
    use std::os::unix::fs::MetadataExt;
    let found: std::fs::Metadata = std::fs::symlink_metadata(directory).ok()?;
    return Some(DirectoryTimes {
        changed: (found.ctime(), found.ctime_nsec()),
        modified: (found.mtime(), found.mtime_nsec()),
    });
}

/// What:
///  The text of one directory's line:
/// the path below the annex,
/// hex-encoded,
/// then its four times.
/// Why:
///   A path is written as its bytes so a name that is not text is recorded whole,
/// and the empty path is the annex itself,
/// which is where a new top-level entry appears.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${DIRECTORY_LINE}${hex(relative)}:${changed.0}.${changed.1}.${modified.0}.${modified.1}`
/// ```
fn directory_line(relative: &Path, times: DirectoryTimes) -> String {
    use std::os::unix::ffi::OsStrExt as _;
    return format!(
        "{DIRECTORY_LINE_PREFIX}{}:{}.{}.{}.{}",
        hex_of_bytes(relative.as_os_str().as_bytes()),
        times.changed.0,
        times.changed.1,
        times.modified.0,
        times.modified.1
    );
}

/// What:
///  One directory's line read back,
/// as the path and the times it records,
/// or nothing when the line is not one this release writes.
/// Why:
///   A line that does not read proves nothing about the folder,
/// so the record does not apply and the walk runs again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const readDirectoryLine = (line) => { const [hex, numbers] = line.split(':'); const [c0, c1, m0, m1] = numbers.split('.').map(Number); return { relative: pathOf(hex), times: { changed: [c0, c1], modified: [m0, m1] } }; };
/// ```
fn read_directory_line(value: &str) -> Option<(PathBuf, DirectoryTimes)> {
    use std::os::unix::ffi::OsStringExt as _;
    let (hex, numbers) = value.split_once(':')?;
    let bytes: Vec<u8> = bytes_of_hex(hex)?;
    let relative: PathBuf = if bytes.is_empty() {
        PathBuf::new()
    } else {
        PathBuf::from(std::ffi::OsString::from_vec(bytes))
    };
    let parts: Vec<&str> = numbers.split('.').collect();
    if parts.len() != 4 {
        return None;
    }
    let mut numbers_read: [i64; 4] = [0; 4];
    for (slot, part) in numbers_read.iter_mut().zip(parts) {
        *slot = part.parse().ok()?;
    }
    return Some((
        relative,
        DirectoryTimes { changed: (numbers_read[0], numbers_read[1]), modified: (numbers_read[2], numbers_read[3]) },
    ));
}

/// What:
///  The reason a kept stage was kept,
/// where every directory the walk read still carries the times it did,
/// or nothing when no such record stands or one of them moved on.
/// Why:
///   The answer is the words the person read the first time,
/// and the times of the directories the walk read are exactly what can change it:
/// an entry added to or removed from any of them,
/// at any depth,
/// makes the record stale and the folder is walked again.
/// So a person who removes the file the line named,
/// at the top of the payload or below a subfolder of it,
/// lets the next command settle the folder.
/// One stat per directory is what the reuse costs,
/// which is far less than one lookup per entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reason = reasonTheStageWasKept(claim, container); // undefined before the first look, or after a change
/// ```
pub fn reason_the_stage_was_kept(claim: &Path, container: &Path) -> Option<String> {
    // The bounded plain-file reader, for the reason the record of what the copy was writing is read through it.
    let text: String = crate::plain_file::read_plain_text(kept_record_beside(claim).as_path()).ok()?;
    let mut reason: Option<&str> = None;
    let mut directories: Vec<(PathBuf, DirectoryTimes)> = Vec::new();
    for line in text.lines() {
        if let Some(value) = line.strip_prefix(REASON_LINE_PREFIX) {
            reason = Some(value);
            continue;
        }
        if let Some(value) = line.strip_prefix(DIRECTORY_LINE_PREFIX) {
            directories.push(read_directory_line(value)?);
            continue;
        }
        // A line this release does not write proves nothing about the folder.
        return None;
    }
    let answer: String = reason.map(str::to_string).filter(|reason_text: &String| { return !reason_text.is_empty(); })?;
    // A record that names no directory says nothing about what the walk read, so it is no record of this release.
    if directories.is_empty() {
        return None;
    }
    for (relative, recorded) in directories {
        let directory: PathBuf = container.join(relative.as_path());
        if times_of(directory.as_path()) != Some(recorded) {
            return None;
        }
    }
    return Some(answer);
}

/// What:
///  Writes down that this stage was examined and kept,
/// with the reason and the times of every directory the walk read,
/// published whole by one rename and flushed by nobody.
/// Why:
///   It saves only the walk, never the decision,
/// and a write that fails only means the next command walks again.
/// The record is only believed while every directory it names carries the times it recorded,
/// so a folder that changed anywhere below a subfolder of its payload is walked again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordTheStageWasKept(claim, container, reason, examined);
/// ```
pub fn record_the_stage_was_kept(claim: &Path, container: &Path, reason: &str, examined: &[PathBuf]) -> Result<(), WorktreeCopyError> {
    let mut recorded: String = format!("{REASON_LINE_PREFIX}{reason}\n");
    let mut written: usize = 0;
    for relative in examined {
        let directory: PathBuf = container.join(relative.as_path());
        let Some(times): Option<DirectoryTimes> = times_of(directory.as_path()) else { continue };
        recorded.push_str(directory_line(relative.as_path(), times).as_str());
        recorded.push('\n');
        written += 1;
    }
    if written == 0 {
        return Ok(());
    }
    return write_atomic_unsynced(kept_record_beside(claim).as_path(), recorded.as_bytes(), "stage kept record");
}

/// What:
///  The path of the record that says a stage was kept,
/// for the claim that names the stage.
/// Why:
///   It keeps the claim's own name and changes only its suffix,
/// so one reader finds it and one release removes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// claim.withExtension(KEPT_SUFFIX)
/// ```
pub fn kept_record_beside(claim: &Path) -> PathBuf {
    return claim.with_extension(&KEPT_SUFFIX[1..]);
}
