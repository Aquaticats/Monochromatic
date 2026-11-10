//! What:
//!  `--bones <out.jsonl>` walks every loud long FLAC of a corpus through the shared
//!  `truepeak_core::flac_bones_profile` walk and writes the profile rows the
//!  `--buckets` report joins.
//! Why:
//!  the side file used to come from a second implementation of the walk written in
//!  JavaScript,
//!  so the offline evidence and the runtime bones seeds could drift apart without
//!  anything failing;
//!  one walk has to produce both.
//!
//! Each test runs the real binary in its own scratch directory under the build
//! directory,
//! the way `out_directory.rs` runs it,
//! so the written file is a throwaway.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde::{Deserialize, Serialize};
use truepeak_core::BONES_SLOT_SECS;

//region The fixture FLAC stream
/// Frames per second the fixture stream declares;
///  the rate every sample-to-second conversion in the walk divides by.
const RATE: u32 = 44_100;
/// Samples per block the fixture stream's frame headers declare,
///  the walk's only source of frame length.
const BLOCK_SAMPLES: usize = 4_096;
/// Payload bytes per fixture frame,
///  smallest first,
///  so the middle frame is the byte-rate hot spot the profile must find.
const PAYLOAD_SIZES: [usize; 3] = [300, 3000, 300];
/// Bytes of the `fLaC` marker plus the STREAMINFO block header and body,
///  the offset the walk starts looking for the first frame at.
const FIRST_FRAME_OFFSET: usize = 4 + 4 + 34;
/// A frame header is the five declared bytes plus its CRC-8 byte.
const FRAME_HEADER_BYTES: usize = 6;
/// Rounding each slot's spread bytes to a whole number can move the total this far,
///  half a byte per slot the fixture spans.
const ROUNDING_SLACK_BYTES: i64 = 3;
/// Linear peak above the `-1 dBTP` ceiling,
///  so the fixture track is one the loud cutoff keeps.
const LOUD_PEAK: f64 = 0.95;
/// Linear peak below the ceiling,
///  so the fixture track is one the loud cutoff drops.
const QUIET_PEAK: f64 = 0.5;
/// Decoded seconds the corpus claims per fixture track:
///  past the short-scan cutoff,
///  so the walk's targets are the long tracks.
const TRACK_SECS: usize = 300;
/// Decoded seconds of the corpus track the short-scan cutoff drops,
///  well under it.
const SHORT_SECS: f64 = 10.0;
/// Seconds covered by each corpus bin.
const BIN_SECONDS: f64 = 0.1;
/// Bins the corpus carries per track;
///  the profile mode reads none of them,
///  but the corpus record declares them.
const BIN_COUNT: usize = 3_000;
/// The corpus bin peak,
///  quiet enough that no scored band depends on it.
const BIN_PEAK: f64 = 0.01;
//endregion

/// One corpus line,
///  the shape `src/corpus.rs` deserializes.
#[derive(Serialize)]
struct CorpusLine {
    /// Track path,
    ///  the profile row's join key.
    path: String,
    /// Decoded track length in seconds.
    duration_secs: f64,
    /// Sample rate in frames per second.
    rate: u32,
    /// Decoded frame count.
    decoded_frames: u64,
    /// Full-track true peak as a linear amplitude.
    full_peak: f64,
    /// Seconds covered by each bin.
    bin_seconds: f64,
    /// Per-bin true peaks (linear).
    bin_peaks: Vec<f64>,
}

/// One written profile row,
///  the shape `src/buckets.rs` reads.
/// `bytes` is whole-number typed on purpose:
///  a fractional slot byte count fails to parse there.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProfileRow {
    /// Track path,
    ///  the corpus join key.
    path: String,
    /// Seconds covered by each slot.
    slot_secs: f64,
    /// Compressed bytes per slot.
    bytes: Vec<u64>,
}

/// The stdout line the extraction prints its counts on.
const PROGRESS_MARKER: &str = "bones profiles:";

/// One corpus line for a track of `duration_secs` seconds whose every bin carries
///  `BIN_PEAK`.
fn corpus_line(path: &str, full_peak: f64, duration_secs: f64) -> String {
    let line = CorpusLine {
        path: path.to_owned(),
        duration_secs,
        rate: RATE,
        decoded_frames: (duration_secs * f64::from(RATE)) as u64,
        full_peak,
        bin_seconds: BIN_SECONDS,
        bin_peaks: vec![BIN_PEAK; BIN_COUNT],
    };
    return serde_json::to_string(&line).expect("serialize the corpus line")
}

/// Bitwise CRC-8 (polynomial 0x07,
///  init 0),
///  the FLAC frame-header checksum,
///  computed here so the fixture does not lean on the walk's own table.
fn crc8_bitwise(bytes: &[u8]) -> u8 {
    return bytes.iter().fold(0u8, |mut crc, &byte| {
        crc ^= byte;
        for _ in 0..8 {
            crc = if crc & 0x80 != 0 { (crc << 1) ^ 0x07 } else { crc << 1 };
        }
        return crc
    })
}

