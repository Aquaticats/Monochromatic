//! What:
//!  The bans on the two values the service client's controls write for the whole test process:
//!  the reach-service tests' stand-in for the manager,
//!  and the stop flag the client tests note.
//!  A control that writes either one starts a child of its own first.
//! Why:
//!  `service/service_client_reach_service_tests.rs` answers every control from one `static` stand-in,
//!  and `service/service_client_tests.rs` notes a stop in one `static` flag,
//!  which every wait resets when its own watch starts and ends.
//!  `cargo test` runs a crate's tests as threads of one process,
//!  so two controls of one threaded run wrote each other's unit state,
//!  read what the other asked,
//!  and wiped the other's stop:
//!  measured on 2026-11-26,
//!  nine of the reach-service controls failed in their own module's threaded run,
//!  and both stop controls failed in the client module's (issue 6522).
//!  A control that hands its body to `run_the_body_in_a_child_of_its_own`
//!  (`testing/process_wide_state_child_test_support.rs`)
//!  owns the state it changes.
//!  The interrupted module's own controls answer the stop note with their lock instead,
//!  so the stop ban names the note by the path the client tests reach it by
//!  (`crate::service_client_interrupted::stop_requested_for_test(`),
//!  and never the bare call those controls make through their own import.
//!  The scan that splits a source into its controls is
//!  `source_guard/process_wide_fact_readers_tests.rs`,
//!  which these bans share with the other bans of their shape.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // every control that names MANAGER or crate::service_client_interrupted::stop_requested_for_test(
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
///  The call that runs one control's body in a child process of the test binary.
/// Why:
///  A control that hands its body to it writes its state where no other control reads it,
///  so a control that writes one of these values and not this call is the finding.
const IN_A_CHILD_OF_ITS_OWN: &str = "run_the_body_in_a_child_of_its_own(";

/// What:
///  The reach-service controls' stand-in for the manager:
///  the one value `service_client_reach_service_tests.rs` answers every control from.
/// Why:
///  A control that writes it and reads it back asks about a value every thread of the run shares,
///  which is the state the starter takes out of the run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_STAND_IN = { what: "the manager stand-in of the reach-service controls", named: [['MANAGER', '...']], excusedBy: [IN_A_CHILD_OF_ITS_OWN], required: '...', files: [['service/service_client_reach_service_tests.rs', '...']] };
/// ```
const THE_STAND_IN: Requirement = Requirement {
    what: "the manager stand-in of the reach-service controls",
    named: &[(
        "MANAGER.lock(",
        "the one stand-in of the reach-service tests, taken through its lock",
    )],
    excused_by: &[IN_A_CHILD_OF_ITS_OWN],
    required: "start the control's body in a child of its own first (`run_the_body_in_a_child_of_its_own`, \
               `testing/process_wide_state_child_test_support.rs`, issue 6522)",
    files: &[(
        "service/service_client_reach_service_tests.rs",
        "each control reads and writes the one stand-in, so each must own its process",
    )],
};

/// What:
///  The stop flag the client controls note,
///  which the wait of every other control resets when its watch starts and ends.
/// Why:
///  A control that notes a stop and then waits reads a flag every thread of the run writes,
///  so its stop can be wiped by a control that only waits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_STOP_FLAG = { what: "the stop flag the client controls note", named: [['crate::service_client_interrupted::stop_requested_for_test(', '...']], excusedBy: [IN_A_CHILD_OF_ITS_OWN], required: '...', files: [['service/service_client_tests.rs', '...']] };
/// ```
const THE_STOP_FLAG: Requirement = Requirement {
    what: "the stop flag the client controls note",
    named: &[(
        "crate::service_client_interrupted::stop_requested_for_test(",
        "writes the one stop word of the process, which every wait resets",
    )],
    excused_by: &[IN_A_CHILD_OF_ITS_OWN],
    required: "start the control's body in a child of its own first (`run_the_body_in_a_child_of_its_own`, \
               `testing/process_wide_state_child_test_support.rs`, issue 6522)",
    files: &[(
        "service/service_client_tests.rs",
        "the two controls that note a stop while their client waits own a child each",
    )],
};

