//! What:
//!  The ban on a control publishing, ending or reading the step a waiting add's notice names
//!  in the run's own process:
//!  a control that publishes a step or a plainly run add's route,
//!  starts a notice that prints the step,
//!  ends the step,
//!  or reads the published step
//!  starts a child of its own first.
//! Why:
//!  `worktree/add_standing_notice.rs` keeps the step an add publishes and the route it is on
//!  in `STANDING`, one `Mutex<Option<Standing>>` for the whole process,
//!  because one process runs one add's steps at a time.
//!  `cargo test` runs a crate's tests as threads of one process,
//!  so a control that publishes a step replaces the step a control beside it just published,
//!  and a control that answers its add ends the step another control's notice is waiting to print
//!  (`add_timing::finish` calls `the_add_ended`).
//!  Measured on 2026-11-27 on `main`,
//!  a threaded run of `worktree::add_standing_notice::tests` failed two of its eight controls:
//!  `a_notice_is_printed_once_the_wait_is_over_and_stops_with_its_value` printed no line
//!  (`left: 0`, `right: 1`),
//!  and `a_plain_add_s_line_names_git_s_own_add_and_the_missing_folder` printed a line that named no route,
//!  while `--test-threads=1` on the same module answered `ok. 8 passed; 0 failed` (issue 6533).
//!  The container gate starts each test in a process of its own,
//!  so the gate cannot show it.
//!  A control that starts `run_the_body_in_a_child_of_its_own` first asks its question in a child of the test
//!  binary,
//!  whose record holds nothing but what that control published
//!  (`testing/process_wide_state_child_test_support.rs`).
//!  The scan that splits a source into its controls is
//!  `source_guard/process_wide_fact_readers_tests.rs`,
//!  which this ban shares with the other bans of its shape.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // every control that names a_step_began(, the_plain_add_began(, add_timing::begin_step(
//! // begin_after(, begin_quiet_after( or the_add_ended(
//! // must also name run_the_body_in_a_child_of_its_own(.
//! ```

use super::process_wide_fact_readers_tests::{
    Requirement,
    Scan,
    findings_in_text,
    missing_reader_files,
    scan,
};

/// What:
///  `IN_A_CHILD_OF_ITS_OWN` is the call that starts one control's body in a child process of the test binary.
/// Why:
///  A control that makes it asks its question where no other test's add publishes or ends a step,
///  so a control that names the record and not this call is the finding.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const IN_A_CHILD_OF_ITS_OWN = 'run_the_body_in_a_child_of_its_own(';
/// ```
const IN_A_CHILD_OF_ITS_OWN: &str = "run_the_body_in_a_child_of_its_own(";

/// What:
///  `WHAT_A_CONTROL_MUST_DO` is the answer this ban gives a control:
///  the same call,
///  for the reason a step of the whole process is one step.
/// Why:
///  The finding has to say what to change,
///  and the sentence names the starter and the issue that measured the sharing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const WHAT_A_CONTROL_MUST_DO = "start the control's body in a child of its own first (run_the_body_in_a_child_of_its_own, ...)";
/// ```
const WHAT_A_CONTROL_MUST_DO: &str = "start the control's body in a child of its own first \
                                      (`run_the_body_in_a_child_of_its_own`, \
                                      `testing/process_wide_state_child_test_support.rs`, issue 6533)";

/// What:
///  `THE_STANDING_STEP` is the ban on publishing,
///  ending or reading in the run's own process the step a waiting add's notice names.
/// Why:
///  `STANDING` is made once for the process,
///  and every control that observes it publishes there and reads there,
///  so a control that asks its question among the run's other tests asks about their steps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_STANDING_STEP = { what: 'the step a waiting add notices', named: [...], excusedBy: [IN_A_CHILD_OF_ITS_OWN], files: [...] };
/// ```
const THE_STANDING_STEP: Requirement = Requirement {
    what: "the step a waiting add's notice names and the route it is on",
    named: &[
        (
            "a_step_began(",
            "publishes the step the whole process's notice prints",
        ),
        (
            "the_plain_add_began(",
            "publishes the step and the route a plainly run add's notice prints",
        ),
        (
            "add_timing::begin_step(",
            "publishes the innermost step the whole process's notice prints",
        ),
        (
            "begin_after(",
            "starts the notice thread that prints the step the whole process holds",
        ),
        (
            "begin_quiet_after(",
            "starts the notice thread that prints the step the whole process holds",
        ),
        (
            "the_add_ended(",
            "ends the step the whole process holds, so a notice beside it names none",
        ),
    ],
    excused_by: &[IN_A_CHILD_OF_ITS_OWN],
    required: WHAT_A_CONTROL_MUST_DO,
    files: &[(
        "worktree/add_standing_notice_tests.rs",
        "six controls publish, print, end or read the step a waiting add's notice names, and one of them makes \
         descriptor 2 a pipe",
    )],
};

/// The step a waiting add's notice names is one record for the whole process.
/// A control that publishes it, prints it, ends it or reads it starts a child of its own first.
#[test]
fn no_control_reads_the_standing_step_in_the_run_s_own_process() {
    let read: Scan = scan(&THE_STANDING_STEP).expect("the crate's sources");
    assert!(
        read.findings.is_empty(),
        "{}: one record for the whole process, and a test run holds its controls as threads of it, so a \
         control that asks about it reads another control's step: {}:\n{}",
        THE_STANDING_STEP.what,
        THE_STANDING_STEP.required,
        read.findings.join("\n")
    );
}

/// Every file whose controls ask about the step is still read,
/// so a run that finds no reader fails here instead of passing every tree.
#[test]
fn the_files_that_read_the_standing_step_still_do() {
    let read: Scan = scan(&THE_STANDING_STEP).expect("the crate's sources");
    let missing: Vec<String> = missing_reader_files(&read, &THE_STANDING_STEP);
    assert!(
        missing.is_empty(),
        "these files hold no control that names {} any more, so this check guards nothing there: move each entry \
         to the file that holds such a control, or delete it:\n{}",
        THE_STANDING_STEP.what,
        missing.join("\n")
    );
}

/// The scan finds a control that prints the step in the run's own process,
/// and says nothing about a control beside it that starts a child first,
/// nor about names a comment or a planted text only carries.
#[test]
fn the_scan_finds_a_control_that_prints_the_step_without_a_child_of_its_own() {
    let planted: &str = "\
#[test]\n\
fn prints_the_step() {\n\
    a_step_began(Some(\"gitAdd\"), Instant::now());\n\
    let notice = begin_quiet_after(false, Box::new(|_line: String| {}), Duration::from_millis(1));\n\
}\n\
\n\
#[test]\n\
fn asks_in_a_child_of_its_own() {\n\
    /// The doc comment names the_add_ended( and it is no call.\n\
    run_the_body_in_a_child_of_its_own(module_path!(), \"asks_in_a_child_of_its_own\", || {\n\
        the_plain_add_began(\"because the folder it goes in does not exist yet\", Instant::now());\n\
        add_timing::begin_step(\"gitAdd\");\n\
        let published = published();\n\
    });\n\
}\n\
\n\
#[test]\n\
fn plants_the_names_in_a_text() {\n\
    let planted: &str = \"a_step_began(; the_plain_add_began(; add_timing::begin_step(; begin_after(; begin_quiet_after(; the_add_ended(\";\n\
}\n";
    assert_eq!(
        findings_in_text(&THE_STANDING_STEP, "planted_tests.rs", planted),
        vec![String::from("planted_tests.rs:1 (prints_the_step)")],
        "the planted control that prints the one step of the process was not the one finding"
    );
}