/// A fixed-blocking stereo 44.1 kHz FLAC stream,
///  one frame per entry of `PAYLOAD_SIZES`.
///
/// What:
///  `fLaC`,
///  a 34-byte STREAMINFO declaring the block size and rate,
///  then each frame's
/// header,
///  CRC-8,
///  and payload.
/// Why:
///  the walk confirms frames by the sync code and the header CRC,
///  so a fixture it accepts
///  is a stream it really walks;
///  payload bytes stay clear of `0xff` so nothing inside them looks like a frame.
fn fixture_flac() -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(b"fLaC");
    // STREAMINFO: last-block flag plus type 0, then its 34-byte body.
    buf.extend_from_slice(&[0x80, 0x00, 0x00, 0x22]);
    let mut streaminfo = [0u8; 34];
    // min/max block size, as the 16-bit pair at bytes 0..4.
    let block_bits = (BLOCK_SAMPLES as u16).to_be_bytes();
    streaminfo[0..2].copy_from_slice(&block_bits);
    streaminfo[2..4].copy_from_slice(&block_bits);
    // Sample rate packed as 20 bits at bytes 10..13.
    streaminfo[10] = ((RATE >> 12) & 0xff) as u8;
    streaminfo[11] = ((RATE >> 4) & 0xff) as u8;
    streaminfo[12] = ((RATE << 4) & 0xf0) as u8;
    buf.extend_from_slice(&streaminfo);
    for (frame_number, &payload) in PAYLOAD_SIZES.iter().enumerate() {
        // Header: 14-bit sync, fixed blocking, block size bits 12 (4096), rate bits 0
        // (STREAMINFO), stereo, 16-bit, then the frame number as a one-byte coded number.
        let header = [0xff, 0xf8, 0xc0, 0x18, frame_number as u8];
        buf.extend_from_slice(&header);
        buf.push(crc8_bitwise(&header));
        buf.extend(std::iter::repeat_n(0x00, payload));
    }
    return buf
}

/// The slot the middle frame dominates,
///  the index a bones seed must lead with.
fn hot_slot() -> usize {
    return PAYLOAD_SIZES.len() / 2
}

/// The slots the fixture's three blocks cover at `BONES_SLOT_SECS` each.
fn fixture_slot_count() -> usize {
    let secs = (PAYLOAD_SIZES.len() * BLOCK_SAMPLES) as f64 / f64::from(RATE);
    return (secs / BONES_SLOT_SECS).ceil() as usize;
}

/// Total compressed bytes after the metadata,
///  the mass a profile spreads over its slots.
fn fixture_audio_bytes() -> i64 {
    let payload: usize = PAYLOAD_SIZES.iter().sum();
    return (payload + PAYLOAD_SIZES.len() * FRAME_HEADER_BYTES) as i64
}

/// A fresh working directory named after the test,
///  emptied of a previous run.
fn scratch(name: &str) -> PathBuf {
    let directory = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    fs::remove_dir_all(&directory).ok();
    fs::create_dir_all(&directory).expect("make the scratch directory");
    return directory
}

/// Write a corpus of `lines` and return its path.
fn write_corpus(directory: &Path, lines: &[String]) -> PathBuf {
    let corpus_path = directory.join("tracks.jsonl");
    fs::write(&corpus_path, lines.join("\n") + "\n").expect("write the corpus fixture");
    return corpus_path
}

/// Run the bench binary with `arguments`.
fn run_bench(arguments: &[&str]) -> Output {
    return Command::new(env!("CARGO_BIN_EXE_truepeak-core-bench"))
        .args(arguments)
        .output()
        .expect("run the bench binary")
}

/// The stdout of one run,
///  failing with the captured stderr when the run did not exit zero.
fn stdout_of(output: &Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    assert!(
        output.status.success(),
        "the run failed: {stdout}stderr: {}",
        String::from_utf8_lossy(&output.stderr),
    );
    return stdout
}

/// The profile rows a run wrote,
///  one per non-empty line.
fn read_profiles(path: &Path) -> Vec<ProfileRow> {
    let text = fs::read_to_string(path).expect("the profile file exists");
    return text
        .lines()
        .filter(|line| return !line.is_empty())
        .map(|line| return serde_json::from_str(line).expect("a profile row parses"))
        .collect()
}

