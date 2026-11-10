//! What:
//!  the bench's first run from an empty working directory still writes its feature
//! dump and prints the whole report.
//! Why:
//!  `out/` is gitignored,
//! so nothing creates it before the dump;
//! a missing directory
//! used to abort the run with a bare `No such file or directory` right after the
//! candidate sweep,
//! losing the classifier,
//! feasibility and provenance sections.
//!
//! Each test runs the real binary with its working directory set to a scratch
//! directory under the build directory,
//! since the dump path is relative to the working directory.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde::Serialize;

/// One corpus line:
/// the shape `src/corpus.rs` deserializes.
#[derive(Serialize)]
struct CorpusLine {
    /// Track path,
    ///  the dump's join key.
    path: String,
    /// Decoded track length in seconds.
    duration_secs: usize,
    /// Sample rate in frames per second.
    rate: u32,
    /// Decoded frame count.
    decoded_frames: usize,
    /// Full-track true peak (linear).
    full_peak: f64,
    /// Seconds covered by each bin.
    bin_seconds: f64,
    /// Per-second bin peaks (linear).
    bin_peaks: Vec<f64>,
}

/// One provenance metadata row:
/// the shape `src/corpus.rs` deserializes.
#[derive(Serialize)]
struct MetadataLine {
    /// Track path,
    ///  the corpus join key.
    path: String,
    /// Whether the codec is lossless.
    lossless: bool,
    /// Whether a yt-dlp provenance tag is present.
    ytdlp: bool,
}

/// Seconds in every corpus track:
///  past the widest swept threshold,
///  so each track is probed rather than fully scanned.
const TRACK_SECS: usize = 300;
/// Frames per second in every corpus track.
const RATE: u32 = 10;
/// The corpus tracks' full true peaks,
///  all quiet enough that the exact gain stays 0 dB.
const FULL_PEAKS: &[f64] = &[0.5, 0.6, 0.7];
/// The bin peak of every quiet second.
const QUIET_BIN: f64 = 0.02;
/// The bin peak of each track's one loud second.
const HOT_BIN_PEAK: f64 = 0.03;
/// The dump path `main` writes,
///  relative to the working directory.
const DUMP_PATH: &str = "out/long_features.jsonl";
/// The report line that precedes the fitted classifier.
const CLASSIFIER_MARKER: &str = ") + fitted classifier:";
/// The report heading of the feasible no-classifier model.
const FEASIBLE_MARKER: &str = "feasible no-classifier model";
/// The report line that precedes the provenance-dependent margin.
const PROVENANCE_MARKER: &str = "provenance-dependent margin";

/// One corpus line for a track whose every second is `QUIET_BIN` except `hot_sec`.
fn corpus_line(path: &str, full_peak: f64, hot_sec: usize) -> String {
    let bins: Vec<f64> = (0..TRACK_SECS)
        .map(|second| {
            if second == hot_sec {
                return HOT_BIN_PEAK
            } else {
                return QUIET_BIN
            }
        })
        .collect();
    let line = CorpusLine {
        path: path.to_owned(),
        duration_secs: TRACK_SECS,
        rate: RATE,
        decoded_frames: TRACK_SECS * RATE as usize,
        full_peak,
        bin_seconds: 1.0,
        bin_peaks: bins,
    };
    return serde_json::to_string(&line).expect("serialize the corpus line")
}

/// Write the corpus and the provenance metadata,
///  returning both paths.
///
/// What:
///  one measurement file with a track per entry of `FULL_PEAKS`,
///  each carrying a loud second at a different place,
///  and a metadata file marking every track lossless.
/// Why:
///  the dump only runs when an in-budget candidate exists (long quiet tracks),
///  and the provenance section only prints when a metadata pass is given.
fn write_fixtures(directory: &Path) -> (PathBuf, PathBuf) {
    let corpus: Vec<String> = FULL_PEAKS
        .iter()
        .enumerate()
        .map(|(index, &full_peak)| {
            let path = format!("/music/track{index}.flac");
            return corpus_line(&path, full_peak, index + 1)
        })
        .collect();
    let corpus_path = directory.join("tracks.jsonl");
    fs::write(&corpus_path, corpus.join("\n") + "\n").expect("write the corpus fixture");
    let metadata: Vec<String> = (0..FULL_PEAKS.len())
        .map(|index| {
            let row = MetadataLine {
                path: format!("/music/track{index}.flac"),
                lossless: true,
                ytdlp: false,
            };
            return serde_json::to_string(&row).expect("serialize the metadata row")
        })
        .collect();
    let metadata_path = directory.join("metadata.jsonl");
    fs::write(&metadata_path, metadata.join("\n") + "\n").expect("write the metadata fixture");
    return (corpus_path, metadata_path)
}

/// A fresh working directory for one test,
///  named after it and emptied of a previous run's files.
fn scratch(name: &str) -> PathBuf {
    let directory = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    fs::remove_dir_all(&directory).ok();
    fs::create_dir_all(&directory).expect("make the scratch directory");
    return directory
}

/// Run the bench binary in `working_directory` with `arguments`.
fn run_bench(working_directory: &Path, arguments: &[&str]) -> Output {
    return Command::new(env!("CARGO_BIN_EXE_truepeak-core-bench"))
        .current_dir(working_directory)
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
        "the run failed: {}stderr: {}",
        stdout,
        String::from_utf8_lossy(&output.stderr),
    );
    return stdout
}

/// The full report prints and the dump lands under `out/`,
///  created on demand in an empty working directory.
#[test]
fn a_first_run_in_an_empty_directory_writes_the_dump_and_prints_every_section() {
    let directory = scratch("first_run_without_out");
    let work = directory.join("empty-working-directory");
    fs::create_dir_all(&work).expect("make the empty working directory");
    let (corpus_path, metadata_path) = write_fixtures(&directory);

    let output = run_bench(&work, &[
        corpus_path.to_str().expect("corpus path is UTF-8"),
        metadata_path.to_str().expect("metadata path is UTF-8"),
    ]);
    let stdout = stdout_of(&output);
    for marker in [CLASSIFIER_MARKER, FEASIBLE_MARKER, PROVENANCE_MARKER] {
        assert!(stdout.contains(marker), "the report lost {marker:?}:\n{stdout}");
    }

    let dump = work.join(DUMP_PATH);
    let rows = fs::read_to_string(&dump).expect("the dump exists under out/");
    let row_count = rows.lines().filter(|line| return !line.is_empty()).count();
    assert_eq!(row_count, FULL_PEAKS.len(), "one dump row per long track: {rows}");
}

/// A dump directory that cannot be created names the file it was trying to write.
#[test]
fn an_unwritable_dump_directory_names_the_file_it_could_not_write() {
    let directory = scratch("dump_directory_blocked");
    let work = directory.join("working-directory");
    fs::create_dir_all(&work).expect("make the working directory");
    let (corpus_path, _metadata_path) = write_fixtures(&directory);
    // A regular file where the dump's directory belongs: `create_dir_all` cannot win.
    fs::write(work.join("out"), b"not a directory").expect("block the dump directory");

    let output = run_bench(&work, &[corpus_path.to_str().expect("corpus path is UTF-8")]);
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    assert!(!output.status.success(), "the blocked dump must fail the run");
    assert!(stderr.contains(DUMP_PATH), "the error must name {DUMP_PATH:?}: {stderr}");
}
