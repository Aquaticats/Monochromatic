//! Unit tests for the FLAC byte-profile extractor:
//!  the target selection,
//!  the slot rounding,
//!  and the row shape the bucket report parses.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use super::*;
use crate::test_fixtures::track;

/// A fresh scratch directory named after the test,
///  emptied of a previous run.
///
/// What:
///  a directory under this package's build directory,
///  since unit tests get no `CARGO_TARGET_TMPDIR`.
/// Why:
///  scratch never goes on a system temporary disk,
/// and the build directory is the package's own throwaway space.
fn scratch(name: &str) -> PathBuf {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("test-scratch")
        .join(name);
    fs::remove_dir_all(&directory).ok();
    fs::create_dir_all(&directory).expect("make the scratch directory");
    return directory
}

/// The linear amplitude a level `db` decibels above `0 dBFS` names.
fn linear(db: f64) -> f32 {
    return 10f64.powf(db / 20.0) as f32
}

/// One corpus track of `duration_secs` seconds with a full peak of `peak`,
///  carrying no bins (the extractor reads none).
fn bare_track(path: &str, duration_secs: u64, peak: f32) -> Track {
    return track(path, duration_secs, peak, Vec::new())
}

/// Rounds a fractional spread weight per slot,
///  so a reader that counts bytes can parse the row.
#[test]
fn whole_bytes_rounds_each_slot() {
    assert_eq!(whole_bytes(&[0.4, 1.5, 2.49]), vec![0, 2, 2]);
}

/// The target is the loud long FLAC,
///  and each of the three other shapes leaves the file alone.
#[test]
fn only_a_long_loud_flac_is_a_target() {
    let policy = truepeak_core::default_policy();
    let long = policy.short_scan_max_secs as u64 + 1;
    let short = policy.short_scan_max_secs as u64;
    let loud = linear(policy.ceiling_dbtp + 1.0);
    let quiet = linear(policy.ceiling_dbtp - 1.0);

    assert!(is_target(&bare_track("/music/a.flac", long, loud), &policy), "the loud long FLAC");
    assert!(
        !is_target(&bare_track("/music/a.flac", short, loud), &policy),
        "a short track is scanned exactly"
    );
    assert!(!is_target(&bare_track("/music/a.flac", long, quiet), &policy), "a quiet track never clamps");
    assert!(!is_target(&bare_track("/music/a.opus", long, loud), &policy), "a lossy codec has no framing walk");
}

/// The row names the slot width the walk spreads bytes over,
///  and its byte counts are whole numbers.
#[test]
fn the_row_names_the_walk_slot_width() {
    let line = ProfileLine {
        path: "/music/a.flac".to_owned(),
        slot_secs: BONES_SLOT_SECS,
        bytes: whole_bytes(&[0.4, 1.5]),
    };
    let text = serde_json::to_string(&line).expect("a row serializes");
    let value: Value = serde_json::from_str(&text).expect("a row is JSON");

    assert_eq!(value["path"].as_str(), Some("/music/a.flac"));
    assert_eq!(value["slotSecs"].as_f64(), Some(BONES_SLOT_SECS), "the join states the walk's width");
    let counts: Vec<u64> = value["bytes"]
        .as_array()
        .expect("the row carries a byte array")
        .iter()
        .map(|count| return count.as_u64().expect("a whole byte count"))
        .collect();
    assert_eq!(counts, vec![0, 2], "the bytes are whole numbers");
}

/// The output path is the argument the flag introduces,
///  and a flag with no value,
///  or another flag for a value,
///  states the usage.
#[test]
fn the_output_path_follows_the_flag() {
    let args = |values: &[&str]| return values.iter().map(|value| return (*value).to_owned()).collect::<Vec<String>>();

    assert_eq!(
        output_path(&args(&["corpus.jsonl", BONES_FLAG, "out.jsonl"])).expect("the path follows the flag"),
        "out.jsonl",
    );
    for missing in [
        args(&["corpus.jsonl"]),
        args(&["corpus.jsonl", BONES_FLAG]),
        args(&["corpus.jsonl", BONES_FLAG, "--proportional"]),
    ] {
        let error = output_path(&missing).expect_err("an incomplete call is a usage error");
        assert!(error.to_string().contains(BONES_FLAG), "the error names the flag: {error}");
    }
}

/// An output path that cannot be created names the file it could not write.
#[test]
fn an_unwritable_output_path_names_the_file() {
    let directory = scratch("bones_profiles_unwritable");
    let out_path = directory.join("absent-directory").join("profiles.jsonl");

    let error = write_profiles(&[], &truepeak_core::default_policy(), &out_path)
        .expect_err("a missing directory cannot hold the output");
    assert!(
        error.to_string().contains(out_path.to_str().expect("output path is UTF-8")),
        "the error names the file: {error}",
    );
}

/// A target that cannot be read at all names the file and stops the run,
///  unlike a target that merely left the disk.
#[test]
fn an_unreadable_target_names_the_file() {
    let directory = scratch("bones_profiles_unreadable");
    let out_path = directory.join("profiles.jsonl");
    let blocked = directory.join("blocked.flac");
    fs::create_dir(&blocked).expect("make a directory where the track belongs");
    let policy = truepeak_core::default_policy();
    let loud = linear(policy.ceiling_dbtp + 1.0);
    let target = bare_track(
        &blocked.to_string_lossy(),
        policy.short_scan_max_secs as u64 + 1,
        loud,
    );

    let error = write_profiles(std::slice::from_ref(&target), &policy, &out_path)
        .expect_err("a directory cannot be read as a track");
    assert!(
        error.to_string().contains(blocked.to_str().expect("blocked path is UTF-8")),
        "the error names the track: {error}",
    );
    assert!(Path::exists(&out_path), "the output is created before the first read");
}