/// One row per loud long FLAC,
///  carrying the walk's slot length,
///  whole-number bytes,
///  and the hot frame's slot first.
///
/// What:
///  the target track is profiled and the short,
///  quiet,
///  and lossy tracks beside it
///  are left out of the file.
#[test]
fn one_row_per_target_and_the_hot_frame_leads() {
    let directory = scratch("bones_profiles_targets");
    let fixture_path = directory.join("fixture.flac");
    fs::write(&fixture_path, fixture_flac()).expect("write the fixture stream");
    let target = fixture_path.to_string_lossy().into_owned();
    let corpus_path = write_corpus(&directory, &[
        corpus_line(&target, LOUD_PEAK, TRACK_SECS as f64),
        corpus_line(&format!("{target}.opus"), LOUD_PEAK, TRACK_SECS as f64),
        corpus_line(&target, QUIET_PEAK, TRACK_SECS as f64),
        corpus_line(&target, LOUD_PEAK, SHORT_SECS),
    ]);
    let out_path = directory.join("flac-profiles.jsonl");

    let stdout = stdout_of(&run_bench(&[
        corpus_path.to_str().expect("corpus path is UTF-8"),
        "--bones",
        out_path.to_str().expect("output path is UTF-8"),
    ]));
    assert!(stdout.contains(PROGRESS_MARKER), "the run must report its counts:\n{stdout}");

    let rows = read_profiles(&out_path);
    assert_eq!(rows.len(), 1, "only the loud long FLAC is a target: {rows:?}");
    let row = &rows[0];
    assert_eq!(row.path, target, "the row keeps the corpus path");
    assert_eq!(row.slot_secs, BONES_SLOT_SECS, "the row names the walk's slot length");
    assert_eq!(row.bytes.len(), fixture_slot_count(), "one byte count per slot the frames cover");
    let spread: i64 = row.bytes.iter().map(|&count| return count as i64).sum();
    assert!(
        (spread - fixture_audio_bytes()).abs() <= ROUNDING_SLACK_BYTES,
        "every audio byte lands in some slot: {spread} vs {}",
        fixture_audio_bytes(),
    );
    let hottest = {
        let mut order: Vec<usize> = (0..row.bytes.len()).collect();
        order.sort_by(|&left, &right| return row.bytes[right].cmp(&row.bytes[left]));
        order[0]
    };
    assert_eq!(hottest, hot_slot(), "the largest payload's slot leads: {:?}", row.bytes);
}

/// A target that is no longer on disk leaves the run alone and is counted.
#[test]
fn a_target_missing_from_disk_is_counted_not_fatal() {
    let directory = scratch("bones_profiles_missing");
    let absent = directory.join("gone.flac").to_string_lossy().into_owned();
    let corpus_path = write_corpus(&directory, &[corpus_line(&absent, LOUD_PEAK, TRACK_SECS as f64)]);
    let out_path = directory.join("flac-profiles.jsonl");

    let stdout = stdout_of(&run_bench(&[
        corpus_path.to_str().expect("corpus path is UTF-8"),
        "--bones",
        out_path.to_str().expect("output path is UTF-8"),
    ]));
    assert!(
        stdout.contains("0 written (1 missing on disk, 0 unparseable)"),
        "the run must count the missing target:\n{stdout}",
    );
    assert!(read_profiles(&out_path).is_empty(), "a missing file writes no row");
}

/// A target whose bytes are not a FLAC stream is counted,
///  named,
///  and the run still writes the rows beside it.
#[test]
fn an_unparseable_target_is_counted_and_named_not_fatal() {
    let directory = scratch("bones_profiles_unparseable");
    let garbage_path = directory.join("garbage.flac");
    fs::write(&garbage_path, [0u8; 32]).expect("write the garbage fixture");
    let garbage = garbage_path.to_string_lossy().into_owned();
    let corpus_path = write_corpus(&directory, &[corpus_line(&garbage, LOUD_PEAK, TRACK_SECS as f64)]);
    let out_path = directory.join("flac-profiles.jsonl");

    let output = run_bench(&[
        corpus_path.to_str().expect("corpus path is UTF-8"),
        "--bones",
        out_path.to_str().expect("output path is UTF-8"),
    ]);
    let stdout = stdout_of(&output);
    assert!(
        stdout.contains("0 written (0 missing on disk, 1 unparseable)"),
        "the run must count the unparseable target:\n{stdout}",
    );
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    assert!(stderr.contains("flac:"), "the diagnostic names the parse failure: {stderr}");
    assert!(read_profiles(&out_path).is_empty(), "an unparseable file writes no row");
}

/// `--bones` with no path after it fails and says how to call it.
#[test]
fn a_bones_flag_without_its_path_names_the_usage() {
    let directory = scratch("bones_profiles_no_path");
    let corpus_path = write_corpus(&directory, &[corpus_line(
        "/music/none.flac",
        LOUD_PEAK,
        TRACK_SECS as f64,
    )]);

    let output = run_bench(&[corpus_path.to_str().expect("corpus path is UTF-8"), "--bones"]);
    assert!(!output.status.success(), "--bones with no path must fail the run");
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    assert!(stderr.contains("--bones"), "the error must name the flag: {stderr}");
    assert!(stderr.contains("usage"), "the error must state the usage: {stderr}");
}
