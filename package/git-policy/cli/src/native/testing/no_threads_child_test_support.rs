//! Runs a test body in a child process that is refused every thread it tries to start,
//! so a site that starts threads is shown to
//! carry on where the system gives none (`Scope::spawn` panics there;
//! a start that falls back runs the work in turn).
//!
//! A test hands this file the body to run,
//! and the body runs in a child whose process limit is one:
//! the parent starts this same test in that child,
//! and the child's success is the parent's assertion.
//! The parent never calls the body.
//! That is the whole reason the body travels as a closure instead of the test holding an answer:
//! while this file answered a yes or no,
//! two controls read the answer inverted
//! (`if !ran_in_child(...) { return; }`),
//! so the child returned at once and the parent ran the body written for a process at its limit,
//! and every other test of that run failed with `EAGAIN`
//! (`Resource temporarily unavailable`,
//! issue 6494).
//! Where the user may exceed the process limit the thread is not refused,
//! so the parent says so and skips.
//! The two setters of that limit refuse in any other process and name the starter,
//! because the limit belongs to the whole process and a control that changes it where the run's tests are
//! threads stops every other test's starts with `EAGAIN` (issue 6496).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! runTheBodyWhereNoThreadCanStart(import.meta.url, 'a_test', () => { /* the body */ });
//! ```

/// The requests for the child of this test binary,
/// its deadline and the check that its one test passed.
use super::child_test_process::{
    ChildTest,
    finished,
    passed,
};
/// `OsStr` is borrowed operating-system text of raw bytes.
use std::ffi::OsStr;

/// The environment variable that marks the child.
const CHILD_VARIABLE: &str = "CLI_GIT_NO_THREADS_CHILD";

/// What:
///  Refuse to change this process's limit on the processes and threads it starts,
///  and to read it as a control,
///  in a process that is not the child `run_the_body_where_no_thread_can_start` started,
///  naming the starter.
/// Why:
///  The limit belongs to the whole process,
///  and `cargo test --lib` runs a crate's tests as threads of one process,
///  so a control that lowers it here stops every other test's starts with `EAGAIN`:
///  measured on 2026-11-26,
///  one control whose child guard was inverted lowered the run's own limit to one,
///  and three other controls of its module failed at their fixture's first Git start
///  (issue 6496).
///  The body runs only in the child,
///  so a control that forgot it now fails at its own first line instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refuseOutsideTheChild(state: string): void { assert(process.env.CLI_GIT_NO_THREADS_CHILD !== undefined, `${state} belongs to the whole process: set it in a child of runTheBodyWhereNoThreadCanStart's own`); }
/// ```
fn refuse_outside_the_child(state: &str) {
    assert!(
        std::env::var_os(CHILD_VARIABLE).is_some(),
        "{state} belongs to the whole process, and `cargo test` runs a crate's tests as threads of one process, so \
         changing it here reaches every other test of that run: set it in a child of the control's own, which \
         `crate::no_threads_child_test_support::run_the_body_where_no_thread_can_start` starts"
    );
}

/// Lowers the soft process limit to one in the child between its fork and its exec.
fn apply_limit() -> std::io::Result<()> {
    let mut found = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    // SAFETY: `getrlimit` and `setrlimit` are async-signal-safe and read and write only this local value.
    unsafe {
        if libc::getrlimit(libc::RLIMIT_NPROC, &mut found) != 0 {
            return Err(std::io::Error::last_os_error());
        }
        let lowered = libc::rlimit {
            rlim_cur: 1,
            rlim_max: found.rlim_max,
        };
        if libc::setrlimit(libc::RLIMIT_NPROC, &lowered) != 0 {
            return Err(std::io::Error::last_os_error());
        }
    }
    return Ok(());
}

/// What:
///  Give this process room for threads again,
/// up to the hard limit it inherited.
/// Why:
///  A control can then show what happens to the work the pool queued while every thread was refused:
///  the pool's first new thread runs it,
///  which is the case a job reported as "never started" must survive.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// setrlimit(RLIMIT_NPROC, { cur: hard, max: hard });
/// ```
pub(crate) fn let_threads_start_again() -> std::io::Result<()> {
    refuse_outside_the_child("the limit on the processes and threads this process starts");
    return set_process_limit(None);
}

