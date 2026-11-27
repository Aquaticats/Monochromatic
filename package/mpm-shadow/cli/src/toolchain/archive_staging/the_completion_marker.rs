//! What:
//! The completion marker an archive install writes last,
//! the facts it records,
//! and the question whether a lock vouches for the copy it sits in.
//! Why:
//! The marker is what tells a finished copy from a truncated unpacking
//! (`archive_staging` says why that matters),
//! and the fact that lets a later install publish that copy instead of downloading the archive again.
//! A marker this build writes records the tool,
//! the version and the archive's SHA-256;
//! a marker an earlier build wrote records the first two and no digest,
//! so no lock can vouch for it and its copy is never published.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // the-completion-marker.ts
//! type CopyRecord = { tool: string; version: string; archiveSha256: string };
//! type CompleteMarker = { kind: 'vouched'; ... } | { kind: 'unvouched'; ... } | { kind: 'unreadable' };
//! function hasCompleteMarker(dir: string): boolean;
//! function writeCompleteMarker(dir: string, toolId: string, version: string, archiveSha256: string): void;
//! function readCompleteMarker(dir: string): CompleteMarker;
//! function hasVouchedMarker(dir: string): boolean;
//! function markerVouches(dir: string, expected: CopyRecord): boolean;
//! ```

/// Imports `fs` from `std`.
use std::fs;
/// Imports what this module uses from `std::path`.
use std::path::{
    Path,
    PathBuf,
};

/// Imports `ToolchainError` from the toolchain error module.
use super::super::error::ToolchainError;

/// The file whose presence shows an install finished;
/// it is the last file written before the version directory takes its name.
pub const COMPLETE_MARKER: &str = ".mpm-shadow-installed";

/// Whether `dir` holds the completion marker as a regular file
/// (a link of that name does not count).
pub fn has_complete_marker(dir: &Path) -> bool {
    return match fs::symlink_metadata(dir.join(COMPLETE_MARKER)) {
        Ok(metadata) => metadata.is_file(),
        Err(_) => false,
    };
}

/// What:
/// What the lock verified when a copy was unpacked:
/// the tool,
/// its version,
/// and the SHA-256 of the archive the copy came from.
/// Why:
/// A later install that publishes the copy instead of downloading the archive again
/// compares these facts with the lock's own row,
/// so a copy the lock does not vouch for is never published.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CopyRecord = { tool: string; version: string; archiveSha256: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CopyRecord {
    /// The tool id the lock names.
    pub(crate) tool: String,
    /// The version the lock pins.
    pub(crate) version: String,
    /// The SHA-256 the lock records for this host's archive.
    pub(crate) archive_sha256: String,
}

/// What:
/// What a completion marker says about the copy it sits in.
/// Why:
/// The marker written by this build records the archive's SHA-256 as well as the tool and version,
/// while a marker an earlier build wrote records only the tool and version,
/// and those two answers mean different things to a publish.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CompleteMarker = { kind: 'vouched'; tool: string; version: string; archiveSha256: string }
///   | { kind: 'unvouched'; tool: string; version: string }
///   | { kind: 'unreadable' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CompleteMarker {
    /// This build wrote it,
    /// and it records the archive's SHA-256.
    Vouched {
        /// The tool the copy was unpacked for.
        tool: String,
        /// The version the copy was unpacked for.
        version: String,
        /// The SHA-256 the download matched before the copy was unpacked.
        archive_sha256: String,
    },
    /// An earlier build wrote it,
    /// naming only the tool and version.
    Unvouched {
        /// The tool the copy was unpacked for.
        tool: String,
        /// The version the copy was unpacked for.
        version: String,
    },
    /// The marker is absent,
    /// is not one regular file,
    /// or does not hold the text this scheme writes.
    Unreadable,
}

/// What:
/// Whether `text` is a lowercase SHA-256 digest in the form the lock writes.
/// Why:
/// The digest field of a marker is compared as text,
/// so a field that is not a digest is not one this build wrote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isSha256(text: string): boolean;
/// ```
fn is_sha256(text: &str) -> bool {
    return text.len() == 64 && text.bytes().all(|byte| return byte.is_ascii_hexdigit());
}

/// Writes the completion marker into `dir`;
/// call it after everything else in the install has been checked.
/// The archive's SHA-256 is recorded so a later install can compare the copy with the lock's row.
pub fn write_complete_marker(
    dir: &Path,
    tool_id: &str,
    version: &str,
    archive_sha256: &str,
) -> Result<(), ToolchainError> {
    let marker: PathBuf = dir.join(COMPLETE_MARKER);
    if let Err(error) = fs::write(
        &marker,
        format!("{tool_id} {version} sha256:{archive_sha256}\n"),
    ) {
        return Err(ToolchainError::failure(format!(
            "cannot write {}: {error}",
            marker.display()
        )));
    }
    return Ok(());
}

/// What:
/// Reads the completion marker in `dir` into the three answers a publish asks of it.
/// Why:
/// A marker this build wrote carries the archive's SHA-256 and can be compared with the lock;
/// a marker an earlier build wrote cannot,
/// and an absent or malformed marker says nothing at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readCompleteMarker(dir: string): CompleteMarker;
/// ```
pub(crate) fn read_complete_marker(dir: &Path) -> CompleteMarker {
    if !has_complete_marker(dir) {
        return CompleteMarker::Unreadable;
    }
    let text: String = match fs::read_to_string(dir.join(COMPLETE_MARKER)) {
        Ok(text) => text,
        Err(_) => return CompleteMarker::Unreadable,
    };
    let parts: Vec<&str> = text.split_whitespace().collect();
    return match parts.as_slice() {
        [tool, version, digest] => match digest.strip_prefix("sha256:") {
            Some(hash) if is_sha256(hash) => CompleteMarker::Vouched {
                tool: (*tool).to_string(),
                version: (*version).to_string(),
                archive_sha256: hash.to_string(),
            },
            _ => CompleteMarker::Unreadable,
        },
        [tool, version] => CompleteMarker::Unvouched {
            tool: (*tool).to_string(),
            version: (*version).to_string(),
        },
        _ => CompleteMarker::Unreadable,
    };
}

/// What:
/// Whether the marker in `dir` is one this build wrote,
/// with a digest field.
/// Why:
/// Only such a copy can be published after a kill,
/// and prune keeps one for a version a known lock names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function hasVouchedMarker(dir: string): boolean;
/// ```
pub(crate) fn has_vouched_marker(dir: &Path) -> bool {
    return matches!(read_complete_marker(dir), CompleteMarker::Vouched { .. });
}

/// What:
/// Whether the marker in `dir` records exactly what the lock's row records.
/// Why:
/// The repair publishes a hidden copy without downloading the archive again,
/// so the marker and the lock must name the same tool,
/// version and archive digest
/// before those bytes become the version.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function markerVouches(dir: string, expected: CopyRecord): boolean;
/// ```
pub(crate) fn marker_vouches(
    dir: &Path,
    expected: &CopyRecord,
) -> bool {
    return match read_complete_marker(dir) {
        CompleteMarker::Vouched {
            tool,
            version,
            archive_sha256,
        } => {
            tool == expected.tool
                && version == expected.version
                && archive_sha256 == expected.archive_sha256
        }
        CompleteMarker::Unvouched {
            tool: _,
            version: _,
        }
        | CompleteMarker::Unreadable => false,
    };
}
