//! What:
//!  One `#[ignore]` child test run in a process that may write no file past a size limit.
//! Why:
//!  A file-size limit is the privilege-free way to make a write fail half way,
//!  and the limit is a property of the whole process,
//!  so the child runs alone and the parent's other tests never see it.
//!  The child runner is a file of its own
//!  so `test_support.rs`,
//! which the crate's controls share,
//! stays inside the code-line limit,
//!  and it sits beside the umask and no-thread child runners that hold the same shape.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! spawnSync(process.execPath, [testBinary, '--exact', name, '--ignored'], { env: { DIRECTORY: directory }, limits: { fsize: limit } });
//! ```

use std::process::Command;

/// Variable that tells a child test the directory it works in.
pub(crate) const LIMITED_DIRECTORY_VARIABLE: &str = "CLI_GIT_LIMITED_CHILD_DIRECTORY";

/// What:
///  Run one `#[ignore]` child test in a process that may write no file past `limit` bytes,
/// and return how it ended.
/// Why:
///  A file-size limit is the privilege-free way to make a write fail half way:
/// the bytes below the limit are written
///  and the rest fails with `EFBIG` (`SIGXFSZ` is ignored so that the error reaches the caller instead of ending the
///  process).
/// The limit is a property of the whole process,
/// so the child runs alone and the parent's other tests
///  never see it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// spawnSync(process.execPath, [testBinary, '--exact', name, '--ignored'], { env: { DIRECTORY: directory }, limits: { fsize: limit } });
/// ```
pub(crate) fn run_child_under_file_size_limit(test_name: &str, limit: u64, directory: &std::path::Path) -> std::process::ExitStatus {
    use std::os::unix::process::CommandExt;
    let mut command: Command = Command::new(crate::test_support::this_test_binary().expect("test binary"));
    command
        .args(["--exact", test_name, "--ignored", "--nocapture"])
        .env(LIMITED_DIRECTORY_VARIABLE, directory)
        .stdin(std::process::Stdio::null());
    // SAFETY: between fork and exec only `signal` and `setrlimit`, both async-signal-safe, run.
    unsafe {
        command.pre_exec(move || {
            libc::signal(libc::SIGXFSZ, libc::SIG_IGN);
            let cap: libc::rlimit = libc::rlimit { rlim_cur: limit, rlim_max: limit };
            if libc::setrlimit(libc::RLIMIT_FSIZE, &cap) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            return Ok(());
        });
    }
    return command.status().expect("the child test starts");
}
