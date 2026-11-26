//! Runs a test body in a child process that is refused every thread it tries to start,
//! so a site that starts threads is shown to
//! carry on where the system gives none (`Scope::spawn` panics there;
//! a start that falls back runs the work in turn).
//!
//! A test calls `ran_in_child` first and returns when it says `true`:
//! the parent has then started this same test in a child whose
//! process limit is one,
//! and the child's success is the parent's assertion.
//! In the child it says `false` and the body follows.
//! Where the user may exceed the process limit the thread is not refused,
//! so the parent says so and skips.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (ranInChild(import.meta.url, 'a_test')) return;
//! ```

/// The requests for the child of this test binary, its deadline and the check that its one test passed.
use super::child_test_process::{
    ChildTest,
    finished,
    passed,
};
/// `OsStr` is borrowed operating-system text of raw bytes.
use std::ffi::OsStr;

/// The environment variable that marks the child.
const CHILD_VARIABLE: &str = "CLI_GIT_NO_THREADS_CHILD";

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
    return set_process_limit(None);
}

/// What:
///  Refuse every thread this process starts,
/// as the child of `ran_in_child` is refused.
/// Why:
///  A control that first needs processes of its own (a Git fixture) can then put the refusal back
/// for the part it measures.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// setrlimit(RLIMIT_NPROC, { cur: 1, max: hard });
/// ```
pub(crate) fn refuse_threads_again() -> std::io::Result<()> {
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
/// `ran_in_child` names it by its path there.
#[cfg(test)]
#[path = "no_threads_child_test_support_tests.rs"]
mod tests;

/// Whether this call ran the test in a child with no thread to start (and asserted that it passed),
/// or skipped because the user may
/// exceed the process limit;
/// `false` in the child itself,
/// where the body follows.
/// `module` is `module_path!()` of the test and `name`
/// its function name.
/// Why:
///  The probe comes first, because a run where a thread still starts under one process proves nothing about the
///  fallback, and the control says so instead of passing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ranInChild(module: string, name: string): boolean {
///   if (process.env.CHILD_VARIABLE) return false;
///   const probe = finished({ exactTest: `${module}::tests::thread_probe`, beforeTheExec: lowerLimit, stderr: 'ignore' });
///   if (probe.stdout.includes('thread-started')) { log('skipped'); return true; }
///   passed({ exactTest: `${module}::${name}`, beforeTheExec: lowerLimit });
///   return true;
/// }
/// ```
pub(crate) fn ran_in_child(
    module: &str,
    name: &str,
) -> bool {
    if std::env::var_os(CHILD_VARIABLE).is_some() {
        return false;
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
        return true;
    }
    let path: String = format!("{}::{name}", strip(module));
    passed(ChildTest {
        exact_test: path.as_str(),
        environment: environment.as_slice(),
        before_the_exec: Some(Box::new(apply_limit)),
        ..ChildTest::default()
    });
    return true;
}
