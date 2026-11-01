//! Probe inputs from the file itself:
//!  provenance and bones without platform tag readers.
//!
//! The bucket policy needs two zero-cost inputs per track:
//!  its provenance (which bucket)
//! and,
//!  for lossless files,
//!  the frame-size bones seeds.
//!  Sniffing a FLAC container and
//! walking its framing is file-format knowledge,
//!  not platform knowledge,
//!  so it lives
//! here;
//!  platforms hand over a path or the raw bytes and get the resolver's inputs back.
//! Degradation contract:
//!  any read or parse failure yields the uninformed provenance
//! (the bare bucket,
//!  the deepest coverage) with no bones,
//!  so a failure can only make the
//! probe more thorough,
//!  never less safe.
//!  Store and youtube provenance need real tag
//! readers,
//!  which stay platform-side;
//!  platforms that have them set the flags themselves.

/// Imports the bones profile walk and the hot-slot selection.
use crate::bones::{bones_hot_bins, flac_bones_profile};
/// Imports the provenance signals the resolver consumes.
use crate::bucketpolicy::TrackProvenance;
/// Imports the policy for the bones seed count.
use crate::policy::Policy;
/// Imports the file reader for the path convenience form.
use std::fs;
/// Imports the borrowed path type.
use std::path::Path;

/// Whether `bytes` start a FLAC container (directly or behind a nonstandard ID3v2 tag).
///
/// What:
///  checks the fLaC magic,
///  skipping a leading ID3v2 tag's syncsafe length.
///  Why:
/// lossless detection must not depend on file names or platform tag readers.
fn sniff_flac(bytes: &[u8]) -> bool {
    if bytes.len() >= 4 && &bytes[0..4] == b"fLaC" {
        return true;
    }
    if bytes.len() >= 14 && &bytes[0..3] == b"ID3" {
        let tag_size = (usize::from(bytes[6] & 0x7f) << 21)
            | (usize::from(bytes[7] & 0x7f) << 14)
            | (usize::from(bytes[8] & 0x7f) << 7)
            | usize::from(bytes[9] & 0x7f);
        let footer = if bytes[5] & 0x10 != 0 { 10 } else { 0 };
        let offset = 10 + tag_size + footer;
        return bytes.len() >= offset + 4 && &bytes[offset..offset + 4] == b"fLaC";
    }
    return false
}

/// The resolver's inputs from a file's raw bytes.
///
/// What:
///  FLAC bytes yield lossless provenance plus bones hot bins (bones drop to None
/// when the walk fails,
///  keeping the lossless bucket);
///  anything else yields the
/// uninformed provenance.
///  Why:
///  one call turns bytes into everything
/// `resolve_decision_for` needs beyond the decoded source.
pub fn probe_inputs_from_bytes(bytes: &[u8], policy: &Policy) -> (TrackProvenance, Option<Vec<usize>>) {
    if !sniff_flac(bytes) {
        // Not FLAC: the uninformed provenance. Debug, not warn, since lossy files are normal.
        tracing::debug!(bytes = bytes.len(), "not a flac container; using the bare bucket");
        return (TrackProvenance::unknown(), None);
    }
    let provenance = TrackProvenance { lossless: true, ..TrackProvenance::unknown() };
    // A failed walk keeps the lossless bucket without bones (its plain, deeper dial); the
    // cause is logged here rather than swallowed by the old `.ok()`.
    let bones = match flac_bones_profile(bytes) {
        Ok(profile) => Some(bones_hot_bins(&profile, policy.bones_top_slots)),
        Err(error) => {
            // A FLAC file we could not walk is worth a warning: a corrupt file or a walker gap.
            tracing::warn!(cause = %error.message, "flac bones walk failed; using the plain lossless dial");
            None
        }
    };
    return (provenance, bones)
}

/// The resolver's inputs from a file path.
///
/// What:
///  reads the file and delegates to `probe_inputs_from_bytes`;
///  an unreadable file
/// yields the uninformed provenance.
///  Why:
///  the one-line form platforms with paths call.
///
/// @example desktop:
///  `let (provenance, bones) = probe_inputs_from_file(path, &policy);`
pub fn probe_inputs_from_file(path: &Path, policy: &Policy) -> (TrackProvenance, Option<Vec<usize>>) {
    // The degradation contract: any read failure lands in the bare bucket, cause logged.
    match fs::read(path) {
        Ok(bytes) => return probe_inputs_from_bytes(&bytes, policy),
        Err(error) => {
            // Unreadable file: record why before degrading to the uninformed provenance.
            tracing::warn!(
                path = %path.display(),
                cause = %error,
                "could not read file for probe inputs; using the bare bucket"
            );
            return (TrackProvenance::unknown(), None)
        }
    }
}

/// What:
///      `#[cfg(test)] #[path = "inputs_tests.rs"] mod tests;`.
///  Test-only submodule in
///           the sibling file,
///  gated to test builds.
/// Why:
///       Keep this file to production code;
///  sibling `*_tests.rs` is max-lines exempt.
#[cfg(test)]
#[path = "inputs_tests.rs"]
mod tests;
