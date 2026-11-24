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
//!  tracked path's facts, and the commit the caller's add checks out. The
//!  digests of the unvouched and racy entries are step 11's, and this capture
//!  holds no digest for a file, so it never claims one.
//!  Every path is read below a held handle of the source root, one plain name
//!  at a time, so a directory swapped for a link while the capture runs is an
//!  error and not a detour.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const capture = captureTheSource({ sourceRoot, indexBytes, hashLength, nonce, ... });
//! ```

use crate::subvolume_sha::sha256;
use crate::worktree_clean::stored_stats;
use crate::worktree_clone_binding_record::binding_encoding::bytes_as_hexadecimal;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
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
    let mut captured: Vec<CapturedEntry> = Vec::with_capacity(entries.len());
    super::add_capture_tree::walk_paths(input.source_root, entries.iter(), |entry| return entry.path.as_slice(), |entry, path, found| {
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
        captured.push(CapturedEntry {
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
        });
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
    });
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

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "add_capture_tests.rs"]
mod tests;
