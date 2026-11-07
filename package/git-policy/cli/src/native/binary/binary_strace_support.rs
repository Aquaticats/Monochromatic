//! What:
//!  Finding `strace`,
//! which the budget,
//! fault and child-bound controls run a wrapped command under.
//! Why:
//!  A traced command runs with the fixture's restricted `PATH`,
//! which lacks the directory of `strace`,
//! so a bare name would
//!  fail to start there even where the tool is installed;
//! a host without it reports that and checks nothing.

use std::path::PathBuf;
use std::process::Command;

/// The absolute path of `strace` on the test process's own `PATH`,
/// or nothing when it is not installed.
///  A traced command runs with the fixture's restricted `PATH`,
/// which lacks the directory of `strace`,
/// so a bare name
///  would fail to start there even where the tool is installed.
pub(super) fn strace_program() -> Option<PathBuf> {
    let search = std::env::var_os("PATH")?;
    return std::env::split_paths(&search).map(|directory| { return directory.join("strace"); })
        .find(|candidate| { return candidate.is_file(); });
}

/// Whether `strace` can be started.
pub(super) fn strace_available() -> bool {
    return strace_program().is_some_and(|program| {
        return Command::new(program).arg("--version").output().is_ok();
    });
}
