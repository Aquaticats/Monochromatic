//! The per-track measurement record and the JSONL corpus loader.
//!
//! Each line of the corpus file is one track measured by the shared meter:
//!  the full
//! true peak and the per-second bin peaks.
//!  The bins let any window policy be simulated
//! by re-slicing,
//!  so the parameter search never re-decodes audio.

/// Imports the safe-provenance path set.
use std::collections::HashSet;
/// Imports the line-by-line file reader pieces.
use std::fs::File;
/// Imports buffered reading so a multi-megabyte corpus streams instead of loading whole.
use std::io::{BufRead, BufReader};
/// Imports the borrowed path type for the corpus location.
use std::path::Path;

/// Imports `anyhow`'s application-level result alias.
use anyhow::Result;
/// Imports serde's derive so the record parses straight from each JSON line.
use serde::Deserialize;

/// One track's measurement:
///  identity,
///  shape,
///  the full true peak,
///  and the per-second
/// Catmull-Rom bin peaks (all linear amplitudes from the shared meter).
#[derive(Clone, Debug, Deserialize)]
pub struct Track {
    /// Absolute path,
    ///  used only for reporting exception lists,
    ///  never for classification.
    pub path: String,
    /// Decoded track length in seconds (frames divided by rate).
    pub duration_secs: f64,
    /// Sample rate in frames per second.
    pub rate: u32,
    /// Decoded interleaved frame count (one frame is one sample per channel).
    pub decoded_frames: u64,
    /// Full-track true peak as a linear amplitude.
    pub full_peak: f32,
    /// Seconds covered by each bin;
    ///  the fine collector emits a tenth,
    ///  the older corpus a
    /// whole second,
    ///  so this defaults to one for corpora that predate the field.
    #[serde(default = "one_second")]
    pub bin_seconds: f64,
    /// Per-bin true peaks:
    ///  bin `i` is the meter's peak over bin `i` of `bin_seconds`.
    pub bin_peaks: Vec<f32>,
}

/// The default bin resolution for corpora that predate the `bin_seconds` field.
fn one_second() -> f64 {
    return 1.0
}

/// Read a JSONL corpus file into one `Track` per non-empty line.
///
/// What:
///  opens the file,
///  streams lines,
///  and parses each into a `Track`.
///  Why:
///  the search
/// needs every track in memory once,
///  but the file is too large to read as one string.
pub fn load_tracks(path: &Path) -> Result<Vec<Track>> {
    // Open the corpus file; a missing file is a hard error the caller surfaces.
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    // Parse each non-empty line into a Track, collecting failures into the error channel.
    let mut tracks = Vec::new();
    for line in reader.lines() {
        let text = line?;
        if text.trim().is_empty() {
            continue;
        }
        let track: Track = serde_json::from_str(&text)?;
        tracks.push(track);
    }
    return Ok(tracks)
}

/// One provenance metadata row:
///  the path and whether it is a reliably-not-hot source.
#[derive(Deserialize)]
struct MetaRow {
    /// Track path,
    ///  the join key against the corpus.
    path: String,
    /// Whether the codec is lossless (FLAC and similar),
    ///  which is never a violator here.
    lossless: bool,
    /// Whether a yt-dlp / youtube provenance tag is present,
    ///  also never a violator.
    ytdlp: bool,
}

/// Read the metadata pass and return the set of paths whose provenance is reliably safe.
///
/// What:
///  a path is "safe" when it is lossless or carries a yt-dlp provenance tag,
///  since
/// the corpus never has a violator in either class.
///  Why:
///  the policy can apply a smaller
/// margin to safe provenance,
///  lowering the average too-quiet error.
pub fn load_safe_paths(path: &Path) -> Result<HashSet<String>> {
    // Read each metadata row and keep the safe paths.
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut safe = HashSet::new();
    for line in reader.lines() {
        let text = line?;
        if text.trim().is_empty() {
            continue;
        }
        let row: MetaRow = serde_json::from_str(&text)?;
        if row.lossless || row.ytdlp {
            safe.insert(row.path);
        }
    }
    return Ok(safe)
}
