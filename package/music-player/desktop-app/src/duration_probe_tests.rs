// What:     Unit tests for `duration_probe.rs`, pulled in by `#[cfg(test)] #[path = "duration_probe_tests.rs"] mod tests;`.
// Why:      The header length must match the decoder's length for the same file (the positive control), and a
//           missing, non-audio, or zero-rate input must give `None` rather than a length.

// What:     `use super::*;` imports the probe and the frame conversion into the test scope.
// Why:      The tests call `probe_duration_secs` and `frames_to_secs` directly.
use super::*;
use std::path::PathBuf;

use crate::decode::open;

// What:     `fn fixture(name: &str) -> PathBuf` names a committed fixture file under the package.
// Why:      The fixture tones are the real audio the decode tests already use.
fn fixture(name: &str) -> PathBuf {
    return PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixture").join(name);
}

#[test]
fn probe_matches_the_decoder_length_for_wav() {
    let path = fixture("tone.wav");
    let probed = probe_duration_secs(&path).expect("the WAV header carries a frame count");
    let decoded = open(&path).expect("the WAV fixture decodes").spec().duration_secs;
    assert!((probed - decoded).abs() < 0.01, "probed {probed}, decoded {decoded}");
}

#[test]
fn probe_matches_the_decoder_length_for_flac() {
    let path = fixture("tone.flac");
    let probed = probe_duration_secs(&path).expect("the FLAC header carries a frame count");
    let decoded = open(&path).expect("the FLAC fixture decodes").spec().duration_secs;
    assert!((probed - decoded).abs() < 0.01, "probed {probed}, decoded {decoded}");
}

#[test]
fn probe_gives_none_for_a_file_that_is_not_audio() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    assert_eq!(probe_duration_secs(&path), None);
}

#[test]
fn probe_gives_none_for_a_missing_file() {
    let path = fixture("does-not-exist.flac");
    assert_eq!(probe_duration_secs(&path), None);
}

#[test]
fn frames_convert_to_seconds_at_the_sample_rate() {
    assert_eq!(frames_to_secs(96_000, 48_000), Some(2.0));
}

#[test]
fn zero_sample_rate_gives_no_length() {
    assert_eq!(frames_to_secs(96_000, 0), None);
}