/// What:
///  Refuse every thread this process starts,
/// as the child of `run_the_body_where_no_thread_can_start` is refused.
/// Why:
///  A control that first needs processes of its own (a Git fixture) can then put the refusal back
/// for the part it measures.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// setrlimit(RLIMIT_NPROC, { cur: 1, max: hard });
/// ```
pub(crate) fn refuse_threads_again() -> std::io::Result<()> {
    refuse_outside_the_child("the limit on the processes and threads this process starts");
    return set_process_limit(Some(1));
}

/// The soft process limit,
/// or the hard one when `allowed` is nothing.
fn set_process_limit(allowed: Option<u64>) -> std::io::Result<()> {
    let mut found = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    // SAFETY: `getrlimit` and `setrlimit` are async-signal-safe and read and write only this local value.
    unsafe {
        if libc::getrlimit(libc::RLIMIT_NPROC, &mut found) != 0 {
            return Err(std::io::Error::last_os_error());
        }
        let wanted = libc::rlimit {
            rlim_cur: allowed.unwrap_or(found.rlim_max),
            rlim_max: found.rlim_max,
        };
        if libc::setrlimit(libc::RLIMIT_NPROC, &wanted) != 0 {
            return Err(std::io::Error::last_os_error());
        }
    }
    return Ok(());
}

/// The probe test the parent starts in a child before the real test:
/// it says whether a thread can start under the limit.
/// It lives in a sidecar,
/// as every test of the crate does;
/// this file names it by its path there.
#[cfg(test)]
#[path = "no_threads_child_test_support_tests.rs"]
mod tests;

/// What:
///  Runs `body` in a child of this test binary whose soft process limit is one,
///  and fails this call with the child's own words when that child's one test did not pass.
///  `module` is `module_path!()` of the test,
///  `name` its function name,
///  and `body` what the test does.
/// Why:
///  A soft process limit belongs to a whole process,
///  and `cargo test` runs a crate's tests as threads of one process,
///  so a body that lowers it where the run holds it makes every other test of that run fail with `EAGAIN`
///  (`Resource temporarily unavailable`,
///  issue 6494).
///  The child is a process of its own,
///  so the limit reaches nothing else.
///  The body is a closure the parent never calls,
///  so no caller holds an answer that can be read inverted.
/// Gotcha:
///  Everything the body needs is made inside it,
///  because nothing before this call runs in the child;
///  a `let` before the call is the parent's value and the child never sees it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runTheBodyWhereNoThreadCanStart(module: string, name: string, body: () => void): void {
///   if (process.env.CHILD_VARIABLE) { body(); return; }
///   const probe = finished({ exactTest: `${module}::tests::thread_probe`, beforeTheExec: lowerLimit, stderr: 'ignore' });
///   if (probe.stdout.includes('thread-started')) { log('skipped'); return; }
///   passed({ exactTest: `${module}::${name}`, environment: [[CHILD_VARIABLE, '1']], beforeTheExec: lowerLimit });
/// }
/// ```
pub(crate) fn run_the_body_where_no_thread_can_start(
    module: &str,
    name: &str,
    body: impl FnOnce(),
) {
    if std::env::var_os(CHILD_VARIABLE).is_some() {
        // The child is the process under the limit, so the body runs here and its panic is this child's failure.
        body();
        return;
    }
    let strip = |full: &str| -> String {
        return full
            .split_once("::")
            .map_or(String::from(full), |(_, rest)| {
                return String::from(rest);
            });
    };
    let probe_path: String = format!("{}::tests::thread_probe", strip(module_path!()));
    let environment: [(&str, &OsStr); 1] = [(CHILD_VARIABLE, OsStr::new("1"))];
    let probe = finished(ChildTest {
        exact_test: probe_path.as_str(),
        environment: environment.as_slice(),
        // The probe's own prints are read from its standard output alone, so its standard error is dropped.
        standard_error: std::process::Stdio::null(),
        before_the_exec: Some(Box::new(apply_limit)),
        ..ChildTest::default()
    })
    .expect("the thread probe ends within the deadline");
    if String::from_utf8_lossy(probe.stdout.as_slice()).contains("thread-started") {
        eprintln!("skipped: this user may exceed RLIMIT_NPROC, so no thread start was refused");
        return;
    }
    let path: String = format!("{}::{name}", strip(module));
    // The parent's whole assertion is this child's result, and a child that ran no test fails here.
    passed(ChildTest {
        exact_test: path.as_str(),
        environment: environment.as_slice(),
        before_the_exec: Some(Box::new(apply_limit)),
        ..ChildTest::default()
    });
}
