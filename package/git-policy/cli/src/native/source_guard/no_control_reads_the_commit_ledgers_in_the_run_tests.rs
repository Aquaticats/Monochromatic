//! What:
//!  The ban on a control reading one of a commit's ledgers in the run's own process:
//!  a control that counts the figures of its own commit starts a child of its own first.
//! Why:
//!  Four ledgers keep a commit's figures until its timing record takes them,
//!  and each is one list for the whole process:
//!  `commit/commit_index_builds.rs` (how each build of the private index was made),
//!  `commit/commit_prepare_part_durations.rs` and `commit/commit_build_part_durations.rs`
//!  (the stretches of a preparation and of a build),
//!  and `transaction/transaction_step_durations.rs` (the Git steps a stretch takes out of itself).
//!  Every commit notes into them and every timing record empties them,
//!  and `cargo test` runs a crate's tests as threads of one process.
//!  So a control that asserts the figures its own commit noted reads the figures of every other test
//!  that ran a commit beside it,
//!  and a control that asserts an empty ledger reads whatever another test left there:
//!  measured on 2026-11-26,
//!  two threaded runs of `commit::` failed the same ten controls
//!  (issue 6509),
//!  `commit_index_builds::tests` read another control's builds in five of its seven failures,
//!  and one read an empty list because another control's record had taken its build.
//!  The container gate starts each test in a process of its own,
//!  so the gate cannot show it.
//!  A control that starts `started_within_a_child_of_its_own` first asks its question in a child of the test
//!  binary,
//!  whose ledgers hold nothing but what that control noted
//!  (`testing/process_wide_state_child_test_support.rs`).
//!  The scan that splits a source into its controls is
//!  `source_guard/process_wide_fact_readers_tests.rs`,
//!  which this ban shares with the other bans of its shape.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // every control that names drain_index_builds(, note_index_build(, commit_index_builds::,
//! // drain_prepare_parts(, note_prepare_part(, drain_build_parts(, note_build_part(, drain_steps( or note_step(
//! // must also name started_within_a_child_of_its_own(.
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
///  A control that makes it asks its question where no other test noted a figure,
///  so a control that names a ledger and not this call is the finding.
const IN_A_CHILD_OF_ITS_OWN: &str = "started_within_a_child_of_its_own(";

/// What:
///  `WHAT_A_CONTROL_MUST_DO` is the answer every one of these four bans gives:
///  the same call,
///  for the same reason.
/// Why:
///  The four ledgers are read into one timing record,
///  so the four bans differ only in their names and not in what they ask a control to do.
const WHAT_A_CONTROL_MUST_DO: &str = "start the control's body in a child of its own first \
                                       (`started_within_a_child_of_its_own`, \
                                       `testing/process_wide_state_child_test_support.rs`, issue 6509)";

/// What:
///  `THE_INDEX_BUILDS` is the ban on counting the builds of the private index in the run's own process.
/// Why:
///  Every `initialize_commit_index` and `initialize_retry_index` notes a build,
///  and every timing record takes them all.
const THE_INDEX_BUILDS: Requirement = Requirement {
    what: "the builds of a commit's private index",
    named: &[
        (
            "drain_index_builds(",
            "takes the builds the whole process noted",
        ),
        (
            "note_index_build(",
            "adds one build to the list the whole process noted",
        ),
        (
            "commit_index_builds::",
            "names the module that keeps the one list of the process",
        ),
    ],
    excused_by: &[IN_A_CHILD_OF_ITS_OWN],
    required: WHAT_A_CONTROL_MUST_DO,
    files: &[
        (
            "commit/commit_index_builds_tests.rs",
            "eight controls count the builds of one command",
        ),
        (
            "commit/commit_timing_tests.rs",
            "one control states the builds its record holds",
        ),
    ],
};

/// What:
///  `THE_PREPARE_PARTS` is the ban on counting a preparation's stretches in the run's own process.
/// Why:
///  Every seam of a preparation notes a stretch,
///  and every timing record takes them all.
const THE_PREPARE_PARTS: Requirement = Requirement {
    what: "the stretches of a commit's preparation",
    named: &[
        (
            "drain_prepare_parts(",
            "takes the stretches the whole process noted",
        ),
        (
            "note_prepare_part(",
            "adds one stretch to the list the whole process noted",
        ),
    ],
    excused_by: &[IN_A_CHILD_OF_ITS_OWN],
    required: WHAT_A_CONTROL_MUST_DO,
    files: &[(
        "commit/commit_prepare_part_durations_tests.rs",
        "five controls ask how the list adds, bounds, empties and writes its stretches",
    )],
};

/// What:
///  `THE_BUILD_PARTS` is the ban on counting a build's stretches in the run's own process.
/// Why:
///  Every seam of a build notes a stretch,
///  and every timing record takes them all.
const THE_BUILD_PARTS: Requirement = Requirement {
    what: "the stretches of a commit's build",
    named: &[
        (
            "drain_build_parts(",
            "takes the stretches the whole process noted",
        ),
        (
            "note_build_part(",
            "adds one stretch to the list the whole process noted",
        ),
    ],
    excused_by: &[IN_A_CHILD_OF_ITS_OWN],
    required: WHAT_A_CONTROL_MUST_DO,
    files: &[
        (
            "commit/commit_build_part_durations_tests.rs",
            "five controls ask how the list adds, bounds, empties and writes its stretches",
        ),
        (
            "shadow/shadow_pool_tests.rs",
            "one control states the refusal a member of the pool noted",
        ),
    ],
};

