//! What:
//!  Reads of the real filesystem under a worktree root that never follow a symlink.
//! Why:
//!  Mirror and export paths must not be redirected outside the worktree,
//! so every component of a path is inspected
//!  with `symlink_metadata` (the `lstat` call) and a link anywhere on the way is refused,
//! not followed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function readReal(root: string, rel: string): RealState; // uses lstat, never stat
//! ```

/// Imports `Diagnostic` from `crate::diagnostic`.
use crate::diagnostic::Diagnostic;
/// Imports `Digest`,
/// `Sha256` from `sha2`.
use sha2::{Digest, Sha256};
/// Imports `fs` from `std`.
use std::fs;
/// Imports `PermissionsExt` from `std::os::unix::fs`.
use std::os::unix::fs::PermissionsExt;
/// Imports `Path`,
/// `PathBuf` from `std::path`.
use std::path::{Path, PathBuf};

/// What:
///  What a path holds on disk right now.
///  `Vec<u8>` is a growable byte buffer (TS:
/// `Uint8Array`);
/// `u32` is an unsigned 32-bit integer (sibling:
/// `u64`).
/// Why:
///  Planning and drift detection compare content and mode;
/// a directory,
/// a link or anything else is a distinct answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RealState = { kind: 'absent' } | { kind: 'file'; content: Uint8Array; mode: number } | { kind: 'directory' } | { kind: 'symlink' } | { kind: 'other' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RealState {
    /// Nothing at the path,
    /// or an ancestor directory is missing.
    Absent,
    /// A regular file with its bytes and permission bits (`0o7777` mask).
    File {
        /// File content.
        content: Vec<u8>,
        /// Permission bits.
        mode: u32,
    },
    /// A directory.
    Directory,
    /// A symbolic link (never followed).
    Symlink,
    /// A socket,
    /// device or other special file.
    Other,
}

/// What:
///  The kind of one directory entry,
/// from `lstat`.
/// Why:
///  Listings drive globs and the mirror,
/// which treat links and special files as errors or drift.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type EntryKind = 'file' | 'directory' | 'symlink' | 'other';
/// ```
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum EntryKind {
    /// Regular file.
    File,
    /// Directory.
    Directory,
    /// Symbolic link.
    Symlink,
    /// Anything else.
    Other,
}

/// What:
///  Lowercase hexadecimal SHA-256 of some bytes.
/// Why:
///  Snapshots and the socket name compare digests instead of keeping whole files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sha256Hex(bytes: Uint8Array): string
/// ```
pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut text = String::new();
    for byte in digest.iter() {
        text.push_str(&format!("{byte:02x}"));
    }
    return text;
}

/// What:
///  Convert a metadata value into an `EntryKind`.
/// Why:
///  One place decides what counts as a regular file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function kindOf(metadata: Stats): EntryKind
/// ```
pub fn kind_of(metadata: &fs::Metadata) -> EntryKind {
    let file_type = metadata.file_type();
    if file_type.is_symlink() {
        return EntryKind::Symlink;
    }
    if file_type.is_dir() {
        return EntryKind::Directory;
    }
    if file_type.is_file() {
        return EntryKind::File;
    }
    return EntryKind::Other;
}

/// What:
///  Join a root and a repo-relative path.
/// Why:
///  Callers validate the relative path first;
/// this only joins.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function absolute(root: string, rel: string): string { return path.join(root, rel); }
/// ```
pub fn absolute(root: &Path, rel: &str) -> PathBuf {
    return root.join(rel);
}

