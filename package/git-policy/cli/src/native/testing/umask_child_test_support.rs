//! Runs a test body in a child process that starts under a chosen umask,
//! so no test changes the umask of a process other tests
//! run in (the test binary runs tests on threads of one process under `cargo test`).
//!
//! A test hands this file the body to run,
//! and the body runs in a child whose file creation mask is the one the test chose:
//! the parent starts this same test in that child, and the child's success is the parent's assertion.
//! The parent never calls the body, so no caller holds an answer that can be read inverted.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! runTheBodyUnderUmask(import.meta.url, 'a_test', 0o077, () => { /* the body */ });
//! ```

/// The request for one child of this test binary, its deadline, and the check that its one test passed.
use super::child_test_process::{
    ChildTest,
    passed,
};
/// The variable that marks the child of this helper.
use std::ffi::OsStr;

/// The environment variable that marks the child.
const CHILD_VARIABLE: &str = "CLI_GIT_UMASK_CHILD";

/// What:
///  Runs `body` in a child of this test binary that starts under `mask`,
///  and fails this call with the child's own words when that child's one test did not pass.
///  `module` is `module_path!()` of the test,
///  `name` its function name,
///  `mask` the file creation mask the child runs under,
///  and `body` what the test does.
/// Why:
///  A umask is one value for a whole process,
///  so the mask is set between the child's `fork` and its `exec` and never in this process;
///  the child is the same test binary on this one test, and its pass is this control's whole assertion.
///  The body is a closure the parent never calls, so no caller holds an answer that can be read inverted.
/// Gotcha:
///  Everything the body needs is made inside it, because nothing before this call runs in the child.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runTheBodyUnderUmask(module: string, name: string, mask: number, body: () => void): void {
///   if (process.env.CHILD_VARIABLE) { body(); return; }
///   passed({ exactTest: `${module}::${name}`, environment: [[CHILD_VARIABLE, '1']], beforeTheExec: () => umask(mask) });
/// }
/// ```
pub(crate) fn run_the_body_under_umask(
    module: &str,
    name: &str,
    mask: u32,
    body: impl FnOnce(),
) {
    if std::env::var_os(CHILD_VARIABLE).is_some() {
        // The child is the process under the mask, so the body runs here and its panic is this child's failure.
        body();
        return;
    }
    // The crate's own name is the first segment of `module_path!()` and is not part of the name libtest answers to.
    let path: String = format!(
        "{}::{name}",
        module.split_once("::").map_or(module, |(_, rest)| {
            return rest;
        })
    );
    let environment: [(&str, &OsStr); 1] = [(CHILD_VARIABLE, OsStr::new("1"))];
    // The parent's whole assertion is this child's result, and a child that ran no test fails here.
    passed(ChildTest {
        exact_test: path.as_str(),
        environment: environment.as_slice(),
        // What: a closure that sets the child's file creation mask between `fork` and `exec`.
        // Why:  `umask` is async-signal-safe, which is the whole rule for that window.
        before_the_exec: Some(Box::new(move || {
            // `as` is the one conversion from the request's `u32` mask to the type `umask` takes.
            // SAFETY: `umask` only sets the file creation mask of the child between its fork and its exec.
            unsafe {
                libc::umask(mask as libc::mode_t);
            }
            return Ok(());
        })),
        ..ChildTest::default()
    });
}
