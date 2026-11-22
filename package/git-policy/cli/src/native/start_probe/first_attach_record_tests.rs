//! What:
//!  Controls of this boot's `first-attach` record:
//!  the claim,
//!  the started record,
//!  the outcome published over it,
//!  and the refusals.
//! Why:
//!  The file is a root-owned record a reader parses,
//!  and a control of its claim and its rename needs only a fixture directory,
//!  so the shapes are proved here and the handler's call to it is proved
//!  beside the sweep.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const claim = place.claim(fingerprint, time);
//! place.record(claim, outcomeText);
//! ```

use super::{
    ATTACHED, FIRST_ATTACH_KIND, FIRST_ATTACH_MODE, FIRST_ATTACH_NAME, FINISHED, FirstAttachOutcome, FirstAttachPlace, FirstAttachRefusal, REFUSED, STARTED, outcome_record, started_record,
};
use crate::start_probe::probe_fingerprint::ProbeFingerprint;
use crate::start_probe::probe_record::ABSENT;
use std::path::PathBuf;

/// The time every record this control writes carries.
const TIME: &str = "2026-11-22T01:00:00-05:00";

/// One fingerprint a control can compare whole.
fn fingerprint() -> ProbeFingerprint {
    return ProbeFingerprint {
        kernel_release: String::from("6.8.0-fixture"),
        module_version: Some(String::from("2.4.4-1")),
        module_srcversion: Some(String::from("ABCDEF")),
        module_build_id: Some(String::from("00112233")),
        boot_id: String::from("00000000-0000-4000-8000-000000000000"),
        handler: Some(String::from("/opt/handler")),
    };
}

/// One fixture directory of this control's own.
fn place(name: &str) -> FirstAttachPlace {
    let directory: PathBuf = crate::test_support::fixture(format!("first-attach-{name}").as_str());
    return FirstAttachPlace::at(directory);
}

/// The started record is one `v1` `operation-outcome` in the design's key order,
/// with the running fingerprint and no outcome.
#[test]
fn the_started_record_is_the_designs_fixed_keys() {
    let text: String = started_record(&fingerprint(), TIME);
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines[0], "v1");
    assert_eq!(lines[1], format!("kind {FIRST_ATTACH_KIND}"));
    assert_eq!(lines[2], format!("state {STARTED}"));
    assert_eq!(lines[3], "boot 00000000-0000-4000-8000-000000000000");
    assert_eq!(lines[4], "kernel 6.8.0-fixture");
    assert_eq!(lines[5], "module 2.4.4-1");
    assert_eq!(lines[6], "srcversion ABCDEF");
    assert_eq!(lines[7], "handler /opt/handler");
    assert_eq!(lines[8], format!("time {TIME}"));
    assert_eq!(lines[9], "operation attach");
    assert_eq!(lines.len(), 10, "the started record holds no outcome: {text}");
    assert!(!text.lines().any(|line| return line.starts_with("outcome ")), "a started record names no outcome: {text}");
}

/// An absent module attribute is written as `absent` rather than invented.
#[test]
fn an_absent_attribute_is_written_as_absent() {
    let bare: ProbeFingerprint = ProbeFingerprint { module_version: None, module_srcversion: None, module_build_id: None, handler: None, ..fingerprint() };
    let text: String = started_record(&bare, TIME);
    assert!(text.contains(&format!("module {ABSENT}")), "{text}");
    assert!(text.contains(&format!("srcversion {ABSENT}")), "{text}");
    assert!(text.contains(&format!("handler {ABSENT}")), "{text}");
}

/// The attached outcome carries the mount,
/// the flags read back,
/// the slot,
/// the clone and its GUID,
/// and the finished state.
#[test]
fn the_attached_outcome_carries_the_designs_keys() {
    let outcome: FirstAttachOutcome = FirstAttachOutcome::Attached {
        mount_id: 41,
        flags: String::from("rw,nosuid,nodev"),
        slot: String::from("one"),
        clone: String::from("zpcachyos/…/clones/one"),
        guid: String::from("7000000000000000003"),
    };
    let text: String = outcome_record(&fingerprint(), TIME, &outcome);
    assert!(text.starts_with("v1\n"), "{text}");
    assert!(text.contains(&format!("state {FINISHED}")), "{text}");
    assert!(text.contains("operation attach"), "{text}");
    assert!(text.contains(&format!("outcome {ATTACHED}")), "{text}");
    assert!(text.contains("mount 41"), "{text}");
    assert!(text.contains("flags rw,nosuid,nodev"), "{text}");
    assert!(text.contains("slot one"), "{text}");
    assert!(text.contains("clone zpcachyos/…/clones/one"), "{text}");
    assert!(text.contains("guid 7000000000000000003"), "{text}");
}

