//! What:
//!  The ban on a control starting the interactive selection in the run's own process:
//!  a control that starts it runs its body in a child of its own first.
//! Why:
//!  `git add --interactive` and `git add --patch` read the standard input of the process that starts them,
//!  by design:
//!  the person's terminal answers the selection
//!  (`commit/commit_index_selection.rs`,
//!  `prepare_interactive_selection`).
//!  `cargo test` runs a crate's tests as threads of one process,
//!  so that process's standard input is the terminal the run was started from:
//!  measured on 2026-11-26,
//!  `commit::commit_index::tests::interactive_selection_succeeds_without_changes` printed its `What now>` prompt,
//!  never ended,
//!  and the run's watchdog aborted the module run after 120 s with no `test result` line (issue 6510).
//!  A control that runs its body in `run_the_body_in_a_child_of_its_own` first starts the selection in a child
//!  of the test binary,
//!  whose standard input is the null device (`testing/process_wide_state_child_test_support.rs`),
//!  so the Git it starts reads no terminal,
//!  and ends at once instead of waiting for an answer.
//!  The container gate starts each test in a process of its own and gives it no terminal,
//!  so the gate cannot show the hang.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // every control that names prepare_interactive_selection( must also name run_the_body_in_a_child_of_its_own(.
//! ```

/// What:
///  `Scan` is what reading every test-only source of a tree answers,
///  `scan` reads the tree,
///  `controls` splits one source into its controls,
///  and `findings_of` names the controls that ask without a child.
///  They are shared with the guard over the preparation's ledger
///  (`controls_naming_a_call_tests.rs`).
/// Why:
///  This guard and that one ask the same question about different calls,
///  so the reading of a source,
///  its code places and its controls is written once.
use super::controls_naming_a_call_tests::{
    Scan,
    controls,
    findings_of,
    scan,
};
/// What:
///  `real_roots` answers the crate roots of this tree,
///  and `source_root` the folder they stand in.
/// Why:
///  The scan reads the crate's own sources,
///  so it needs where they start.
use crate::source_scan_tests::{
    real_roots,
    source_root,
};

/// What:
///  `INTERACTIVE_GIT_CALLS` is each entry point of this crate whose Git reads the standard input of the process
///  that calls it,
///  with why that entry point's Git reads it.
///  The siblings a reader might expect are the option words alone (`--interactive`,
///  `--patch`),
///  which a control also names while it parses a command line and starts no Git,
///  and a list of the two programs by name,
///  which names no call a control makes.
/// Why:
///  A control that names one of these asks a question of a process whose standard input is the run's,
///  so it is the call a control reaches an interactive Git by that has to be found,
///  not one spelling of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const INTERACTIVE_GIT_CALLS: ReadonlyArray<readonly [string, string]> = [['prepare_interactive_selection(', 'runs git add --interactive or --patch with the standard input of this process']];
/// ```
const INTERACTIVE_GIT_CALLS: &[(&str, &str)] = &[(
    "prepare_interactive_selection(",
    "runs `git add --interactive` or `--patch`, whose answers are the standard input of this process",
)];

/// What:
///  `FILES_THAT_START_THE_INTERACTIVE_SELECTION` is each file whose controls start the interactive selection,
///  with what its controls start it for.
/// Why:
///  A scan that found no caller at all would pass every tree and guard nothing,
///  so a run that stops seeing these files fails here instead,
///  and a file that moves or drops its control shrinks this list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const FILES_THAT_START_THE_INTERACTIVE_SELECTION = [['commit/commit_index_tests.rs', 'two controls start the selection, once over a clean index and once over a pathspec file that cannot be read']];
/// ```
const FILES_THAT_START_THE_INTERACTIVE_SELECTION: &[(&str, &str)] = &[(
    "commit/commit_index_tests.rs",
    "two controls start the selection, once over a clean index and once over a pathspec file that cannot be read",
)];

/// No control starts the interactive selection in the run's own process.
/// A control that starts it gives its Git a process whose standard input is not the run's.
#[test]
fn no_control_starts_the_interactive_selection_in_the_run_s_own_process() {
    // What: the whole crate's answer, which is a reason the tree could not be read when it is `Err`.
    // Why: a guard that could not read a source would pass a tree it never looked at.
    let read: Scan =
        scan(&source_root(), &real_roots(), INTERACTIVE_GIT_CALLS).expect("the crate's sources");
    assert!(
        read.findings.is_empty(),
        "the interactive selection's Git reads the standard input of the process that starts it, and a test run \
         holds its controls as threads of one process, whose standard input is the terminal the run was started \
         from, so a control that starts the selection hangs the whole run waiting for an answer: start the \
         control's body in a child of its own first (`run_the_body_in_a_child_of_its_own`, \
         `testing/process_wide_state_child_test_support.rs`, issue 6510):\n{}",
        read.findings.join("\n")
    );
}

/// Every file whose controls start the interactive selection is still read,
/// so a run that finds no caller fails here instead of passing every tree.
#[test]
fn the_files_that_start_the_interactive_selection_still_do() {
    let read: Scan =
        scan(&source_root(), &real_roots(), INTERACTIVE_GIT_CALLS).expect("the crate's sources");
    // What: the entries of the list above whose file no longer holds such a control, with why each was listed.
    // Why: an entry whose control moved or went guards nothing, and the failure has to say which file to look at.
    let missing: Vec<String> = FILES_THAT_START_THE_INTERACTIVE_SELECTION
        .iter()
        .filter(|(path, _why)| return !read.files_naming_a_call.contains(&String::from(*path)))
        .map(|(path, why)| return format!("{path}: {why}"))
        .collect();
    assert!(
        missing.is_empty(),
        "these files hold no control that starts the interactive selection any more, so this check guards nothing \
         there: move each entry to the file that holds such a control, or delete it:\n{}",
        missing.join("\n")
    );
}

/// The scan finds a control that starts the selection in the run's own process,
/// and says nothing about the control beside it that starts a child first,
/// nor about a call a comment or a planted text only names.
#[test]
fn the_scan_finds_a_control_that_starts_the_selection_without_a_child_of_its_own() {
    // What: a source holding three controls: one that starts the selection in the run's own process, one that
    //       starts a child first, and one that only writes the call's name into a string.
    // Why: the scan has to name the first and only the first, so a guard that excused it would fail here.
    let planted: &str = "\
#[test]\n\
fn starts_in_the_run_s_own_process() {\n\
    let started = prepare_interactive_selection(&context, &repo, &index, &objects, false, &choice);\n\
}\n\
\n\
#[test]\n\
fn starts_in_a_child_of_its_own() {\n\
    /// The doc comment names prepare_interactive_selection( and it is no call.\n\
    run_the_body_in_a_child_of_its_own(module_path!(), \"starts_in_a_child_of_its_own\", || {\n\
        let started = prepare_interactive_selection(&context, &repo, &index, &objects, false, &choice);\n\
    });\n\
}\n\
\n\
#[test]\n\
fn plants_the_name_in_a_text() {\n\
    let planted: &str = \"prepare_interactive_selection(\";\n\
}\n";
    assert_eq!(
        findings_of(
            "planted_tests.rs",
            controls(planted, INTERACTIVE_GIT_CALLS).as_slice()
        ),
        vec![String::from(
            "planted_tests.rs:1 (starts_in_the_run_s_own_process)"
        )],
        "the planted control that starts the selection in the run's own process was not the one finding"
    );
}
