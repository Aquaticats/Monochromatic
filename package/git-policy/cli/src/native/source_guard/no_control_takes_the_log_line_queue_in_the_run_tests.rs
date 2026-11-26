//! What:
//!  The ban on a control taking the process's one queue of log lines in the run's own process:
//!  a control that gives that queue a stream,
//!  reads the count of lines it could not take,
//!  or reads whether a caller gave up waiting
//!  starts a child of its own first.
//! Why:
//!  The service's log lines and the command's diagnostic lines are each written by one thread of the process
//! (`foundation/lines_written_by_a_thread.rs`),
//!  and `give_the_thread` puts a control's stream in the one place that thread writes.
//!  `cargo test` runs a crate's tests as threads of one process,
//!  so a control that gives the queue a stream replaces the stream a control beside it just gave it,
//!  and a control that reads the queue's figures reads another control's lines:
//!  measured on 2026-11-26 on `main` at `4674e5b44a`,
//!  a threaded run of `service::service_log_lines_off_the_calling_thread::tests` failed 5 of its 6 controls,
//!  the same run with `--test-threads=1` failed 2,
//!  and each of the six passed alone in a process of its own (issue 6521).
//!  A probe of that run logged four controls giving the one queue a stream in one process,
//!  and one control's line taking ticket 5 after the four lines of the others.
//!  The container gate starts each test in a process of its own, so the gate cannot show it.
//!  A control that starts `run_the_body_in_a_child_of_its_own` first asks its question in a child of the test
//!  binary,
//!  whose queue holds nothing but what that control handed it
//!  (`testing/process_wide_state_child_test_support.rs`).
//!  The scan that splits a source into its controls is
//!  `source_guard/process_wide_fact_readers_tests.rs`,
//!  which this ban shares with the other bans of its shape.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // every control that names give_the_thread(, dropped_so_far( or the_reader_stopped(
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
///  A control that makes it asks its question where no other test's lines wait in the queue,
///  so a control that names the queue and not this call is the finding.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const IN_A_CHILD_OF_ITS_OWN = 'run_the_body_in_a_child_of_its_own(';
/// ```
const IN_A_CHILD_OF_ITS_OWN: &str = "run_the_body_in_a_child_of_its_own(";

/// What:
///  `WHAT_A_CONTROL_MUST_DO` is the answer this ban gives a control:
///  the same call,
///  for the reason a queue of the whole process is one queue.
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
                                      `testing/process_wide_state_child_test_support.rs`, issue 6521)";

/// What:
///  `THE_LOG_LINE_QUEUE` is the ban on taking the one queue of log lines in the run's own process.
/// Why:
///  Both queues of `lines_written_by_a_thread.rs` are made once for the process,
///  and each of their controls gives that one queue a stream and reads its figures,
///  so a control that asks its question among the run's other tests asks about their lines.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_LOG_LINE_QUEUE = { what: 'the one queue of log lines', named: [...], excusedBy: [IN_A_CHILD_OF_ITS_OWN], files: [...] };
/// ```
const THE_LOG_LINE_QUEUE: Requirement = Requirement {
    what: "the one queue of log lines and the stream its thread writes",
    named: &[
        (
            "give_the_thread(",
            "puts a control's stream in the one place the process's queue writes",
        ),
        (
            "dropped_so_far(",
            "reads the count of lines the whole process's queue could not take",
        ),
        (
            "the_reader_stopped(",
            "reads whether a caller of the whole process gave up waiting for its line",
        ),
    ],
    excused_by: &[IN_A_CHILD_OF_ITS_OWN],
    required: WHAT_A_CONTROL_MUST_DO,
    files: &[
        (
            "foundation/diagnostic_lines_off_the_calling_thread_tests.rs",
            "seven controls ask how the one queue of the process takes, drops and waits for lines",
        ),
        (
            "service/service_log_lines_off_the_calling_thread_tests.rs",
            "six controls ask how the service's queue takes, drops and waits for lines",
        ),
    ],
};

/// One queue of log lines is one for the whole process.
/// A control that gives it a stream or reads its figures starts a child of its own first.
#[test]
fn no_control_takes_the_log_line_queue_in_the_run_s_own_process() {
    let read: Scan = scan(&THE_LOG_LINE_QUEUE).expect("the crate's sources");
    assert!(
        read.findings.is_empty(),
        "{}: one queue for the whole process, and a test run holds its controls as threads of it, so a control that \
         asks about it reads or replaces another control's queue: {}:\n{}",
        THE_LOG_LINE_QUEUE.what,
        THE_LOG_LINE_QUEUE.required,
        read.findings.join("\n")
    );
}

/// Every file whose controls ask about the queue is still read,
/// so a run that finds no reader fails here instead of passing every tree.
#[test]
fn the_files_that_read_the_log_line_queue_still_do() {
    let read: Scan = scan(&THE_LOG_LINE_QUEUE).expect("the crate's sources");
    let missing: Vec<String> = missing_reader_files(&read, &THE_LOG_LINE_QUEUE);
    assert!(
        missing.is_empty(),
        "these files hold no control that names {} any more, so this check guards nothing there: move each entry \
         to the file that holds such a control, or delete it:\n{}",
        THE_LOG_LINE_QUEUE.what,
        missing.join("\n")
    );
}

/// The scan finds a control that takes the queue in the run's own process,
/// and says nothing about a control beside it that starts a child first,
/// nor about names a comment or a planted text only carries.
#[test]
fn the_scan_finds_a_control_that_takes_the_queue_without_a_child_of_its_own() {
    let planted: &str = "\
#[test]\n\
fn gives_the_queue_a_stream() {\n\
    let (reader, writer) = std::io::pipe().expect(\"a pipe\");\n\
    give_the_thread(Box::new(writer));\n\
}\n\
\n\
#[test]\n\
fn asks_in_a_child_of_its_own() {\n\
    /// The doc comment names give_the_thread( and it is no call.\n\
    run_the_body_in_a_child_of_its_own(module_path!(), \"asks_in_a_child_of_its_own\", || {\n\
        give_the_thread(Box::new(std::io::sink()));\n\
        let dropped = dropped_so_far();\n\
    });\n\
}\n\
\n\
#[test]\n\
fn plants_the_names_in_a_text() {\n\
    let planted: &str = \"give_the_thread(; dropped_so_far(); the_reader_stopped(\";\n\
}\n";
    assert_eq!(
        findings_in_text(&THE_LOG_LINE_QUEUE, "planted_tests.rs", planted),
        vec![String::from(
            "planted_tests.rs:1 (gives_the_queue_a_stream)"
        )],
        "the planted control that gives the one queue of the process a stream was not the one finding"
    );
}
