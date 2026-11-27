//! What:
//!  The ban on a control that changes `SIGPIPE`'s action in the run's own process:
//!  a control that names one of these calls starts a child of its own first.
//! Why:
//!  A Rust program ignores `SIGPIPE` from its start,
//!  so the first write to a pipe nobody reads only answers `EPIPE`.
//!  Once the action is the default instead,
//!  that same write ends every thread of the run,
//!  and a threaded run of ten library modules ended with `signal: 13`
//!  after 2,985 of its 7,764 tests (issue 6604).
//!  `std` sets `SIGPIPE` back to its default in whatever process calls `exec`:
//!  a start that cannot run its program returns its error and leaves the default in place
//!  (`library/std/src/sys/process/unix/unix.rs:374`;
//!  `doc/troubleshooting/rust-exec-leaves-sigpipe-at-its-default-in-the-calling-process.md`).
//!  Measured on 2026-11-27 with a probe interposing on `sigaction` and `signal`:
//!  a threaded run of `invocation::forwarding::tests` and `push::start_retry::tests`
//!  read `SIG_IGN` at the process's start,
//!  then `SIG_DFL` on each of the two threads that ran a start that could not run its program,
//!  and never again `SIG_IGN`.
//!  The process-wide starter is `testing/process_wide_state_child_test_support.rs`,
//!  and a control that ends by a signal for its own assertion runs its body in a child through
//!  `testing/child_test_process.rs`.
//!  The scan that splits a source into its controls is
//!  `source_guard/process_wide_fact_readers_tests.rs`,
//!  which this ban shares with the other bans of its shape.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // every control that names replace_process_with_real_git(, exec_retrying(, end_as_git_ended( or print_summary(
//! // must also name run_the_body_in_a_child_of_its_own( or child_test_process::finished(.
//! ```

use super::process_wide_fact_readers_tests::{
    Requirement,
    Scan,
    findings_in_text,
    missing_reader_files,
    scan,
};

/// What:
///  `SIGPIPE_DEFAULT_NAMES` is each call that leaves `SIGPIPE` at its default in the calling process,
///  with why that name is the signal's.
///  Its siblings a reader might expect are the calls that only read the action
///  (`testing/signal_disposition_of_the_process.rs`) and the one that puts the ignore back,
///  which a control may call anywhere because they leave the process as they found it.
/// Why:
///  A control that names one of these asks a question about the whole process,
///  so it is each call that reaches the change that has to be found,
///  not one spelling of it:
///  a control that only runs the retrying start changes the same action as one that replaces the process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SIGPIPE_DEFAULT_NAMES: ReadonlyArray<readonly [string, string]> = [['replace_process_with_real_git(', '...'], ...];
/// ```
const SIGPIPE_DEFAULT_NAMES: &[(&str, &str)] = &[
    (
        "replace_process_with_real_git(",
        "replaces this process with Git, and a start that cannot run it leaves this process's `SIGPIPE` at its \
         default",
    ),
    (
        "exec_retrying(",
        "replaces this process with its program, and a start that cannot run it leaves this process's `SIGPIPE` at \
         its default",
    ),
    (
        "end_as_git_ended(",
        "puts back a signal's default action and raises it, which ends every thread of this process",
    ),
    (
        "print_summary(",
        "writes a landed commit's summary and ends this process by `SIGPIPE` when the stream has no reader",
    ),
];

/// What:
///  The call that runs one control's body in a child process of the test binary.
/// Why:
///  A control that hands its body to it asks its question where its own change reaches nothing,
///  so a control that names one of the calls above and not this one is the finding.
const IN_A_CHILD_OF_ITS_OWN: &str = "run_the_body_in_a_child_of_its_own(";

/// What:
///  The call that starts a child of the test binary for a control whose own assertion is a process's ending,
///  such as the death by a signal.
///  The sibling a reader might expect is the starter above,
///  which runs the same control's body in a child instead of a body written for one child.
/// Why:
///  `invocation/forwarding_tests.rs` and `push/push_stacking_tests.rs` keep the body of their death controls
///  behind one variable and judge the child's ending,
///  so those controls own a process of their own without naming the starter.
const IN_A_CHILD_WHOSE_ENDING_IS_THE_ASSERTION: &str = "child_test_process::finished(";

/// What:
///  `FILES_THAT_LEAVE_SIGPIPE_AT_ITS_DEFAULT` is each file whose controls run one of the calls above,
///  with what its controls ask.
/// Why:
///  A scan that found no such control at all would pass every tree and guard nothing,
///  so a run that stops seeing these files fails here instead,
///  and a file that moves or drops its control shrinks this list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const FILES_THAT_LEAVE_SIGPIPE_AT_ITS_DEFAULT = [['invocation/forwarding_tests.rs', '...'], ...];
/// ```
const FILES_THAT_LEAVE_SIGPIPE_AT_ITS_DEFAULT: &[(&str, &str)] = &[
    (
        "invocation/forwarding_tests.rs",
        "three controls replace the process, read what that leaves or end by a signal, each in a child of its own",
    ),
    (
        "push/start_retry_tests.rs",
        "one control runs a replacement that cannot run its program, in a child of its own",
    ),
];

