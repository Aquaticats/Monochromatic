//! What:
//!  The ban on a control recording, preparing or reading the service's stop in the run's own process:
//!  a control that touches it starts a child of its own first.
//! Why:
//!  `process/stop_of_the_service_noticed_by_its_children.rs` keeps the signal the handler recorded,
//!  the pipe it wakes the shutdown through,
//!  the level every waiter reads and the count of unfinished work
//!  in statics of the whole process,
//!  and `cargo test` runs a crate's tests as threads of one process.
//!  So a control that records a stop refuses every push,
//!  query and step another thread of the run starts,
//!  and a control that asserts the state a process starts with reads whatever another control left:
//!  measured on 2026-11-27,
//!  a threaded run of `push::` failed 110 of its 599 tests
//!  and a threaded run of `process::` failed eleven,
//!  and a threaded run of the stop's own module answered
//!  `test result: FAILED. 2 passed; 8 failed`
//!  (issues 6530 and 6531).
//!  The container gate starts each test in a process of its own,
//!  so the gate cannot show it.
//!  A control that hands its body to `run_the_body_in_a_child_of_its_own` first asks its question in a child of the
//!  test binary, whose statics hold nothing but what that control recorded
//!  (`testing/process_wide_state_child_test_support.rs`).
//!  The scan that splits a source into its controls is
//!  `source_guard/process_wide_fact_readers_tests.rs`,
//!  which this ban shares with the other bans of its shape.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // every control that names prepare_to_notice(, record_from_a_signal_handler( or any read of the stop must also
//! // name run_the_body_in_a_child_of_its_own(.
//! ```

use super::process_wide_fact_readers_tests::{
    Requirement,
    Scan,
    findings_in_text,
    missing_reader_files,
    scan,
};

/// What:
///  `STOP_NAMES` is each call that reaches the one stop of the process,
///  with why that name is the stop's.
///  Its siblings a reader might expect are the state's own statics,
///  which no control names because they are private to the module,
///  and the constants beside them (`SETTLE_CEILING`, `LAST_RESORT`),
///  which a control may read anywhere because no process changes them.
/// Why:
///  A control that names one of these asks a question about the whole process,
///  so it is each call that reaches the state that has to be found,
///  not one spelling of it:
///  a control that only reads `is_stopping` refuses the same work as one that records a signal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const STOP_NAMES: ReadonlyArray<readonly [string, string]> = [['prepare_to_notice(', '...'], ...];
/// ```
const STOP_NAMES: &[(&str, &str)] = &[
    (
        "prepare_to_notice(",
        "makes the one wake pipe of the process and turns it into a process that notices a stop",
    ),
    (
        "record_from_a_signal_handler(",
        "stores the first stop signal in the whole process, and answers no to every later one",
    ),
    (
        "wake_the_shutdown_from_a_signal_handler(",
        "writes the byte the one shutdown of the process reads",
    ),
    (
        "tell_the_stop_and_wait_for_the_work(",
        "tells every waiter of the process, and every waiter's arm then resolves",
    ),
    (
        "stopping_signal(",
        "reads the signal the whole process recorded",
    ),
    (
        "is_noticing(",
        "reads whether the whole process prepared to notice a stop",
    ),
    (
        "is_stopping(",
        "reads whether the whole process recorded a stop",
    ),
    (
        "unfinished_work(",
        "reads how much work the whole process has in flight",
    ),
];

/// What:
///  The call that runs one control's body in a child process of the test binary.
/// Why:
///  A control that hands its body to it asks its question where nothing else recorded a stop,
///  so a control that names the stop and not this call is the finding.
const IN_A_CHILD_OF_ITS_OWN: &str = "run_the_body_in_a_child_of_its_own(";

/// What:
///  The call that answers the verdict of a body moved into a helper process of the test binary.
///  The sibling a reader might expect is the starter above,
///  which a moved body does not call: its control starts the helper and reads what the body answered.
/// Why:
///  `process/signal_relay_tests.rs` moves the body of its service-stop scenario into such a helper,
///  and that body records a stop,
///  so the file holds one control naming the stop that owns a process of its own without naming the starter.
const IN_A_HELPER_OF_THE_MOVED_BODY: &str = "verdict_in_a_helper(";