/// What:
///  `THE_STEPS` is the ban on counting the Git steps of the run's own process.
/// Why:
///  Every Git step of the wrapper notes itself,
///  and `step_time_between` reads the whole process's steps to take them out of a stretch,
///  so a control that notes one step to measure that subtraction reads every other test's steps.
const THE_STEPS: Requirement = Requirement {
    what: "the Git steps a commit's stretches take out of themselves",
    named: &[
        ("drain_steps(", "takes the steps the whole process noted"),
        (
            "note_step(",
            "adds one step to the list the whole process noted",
        ),
    ],
    excused_by: &[IN_A_CHILD_OF_ITS_OWN],
    required: WHAT_A_CONTROL_MUST_DO,
    files: &[
        (
            "commit/commit_build_part_durations_tests.rs",
            "one control measures the step a stretch took out of itself",
        ),
        (
            "commit/commit_prepare_part_durations_tests.rs",
            "one control measures the step a stretch took out of itself",
        ),
        (
            "repo_read/git_metadata_tests.rs",
            "one control states the step a declined in-process read ran",
        ),
        (
            "transaction/transaction_step_durations_tests.rs",
            "five controls ask how the list orders, bounds, names and writes its steps",
        ),
    ],
};

/// Every ledger of a commit is one list for the whole process.
/// A control that counts its own command's figures starts a child of its own first.
#[test]
fn no_control_reads_a_commit_ledger_in_the_run_s_own_process() {
    for requirement in [
        &THE_INDEX_BUILDS,
        &THE_PREPARE_PARTS,
        &THE_BUILD_PARTS,
        &THE_STEPS,
    ] {
        let read: Scan = scan(requirement).expect("the crate's sources");
        assert!(
            read.findings.is_empty(),
            "{} are one ledger for the whole process, and a test run holds its controls as threads of it, so \
             a control that asks about them reads another control's work: {}:\n{}",
            requirement.what,
            requirement.required,
            read.findings.join("\n")
        );
    }
}

/// Every file whose controls ask about a ledger is still read,
/// so a run that finds no reader fails here instead of passing every tree.
#[test]
fn the_files_that_read_each_ledger_still_do() {
    for requirement in [
        &THE_INDEX_BUILDS,
        &THE_PREPARE_PARTS,
        &THE_BUILD_PARTS,
        &THE_STEPS,
    ] {
        let read: Scan = scan(requirement).expect("the crate's sources");
        let missing: Vec<String> = missing_reader_files(&read, requirement);
        assert!(
            missing.is_empty(),
            "these files hold no control that names {} any more, so this check guards nothing there: move \
             each entry to the file that holds such a control, or delete it:\n{}",
            requirement.what,
            missing.join("\n")
        );
    }
}

/// The scan finds a control that asks in the run's own process for each of the four ledgers,
/// and says nothing about the control beside them that starts a child first,
/// nor about a call a comment or a planted text only names.
#[test]
fn the_scan_finds_a_control_that_asks_without_a_child_of_its_own() {
    let planted: &str = "\
#[test]\n\
fn asks_for_the_index_builds() {\n\
    let taken = crate::commit_index_builds::drain_index_builds();\n\
}\n\
\n\
#[test]\n\
fn asks_for_the_prepare_parts() {\n\
    let taken = drain_prepare_parts();\n\
}\n\
\n\
#[test]\n\
fn asks_for_the_build_parts() {\n\
    let taken = drain_build_parts();\n\
}\n\
\n\
#[test]\n\
fn asks_for_the_steps() {\n\
    let taken = drain_steps(Instant::now());\n\
}\n\
\n\
#[test]\n\
fn asks_in_a_child_of_its_own() {\n\
    /// The doc comment names drain_build_parts( and it is no call.\n\
    if started_within_a_child_of_its_own(module_path!(), \"asks_in_a_child_of_its_own\") {\n\
        return;\n\
    }\n\
    let taken = crate::commit_build_part_durations::drain_build_parts();\n\
}\n\
\n\
#[test]\n\
fn plants_the_names_in_a_text() {\n\
    let planted: &str = \"drain_index_builds(); note_prepare_part(\";\n\
}\n";
    assert_eq!(
        findings_in_text(&THE_INDEX_BUILDS, "planted_tests.rs", planted),
        vec![String::from(
            "planted_tests.rs:1 (asks_for_the_index_builds)"
        )],
        "the planted control that counts builds in the run's own process was not the one finding"
    );
    assert_eq!(
        findings_in_text(&THE_PREPARE_PARTS, "planted_tests.rs", planted),
        vec![String::from(
            "planted_tests.rs:6 (asks_for_the_prepare_parts)"
        )],
        "the planted control that counts a preparation's stretches was not the one finding"
    );
    assert_eq!(
        findings_in_text(&THE_BUILD_PARTS, "planted_tests.rs", planted),
        vec![String::from(
            "planted_tests.rs:11 (asks_for_the_build_parts)"
        )],
        "the planted control that counts a build's stretches was not the one finding"
    );
    assert_eq!(
        findings_in_text(&THE_STEPS, "planted_tests.rs", planted),
        vec![String::from("planted_tests.rs:16 (asks_for_the_steps)")],
        "the planted control that counts the run's steps was not the one finding"
    );
}
