//! Runs a test body in a child process that starts under a chosen umask,
//! so no test changes the umask of a process other tests
//! run in (the test binary runs tests on threads of one process under `cargo test`).
//!
//! A test calls `ran_in_child` first and returns when it says `true`:
//! the parent has then started this same test in a child
//! with the mask,
//! and the child's success is the parent's assertion.
//! In the child it says `false` and the test goes on.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (ranInChild(import.meta.url, 'a_test', 0o077)) return;
//! ```

/// The request for one child of this test binary, its deadline, and the check that its one test passed.
use super::child_test_process::{ChildTest, passed};
/// The variable that marks the child of this helper.
use std::ffi::OsStr;

/// The environment variable that marks the child.
const CHILD_VARIABLE: &str = "CLI_GIT_UMASK_CHILD";

/// What:
///  Whether this call ran the test in a child under `mask` (and asserted that it passed);
///  `false` in the child itself,
///  where the body follows.
///  `module` is `module_path!()` of the test and `name` its function name.
/// Why:
///  A umask is one value for a whole process, so the mask is set between the child's `fork` and its `exec`
///  and never in this process;
///  the child is the same test binary on this one test, and its pass is this control's whole assertion.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ranInChild(module: string, name: string, mask: number): boolean {
///   if (process.env.CHILD_VARIABLE) return false;
///   passed({ exactTest: `${module}::${name}`, environment: [[CHILD_VARIABLE, '1']], beforeTheExec: () => umask(mask) });
///   return true;
/// }
/// ```
pub(crate) fn ran_in_child(module: &str, name: &str, mask: u32) -> bool {
    if std::env::var_os(CHILD_VARIABLE).is_some() {
        return false;
    }
    // The crate's own name is the first segment of `module_path!()` and is not part of the name libtest answers to.
    let path: String = format!("{}::{name}", module.split_once("::").map_or(module, |(_, rest)| { return rest; }));
    let environment: [(&str, &OsStr); 1] = [(CHILD_VARIABLE, OsStr::new("1"))];
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
    return true;
}