/// What:
///  `FILES_THAT_TOUCH_THE_STOP` is each file whose controls record, prepare or read the stop,
///  with what its controls ask.
/// Why:
///  A scan that found no reader at all would pass every tree and guard nothing,
///  so a run that stops seeing these files fails here instead,
///  and a file that moves or drops its control shrinks this list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const FILES_THAT_TOUCH_THE_STOP = [['process/stop_of_the_service_noticed_by_its_children_tests.rs', '...'], ...];
/// ```
const FILES_THAT_TOUCH_THE_STOP: &[(&str, &str)] = &[
    (
        "process/stop_of_the_service_noticed_by_its_children_tests.rs",
        "nine controls record a stop, prepare to notice one or read the state a process starts with",
    ),
    (
        "process/child_awaited_until_it_exits_the_stop_or_the_ceiling_tests.rs",
        "one control records a stop of its own and ends its step with it",
    ),
    (
        "process/signal_relay_tests.rs",
        "the moved body of the service-stop scenario records a stop in a helper process",
    ),
    (
        "push/bounded_push_awaiting_tests.rs",
        "three controls record a stop or prepare to notice one",
    ),
];

/// What:
///  `THE_SERVICE_STOP` is the ban itself:
///  the stop's names,
///  the calls that answer it,
///  what a control must do instead,
///  and the files that must still touch the stop.
/// Why:
///  The scan in `process_wide_fact_readers_tests.rs` takes a requirement like this one,
///  so the ban states its own names and the scan stays the same for every ban of this shape.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_SERVICE_STOP = { what: "the service's stop", named: STOP_NAMES, excusedBy: [IN_A_CHILD_OF_ITS_OWN, IN_A_HELPER_OF_THE_MOVED_BODY], required: '...', files: FILES_THAT_TOUCH_THE_STOP };
/// ```
const THE_SERVICE_STOP: Requirement = Requirement {
    what: "the service's stop",
    named: STOP_NAMES,
    excused_by: &[IN_A_CHILD_OF_ITS_OWN, IN_A_HELPER_OF_THE_MOVED_BODY],
    required: "start the control's body in a child of its own first (`run_the_body_in_a_child_of_its_own`, \
               `testing/process_wide_state_child_test_support.rs`, issue 6530)",
    files: FILES_THAT_TOUCH_THE_STOP,
};

/// No control records, prepares or reads the service's stop in the run's own process.
/// A control that touches it starts a child of its own first.
#[test]
fn no_control_records_or_reads_the_service_stop_in_the_run_s_own_process() {
    let touch: Scan = scan(&THE_SERVICE_STOP).expect("the crate's sources");
    assert!(
        touch.findings.is_empty(),
        "{} is one set of statics for the whole process, and a test run holds its controls as threads of it, so a \
         control that records a stop refuses another control's work, and one that reads the state a process starts \
         with reads another control's record: {}:\n{}",
        THE_SERVICE_STOP.what,
        THE_SERVICE_STOP.required,
        touch.findings.join("\n")
    );
}

/// Every file whose controls touch the stop is still read,
/// so a run that finds none fails here instead of passing every tree.
#[test]
fn the_files_that_touch_the_stop_still_do() {
    let touch: Scan = scan(&THE_SERVICE_STOP).expect("the crate's sources");
    let missing: Vec<String> = missing_reader_files(&touch, &THE_SERVICE_STOP);
    assert!(
        missing.is_empty(),
        "these files hold no control that names {} any more, so this check guards nothing there: move each \
         entry to the file that holds such a control, or delete it:\n{}",
        THE_SERVICE_STOP.what,
        missing.join("\n")
    );
}

/// The scan finds a control that touches the stop in the run's own process,
/// and says nothing about the control beside it that starts a child first,
/// nor about a call a comment or a planted text only names.
#[test]
fn the_scan_finds_a_control_that_touches_the_stop_without_a_child_of_its_own() {
    let planted: &str = "\
#[test]\n\
fn records_in_the_run_s_own_process() {\n\
    let first = crate::stop_of_the_service_noticed_by_its_children::record_from_a_signal_handler(libc::SIGTERM);\n\
}\n\
\n\
#[test]\n\
fn records_in_a_child_of_its_own() {\n\
    /// The doc comment names prepare_to_notice( and it is no call.\n\
    run_the_body_in_a_child_of_its_own(module_path!(), \"records_in_a_child_of_its_own\", || {\n\
        crate::stop_of_the_service_noticed_by_its_children::prepare_to_notice().expect(\"the pipe\");\n\
    });\n\
}\n\
\n\
#[test]\n\
fn plants_the_name_in_a_text() {\n\
    let planted: &str = \"prepare_to_notice(); is_stopping()\";\n\
}\n";
    assert_eq!(
        findings_in_text(&THE_SERVICE_STOP, "planted_tests.rs", planted),
        vec![String::from(
            "planted_tests.rs:1 (records_in_the_run_s_own_process)"
        )],
        "the planted control that recorded in the run's own process was not the one finding"
    );
}
