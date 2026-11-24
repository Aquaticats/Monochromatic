//! What:
//!  The frozen capture of a clean source the clone route proves its clone
//!  against: the index's bytes by size and digest, and every tracked path's
//!  kind, size, times and execute bit.
//! Why:
//!  A snapshot and a clone are taken after the capture, so a write between the
//!  two would otherwise reach the new worktree unproven. The proof reads the
//!  clone's own files and no Git run against its copied administration
//!  (`doc/planning/cli-git-zfs-dirty-main-restore-on-rung-1-design.md`,
//!  "The frozen proof", parts C1, C2 and C5).
//!  Step 10 builds the parts a clean source needs: the index's bytes, every
//!  tracked path's facts, and the commit the caller's add checks out. Step 11 adds
//!  the two digests a dirty source needs (C3 and C4): the content of every path
//!  the index's stat data cannot vouch for, and the content of every path in the
//!  racy set, which is the set a same-tick write can hide in.
//!  A path the source has deleted is recorded as absent and carries no digest,
//!  because there are no bytes to digest and the proof's kind comparison is what
//!  refuses such a path coming back.
//!  Every path is read below a held handle of the source root, one plain name
//!  at a time, so a directory swapped for a link while the capture runs is an
//!  error and not a detour.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const capture = captureTheSource({ sourceRoot, indexBytes, hashLength, nonce, ... });
//! ```

use crate::fs_beneath::open_file_at;
use crate::subvolume_index::StoredStat;
use crate::subvolume_index_files::matches_stored_exactly;
use crate::subvolume_sha::sha256;
use crate::worktree_clean::stored_stats;
use crate::worktree_clone_binding_record::binding_encoding::bytes_as_hexadecimal;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use sha2::Digest as _;
use std::ffi::OsStr;
use std::io::Read as _;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

/// The widest path one captured entry may name, which the kernel's `PATH_MAX`
/// allows without its terminator.
pub const MAXIMUM_PATH_BYTES: usize = 4095;

/// What:
///  What one tracked path is in the source at the capture's instant.
/// Why:
///  The clone's own file is compared with exactly this,
///  and an absent path is recorded so a write that brings it back is comparable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Kind = 'regular' | 'link' | 'absent';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapturedKind {
    /// A regular file.
    Regular,
    /// A symbolic link.
    Link,
    /// A tracked path the source has deleted.
    Absent,
}

impl CapturedKind {
    /// The word the record writes.
    pub fn word(self) -> &'static str {
        return match self {
            CapturedKind::Regular => "regular",
            CapturedKind::Link => "link",
            CapturedKind::Absent => "absent",
        };
    }

    /// The kind one word names.
    pub fn parse(word: &str) -> Option<CapturedKind> {
        return match word {
            "regular" => Some(CapturedKind::Regular),
            "link" => Some(CapturedKind::Link),
            "absent" => Some(CapturedKind::Absent),
            _ => None,
        };
    }
}

/// What:
///  One tracked path's facts at the capture.
/// Why:
///  Every field is compared with the clone's own file, and the refusal names
///  the path and the field that differed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CapturedEntry = { path; kind; size; mtime; ctime; executable; linkTarget? };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapturedEntry {
    /// The path below the source root,
    /// `/` separated as the index writes it.
    pub path: Vec<u8>,
    /// What stands there.
    pub kind: CapturedKind,
    /// The size in bytes,
    /// zero for an absent path.
    pub size: u64,
    /// The modification time,
    /// seconds and nanoseconds.
    pub mtime: (i64, u32),
    /// The change time,
    /// seconds and nanoseconds.
    pub ctime: (i64, u32),
    /// Whether the owner may execute it.
    pub executable: bool,
    /// A symbolic link's target bytes.
    pub link_target: Option<Vec<u8>>,
}

/// What:
///  One path's content digest at the capture,
///  for the entries the index's stat data cannot vouch for (C3) and the entries
///  in the racy set (C4).
/// Why:
///  A write between the capture and the snapshot that leaves every compared
///  field equal is caught by the bytes alone, and these digests are the bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CapturedDigest = { path; digest };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapturedDigest {
    /// The path below the source root.
    pub path: Vec<u8>,
    /// The SHA-256 of the file's bytes,
    /// or of a link's target bytes.
    pub digest: [u8; 32],
}

/// What:
///  The index's own facts at the capture.
/// Why:
///  D1 compares the clone's copied index with these, so any write to the index
///  between the capture and the snapshot refuses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CapturedIndex = { size; mtime; digest };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapturedIndex {
    /// The index file's length.
    pub size: u64,
    /// Its modification time.
    pub mtime: (i64, u32),
    /// The SHA-256 of its bytes.
    pub digest: [u8; 32],
}

