// What:     Unit tests for `folder_chooser.rs`, pulled in by `#[cfg(test)] #[path = "folder_chooser_tests.rs"] mod tests;`.
// Why:      Issue 1010: a chooser that exists but cannot open a window must be told apart from a cancel, with the
//           program's own explanation kept. Every chooser here is a stub script in a disposable folder, so no dialog
//           ever opens on the desktop this test runs on.

use super::*;
use crate::logging::log_capture::capture_records;

/// What:
///     The program file name the stubs carry,
/// so the messages under test read as they do in production.
/// Why:
///      A message that names `zenity` is checked by the failure tests,
/// and the stub must be found under that name.
#[cfg(target_os = "linux")]
const ZENITY_NAME: &str = "zenity";

/// What:
///     The permission bit a stub needs to be started at all.
/// Why:
///      The chooser is run by absolute path,
/// and a file without the bit is a start failure,
/// which one test needs.
#[cfg(target_os = "linux")]
use std::os::unix::fs::PermissionsExt;

/// What:
///     The byte view of a folder name,
/// for the test that a path which is not valid text survives.
/// Why:
///      A Linux path is bytes,
/// and the assertion compares those bytes.
#[cfg(target_os = "linux")]
use std::os::unix::ffi::OsStrExt;

/// What:
///     An executable chooser stub in a fresh disposable folder.
/// Why:
///      The chooser is run by absolute path,
/// so the stub is the only program any test starts,
/// whatever the real
///           `PATH` of the machine holds.
#[cfg(target_os = "linux")]
fn stub(tag: &str, body: &str) -> PathBuf {
    let program: PathBuf = crate::test_scratch::scratch_root().join(format!("music-player-chooser-{tag}")).join(ZENITY_NAME);
    std::fs::create_dir_all(program.parent().expect("the stub folder is named")).expect("the stub folder is created");
    std::fs::write(&program, format!("#!/bin/sh\n{body}")).expect("the stub is written");
    let mut permissions: std::fs::Permissions = std::fs::metadata(&program).expect("the stub exists").permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&program, permissions).expect("the stub is made executable");
    return program;
}

#[test]
#[cfg(target_os = "linux")]
fn a_chosen_folder_is_reported() {
    let program: PathBuf = stub("chosen", "printf '/a/private/album\\n'\n");
    let outcome: ChooserOutcome = run_chooser_program(&program);
    assert_eq!(outcome, ChooserOutcome::Chosen { folder: PathBuf::from("/a/private/album") });
}

#[test]
#[cfg(target_os = "linux")]
fn the_chosen_folder_stays_out_of_the_log_while_the_outcome_is_logged() {
    let program: PathBuf = stub("logged", "printf '/a/private/album\\n'\n");
    let records: String = capture_records(None, || {
        run_chooser_program(&program);
    });
    assert!(
        records.contains("the folder chooser chose a folder"),
        "the outcome is worth a record; got {records:?}",
    );
    assert!(
        !records.contains("private"),
        "the chosen folder is the user's private path and stays out of the log; got {records:?}",
    );
}

#[test]
#[cfg(target_os = "linux")]
fn a_folder_name_that_is_not_valid_text_is_still_chosen() {
    // 0xFF 0xFE is a legal pair of bytes in a Linux path and not valid UTF-8.
    let program: PathBuf = stub("bytes", "printf '\\377\\376\\n'\n");
    let chosen: Option<PathBuf> = match run_chooser_program(&program) {
        ChooserOutcome::Chosen { folder } => Some(folder),
        other => panic!("a printed folder is a chosen folder, got {other:?}"),
    };
    assert_eq!(chosen.expect("a folder was chosen").as_os_str().as_bytes(), &[0xFF, 0xFE]);
}

#[test]
#[cfg(target_os = "linux")]
fn a_cancel_is_not_a_failure() {
    // zenity's cancel: exit 1 and nothing on either stream.
    let program: PathBuf = stub("cancel", "exit 1\n");
    assert_eq!(run_chooser_program(&program), ChooserOutcome::Cancelled);
}

#[test]
#[cfg(target_os = "linux")]
fn a_program_that_prints_nothing_and_exits_zero_chose_nothing() {
    let program: PathBuf = stub("quiet", "exit 0\n");
    assert_eq!(run_chooser_program(&program), ChooserOutcome::Cancelled);
}

