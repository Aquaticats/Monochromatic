//! The FLAC byte-profile extractor:
//!  the one writer of the bones side file the bucket report joins.
//!
//! What:
//!  walks every loud long FLAC of a corpus through
//!  `truepeak_core::flac_bones_profile` and writes one
//! `{"path", "slotSecs", "bytes"}` row per track.
//! Why:
//!  `--buckets` seeds the lossless probe from that file,
//!  so the bytes have to come from the same framing walk playback seeds its probe
//!  with;
//!  a second walk in another language could drift from this one with nothing
//!  failing.

/// Imports the directory entry point for reading a track's bytes.
use std::fs;
/// Imports the output file handle.
use std::fs::File;
/// Imports the buffered writer,
///  the buffering trait's `flush`,
///  and the error kind that marks a vanished track.
use std::io::{BufWriter, ErrorKind, Write};
/// Imports the borrowed output path type.
use std::path::Path;
/// Imports scoped threads,
///  so the readers borrow the corpus instead of cloning it.
use std::thread;

/// Imports `anyhow`'s result alias,
///  its `Context` trait,
///  and the macro that carries a usage string.
use anyhow::{Context, Result, anyhow};
/// Imports serde's derive so a row writes itself as one JSON line.
use serde::Serialize;
/// Imports the shared frame walk,
///  the slot width it spreads bytes over,
///  and the policy type the target selection reads.
use truepeak_core::{BONES_SLOT_SECS, Policy, flac_bones_profile};

/// Imports the corpus track record.
use crate::corpus::Track;
/// Imports the bench's one dBTP conversion for the loudness cutoff.
use crate::levels::peak_dbtp_or_silence;

//region The mode's flag and usage
/// The flag that selects the extraction:
///  `--bones <out.jsonl>`.
pub const BONES_FLAG: &str = "--bones";

/// How to call the extraction,
///  carried by every argument error it can raise.
const USAGE: &str = "usage: truepeak-core-bench <tracks.jsonl> --bones <out.jsonl>";

/// How many targets are read and walked at once.
///
/// What:
///  the reading pool's width,
///  the same four the retired script used.
/// Why:
///  a regeneration spends its wall clock waiting on file reads:
///  over the recorded corpus (1173 tracks,
///  48.6 GB) one reader took 572 s and four took 275 s,
///  though the second pass also found more of the library in the page cache,
///  and on eight cached tracks the four-reader run finishes four to six times
///  sooner.
///  Four is the width the retired script used,
///  so the replacement costs no more than the profiler it replaced.
const PROFILE_WORKERS: usize = 4;
//endregion

/// One written profile row,
///  the shape `--buckets` parses.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProfileLine {
    /// Track path,
    ///  the corpus join key.
    path: String,
    /// Seconds covered by each slot,
    ///  the walk's own width.
    slot_secs: f64,
    /// Compressed bytes per slot,
    ///  whole bytes so a reader can count them.
    bytes: Vec<u64>,
}

/// How many targets a run wrote and how many it could not.
#[derive(Debug, Default)]
struct Counts {
    /// Targets that produced a row.
    written: usize,
    /// Targets whose file is no longer on disk;
    ///  library churn is expected and never fails a run.
    missing: usize,
    /// Targets whose bytes are not a walkable FLAC stream.
    unparseable: usize,
}

/// What one target produced,
///  so the writer can count the outcomes that are not errors.
enum Walked {
    /// The walk succeeded.
    Line {
        /// The row this track's profile becomes.
        row: ProfileLine,
    },
    /// The file is no longer on disk.
    Missing,
    /// The bytes are not a walkable FLAC stream.
    Unparseable,
    /// The file exists but cannot be read at all;
    ///  carried out of the worker so the run names it and stops.
    Unreadable {
        /// The track the run failed on.
        path: String,
        /// What the read reported.
        message: String,
    },
}

/// Read one track and walk its framing.
fn walk_track(track: &Track) -> Walked {
    let buf = match fs::read(&track.path) {
        Ok(buf) => buf,
        Err(error) if error.kind() == ErrorKind::NotFound => return Walked::Missing,
        Err(error) => {
            return Walked::Unreadable { path: track.path.clone(), message: error.to_string() }
        }
    };
    let profile = match flac_bones_profile(&buf) {
        Ok(profile) => profile,
        Err(error) => {
            tracing::warn!(path = %track.path, cause = %error.message, "bones profile failed");
            return Walked::Unparseable
        }
    };
    return Walked::Line {
        row: ProfileLine {
            path: track.path.clone(),
            slot_secs: BONES_SLOT_SECS,
            bytes: whole_bytes(&profile),
        },
    }
}

