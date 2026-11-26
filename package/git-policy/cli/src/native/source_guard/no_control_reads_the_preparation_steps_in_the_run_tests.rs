//! What:
//!  The ban on a control reading the preparation's named steps in the run's own process:
//!  a control that asks about that ledger starts a child of its own first.
//! Why:
//!  `commit/prepare_work_spans.rs` keeps one `static` ledger for the whole process,
//!  which every preparation notes into and every timing record empties,
//!  and `cargo test` runs a crate's tests as threads of one process.
//!  So a control that asserts the steps its own preparation noted reads the steps of every other test
//!  that ran a preparation beside it,
//!  and a control that asserts an empty ledger reads whatever another test left there:
//!  measured on 2026-11-26,
//!  `commit::commit_index::tests` failed `every_selection_mode_states_its_one_read_and_its_seed_walk`
//!  with a second `index read` in both threaded runs (issue 6507),
//!  and one process holding a preparation's test and another's record failed the record's list.
//!  The container gate starts each test in a process of its own,
//!  so the gate cannot show it.
//!  A control that hands its body to `run_the_body_in_a_child_of_its_own` first asks its question in a child of the test
//!  binary,
//!  whose ledger holds nothing but what that control noted
//!  (`testing/process_wide_state_child_test_support.rs`).
//!  The scan that splits a source into its controls is
//!  `source_guard/process_wide_fact_readers_tests.rs`,
//!  which this ban shares with the other bans of its shape.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // every control that names drain_spans(, note_span(, prepare_work_spans:: or prepareSpans must also name
//! // run_the_body_in_a_child_of_its_own(.
//! ```

use super::process_wide_fact_readers_tests::{
    Requirement,
    Scan,
    findings_in_text,
    missing_reader_files,
    scan,
};

/// What:
///  `LEDGER_NAMES` is each call and each name that reaches the preparation's one ledger for the process,
///  with why that name is the ledger's.
///  Its siblings a reader might expect are a list of the two functions alone,
///  which misses an import of the module and the record's own field,
///  and one needle `prepare_work_spans`,
///  which misses a control whose file takes the two functions through `use super::*`.
/// Why:
///  A control that names the ledger asks a question about the whole process,
///  so it is the name a control reaches the ledger by that has to be found,
///  not one spelling of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const LEDGER_NAMES: ReadonlyArray<readonly [string, string]> = [['drain_spans(', '...'], ...];
/// ```
const LEDGER_NAMES: &[(&str, &str)] = &[
    ("drain_spans(", "empties the steps the whole process noted"),
    (
        "note_span(",
        "adds one step to the list the whole process noted",
    ),
    (
        "prepare_work_spans::",
        "names the module that keeps the one ledger of the process",
    ),
    (
        "prepareSpans",
        "names the record's field for the steps the whole process noted",
    ),
];

/// What:
///  The call that runs one control's body in a child process of the test binary.
/// Why:
///  A control that hands its body to it asks its question where nothing else noted a step,
///  so a control that names the ledger and not this call is the finding.
const IN_A_CHILD_OF_ITS_OWN: &str = "run_the_body_in_a_child_of_its_own(";

/// What:
///  `FILES_THAT_READ_THE_LEDGER` is each file whose controls still ask about the ledger,
///  with what its controls ask.
/// Why:
///  A scan that found no reader at all would pass every tree and guard nothing,
///  so a run that stops seeing these files fails here instead,
///  and a file that moves or drops its control shrinks this list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const FILES_THAT_READ_THE_LEDGER = [['commit/commit_index_tests.rs', 'one control counts the steps of one preparation'], ...];
/// ```
const FILES_THAT_READ_THE_LEDGER: &[(&str, &str)] = &[
    (
        "commit/commit_index_tests.rs",
        "one control counts the steps one preparation notes",
    ),
    (
        "commit/commit_timing_tests.rs",
        "one control states the steps its record holds",
    ),
    (
        "commit/prepare_work_spans_tests.rs",
        "three controls ask how the ledger adds, bounds and empties its steps",
    ),
];

/// What:
///  `THE_PREPARATION_STEPS` is the ban itself:
///  the ledger's names,
///  the one call that answers it,
///  what a control must do instead,
///  and the files that must still read it.
/// Why:
///  The scan in `process_wide_fact_readers_tests.rs` takes a requirement like this one,
///  so the ban states its own names and the scan stays the same for every ban of this shape.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_PREPARATION_STEPS = { what: "the preparation's named steps", named: LEDGER_NAMES, excusedBy: [IN_A_CHILD_OF_ITS_OWN], required: '...', files: FILES_THAT_READ_THE_LEDGER };
/// ```
const THE_PREPARATION_STEPS: Requirement = Requirement {
    what: "the preparation's named steps",
    named: LEDGER_NAMES,
    excused_by: &[IN_A_CHILD_OF_ITS_OWN],
    required: "start the control's body in a child of its own first (`run_the_body_in_a_child_of_its_own`, \
               `testing/process_wide_state_child_test_support.rs`, issue 6507)",
    files: FILES_THAT_READ_THE_LEDGER,
};

/// No control reads the preparation's steps in the run's own process.
/// A control that asks about them starts a child of its own first.
#[test]
fn no_control_reads_the_preparation_steps_in_the_run_s_own_process() {
    let read: Scan = scan(&THE_PREPARATION_STEPS).expect("the crate's sources");
    assert!(
        read.findings.is_empty(),
        "{} are one ledger for the whole process, and a test run holds its controls as threads of it, so a \
         control that asks about them reads another control's work: {}:\n{}",
        THE_PREPARATION_STEPS.what,
        THE_PREPARATION_STEPS.required,
        read.findings.join("\n")
    );
}

/// Every file whose controls ask about the ledger is still read,
/// so a run that finds no reader fails here instead of passing every tree.
#[test]
fn the_files_that_read_the_ledger_still_do() {
    let read: Scan = scan(&THE_PREPARATION_STEPS).expect("the crate's sources");
    let missing: Vec<String> = missing_reader_files(&read, &THE_PREPARATION_STEPS);
    assert!(
        missing.is_empty(),
        "these files hold no control that names {} any more, so this check guards nothing there: move each \
         entry to the file that holds such a control, or delete it:\n{}",
        THE_PREPARATION_STEPS.what,
        missing.join("\n")
    );
}

/// The scan finds a control that asks in the run's own process,
/// and says nothing about the control beside it that starts a child first,
/// nor about a call a comment or a planted text only names.
#[test]
fn the_scan_finds_a_control_that_asks_without_a_child_of_its_own() {
    let planted: &str = "\
#[test]\n\
fn asks_in_the_run_s_own_process() {\n\
    let taken = crate::prepare_work_spans::drain_spans();\n\
}\n\
\n\
#[test]\n\
fn asks_in_a_child_of_its_own() {\n\
    /// The doc comment names drain_spans( and it is no call.\n\
    run_the_body_in_a_child_of_its_own(module_path!(), \"asks_in_a_child_of_its_own\", || {\n\
        let taken = crate::prepare_work_spans::drain_spans();\n\
    });\n\
}\n\
\n\
#[test]\n\
fn plants_the_name_in_a_text() {\n\
    let planted: &str = \"drain_spans(); prepareSpans\";\n\
}\n";
    assert_eq!(
        findings_in_text(&THE_PREPARATION_STEPS, "planted_tests.rs", planted),
        vec![String::from(
            "planted_tests.rs:1 (asks_in_the_run_s_own_process)"
        )],
        "the planted control that asks in the run's own process was not the one finding"
    );
}
