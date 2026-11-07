//! Runs a test body in a child process that starts under a chosen umask,
//! so no test changes the umask of a process other tests
//! run in (the test binary runs tests on threads of one process under `cargo test`).
//!
//! A test calls `ran_in_child` first and returns when it says `true`:
//! the parent has then started this same test in a child
//! with the mask,
//! and the child's success is the parent's assertion.
//! In the child it says `false` and the test goes on.

use std::os::unix::process::CommandExt as _;

/// The environment variable that marks the child.
const CHILD_VARIABLE: &str = "CLI_GIT_UMASK_CHILD";

/// Whether this call ran the test in a child under `mask` (and asserted that it passed);
/// `false` in the child itself,
/// where the
/// body follows.
/// `module` is `module_path!()` of the test and `name` its function name.
pub(crate) fn ran_in_child(module: &str, name: &str, mask: u32) -> bool {
    if std::env::var_os(CHILD_VARIABLE).is_some() {
        return false;
    }
    let path: String = format!("{}::{name}", module.split_once("::").map_or(module, |(_, rest)| { return rest; }));
    let mut command: std::process::Command = std::process::Command::new(crate::test_support::this_test_binary().expect("the test binary"));
    command.args(["--exact", path.as_str(), "--nocapture"]).env(CHILD_VARIABLE, "1");
    // SAFETY: `umask` only sets the file creation mask of the child between its fork and its exec and is async-signal-safe.
    unsafe {
        command.pre_exec(move || {
            libc::umask(mask as libc::mode_t);
            return Ok(());
        });
    }
    command.stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped());
    let child: std::process::Child = crate::test_support::spawn_owned(&mut command).expect("start the child test");
    let output: std::process::Output = child.wait_with_output().expect("child test");
    let said: String = String::from_utf8_lossy(&output.stdout).into_owned();
    assert!(output.status.success(), "the child test {path} under umask {mask:o} failed:\n{said}\n{}", String::from_utf8_lossy(&output.stderr));
    // A name that matches no test would end with success and run nothing.
    assert!(said.contains("running 1 test") && said.contains("1 passed"), "the child ran no test named {path}:\n{said}");
    return true;
}