#[test]
#[cfg(target_os = "linux")]
fn a_program_that_cannot_open_a_window_is_a_failure_with_its_own_explanation() {
    // Measured on zenity 4.2.2 with no display: exit 1 and `Failed to open display` on standard error, which is the
    // whole difference from a cancel.
    let program: PathBuf = stub("broken", "printf 'Failed to open display\\n' >&2\nexit 1\n");
    let outcome: ChooserOutcome = run_chooser_program(&program);
    assert_eq!(
        outcome,
        ChooserOutcome::Failed {
            failure: ChooserFailure {
                cause: "zenity exited with status 1".to_string(),
                standard_error: "Failed to open display".to_string(),
            },
        },
    );
}

#[test]
#[cfg(target_os = "linux")]
fn a_program_that_cannot_start_is_a_failure() {
    let program: PathBuf = stub("unrunnable", "exit 0\n");
    let mut permissions: std::fs::Permissions = std::fs::metadata(&program).expect("the stub exists").permissions();
    permissions.set_mode(0o644);
    std::fs::set_permissions(&program, permissions).expect("the stub loses its permission bit");
    let cause: String = match run_chooser_program(&program) {
        ChooserOutcome::Failed { failure } => failure.cause,
        other => panic!("a program that cannot start is a failure, got {other:?}"),
    };
    assert!(
        cause.starts_with("zenity could not be started:"),
        "the failure names the program and the start error; got {cause:?}",
    );
}

#[test]
#[cfg(target_os = "linux")]
fn a_folder_beside_a_warning_is_still_the_answer() {
    let program: PathBuf = stub("noisy", "printf '/a/chosen\\n'\nprintf 'Gtk-WARNING: something\\n' >&2\nexit 1\n");
    assert_eq!(run_chooser_program(&program), ChooserOutcome::Chosen { folder: PathBuf::from("/a/chosen") });
}

#[test]
#[cfg(target_os = "linux")]
fn the_way_a_chooser_opens_follows_the_two_probes() {
    let program: PathBuf = PathBuf::from("/usr/bin/zenity");
    assert_eq!(chooser_source(None, false), ChooserSource::Missing);
    assert_eq!(chooser_source(Some(program.clone()), false), ChooserSource::Program { program: program.clone() });
    // The portal is the desktop's own chooser, so it is preferred when both exist.
    assert_eq!(chooser_source(None, true), ChooserSource::Portal);
    assert_eq!(chooser_source(Some(program), true), ChooserSource::Portal);
}

#[test]
fn the_failure_message_names_the_program_its_status_and_what_it_said() {
    let failure = ChooserFailure {
        cause: "zenity exited with status 1".to_string(),
        standard_error: "Failed to open display".to_string(),
    };
    let message: String = failure_message(&failure);
    assert!(message.contains("zenity exited with status 1"), "got {message:?}");
    assert!(message.contains("Failed to open display"), "got {message:?}");
    assert!(message.contains(crate::dialog_backend::NO_DIALOG_REMEDY), "the ways forward are stated; got {message:?}");
    assert!(!message.contains('—') && !message.contains('–'), "no dashes; got {message:?}");
}

#[test]
fn a_failure_without_a_reported_line_still_reads_whole() {
    let failure = ChooserFailure {
        cause: "zenity could not be started: Permission denied (os error 13)".to_string(),
        standard_error: String::new(),
    };
    let message: String = failure_message(&failure);
    assert!(message.ends_with(crate::dialog_backend::NO_DIALOG_REMEDY), "got {message:?}");
    assert!(!message.contains(": ."), "no empty detail; got {message:?}");
}

#[test]
fn every_outcome_has_its_own_advice() {
    assert_eq!(
        advice(ChooserOutcome::Chosen { folder: PathBuf::from("/a") }),
        ChooserAdvice::Open { folder: PathBuf::from("/a") },
    );
    assert_eq!(advice(ChooserOutcome::Cancelled), ChooserAdvice::Nothing);
    assert_eq!(advice(ChooserOutcome::NoChooser), ChooserAdvice::TellNoChooser);
    let failure = ChooserFailure {
        cause: "zenity exited with status 1".to_string(),
        standard_error: "Failed to open display".to_string(),
    };
    assert_eq!(
        advice(ChooserOutcome::Failed { failure: failure.clone() }),
        ChooserAdvice::TellFailure { message: failure_message(&failure) },
    );
}
