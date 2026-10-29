//! What:
//!  A real-Git stand-in for unit tests that writes one line per start into a log file and then runs the real Git.
//! Why:
//!  A replacement that saves a Git process is only proven when a control sees the start did not happen,
//!  while the real Git remains the oracle for what the answer must be.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const context = { gitPath: recordingShim, overlay, globalPrefix: [] }; // shim appends argv to a log, then execs git
//! ```

/// The fixture root,
///  the real-Git paths and the isolating overlay.
use crate::test_support::{REAL_GIT, executable, isolated_overlay};
/// The Git context the production runners take.
use crate::transaction_git::GitContext;
/// Borrowed and owned paths.
use std::path::{Path, PathBuf};

/// What:
///  A Git context whose executable logs its arguments to a file in `root` and then runs the real Git,
///  with the log's path.
/// Why:
///   Controls count and name the Git starts a function made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { context, log } = recordingContext(root);
/// ```
#[cfg(unix)]
pub(crate) fn recording_context(root: &Path) -> (GitContext, PathBuf) {
    let log: PathBuf = root.join("git-starts.log");
    let shim: PathBuf = root.join("recording-git");
    let script: String = format!("#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{}'\nexec {REAL_GIT} \"$@\"\n", log.display());
    executable(shim.as_path(), script.as_bytes());
    let context: GitContext = GitContext {
        real_git: shim,
        overlay: isolated_overlay(),
        global_prefix: Vec::new(),
        common_dir: None,
    };
    return (context, log);
}

/// What:
///  The arguments of every Git start the log holds,
///  one string per start.
/// Why:
///   A log that was never written means no start happened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await readFile(log, 'utf8').catch(() => '')).split('\n').filter(Boolean)
/// ```
pub(crate) fn recorded_starts(log: &Path) -> Vec<String> {
    let text: String = std::fs::read_to_string(log).unwrap_or_default();
    let mut starts: Vec<String> = Vec::new();
    for line in text.lines() {
        starts.push(String::from(line));
    }
    return starts;
}