/// What:
///  Check that no ancestor directory of `rel` is a link or a non-directory.
///  Returns `Ok(false)` when an ancestor is missing,
/// so the path is simply absent.
/// Why:
///  A link anywhere on the way could redirect a read or a write outside the worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ancestorsAreDirectories(root: string, rel: string): boolean // throws on a link
/// ```
pub fn ancestors_are_real_directories(root: &Path, rel: &str) -> Result<bool, Diagnostic> {
    let mut current = root.to_path_buf();
    let segments: Vec<&str> = rel.split('/').collect();
    let ancestor_count = segments.len().saturating_sub(1);
    for segment in segments.iter().take(ancestor_count) {
        current.push(segment);
        match fs::symlink_metadata(&current) {
            Ok(metadata) => match kind_of(&metadata) {
                EntryKind::Directory => {}
                EntryKind::Symlink => {
                    let message = format!("{} is a symbolic link; paths are never followed through links", current.display());
                    return Err(Diagnostic::new("symlink-rejected", &message).with_path(rel));
                }
                _ => {
                    let message = format!("{} is not a directory, so {rel} cannot exist", current.display());
                    return Err(Diagnostic::new("path-blocked", &message).with_path(rel));
                }
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
            Err(error) => {
                let message = format!("cannot inspect {}: {error}", current.display());
                return Err(Diagnostic::new("io-failure", &message).with_path(rel));
            }
        }
    }
    return Ok(true);
}

/// What:
///  Read what `rel` holds under `root` without following links.
/// Why:
///  The one read primitive under the virtual view and the publish step's rechecks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readReal(root: string, rel: string): RealState
/// ```
pub fn read_real(root: &Path, rel: &str) -> Result<RealState, Diagnostic> {
    if !ancestors_are_real_directories(root, rel)? {
        return Ok(RealState::Absent);
    }
    let full = absolute(root, rel);
    let metadata = match fs::symlink_metadata(&full) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(RealState::Absent),
        Err(error) => {
            let message = format!("cannot inspect {}: {error}", full.display());
            return Err(Diagnostic::new("io-failure", &message).with_path(rel));
        }
    };
    match kind_of(&metadata) {
        EntryKind::File => {
            let content = match fs::read(&full) {
                Ok(content) => content,
                Err(error) => {
                    let message = format!("cannot read {}: {error}", full.display());
                    return Err(Diagnostic::new("io-failure", &message).with_path(rel));
                }
            };
            return Ok(RealState::File { content, mode: metadata.permissions().mode() & 0o7777 });
        }
        EntryKind::Directory => return Ok(RealState::Directory),
        EntryKind::Symlink => return Ok(RealState::Symlink),
        EntryKind::Other => return Ok(RealState::Other),
    }
}

/// What:
///  List a real directory's entries with their kinds,
/// sorted by name.
///  `Vec<(String, EntryKind)>` is a list of pairs (TS:
/// `[string, EntryKind][]`).
/// Why:
///  Sorting makes every walk deterministic across filesystems.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function listReal(root: string, rel: string): [string, EntryKind][]
/// ```
pub fn list_real(root: &Path, rel: &str) -> Result<Vec<(String, EntryKind)>, Diagnostic> {
    let mut entries: Vec<(String, EntryKind)> = Vec::new();
    let probe = format!("{rel}/x");
    if !rel.is_empty() && !ancestors_are_real_directories(root, &probe)? {
        return Ok(entries);
    }
    let full = if rel.is_empty() { root.to_path_buf() } else { absolute(root, rel) };
    let reader = match fs::read_dir(&full) {
        Ok(reader) => reader,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(entries),
        Err(error) if error.kind() == std::io::ErrorKind::NotADirectory => return Ok(entries),
        Err(error) => {
            let message = format!("cannot list {}: {error}", full.display());
            return Err(Diagnostic::new("io-failure", &message).with_path(rel));
        }
    };
    for item in reader {
        let entry = match item {
            Ok(entry) => entry,
            Err(error) => {
                let message = format!("cannot list {}: {error}", full.display());
                return Err(Diagnostic::new("io-failure", &message).with_path(rel));
            }
        };
        let name = entry.file_name().to_string_lossy().to_string();
        let kind = match fs::symlink_metadata(entry.path()) {
            Ok(metadata) => kind_of(&metadata),
            Err(_) => EntryKind::Other,
        };
        entries.push((name, kind));
    }
    entries.sort();
    return Ok(entries);
}