/// What:
///  `SIGPIPE_AT_ITS_DEFAULT` is the ban itself:
///  the calls that leave the action at its default,
///  the calls that answer it,
///  what a control must do instead,
///  and the files that must still name one.
/// Why:
///  The scan in `process_wide_fact_readers_tests.rs` takes a requirement like this one,
///  so the ban states its own names and the scan stays the same for every ban of this shape.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SIGPIPE_AT_ITS_DEFAULT = { what: "the action in place for SIGPIPE", named: SIGPIPE_DEFAULT_NAMES, excusedBy: [IN_A_CHILD_OF_ITS_OWN, IN_A_CHILD_WHOSE_ENDING_IS_THE_ASSERTION], required: '...', files: FILES_THAT_LEAVE_SIGPIPE_AT_ITS_DEFAULT };
/// ```
const SIGPIPE_AT_ITS_DEFAULT: Requirement = Requirement {
    what: "the action in place for `SIGPIPE`",
    named: SIGPIPE_DEFAULT_NAMES,
    excused_by: &[
        IN_A_CHILD_OF_ITS_OWN,
        IN_A_CHILD_WHOSE_ENDING_IS_THE_ASSERTION,
    ],
    required: "start the control's body in a child of its own first (`run_the_body_in_a_child_of_its_own`, \
               `testing/process_wide_state_child_test_support.rs`, issue 6604)",
    files: FILES_THAT_LEAVE_SIGPIPE_AT_ITS_DEFAULT,
};

/// No control leaves `SIGPIPE` at its default in the run's own process.
/// A control that runs such a call starts a child of its own first.
#[test]
fn no_control_leaves_sigpipe_at_its_default_in_the_run_s_own_process() {
    let touch: Scan = scan(&SIGPIPE_AT_ITS_DEFAULT).expect("the crate's sources");
    assert!(
        touch.findings.is_empty(),
        "an action in place for a signal belongs to the whole process, and a test run holds its controls as threads \
         of it, so one control that leaves `SIGPIPE` at its default arms every other thread's next write to a pipe \
         nobody reads, which ends the run: {}:\n{}",
        SIGPIPE_AT_ITS_DEFAULT.required,
        touch.findings.join("\n")
    );
}

/// Every file whose controls run such a call is still read,
/// so a run that finds none fails here instead of passing every tree.
#[test]
fn the_files_that_leave_sigpipe_at_its_default_still_do() {
    let touch: Scan = scan(&SIGPIPE_AT_ITS_DEFAULT).expect("the crate's sources");
    let missing: Vec<String> = missing_reader_files(&touch, &SIGPIPE_AT_ITS_DEFAULT);
    assert!(
        missing.is_empty(),
        "these files hold no control that names {} any more, so this check guards nothing there: move each entry \
         to the file that holds such a control, or delete it:\n{}",
        SIGPIPE_AT_ITS_DEFAULT.what,
        missing.join("\n")
    );
}

/// The scan finds a control that replaces the process in the run's own process,
/// and says nothing about the control beside it that starts a child first,
/// nor about a name a comment or a planted text only holds.
#[test]
fn the_scan_finds_a_control_that_replaces_the_process_without_a_child_of_its_own() {
    let planted: &str = "\
#[test]\n\
fn replaces_in_the_run_s_own_process() {\n\
    let _ending = crate::forwarding::replace_process_with_real_git(missing, &[], &[]);\n\
}\n\
\n\
#[test]\n\
fn replaces_in_a_child_of_its_own() {\n\
    /// The doc comment names exec_retrying( and it is no call.\n\
    run_the_body_in_a_child_of_its_own(module_path!(), \"replaces_in_a_child_of_its_own\", || {\n\
        let _ending = crate::forwarding::end_as_git_ended(libc::SIGTERM);\n\
    });\n\
}\n\
\n\
#[test]\n\
fn plants_the_name_in_a_text() {\n\
    let planted: &str = \"print_summary(); exec_retrying()\";\n\
}\n";
    assert_eq!(
        findings_in_text(&SIGPIPE_AT_ITS_DEFAULT, "planted_tests.rs", planted),
        vec![String::from(
            "planted_tests.rs:1 (replaces_in_the_run_s_own_process)"
        )],
        "the planted control that replaced the process in the run's own process was not the one finding"
    );
}
