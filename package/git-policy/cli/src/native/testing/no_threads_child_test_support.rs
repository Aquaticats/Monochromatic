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

use std::os::unix::process::CommandExt as _;

/// The environment variable that marks the child.
const CHILD_VARIABLE: &str = "CLI_GIT_NO_THREADS_CHILD";

/// Lowers the soft process limit to one in the child between its fork and its exec.
fn apply_limit() -> std::io::Result<()> {
    let mut found = libc::rlimit { rlim_cur: 0, rlim_max: 0 };
    // SAFETY: `getrlimit` and `setrlimit` are async-signal-safe and read and write only this local value.
    unsafe {
        if libc::getrlimit(libc::RLIMIT_NPROC, &mut found) != 0 {
            return Err(std::io::Error::last_os_error());
        }
        let lowered = libc::rlimit { rlim_cur: 1, rlim_max: found.rlim_max };
        if libc::setrlimit(libc::RLIMIT_NPROC, &lowered) != 0 {
            return Err(std::io::Error::last_os_error());
        }
    }
    return Ok(());
}

/// The command that starts this test binary on one test,
/// under the limit.
fn child_command(path: &str) -> std::process::Command {
    let mut command: std::process::Command = std::process::Command::new(crate::test_support::this_test_binary().expect("the test binary"));
    command.args(["--exact", path, "--nocapture", "--test-threads=1"]).env(CHILD_VARIABLE, "1").stdin(std::process::Stdio::null());
    // SAFETY: between fork and exec only `getrlimit` and `setrlimit` run.
    unsafe {
        command.pre_exec(apply_limit);
    }
    return command;
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
pub(crate) fn ran_in_child(module: &str, name: &str) -> bool {
    if std::env::var_os(CHILD_VARIABLE).is_some() {
        return false;
    }
    let strip = |full: &str| -> String { return full.split_once("::").map_or(String::from(full), |(_, rest)| { return String::from(rest); }); };
    let probe_path: String = format!("{}::tests::thread_probe", strip(module_path!()));
    let probe = child_command(probe_path.as_str()).stderr(std::process::Stdio::null()).output().expect("the probe runs");
    if String::from_utf8_lossy(probe.stdout.as_slice()).contains("thread-started") {
        eprintln!("skipped: this user may exceed RLIMIT_NPROC, so no thread start was refused");
        return true;
    }
    let path: String = format!("{}::{name}", strip(module));
    let output = child_command(path.as_str()).output().expect("the child test runs");
    let said: String = String::from_utf8_lossy(output.stdout.as_slice()).into_owned();
    assert!(output.status.success(), "the child test {path} without a thread to start failed:\n{said}\n{}", String::from_utf8_lossy(output.stderr.as_slice()));
    // A name that matches no test would end with success and run nothing.
    assert!(said.contains("running 1 test") && said.contains("1 passed"), "the child ran no test named {path}:\n{said}");
    return true;
}
