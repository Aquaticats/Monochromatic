//! What:
//!  The ban on a control that steps a notice reading the notice's control window in the run's own process:
//!  a control that steps a step of a notice takes the window first,
//!  or starts a child of its own.
//! Why:
//!  `commit/native_commit_waiting_notice_control_window.rs` and
//!  `commit/scratch_step_look_below_control_window.rs` each keep one window for the whole process:
//!  the ceiling a control plants,
//!  the bound a step the kernel holds is waited for,
//!  the looks a control states cannot be made,
//!  and the process a control states the kernel holds.
//!  A control that plants one takes its window,
//!  and a control that reads one without taking it fails by which control ran first:
//!  measured on 2026-11-26,
//!  `commit::native_commit_waiting_notice::tests::the_notice_reports_the_programs_that_ran_below_the_step`
//!  credited one 250 ms interval of a step it had watched for 1.2 s,
//!  because `commit_preparation_tests` planted six looks that cannot be made while its notice looked
//!  (issue 6509).
//!  A look that cannot be made clears the look before it,
//!  so the credit starts again and the figures are a control that ran beside it.
//!  A control that starts `run_the_body_in_a_child_of_its_own` first asks its question in a child of the test
//!  binary,
//!  whose windows nothing else planted in,
//!  and one that starts `started_this_test_in_a_child` does the same for the descriptor limit
//!  (`testing/descriptor_limit_child_test_support.rs`).
//!  The scan that splits a source into its controls is
//!  `source_guard/process_wide_fact_readers_tests.rs`,
//!  which this ban shares with the other bans of its shape.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // every control that names step_started( must also name take_the_window(, started_this_test_in_a_child(
//! // or run_the_body_in_a_child_of_its_own(.
//! ```

use super::process_wide_fact_readers_tests::{
    Requirement,
    Scan,
    findings_in_text,
    missing_reader_files,
    scan,
};

/// What:
///  `STEPPED` is the one call that makes a control's figures depend on the window:
///  the notice that carries the step looks below it at the interval the window decides,
///  and stops it at the ceiling the window names.
/// Why:
///  A notice that never carries a step reads no window,
///  so the call is the fact that a control's figures are about a step the window changed.
const STEPPED: &[(&str, &str)] = &[(
    "step_started(",
    "tells a notice the step whose ceiling and looks the window decides",
)];

/// What:
///  `EXCUSED_BY_A_WINDOW_OR_A_CHILD` is each call that gives the control a window of its own.
/// Why:
///  A control that takes the window asks its question where the other controls' plants cannot land,
///  and a child process has windows of its own that nothing else planted in.
const EXCUSED_BY_A_WINDOW_OR_A_CHILD: &[&str] = &[
    "take_the_window(",
    "take_the_windows_a_notice_control_reads(",
    "started_this_test_in_a_child(",
    "run_the_body_in_a_child_of_its_own(",
];

/// What:
///  `THE_NOTICE_WINDOW` is the ban:
///  the call that makes a control read the window,
///  the calls that answer it,
///  what a control must do instead,
///  and the file that must still step a notice.
/// Why:
///  One file holds every control that steps a notice of this crate,
///  so one requirement covers them all,
///  and a run that finds no such control fails instead of passing every tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_NOTICE_WINDOW = { what: 'the control window a notice reads', named: STEPPED, excusedBy: EXCUSED_BY_A_WINDOW_OR_A_CHILD, required: '...', files: [['commit/native_commit_waiting_notice_tests.rs', '...']] };
/// ```
const THE_NOTICE_WINDOW: Requirement = Requirement {
    what: "the control window a notice's step is looked at and stopped through",
    named: STEPPED,
    excused_by: EXCUSED_BY_A_WINDOW_OR_A_CHILD,
    required: "take the control window first (`take_the_window` of the look's window and of the notice's, in \
               that order) or start the control's body in a child of its own (`run_the_body_in_a_child_of_its_own`, \
               `testing/process_wide_state_child_test_support.rs`, issue 6509)",
    files: &[(
        "commit/native_commit_waiting_notice_tests.rs",
        "seven controls step a notice and read the ceiling and looks the window decides",
    )],
};

/// No control steps a notice without a window of its own.
/// A control that reads the window takes it first or starts a child of its own.
#[test]
fn no_control_steps_a_notice_without_its_window() {
    let read: Scan = scan(&THE_NOTICE_WINDOW).expect("the crate's sources");
    assert!(
        read.findings.is_empty(),
        "{} is one window for the whole process, and a test run holds its controls as threads of it, so a \
         control that reads it without taking it measures another control's plant: {}:\n{}",
        THE_NOTICE_WINDOW.what,
        THE_NOTICE_WINDOW.required,
        read.findings.join("\n")
    );
}

/// The file whose controls step a notice is still read,
/// so a run that finds none fails here instead of passing every tree.
#[test]
fn the_file_that_steps_a_notice_still_does() {
    let read: Scan = scan(&THE_NOTICE_WINDOW).expect("the crate's sources");
    let missing: Vec<String> = missing_reader_files(&read, &THE_NOTICE_WINDOW);
    assert!(
        missing.is_empty(),
        "these files hold no control that names {} any more, so this check guards nothing there: move each \
         entry to the file that holds such a control, or delete it:\n{}",
        THE_NOTICE_WINDOW.what,
        missing.join("\n")
    );
}

/// The scan finds a control that steps a notice in the run's own process,
/// and says nothing about the control beside it that takes the window first,
/// nor about a call a comment or a planted text only names.
#[test]
fn the_scan_finds_a_control_that_steps_a_notice_without_its_window() {
    let planted: &str = "\
#[test]\n\
fn steps_in_the_run_s_own_process() {\n\
    notice.step_started(step_number(&step));\n\
}\n\
\n\
#[test]\n\
fn steps_with_the_window_taken() {\n\
    let window = the_controls_window::take_the_window();\n\
    notice.step_started(step_number(&step));\n\
}\n\
\n\
#[test]\n\
fn steps_in_a_child_of_its_own() {\n\
    /// The doc comment names step_started( and it is no call.\n\
    run_the_body_in_a_child_of_its_own(module_path!(), \"steps_in_a_child_of_its_own\", || {\n\
        notice.step_started(step_number(&step));\n\
    });\n\
}\n\
\n\
#[test]\n\
fn plants_the_name_in_a_text() {\n\
    let planted: &str = \"step_started(\";\n\
}\n";
    assert_eq!(
        findings_in_text(&THE_NOTICE_WINDOW, "planted_tests.rs", planted),
        vec![String::from(
            "planted_tests.rs:1 (steps_in_the_run_s_own_process)"
        )],
        "the planted control that steps a notice in the run's own process was not the one finding"
    );
}
