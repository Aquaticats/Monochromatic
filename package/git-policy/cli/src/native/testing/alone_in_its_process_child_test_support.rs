//! What:
//!  A control's own body run in a child process of this test binary,
//!  which holds that one test and no other,
//!  and the refusal that keeps a control from arming a seam of the whole process anywhere else.
//! Why:
//!  `cargo test --lib` runs a module's tests as threads of one process,
//!  so what a control shares with the tests beside it is the state of the whole process:
//!  the hold the size read's own thread waits at,
//!  the note the last failed Git step leaves,
//!  the repository whose record every Git start is noted in,
//!  the one add a process records,
//!  the runtime the process built,
//!  and the environment the process was started with.
//!  A control that armed or read such state beside its module's other tests changed their run and read theirs,
//!  so the module failed in a different set of tests on each run
//!  and every one of them passed alone (issue 6493).
//!  The child runs one test and ends with it,
//!  so the state that test touches is its own,
//!  and the parent's assertion is the child's one result,
//!  read with a deadline by `child_test_process::passed`.
//! Gotcha:
//!  The child is a second start of the same test,
//!  so the body has to be the whole of what the test does and the parent has to return before it:
//!  a fixture the parent made would be made twice,
//!  and the child's fixture root is the one named for the child's own process number.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // in the parent: if (ranAloneInAChildOfItsOwn(import.meta.url, 'the_control')) return;
//! // in the child:  // ...the body, the only test this process runs...
//! ```

/// What:
///  The request for a child of this test binary, and the check that the one test it ran passed.
/// Why:
///  Every child of this crate's test binary is started in one place (`child_test_process.rs`),
///  so the process is owned by the test thread and its wait has a deadline.
use super::child_test_process::{
    ChildTest,
    passed,
};
/// What:
///  `OsStr` is borrowed operating-system text of raw bytes;
///  its sibling is `String`, an owned UTF-8 buffer.
/// Why:
///  An environment value is bytes and not necessarily UTF-8,
///  and the marker this file hands the child is one such value.
use std::ffi::OsStr;

/// The variable whose presence marks the child this file starts,
/// which is the only process a control may arm a seam of the whole process in.
const CHILD_VARIABLE: &str = "CLI_GIT_ALONE_IN_ITS_PROCESS_CHILD";

/// What:
///  Whether this process is a child this file started,
///  which holds one test and nothing else.
/// Why:
///  The refusal and the control's own body ask one question,
///  so a process is that child,
///  or is not,
///  by one variable read in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function thisProcessHoldsThisTestAlone(): boolean { return process.env[CHILD_VARIABLE] !== undefined; }
/// ```
pub(crate) fn this_process_holds_this_test_alone() -> bool {
    // What: `var_os` answers the variable's value as an operating-system string, or nothing when it is absent; `is_some`
    //  turns that into yes or no.
    // Why: only the presence of the marker matters, and the child sets it to `1`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return process.env[CHILD_VARIABLE] !== undefined;
    // ```
    return std::env::var_os(CHILD_VARIABLE).is_some();
}

/// What:
///  Refuse to arm the seam of the whole process that `seam` names in a process that holds other tests too,
///  naming the starter that makes the calling process one that holds this test alone.
/// Why:
///  A seam such as a hold the shipped code consults for every read reaches every test running beside the one that
///  armed it,
///  so a control that forgot its child must fail at its first line,
///  every time it runs,
///  instead of making the rest of a threaded run fail now and then.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refuseOutsideTheChild(seam: string): void { assert(thisProcessHoldsThisTestAlone(), `${seam} belongs to the whole process, so arm it in a child of the control's own, which ranAloneInAChildOfItsOwn starts`); }
/// ```
pub(crate) fn refuse_outside_the_child(seam: &str) {
    // What: `assert!` ends the test with the message when the condition is false and does nothing when it is true.
    // Why: the message names the seam and the way out, so a failing run says what to do rather than only what broke.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (!thisProcessHoldsThisTestAlone()) throw new Error(`${seam} belongs to the whole process: run this control in a child of its own, which ranAloneInAChildOfItsOwn starts.`);
    // ```
    assert!(
        this_process_holds_this_test_alone(),
        "{seam} belongs to the whole process, and `cargo test` runs a crate's tests as threads of one process, so \
         arming it here changes every test of this run: run this control in a child of its own, which \
         `crate::alone_in_its_process_child_test_support::ran_alone_in_a_child_of_its_own` starts"
    );
}

/// What:
///  Start the body of the test `name` of `module` in a child process of this test binary
///  and fail this test with everything the child said when its one test did not pass,
///  or answer that this process is that child.
/// Why:
///  A control whose body arms or reads state of the whole process needs a process that holds no other test,
///  and this is the one call that gives every such control that process;
///  `true` in the parent, which has then asserted the child's result and returns,
///  and `false` in the child, where the body follows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ranAloneInAChildOfItsOwn(module: string, name: string): boolean {
///   if (process.env[CHILD_VARIABLE]) return false;
///   passed({ exactTest: pathOf(module, name), environment: [[CHILD_VARIABLE, '1']], ...plainRequest() });
///   return true;
/// }
/// ```
pub(crate) fn ran_alone_in_a_child_of_its_own(
    module: &str,
    name: &str,
) -> bool {
    // What: this process is the child, so the body that called this runs here and nothing is started.
    // Why: the child was started with the marker set, and its own call has to answer "go on".
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (thisProcessHoldsThisTestAlone()) return false;
    // ```
    if this_process_holds_this_test_alone() {
        return false;
    }
    // What: the path libtest prints for this test: the module path without the crate's own first segment, then the
    //  test's name; `split_once` answers the text before and after the first `::`, and `match` chooses between the two
    //  answers, where `_the_crate` is a part that is read and thrown away.
    // Why: `--exact` matches that text and nothing else, so a path that keeps the crate's name would run no test and
    //  assert nothing.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const inside = module.includes('::') ? module.slice(module.indexOf('::') + 2) : module;
    // const path = `${inside}::${name}`;
    // ```
    let module_inside_the_crate: &str = match module.split_once("::") {
        Some((_the_crate, inside)) => inside,
        None => module,
    };
    let path: String = format!("{module_inside_the_crate}::{name}");
    // What: one entry naming the marker variable and the value `1`, as a fixed-length slice of borrows.
    // Why: the child learns from this value that it is the child, and nothing else of this process's state is passed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const environment: [string, string][] = [[CHILD_VARIABLE, '1']];
    // ```
    let environment: [(&str, &OsStr); 1] = [(CHILD_VARIABLE, OsStr::new("1"))];
    // What: the one call that starts the child and checks its one result; the two dots take every field the request
    //  leaves out from `ChildTest::default()`, which is the plain child with two pipes and the file's deadline.
    // Why: a child that failed, hung or ran no test fails this call with its own words, in the parent's own panic.
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
    // What: the tail expression, so the parent answers `true`: it has started the child and asserted its result.
    // Why: the caller returns on `true` and runs the body on `false`, which is the child's answer alone.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return true;
    // ```
    return true;
}

/// The controls of the refusal and of the child's own answer;
/// never part of the release executable.
#[cfg(test)]
#[path = "alone_in_its_process_child_test_support_tests.rs"]
mod tests;