/// The refused outcome names the step and the errno and carries no mount.
#[test]
fn the_refused_outcome_names_the_step_and_the_errno() {
    let outcome: FirstAttachOutcome = FirstAttachOutcome::Refused {
        slot: String::from("one"),
        clone: String::from("zpcachyos/…/clones/one"),
        guid: String::from("7000000000000000003"),
        step: String::from("move-mount"),
        errno: String::from("EXDEV"),
    };
    let text: String = outcome_record(&fingerprint(), TIME, &outcome);
    assert!(text.contains(&format!("outcome {REFUSED}")), "{text}");
    assert!(text.contains("step move-mount"), "{text}");
    assert!(text.contains("errno EXDEV"), "{text}");
    assert!(!text.contains("mount "), "a refusal carries no mount: {text}");
}

/// The claim writes a started record at the design's mode,
/// and a second process's claim answers that the name is taken and changes
/// nothing.
#[test]
fn one_claim_writes_the_started_record_and_a_second_claim_is_refused() {
    use std::os::unix::fs::PermissionsExt;
    let place: FirstAttachPlace = place("claim");
    let claim = place.claim(&fingerprint(), TIME).expect("the first claim wins the name");
    let path: PathBuf = place.path();
    assert_eq!(claim.path(), path.as_path());
    assert_eq!(path.file_name().and_then(|name| return name.to_str()), Some(FIRST_ATTACH_NAME));
    let text: String = std::fs::read_to_string(path.as_path()).expect("the started record is there");
    assert_eq!(text, started_record(&fingerprint(), TIME));
    let mode: u32 = std::fs::metadata(path.as_path()).expect("the record's metadata reads").permissions().mode() & 0o7777;
    assert_eq!(mode, FIRST_ATTACH_MODE);
    let second: FirstAttachRefusal = place.claim(&fingerprint(), TIME).expect_err("the name is this boot's and it is taken");
    assert!(matches!(second, FirstAttachRefusal::AlreadyClaimed { .. }), "{second:?}");
    assert_eq!(std::fs::read_to_string(path.as_path()).expect("the record still reads"), text, "a refused claim changed nothing");
}

/// The outcome is published over the claimed name in one rename,
/// at the design's mode,
/// and leaves no staging file behind.
#[test]
fn the_outcome_replaces_the_started_record_in_one_rename() {
    use std::os::unix::fs::PermissionsExt;
    let place: FirstAttachPlace = place("record");
    let claim = place.claim(&fingerprint(), TIME).expect("the claim wins the name");
    let outcome: FirstAttachOutcome = FirstAttachOutcome::Attached {
        mount_id: 41,
        flags: String::from("rw,nosuid,nodev"),
        slot: String::from("one"),
        clone: String::from("zpcachyos/…/clones/one"),
        guid: String::from("7000000000000000003"),
    };
    place.record(&claim, outcome_record(&fingerprint(), TIME, &outcome).as_str()).expect("the outcome is published");
    let text: String = std::fs::read_to_string(place.path().as_path()).expect("the outcome is there");
    assert!(text.contains(&format!("outcome {ATTACHED}")), "{text}");
    let mode: u32 = std::fs::metadata(place.path().as_path()).expect("the record's metadata reads").permissions().mode() & 0o7777;
    assert_eq!(mode, FIRST_ATTACH_MODE);
    assert!(!place.staging().exists(), "the staging file was renamed, not left");
}

/// A record longer than the bound is refused rather than written,
/// so a reader never meets a file one page cannot hold.
#[test]
fn a_record_longer_than_the_bound_is_refused() {
    let place: FirstAttachPlace = place("too-long");
    let claim = place.claim(&fingerprint(), TIME).expect("the claim wins the name");
    let long: String = format!("{}\n", "x".repeat(super::RECORD_BOUND + 1));
    let refusal: FirstAttachRefusal = place.record(&claim, long.as_str()).expect_err("a record over the bound is refused");
    assert!(matches!(refusal, FirstAttachRefusal::Publish { .. }), "{refusal:?}");
    let text: String = std::fs::read_to_string(place.path().as_path()).expect("the started record still reads");
    assert!(text.contains(&format!("state {STARTED}")), "the started record stands: {text}");
}
