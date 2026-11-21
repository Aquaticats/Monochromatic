//! What:
//!  Controls of the mode at the user boundary:
//!  a gate refusal writes a skipped record naming the gate's step,
//!  and a run whose worker cannot unshare writes a record naming that step
//!  with the errno the kernel answered,
//!  leaving no mount anywhere.
//! Why:
//!  The record is what the add preflight and `doctor` read,
//!  so the two endings that decide a boot's route must be visible in it,
//!  and a non-root control must be able to run the mode without mounting
//!  anything:
//! its first call refuses before any mount exists.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! skipAfterGateRefusal(place, paths, 'member'); expect(read(place.result())).toContain('verdict skipped');
//! ```

use super::super::probe_fingerprint::FingerprintPaths;
use super::super::probe_record_files::RecordPlace;
use super::super::probe_mode_words::skip_after_gate_refusal;
use crate::testing::test_support::fixture;
use std::path::{Path, PathBuf};

/// A fixture set of fingerprint files,
/// which the mode reads before it writes anything.
fn fingerprint_paths(root: &Path) -> FingerprintPaths {
    let write = |name: &str, text: &str| -> PathBuf {
        let path: PathBuf = root.join(name);
        std::fs::write(&path, text).expect("the fixture file is written");
        return path;
    };
    return FingerprintPaths {
        kernel_release: write("osrelease", "6.8.0-147-generic\n"),
        module_version: write("version", "2.4.4-1\n"),
        module_srcversion: write("srcversion", "A3553C308C29FD4B6595A72\n"),
        module_build_id: root.join("no-note"),
        boot_id: write("boot_id", "6f0f1a2b-3c4d-4e5f-8a9b-0c1d2e3f4a5b\n"),
        handler: root.join("no-handler"),
    };
}

/// A gate refusal is recorded as a skipped probe,
/// naming the gate's own failing step,
/// and the mode still answers a line rather than a failure.
#[test]
fn a_gate_refusal_is_a_skipped_record() {
    let root: PathBuf = fixture("start-probe-mode-skip");
    let place: RecordPlace = RecordPlace::at(root.join("records"));
    let paths: FingerprintPaths = fingerprint_paths(root.as_path());
    let line: String = skip_after_gate_refusal(&place, &paths, "member").expect("the skipped record is written");
    let text: String = std::fs::read_to_string(place.result()).expect("the record is read");
    assert!(text.contains("state finished"));
    assert!(text.contains("verdict skipped"));
    assert!(line.contains("skipped"));
    assert!(line.contains("member"));
}

/// A mode whose fingerprint cannot be read writes nothing at all,
/// so no reader mistakes an unwritten record for a probe that never started.
#[test]
fn a_mode_without_a_fingerprint_writes_no_record() {
    let root: PathBuf = fixture("start-probe-mode-no-fingerprint");
    let place: RecordPlace = RecordPlace::at(root.join("records"));
    let mut paths: FingerprintPaths = fingerprint_paths(root.as_path());
    paths.kernel_release = root.join("no-such-release");
    let configuration: PathBuf = root.join("handler.jsonc");
    std::fs::write(&configuration, "{}").expect("the configuration is written");
    let mode = super::super::probe_mode::ProbeMode::new(&place, configuration.as_path());
    let refusal: String = mode.run_with(&paths).expect_err("the fingerprint is refused");
    assert!(refusal.contains("kernel"), "{refusal}");
    assert!(!place.result().exists(), "no record is written");
}
