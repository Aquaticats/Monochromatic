//! Controls for the test-only transaction phase markers.

use super::*;
use crate::test_support::{fixture, remove};

/// The phases are named in transaction order, as the incumbent lists them.
#[test]
fn phases_are_named_in_transaction_order() {
    let names: Vec<&str> = TRANSACTION_TEST_PHASES.iter().map(|phase| { return phase.name(); }).collect();
    assert_eq!(
        names,
        [
            "capture-locked",
            "preparation-done",
            "landing-locked",
            "objects-migrated",
            "ref-updated",
            "index-installed",
            "race-lost",
            "merge-intent",
            "merge-applied",
        ]
    );
}

/// A kill marker without a directory parses.
#[test]
fn kill_without_directory_parses() {
    let signal = parse_phase_signal("ref-updated:kill").expect("valid marker");
    assert_eq!(signal.phase, Phase::RefUpdated);
    assert_eq!(signal.action, PhaseAction::Kill);
    assert_eq!(signal.directory, None);
}

/// A kill or pause marker with an absolute directory parses.
#[test]
fn kill_and_pause_with_absolute_directory_parse() {
    let directory: PathBuf = crate::test_support::fixture_root();
    let text: String = format!("landing-locked:kill:{}", directory.display());
    let signal = parse_phase_signal(&text).expect("valid kill");
    assert_eq!(signal.action, PhaseAction::Kill);
    assert_eq!(signal.directory.as_deref(), Some(directory.as_path()));
    let paused: String = format!("race-lost:pause:{}", directory.display());
    let pause_signal = parse_phase_signal(&paused).expect("valid pause");
    assert_eq!(pause_signal.phase, Phase::RaceLost);
    assert_eq!(pause_signal.action, PhaseAction::Pause);
}

/// An empty directory after the action is no directory, as the incumbent's parser reads it.
#[test]
fn kill_with_empty_directory_is_no_directory() {
    let signal = parse_phase_signal("ref-updated:kill:").expect("accepted by the incumbent");
    assert_eq!(signal.directory, None);
}

/// Every malformed value is refused: unknown phase, unknown action, pause without directory,
/// relative directory, missing phase.
#[test]
fn malformed_values_are_refused() {
    for value in [
        "",
        "ref-updated",
        "bogus:kill",
        "ref-updated:stop",
        "ref-updated:pause",
        "ref-updated:pause:",
        "ref-updated:kill:relative",
        "ref-updated:pause:relative",
        ":kill",
    ] {
        assert!(parse_phase_signal(value).is_err(), "{value:?} must be refused");
    }
}

/// The refusal names the variable, the JSON-quoted value and the accepted forms.
#[test]
fn refusal_message_names_the_value_and_the_forms() {
    let error = parse_phase_signal("bogus:kill").expect_err("unknown phase");
    assert_eq!(
        error.to_string(),
        "CLI_GIT_TEST_ONLY_PHASE_SIGNAL=\"bogus:kill\" is not <phase>:kill[:<absolute directory>] or <phase>:pause:<absolute directory> with a phase among capture-locked, preparation-done, landing-locked, objects-migrated, ref-updated, index-installed, race-lost, merge-intent, merge-applied."
    );
}

/// An absent or empty variable marks nothing and writes nothing.
#[test]
fn absent_variable_reaches_silently() {
    let root: PathBuf = fixture("phase-absent");
    reach_with_value(None, Phase::LandingLocked, None).expect("absent marker");
    reach_with_value(Some(""), Phase::LandingLocked, None).expect("empty marker");
    assert_eq!(std::fs::read_dir(&root).expect("fixture").count(), 0);
    remove(&root);
}

/// A marker for another phase writes nothing, but a malformed marker fails at every phase.
#[test]
fn other_phases_are_ignored_and_malformed_markers_fail_everywhere() {
    let root: PathBuf = fixture("phase-other");
    let value: String = format!("ref-updated:kill:{}", root.display());
    reach_with_value(Some(value.as_str()), Phase::LandingLocked, None).expect("other phase");
    assert_eq!(std::fs::read_dir(&root).expect("fixture").count(), 0);
    let malformed: String = format!("ref-updated:stop:{}", root.display());
    assert!(reach_with_value(Some(malformed.as_str()), Phase::LandingLocked, None).is_err());
    remove(&root);
}

/// Reach a phase on a thread and fail instead of hanging when the wait never ends,
///  so a defect in the wait is a failure and not a timeout.
fn reach_bounded(value: String, phase: Phase, occurrence: Option<u64>) -> Result<(), PhaseMarkerError> {
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = sender.send(reach_with_value(Some(value.as_str()), phase, occurrence));
    });
    return receiver
        .recv_timeout(std::time::Duration::from_secs(30))
        .expect("the phase was reached instead of waiting for ever");
}

/// A pause writes the reached marker with the process ID, then waits for the release file:
///  it does not return before the file exists.
#[test]
fn pause_waits_for_the_release_file() {
    let root: PathBuf = fixture("phase-pause");
    let value: String = format!("landing-locked:pause:{}", root.display());
    let releaser_root: PathBuf = root.clone();
    let releaser = std::thread::spawn(move || {
        let reached: PathBuf = releaser_root.join("landing-locked.reached");
        while !reached.exists() {
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
        std::fs::write(releaser_root.join("landing-locked.release"), b"").expect("release file");
    });
    let started: std::time::Instant = std::time::Instant::now();
    reach_bounded(value, Phase::LandingLocked, None).expect("released");
    assert!(started.elapsed() >= std::time::Duration::from_millis(150), "the pause returned before the release");
    assert!(root.join("landing-locked.release").exists());
    releaser.join().expect("releaser");
    let reached: String = std::fs::read_to_string(root.join("landing-locked.reached")).expect("reached");
    assert_eq!(reached, std::process::id().to_string());
    remove(&root);
}

/// A repeatable phase numbers its marker files by occurrence.
#[test]
fn repeatable_phase_numbers_its_files() {
    let root: PathBuf = fixture("phase-occurrence");
    let value: String = format!("race-lost:pause:{}", root.display());
    std::fs::write(root.join("race-lost-2.release"), b"").expect("release file");
    reach_bounded(value, Phase::RaceLost, Some(2)).expect("second occurrence");
    assert!(root.join("race-lost-2.reached").exists());
    assert!(!root.join("race-lost.reached").exists());
    remove(&root);
}

/// Killing the process at a marker ends it with SIGKILL, after its reached marker is written.
/// The child is this test binary running the ignored helper below.
#[cfg(unix)]
#[test]
fn kill_marker_sigkills_the_process_after_writing_its_marker() {
    use std::os::unix::process::ExitStatusExt;
    let root: PathBuf = fixture("phase-kill");
    let value: String = format!("objects-migrated:kill:{}", root.display());
    let status = std::process::Command::new(std::env::current_exe().expect("test binary"))
        .args(["--exact", "transaction_phase::tests::child_reaches_objects_migrated", "--ignored"])
        .env(PHASE_SIGNAL_VARIABLE, &value)
        .status()
        .expect("child test binary");
    assert_eq!(status.signal(), Some(9));
    assert!(root.join("objects-migrated.reached").exists());
    remove(&root);
}

/// Helper run only by the kill control above, in its own process.
#[test]
#[ignore = "child process of kill_marker_sigkills_the_process_after_writing_its_marker"]
fn child_reaches_objects_migrated() {
    reach_transaction_phase(Phase::ObjectsMigrated, None).expect("marker is armed in the environment");
    panic!("the kill marker did not end the process");
}