/// No control writes the manager stand-in in the run's own process.
#[test]
fn no_control_writes_the_manager_stand_in_in_the_run_s_own_process() {
    let read: Scan = scan(&THE_STAND_IN).expect("the crate's sources");
    assert!(
        read.findings.is_empty(),
        "{} is one value for the whole process, and a test run holds its controls as threads of it, so a control \
         that writes it answers another control's question with this control's word: {}:\n{}",
        THE_STAND_IN.what,
        THE_STAND_IN.required,
        read.findings.join("\n")
    );
}

/// Every file whose controls write the manager stand-in is still read,
/// so a run that finds no writer fails here instead of passing every tree.
#[test]
fn the_files_that_write_the_manager_stand_in_still_do() {
    let read: Scan = scan(&THE_STAND_IN).expect("the crate's sources");
    let missing: Vec<String> = missing_reader_files(&read, &THE_STAND_IN);
    assert!(
        missing.is_empty(),
        "these files hold no control that names {} any more, so this ban guards nothing there: move each entry \
         to the file that holds such a control, or delete it:\n{}",
        THE_STAND_IN.what,
        missing.join("\n")
    );
}

/// No control notes a stop in the run's own process.
#[test]
fn no_control_notes_a_stop_in_the_run_s_own_process() {
    let read: Scan = scan(&THE_STOP_FLAG).expect("the crate's sources");
    assert!(
        read.findings.is_empty(),
        "{} is one flag for the whole process, and every wait resets it, so a control that notes a stop beside \
         another control's wait loses it: {}:\n{}",
        THE_STOP_FLAG.what,
        THE_STOP_FLAG.required,
        read.findings.join("\n")
    );
}

/// Every file whose controls note a stop is still read,
/// so a run that finds no writer fails here instead of passing every tree.
#[test]
fn the_files_that_note_a_stop_still_do() {
    let read: Scan = scan(&THE_STOP_FLAG).expect("the crate's sources");
    let missing: Vec<String> = missing_reader_files(&read, &THE_STOP_FLAG);
    assert!(
        missing.is_empty(),
        "these files hold no control that names {} any more, so this ban guards nothing there: move each entry \
         to the file that holds such a control, or delete it:\n{}",
        THE_STOP_FLAG.what,
        missing.join("\n")
    );
}

/// The scan finds a control that writes either value in the run's own process,
/// and says nothing about the control beside it that starts a child first.
#[test]
fn the_scan_finds_a_control_that_writes_either_value_without_a_child() {
    let planted: &str = "#[test]\n\
fn writes_the_stand_in_in_the_run_s_own_process() {\n\
let _state = MANAGER.lock();\n\
}\n\
\n\
#[test]\n\
fn writes_the_stand_in_in_a_child_of_its_own() {\n\
run_the_body_in_a_child_of_its_own(module_path!(), \"writes_the_stand_in_in_a_child_of_its_own\", || {\n\
let _state = MANAGER.lock();\n\
});\n\
}\n\
\n\
#[test]\n\
fn notes_a_stop_in_the_run_s_own_process() {\n\
crate::service_client_interrupted::stop_requested_for_test(2, None);\n\
}\n\
\n\
#[test]\n\
fn notes_a_stop_in_a_child_of_its_own() {\n\
run_the_body_in_a_child_of_its_own(module_path!(), \"notes_a_stop_in_a_child_of_its_own\", || {\n\
crate::service_client_interrupted::stop_requested_for_test(2, None);\n\
});\n\
}\n";
    assert_eq!(
        findings_in_text(&THE_STAND_IN, "planted_tests.rs", planted),
        vec![String::from(
            "planted_tests.rs:1 (writes_the_stand_in_in_the_run_s_own_process)"
        )],
        "the planted control that writes the stand-in in the run's own process was not the one finding"
    );
    assert_eq!(
        findings_in_text(&THE_STOP_FLAG, "planted_tests.rs", planted),
        vec![String::from(
            "planted_tests.rs:13 (notes_a_stop_in_the_run_s_own_process)"
        )],
        "the planted control that notes a stop in the run's own process was not the one finding"
    );
}
