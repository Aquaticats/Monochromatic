//! What:
//!  A control's own body run in a child process of this test binary,
//!  which lowers its own descriptor limit.
//! Why:
//!  A soft descriptor limit belongs to the whole process,
//!  and `cargo test` runs a crate's tests as threads of one process,
//!  so a control that lowered it in the process the run holds
//!  made every other test of that run fail with `EMFILE` (`Too many open files`) on its next open,
//!  for a reason that had nothing to do with what it tested (issue 6492).
//!  The child runs one test and ends with it,
//!  so the limit reaches nothing else,
//!  and `set_soft_limit` refuses to lower the limit in any other process,
//!  naming the starter below.
//!  A child whose control works in a fixture takes the directory from `child_directory` rather than `fixture`,
//!  because a fixture arms the run's watchdog,
//!  and the thread it starts opens descriptors of `/proc` between the count a control reads and the work it measures,
//!  which left a walk one descriptor short in the container gate (issue 6478).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // in the parent: startedThisTestInAChild(modulePath, 'the_control', fixture('some-fixture')); return;
//! // in the child:  const root = childDirectory(); // ...the measured work, under this process's own limit...
//! ```

/// What:
///  The request for one child of this test binary,
///  and the check that the one test it ran passed.
///  `ChildTest` is that request,
///  and `passed` starts it,
///  waits for it with a deadline and reads what it said.
/// Why:
///  Every child of this crate's test binary is started in one place (`child_test_process.rs`),
///  so the process is owned and its wait is bounded,
///  and its own failure reaches the control that started it,
///  with the child's own words.
use super::child_test_process::{
    ChildTest,
    passed,
};
/// What:
///  `OsStr` is borrowed operating-system text of raw bytes,
///  whose sibling is `String`,
///  an owned UTF-8 buffer.
/// Why:
///  The fixture directory travels to the child in the environment,
///  which holds bytes and not necessarily UTF-8.
use std::ffi::OsStr;
/// What:
///  `Path` is a borrowed filesystem path and `PathBuf` an owned one,
///  whose siblings are `&str` and `String`,
///  which are text.
/// Why:
///  The parent names the directory it made,
///  and the child answers it as a path of its own.
use std::path::{
    Path,
    PathBuf,
};

/// The variable whose presence marks the child this file starts,
/// which is the only process a control may lower a descriptor limit in.
const CHILD_VARIABLE: &str = "CLI_GIT_DESCRIPTOR_LIMIT_CHILD";

/// The variable that carries the fixture directory the child works in,
/// set only by a control that works in one.
const CHILD_DIRECTORY_VARIABLE: &str = "CLI_GIT_DESCRIPTOR_LIMIT_CHILD_DIRECTORY";

/// What:
///  Whether this process is a child this file started,
///  which is the only process a control may lower a descriptor limit in.
/// Why:
///  The refusal and the control's own body ask this one question,
///  so a process is the child,
///  or is not,
///  by one variable read in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function thisProcessIsTheChild(): boolean { return process.env[CHILD_VARIABLE] !== undefined; }
/// ```
pub(crate) fn this_process_is_the_child() -> bool {
    return std::env::var_os(CHILD_VARIABLE).is_some();
}

/// What:
///  Refuse to lower a soft descriptor limit in a process that is not a child this file started,
///  naming the starter that makes the calling process one.
/// Why:
///  `set_soft_limit` calls this before it lowers anything,
///  so a control that forgot the child fails at its own first line,
///  every time it runs,
///  instead of making the rest of a threaded run fail with `EMFILE` now and then.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refuseOutsideTheChild(): void { assert(thisProcessIsTheChild(), 'lower it in a child of the control\'s own, which started_this_test_in_a_child starts'); }
/// ```
pub(crate) fn refuse_outside_the_child() {
    assert!(
        this_process_is_the_child(),
        "the soft descriptor limit belongs to the whole process, and `cargo test` runs a crate's tests as threads of \
         one process, so lowering it here fails every other test of this run with `EMFILE`: lower it in a child of \
         the control's own, which `crate::descriptor_limit_child_test_support::started_this_test_in_a_child` \
         starts"
    );
}

/// What:
///  The fixture directory the parent made for this child,
///  in the child,
///  and nothing in the process that started it.
/// Why:
///  The body of a control is one function in both processes,
///  so this answer is what hands the child its fixture;
///  taking the directory this way keeps the watchdog,
///  and the descriptors its looks open,
///  out of the process whose room is measured.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function childDirectory(): string | undefined { return process.env[CHILD_DIRECTORY_VARIABLE]; }
/// ```
pub(crate) fn child_directory() -> Option<PathBuf> {
    return std::env::var_os(CHILD_DIRECTORY_VARIABLE).map(PathBuf::from);
}

/// What:
///  Start this same test in a child process of the test binary,
///  working in `directory` when the control works in a fixture,
///  and fail this control with everything the child said when its one test did not pass.
/// Why:
///  The descriptor limit the child lowers then belongs to a process of its own,
///  and a child that panics,
///  hangs or runs no test,
///  fails here with its own words
///  instead of leaving the parent's other tests to fail with `EMFILE`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startedThisTestInAChild(module: string, name: string, directory?: string): void {
///   passed({ exactTest: `${module.split('::').slice(1).join('::')}::${name}`, environment: [[CHILD_VARIABLE, '1'], ...(directory === undefined ? [] : [[CHILD_DIRECTORY_VARIABLE, directory]])] });
/// }
/// ```
pub(crate) fn started_this_test_in_a_child(
    module: &str,
    name: &str,
    directory: Option<&Path>,
) {
    // What:     a growable list of the child's environment, each entry a variable name and the value it holds.
    //           A fixed-length array (`[(&str, &OsStr); 2]`) would need a value for the entry a control without a
    //           fixture never sets.
    // Why:      the child has to learn that it is the child, and which fixture is its own, before it starts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const environment: [string, string][] = [[CHILD_VARIABLE, '1']];
    // ```
    let mut environment: Vec<(&str, &OsStr)> = vec![(CHILD_VARIABLE, OsStr::new("1"))];
    if let Some(fixture) = directory {
        environment.push((CHILD_DIRECTORY_VARIABLE, fixture.as_os_str()));
    }
    // What:     the path libtest answers to: the module path without the crate's own first segment, and the test's name.
    // Why:      `--exact` matches this text and nothing else, so a path that is wrong would run no test at all.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const path = `${module.split('::').slice(1).join('::')}::${name}`;
    // ```
    let path: String = format!(
        "{}::{name}",
        module.split_once("::").map_or(module, |(_, rest)| {
            return rest;
        })
    );
    // What:     the one call that starts the child: `ChildTest` is the request, its two streams are pipes this call
    //           reads, and `..ChildTest::default()` takes every field the request leaves out from the plain request.
    // Why:      the child's own words reach the control that started it when its one test does not pass.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // passed({ exactTest: path, environment, ...plainRequest() });
    // ```
    passed(ChildTest {
        exact_test: path.as_str(),
        environment: environment.as_slice(),
        ..ChildTest::default()
    });
}
