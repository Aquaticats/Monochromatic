//! What:
//!  The one way a control whose state belongs to the whole process runs its body in a child of this test binary,
//!  so the state it sets reaches no other control of the run.
//! Why:
//!  `cargo test --lib` runs a crate's tests as threads of one process,
//!  and the container gate runs each test in a process of its own (nextest),
//!  so a control that sets process-wide state passes the gate and fails a threaded run:
//!  measured on 2026-11-25,
//!  `cargo test --lib foundation::` failed 16 tests and then deadlocked,
//!  and each of them passed alone (issue 6487).
//!  The child runs one test and ends with it,
//!  so what the control sets reaches nothing else,
//!  and the setters of that state refuse in any other process,
//!  naming the starter below.
//!  A child makes its own fixtures:
//!  unlike the descriptor-limit child (`descriptor_limit_child_test_support.rs`),
//!  nothing here counts this process's open descriptors,
//!  so the fixture the watchdog arms while it is made costs
//!  nothing a control measures.
//! Gotcha:
//!  The variable names the control it started,
//!  and a child of that child inherits it:
//!  a grandchild is inside a process of its own too,
//!  which is exactly what a control that sets this state needs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // in the parent: runTheBodyInAChildOfItsOwn(import.meta.url, 'the_control', () => { /* ...the body... */ });
//! // in the child:  the same body runs here, free to set what belongs to this process.
//! ```

/// What:
///  `ChildTest` is one request for a child of this test binary,
///  and `passed` starts it,
///  waits for it with a deadline
///  and checks that the one test it ran passed.
/// Why:
///  Every child of this crate's test binary is started in one place (`child_test_process.rs`,
///  debt ledger item 10),
///  so this file states a request and never builds a command of its own.
use super::child_test_process::{
    ChildTest,
    passed,
};
/// What:
///  `OsStr` is borrowed operating-system text of raw bytes,
///  whose sibling is `String`,
///  an owned UTF-8 buffer.
/// Why:
///  An environment value is bytes,
///  and this file hands the child the path of the one test it runs as one.
use std::ffi::OsStr;

/// The variable whose value marks the child this file started,
/// and names the control that child runs,
/// which is the only process a control may give this process room or a bound in.
const CHILD_VARIABLE: &str = "CLI_GIT_PROCESS_WIDE_STATE_CHILD";

/// What:
///  Whether this process is a child this file started,
///  which is the only process a control may set state that belongs to the whole process in.
///  `mod tests` is a child module and reaches it; nothing outside this file does,
///  so no control can ask the question instead of handing the starter its body.
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
fn this_process_is_the_child() -> bool {
    return std::env::var_os(CHILD_VARIABLE).is_some();
}

/// What:
///  Refuse to set `state`,
///  a fact that belongs to the whole process,
///  in a process that is not a child this file
///  started,
///  naming the starter that makes the calling process one.
/// Why:
///  A setter that changes this process's room or its bound calls this before it changes anything,
///  so a control that forgot the child fails at its own first line,
///  every time it runs,
///  instead of making the rest of a threaded run fail or deadlock now and then.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refuseOutsideTheChild(state: string): void { assert(thisProcessIsTheChild(), `${state} belongs to the whole process: set it in a child of the control's own`); }
/// ```
pub(crate) fn refuse_outside_the_child(state: &str) {
    assert!(
        this_process_is_the_child(),
        "{state} belongs to the whole process, and `cargo test` runs a crate's tests as threads of one process, so \
         setting it here reaches every other test of that run: set it in a child of the control's own, which \
         `crate::process_wide_state_child_test_support::run_the_body_in_a_child_of_its_own` starts"
    );
}

/// What:
///  Run `body` in a child of this test binary that holds one test,
///  `name` of `module`,
///  and fail this call with the child's own words when that child's one test did not pass.
///  `module` is `module_path!()` of the test,
///  `name` its function name,
///  and `body` what the test does.
/// Why:
///  A runtime, the room a process gives blocking jobs, the bound on them and a hold every read of the process consults
///  all belong to the whole process,
///  so a control that sets one where the run holds them fails or deadlocks the rest of that run (issue 6487).
///  The child is a process of its own,
///  so what the body sets reaches nothing else,
///  and its one passing test is this call's whole assertion.
///  The body travels as a closure the parent never calls,
///  so no caller holds an answer it could read inverted:
///  while this file answered a yes or a no,
///  a control that read the answer inverted ran its body in the parent,
///  the process the body was written to keep its state out of (issue 6503).
/// Gotcha:
///  Everything the body needs is made inside it,
///  because nothing before this call runs in the child.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runTheBodyInAChildOfItsOwn(module: string, name: string, body: () => void): void {
///   if (thisProcessIsTheChild()) { body(); return; }
///   passed({ exactTest: `${module.split('::').slice(1).join('::')}::${name}`, environment: [[CHILD_VARIABLE, thePath]] });
/// }
/// ```
pub(crate) fn run_the_body_in_a_child_of_its_own(
    module: &str,
    name: &str,
    body: impl FnOnce(),
) {
    // What:     the child is the process under the control's own state, so the body runs here and its panic is this
    //           child's ending.
    // Why:      the parent never runs it, which is the whole reason the body travels as a closure.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (thisProcessIsTheChild()) { body(); return; }
    // ```
    if this_process_is_the_child() {
        body();
        return;
    }
    // What:     the path libtest answers to: the module path without the crate's own first segment, and the test's
    //           name.
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
    // What:     the one variable the child gets, whose value is the path of the test it runs.
    // Why:      the child has to learn that it is a child with a process of its own, and which control it is there
    //           for, before it starts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const environment: [string, string][] = [[CHILD_VARIABLE, path]];
    // ```
    let environment: [(&str, &OsStr); 1] = [(CHILD_VARIABLE, OsStr::new(path.as_str()))];
    // What:     the one call that starts the child: `ChildTest` is the request, and `..ChildTest::default()` takes
    //           every field the request leaves out from the plain request.
    // Why:      the child's own words reach the control that started it when its one test does not pass.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // passed({ exactTest: path, environment, ...plainRequest() });
    // ```
    // The parent's whole assertion is this child's result, and a child that ran no test fails here.
    let _output = passed(ChildTest {
        exact_test: path.as_str(),
        environment: environment.as_slice(),
        ..ChildTest::default()
    });
}

/// What:
///  Hold this process's blocking jobs to at most `places` at once,
///  for a control whose subject needs a bound small enough to see.
/// Why:
///  The bound belongs to the whole process and is made on the first job that asks for a place,
///  so a control that holds it here and does not own the process decides what every other test of the run gets;
///  this states the child's ownership once,
///  and names the setter it stands in front of.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function holdTheBlockingJobsTo(places: number): void { refuseOutsideTheChild('the bound on blocking jobs'); holdBlockingJobsTo(places); }
/// ```
pub(crate) fn hold_the_blocking_jobs_to(places: usize) {
    refuse_outside_the_child("the bound on blocking jobs");
    crate::blocking_jobs_within_the_budget::hold_blocking_jobs_to(places);
}

/// The controls of the refusal and of the child's own answer;
/// never part of the release executable.
#[cfg(test)]
#[path = "process_wide_state_child_test_support_tests.rs"]
mod tests;
