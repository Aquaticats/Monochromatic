//! What:
//!  The file a commit service's output goes to when the wrapper starts it itself:
//! its path beside the worktree's socket,
//! its size cap,
//! and its truncation at each start.
//! Why:
//!   The wrapper redirects the server's standard output and standard error into one file,
//! because the process has no terminal and no service manager to capture them.
//! A file that only ever grows is a leak,
//! and a start that appends to another server's history is a log nobody can read,
//! so the file is truncated when a start finds it past its cap,
//! which is Bazel's answer (`daemonize -l`, whose log is truncated at each start
//!  unless the person names a file to append to;
//! `src/main/tools/daemonize.cc`, `SetupStdio`).
//!  The cap inside one server's life is ours:
//! Bazel has none.
//! The service writes about one line per start and about one per landed commit
//! (`service_timing.rs`),
//! and the service journal of this repository held 1,335 landings by 2026-11-03
//! (`wait_ceilings.rs`, the answer ceiling's measurement),
//! so four million bytes hold roughly sixteen thousand landings,
//! more than any recorded period,
//! for one worktree.
//! A written file is 0600 and a written directory 0700:
//! the server's output can name branches and paths,
//! and the socket it sits beside carries a review credential.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const log = join(runtimeDir, 'monochromatic-git-policy-cli', `${instance}.log`);
//! truncateWhenLargerThanItsCap(log); const fd = open(log, 'a', 0o600);
//! ```

use std::path::{Path, PathBuf};

/// The ending of the file one instance's output goes to.
pub const LOG_ENDING: &str = ".log";

/// The most bytes the file keeps;
/// see the module's reason for the number.
pub const LOG_BYTES_KEPT: u64 = 4 * 1024 * 1024;

/// The file one instance's output goes to, beside its socket.
pub fn path(socket_directory: &Path, instance: &str) -> PathBuf {
    return socket_directory.join(format!("{instance}{LOG_ENDING}"));
}

/// What:
///  Removes the file's content when it has grown past its cap.
/// Why:
///   A start must not append to a history nobody can read,
/// and the cap is what makes the file's size bounded across starts.
/// A file that is not there,
/// or is under the cap,
/// is left as it is:
/// the last server's lines are what explains a crash.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (size(log) > LOG_BYTES_KEPT) truncate(log, 0);
/// ```
pub fn truncate_when_past_its_cap(log: &Path) -> std::io::Result<()> {
    let size: u64 = match std::fs::metadata(log) {
        Ok(metadata) => metadata.len(),
        // Nothing was written yet, which is not a failure: the file is created by the open that follows.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    if size <= LOG_BYTES_KEPT {
        return Ok(());
    }
    return std::fs::OpenOptions::new().write(true).truncate(true).open(log).map(|_truncated| ());
}

/// Controls of the path and the cap stay out of the release executable.
#[cfg(test)]
#[path = "service_server_log_file_of_an_instance_tests.rs"]
mod tests;