/// What:
///  Everything one capture holds.
/// Why:
///  The capture is the record a resumed add proves its clone from, so it names
///  the add by nonce and the repository and source it was taken in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Capture = { nonce; repository; source; checkoutGuid; index; head; sourceHead; entries };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Capture {
    /// The add's nonce,
    /// which says whose capture this is.
    pub nonce: RequestNonce,
    /// The Git common directory's path.
    pub repository: Vec<u8>,
    /// The source root's path.
    pub source: Vec<u8>,
    /// The checkout dataset's GUID.
    pub checkout_guid: DatasetGuid,
    /// The index's facts.
    pub index: CapturedIndex,
    /// The commit the new worktree's `HEAD` names after registration.
    pub head: String,
    /// The commit the source's own `HEAD` names.
    pub source_head: String,
    /// One entry per tracked path,
    /// in the index's order.
    pub entries: Vec<CapturedEntry>,
    /// One digest per path C3 or C4 named,
    /// in the index's order.
    pub digests: Vec<CapturedDigest>,
}

/// What:
///  What the capture reads the source with.
/// Why:
///  The caller holds the proven index bytes and the registration, and this
///  value keeps the capture's own steps free of the add's other state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CaptureInput = { sourceRoot; repository; indexBytes; hashLength; nonce; head; sourceHead; checkoutGuid };
/// ```
pub struct CaptureInput<'a> {
    /// The main checkout's root.
    pub source_root: &'a Path,
    /// The repository's common Git directory.
    pub repository: &'a Path,
    /// The index's proven bytes.
    pub index_bytes: &'a [u8],
    /// The index file's modification time,
    /// seconds and nanoseconds.
    pub index_mtime: (i64, u32),
    /// The repository's object name length in bytes.
    pub hash_length: usize,
    /// The add's nonce.
    pub nonce: &'a RequestNonce,
    /// The commit the new worktree's `HEAD` names.
    pub head: String,
    /// The commit the source's `HEAD` names.
    pub source_head: String,
    /// The checkout dataset's GUID.
    pub checkout_guid: DatasetGuid,
}

/// The mode bits that carry an entry's type.
const TYPE_MASK: u32 = 0o170_000;

/// The mode that says an entry is a symbolic link.
const LINK_MODE: u32 = 0o120_000;

/// The mode that says an entry is a regular file.
const REGULAR_MODE: u32 = 0o100_000;

/// What:
///  A path the capture cannot record, in words naming the path and the reason.
/// Why:
///  A refusal here ends the add before the snapshot, with nothing mutated but
///  the claim, the slot and the registration, and the person needs the path.
pub type CaptureRefusal = String;

/// What:
///  Take the frozen capture of a clean source.
/// Why:
///  It runs immediately before the snapshot, and every read is below a held
///  handle of the source root.
///
/// # Errors
/// A tracked path is not a regular file or a link,
/// a path is not a plain name,
/// or one read of the source failed.
pub fn capture_the_source(input: &CaptureInput<'_>) -> Result<Capture, CaptureRefusal> {
    let Some(entries): Option<Vec<crate::worktree_clean_stat_verdicts::StatEntry>> = stored_stats(input.index_bytes, input.hash_length) else {
        return Err(String::from("the source's index is not one this reader knows, so the clone cannot be proven against it"));
    };
    let tick: Option<u64> = coarse_tick_nanos();
    let mut captured: Vec<CapturedEntry> = Vec::with_capacity(entries.len());
    let mut digests: Vec<CapturedDigest> = Vec::new();
    super::add_capture_tree::walk_paths(input.source_root, entries.iter(), |entry| return entry.path.as_slice(), |entry, path, found, parent, name| {
        let kind_bits: u32 = entry.mode & TYPE_MASK;
        if kind_bits != REGULAR_MODE && kind_bits != LINK_MODE {
            return Err(format!(
                "the tracked path {} is a folder or a submodule, which the clone route cannot prove (mode {:o}); Ways forward: remove it from the checkout or make the worktree with Git's own checkout",
                path.display(),
                entry.mode
            ));
        }
        if (kind_bits == LINK_MODE) != (found.kind == super::add_capture_tree::FoundKind::Link) {
            return Err(format!("the tracked path {} is {} in the index and {} on disk; Ways forward: check the source out again", path.display(), if kind_bits == LINK_MODE { "a link" } else { "a regular file" }, found.kind.word()));
        }
        let captured_entry: CapturedEntry = CapturedEntry {
            path: entry.path.clone(),
            kind: match found.kind {
                super::add_capture_tree::FoundKind::Link => CapturedKind::Link,
                super::add_capture_tree::FoundKind::Absent => CapturedKind::Absent,
                super::add_capture_tree::FoundKind::Regular => CapturedKind::Regular,
            },
            size: found.size,
            mtime: found.mtime,
            ctime: found.ctime,
            executable: found.executable,
            link_target: found.link_target.clone(),
        };
        let kind: CapturedKind = captured_entry.kind;
        captured.push(captured_entry);
        if kind == CapturedKind::Absent {
            return Ok(());
        }
        let vouched: bool = found.fields.as_ref().map(|fields| return matches_stored_exactly(fields, &entry.stored)).unwrap_or(false);
        if vouched && !is_racy(&entry.stored, input.index_mtime, tick) {
            return Ok(());
        }
        let digest: [u8; 32] = match kind {
            CapturedKind::Regular => digest_the_file(parent, name, path)?,
            CapturedKind::Link => sha256(found.link_target.as_deref().unwrap_or_default()),
            CapturedKind::Absent => return Ok(()),
        };
        digests.push(CapturedDigest { path: entry.path.clone(), digest });
        return Ok(());
    })?;
    return Ok(Capture {
        nonce: input.nonce.clone(),
        repository: input.repository.as_os_str().as_bytes().to_vec(),
        source: input.source_root.as_os_str().as_bytes().to_vec(),
        checkout_guid: input.checkout_guid,
        index: CapturedIndex { size: input.index_bytes.len() as u64, mtime: input.index_mtime, digest: sha256(input.index_bytes) },
        head: input.head.clone(),
        source_head: input.source_head.clone(),
        entries: captured,
        digests,
    });
}