/// Walk every target,
///  `PROFILE_WORKERS` of them at a time,
///  returning the outcomes in corpus order.
///
/// What:
///  splits the targets into contiguous chunks,
///  gives each chunk a thread,
///  and joins the chunks in order.
/// Why:
///  the reads dominate and one reader leaves the disk queue empty;
///  joining in chunk order keeps the file byte-identical between runs.
fn walk_targets(targets: &[&Track]) -> Vec<Walked> {
    let chunk_size = targets.len().div_ceil(PROFILE_WORKERS).max(1);
    return thread::scope(|scope| {
        let handles: Vec<_> = targets
            .chunks(chunk_size)
            .map(|chunk| {
                return scope.spawn(move || {
                    return chunk.iter().map(|track| return walk_track(track)).collect::<Vec<Walked>>()
                })
            })
            .collect();
        let mut walked: Vec<Walked> = Vec::with_capacity(targets.len());
        for handle in handles {
            walked.extend(handle.join().expect("a profile worker does not panic"));
        }
        return walked
    })
}

/// The path written after `--bones`.
///
/// What:
///  the argument that follows the flag,
///  or a usage error when the flag has no value.
/// Why:
///  the extraction has one output and one place to name it,
///  so a missing value must say how to call the mode.
fn output_path(args: &[String]) -> Result<&str> {
    let Some(position) = args.iter().position(|arg| return arg == BONES_FLAG) else {
        return Err(anyhow!(USAGE))
    };
    let Some(value) = args.get(position + 1) else {
        return Err(anyhow!(USAGE))
    };
    if value.starts_with('-') {
        return Err(anyhow!(USAGE))
    }
    return Ok(value.as_str())
}

/// Whether a corpus track is one the walk profiles.
///
/// What:
///  a FLAC path,
///  longer than the shipped short-scan cutoff,
///  and louder than the ceiling.
/// Why:
///  a short track is scanned exactly and a quiet one never clamps,
///  so only the loud long lossless tracks can act on a bones seed;
///  profiling the rest would spend read time on rows nothing reads.
fn is_target(track: &Track, policy: &Policy) -> bool {
    let lossless = track.path.to_lowercase().ends_with(".flac");
    let long = track.duration_secs > policy.short_scan_max_secs;
    let loud = peak_dbtp_or_silence(f64::from(track.full_peak)) > policy.ceiling_dbtp;
    return lossless && long && loud
}

/// Whole bytes per slot.
///
/// What:
///  rounds each slot's spread weight to the nearest byte.
/// Why:
///  the walk spreads a frame's bytes proportionally across the slots it covers,
///  so most weights are fractional,
///  and the side file counts bytes.
fn whole_bytes(profile: &[f64]) -> Vec<u64> {
    return profile.iter().map(|&weight| return weight.round() as u64).collect()
}

/// Write one row per target,
///  counting the files it could not profile instead of stopping.
///
/// What:
///  walks every target,
///  then appends the rows in corpus order.
/// Why:
///  a library changes under a corpus,
///  so a vanished or re-encoded file must be reported and skipped rather than
///  discard every row beside it;
///  and a fixed order keeps two runs of the same corpus byte-identical,
///  which is what lets a regenerated side file be diffed against an older one.
fn write_profiles(tracks: &[Track], policy: &Policy, out_path: &Path) -> Result<Counts> {
    // Create the output before the first read, so a bad path fails at once.
    let file = File::create(out_path)
        .with_context(|| return format!("bones: cannot create {}", out_path.display()))?;
    let mut writer = BufWriter::new(file);
    let targets: Vec<&Track> = tracks.iter().filter(|track| return is_target(track, policy)).collect();
    let mut counts = Counts::default();
    for walked in walk_targets(&targets) {
        let line = match walked {
            Walked::Line { row } => row,
            Walked::Missing => {
                counts.missing += 1;
                continue;
            }
            Walked::Unparseable => {
                counts.unparseable += 1;
                continue;
            }
            Walked::Unreadable { path, message } => {
                return Err(anyhow!("bones: cannot read {path}: {message}"))
            }
        };
        serde_json::to_writer(&mut writer, &line)
            .with_context(|| return format!("bones: cannot write {}", out_path.display()))?;
        writer
            .write_all(b"\n")
            .with_context(|| return format!("bones: cannot write {}", out_path.display()))?;
        counts.written += 1;
    }
    writer
        .flush()
        .with_context(|| return format!("bones: cannot flush {}", out_path.display()))?;
    return Ok(counts)
}

/// Extract the FLAC byte profiles the `--buckets` report joins and print the counts.
///
/// What:
///  runs `write_profiles` over the corpus and reports what it wrote,
///  what left the disk,
///  and what did not parse.
pub fn report_bones(tracks: &[Track], args: &[String]) -> Result<()> {
    let out_path = output_path(args)?;
    let policy = truepeak_core::default_policy();
    let counts = write_profiles(tracks, &policy, Path::new(out_path))?;
    println!(
        "bones profiles: {} written ({} missing on disk, {} unparseable) -> {out_path}",
        counts.written, counts.missing, counts.unparseable
    );
    return Ok(())
}

/// What:
///      `#[cfg(test)] #[path = "bones_profiles_tests.rs"] mod tests;`.
///  Test-only submodule in
///           the sibling file,
///  gated to test builds.
/// Why:
///       keep this file to production code;
///  sibling `*_tests.rs` is max-lines exempt.
#[cfg(test)]
#[path = "bones_profiles_tests.rs"]
mod tests;
