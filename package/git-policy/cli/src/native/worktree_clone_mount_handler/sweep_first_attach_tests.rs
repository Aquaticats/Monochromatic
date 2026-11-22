//! What:
//!  Controls of the release recorder's two calls:
//!  the claim of this boot's name and the outcome written over it.
//! Why:
//!  The sweep's own order is proved beside it,
//!  and what this control adds is that the release recorder really writes the
//!  fingerprint of the running machine and the outcome a reader parses,
//!  which a fixture directory can answer without `/run`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const claim = LiveFirstAttach.at(place).claim();
//! recorder.record(claim, outcome);
//! ```

use super::{FirstAttachRecorder, LiveFirstAttach};
use crate::start_probe::first_attach_record::{ATTACHED, FirstAttachOutcome, FirstAttachPlace};
use crate::start_probe::probe_fingerprint::{BOOT_ID_PATH, ProbeFingerprint};
use std::path::PathBuf;

/// The time the outcome is written at is the recorder's own,
/// so this control reads no clock of its own.
fn outcome() -> FirstAttachOutcome {
    return FirstAttachOutcome::Attached {
        mount_id: 41,
        flags: String::from("rw,nosuid,nodev"),
        slot: String::from("one"),
        clone: String::from("zpcachyos/…/clones/one"),
        guid: String::from("7000000000000000003"),
    };
}

/// This boot's name is claimed once,
/// the running boot's identity is in the record,
/// and the outcome replaces the started one.
#[test]
fn the_release_recorder_claims_writes_the_boot_and_publishes_one_outcome() {
    let directory: PathBuf = crate::test_support::fixture("sweep-first-attach");
    let place: FirstAttachPlace = FirstAttachPlace::at(directory.clone());
    let recorder: LiveFirstAttach = LiveFirstAttach::at(place.clone());
    let claim = recorder.claim().expect("this boot's name is free in a fresh directory");
    let started: String = std::fs::read_to_string(place.path().as_path()).expect("the started record is there");
    let boot: String = std::fs::read_to_string(BOOT_ID_PATH).expect("this boot's identity reads").trim().to_string();
    assert!(started.contains(&format!("boot {boot}")), "the started record pins this boot: {started}");
    assert!(started.contains("state started"), "{started}");
    recorder.record(&claim, &outcome());
    let finished: String = std::fs::read_to_string(place.path().as_path()).expect("the outcome is there");
    assert!(finished.contains(&format!("outcome {ATTACHED}")), "{finished}");
    assert!(finished.contains("mount 41"), "{finished}");
    assert!(finished.contains(&format!("boot {boot}")), "the outcome keeps this boot: {finished}");
    assert!(finished.contains("state finished"), "{finished}");
    assert!(ProbeFingerprint::read(&crate::start_probe::probe_fingerprint::FingerprintPaths::live()).is_ok(), "the running fingerprint reads on this machine");
}

/// A second process claims nothing,
/// and the record the first one wrote stands.
#[test]
fn a_second_recorder_claims_nothing() {
    let directory: PathBuf = crate::test_support::fixture("sweep-first-attach-taken");
    let place: FirstAttachPlace = FirstAttachPlace::at(directory);
    let first: LiveFirstAttach = LiveFirstAttach::at(place.clone());
    let second: LiveFirstAttach = LiveFirstAttach::at(place.clone());
    let _claim = first.claim().expect("the first recorder wins the name");
    let before: String = std::fs::read_to_string(place.path().as_path()).expect("the started record is there");
    assert!(second.claim().is_none(), "the second recorder claims nothing");
    let after: String = std::fs::read_to_string(place.path().as_path()).expect("the record still reads");
    assert_eq!(before, after, "a refused claim changed nothing");
}