/// What:
///  The kernel's coarse clock resolution in nanoseconds,
///  which is the tick a ZFS write's change and modification times come from.
/// Why:
///  C4's set is the entries whose recorded time is not at least one tick older
///  than the index's own modification time, and an answer this process cannot
///  read treats every entry as racy, which is the safe direction.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const tick = clockGetres(CLOCK_REALTIME_COARSE) ?? Number.POSITIVE_INFINITY;
/// ```
fn coarse_tick_nanos() -> Option<u64> {
    let mut resolution: libc::timespec = libc::timespec { tv_sec: 0, tv_nsec: 0 };
    // SAFETY: `clock_getres` writes the resolution into the pointer it is given, which is this stack value.
    let answered: libc::c_int = unsafe { libc::clock_getres(libc::CLOCK_REALTIME_COARSE, &mut resolution) };
    if answered != 0 {
        return None;
    }
    let nanos: u64 = u64::try_from(resolution.tv_sec).unwrap_or(u64::MAX).saturating_mul(1_000_000_000).saturating_add(u64::try_from(resolution.tv_nsec).unwrap_or(u64::MAX));
    return Some(nanos.max(1));
}

/// What:
///  Whether one entry is in the racy set:
///  its recorded modification time or change time is not at least one tick older
///  than the index's own modification time.
/// Why:
///  A write in the file's own tick leaves both recorded times equal, so only the
///  digest of its bytes can refuse it.
///  An unreadable tick makes every entry racy rather than none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const racy = (stored, indexMtime, tick) => Math.max(stored.mtime, stored.ctime) + tick > indexMtime;
/// ```
fn is_racy(stored: &StoredStat, index_mtime: (i64, u32), tick: Option<u64>) -> bool {
    let Some(known) = tick else { return true };
    let index_nanos: i128 = i128::from(index_mtime.0) * 1_000_000_000 + i128::from(index_mtime.1);
    let newest: i128 = [(stored.mtime_seconds, stored.mtime_nanos), (stored.ctime_seconds, stored.ctime_nanos)]
        .into_iter()
        .map(|(seconds, nanos)| return i128::from(seconds) * 1_000_000_000 + i128::from(nanos))
        .max()
        .unwrap_or(0);
    return newest + i128::from(known) > index_nanos;
}

/// What:
///  The SHA-256 of one regular file's bytes,
///  read in one linear pass with a fixed buffer.
/// Why:
///  A tracked file may be any size, and the capture streams it rather than
///  holding it.
///
/// # Errors
/// The file cannot be opened without a link,
/// or reading it was refused.
fn digest_the_file(parent: &std::fs::File, name: &OsStr, path: &Path) -> Result<[u8; 32], CaptureRefusal> {
    let file: std::fs::File = open_file_at(parent, name).map_err(|error| return format!("{} cannot be read for its digest ({error})", path.display()))?;
    let mut hasher: sha2::Sha256 = sha2::Sha256::new();
    let mut buffer: [u8; 64 * 1024] = [0; 64 * 1024];
    let mut reader: std::io::BufReader<std::fs::File> = std::io::BufReader::new(file);
    loop {
        let read: usize = reader.read(&mut buffer).map_err(|error| return format!("{} cannot be read for its digest ({error})", path.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    return Ok(hasher.finalize().into());
}

/// What:
///  One entry's facts as the words the record writes after `entry=`.
/// Why:
///  The record is the durable half of the capture, so the writer and the reader
///  share one spelling.
pub fn entry_words(entry: &CapturedEntry) -> String {
    let target: String = match entry.link_target.as_deref() {
        Some(bytes) => bytes_as_hexadecimal(bytes),
        None => String::from("-"),
    };
    return format!(
        "{} {} {} {}.{} {}.{} {} {target}",
        bytes_as_hexadecimal(entry.path.as_slice()),
        entry.kind.word(),
        entry.size,
        entry.mtime.0,
        entry.mtime.1,
        entry.ctime.0,
        entry.ctime.1,
        u8::from(entry.executable)
    );
}

/// What:
///  One digest's facts as the words the record writes after `digest=`.
/// Why:
///  The record is the durable half of the capture, so the writer and the reader
///  share one spelling.
pub fn digest_words(digest: &CapturedDigest) -> String {
    return format!("{} {}", bytes_as_hexadecimal(digest.path.as_slice()), bytes_as_hexadecimal(digest.digest.as_slice()));
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "add_capture_tests.rs"]
mod tests;
