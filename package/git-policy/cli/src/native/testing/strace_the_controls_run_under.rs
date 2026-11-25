//! What:
//!  Finding `strace`,
//! which the budget,
//! fault,
//! child-bound and stop-lock controls run a wrapped command under.
//! Why:
//!  A traced command runs with the fixture's restricted `PATH`,
//! which lacks the directory of `strace`,
//! so a bare name would
//!  fail to start there even where the tool is installed;
//! a host without it reports that and checks nothing.
//!  This host installs none
//! (`pacman -Q strace` answers "package 'strace' was not found",
//! 2026-11-25),
//! while the container gate's image installs it
//!  (`bin/build-git-test-image.ts`),
//! so a control that names the tracer bare fails here and passes there
//! (`doc/troubleshooting/git-policy-cli-cut-control-holds-a-tracer-the-stop-kills.md`,
//! "A machine without the tracer").
//!  One copy serves both crates that need it:
//! the unit tests reach it as `crate::testing::strace_the_controls_run_under`,
//! and the binary controls' own crate includes this file by path
//!  (`src/native/binary/binary_tests.rs`),
//! as it includes `real_git_lfs_the_run_requires.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const tracer = path?.split(':').map((folder) => join(folder, 'strace')).find(isFile); // absent: one line, then check nothing
//! ```

use std::path::PathBuf;
use std::process::Command;

/// The absolute path of `strace` on the test process's own `PATH`,
/// or nothing when it is not installed.
///  A traced command runs with the fixture's restricted `PATH`,
/// which lacks the directory of `strace`,
/// so a bare name
///  would fail to start there even where the tool is installed.
pub(crate) fn strace_program() -> Option<PathBuf> {
    let search = std::env::var_os("PATH")?;
    return std::env::split_paths(&search).map(|directory| { return directory.join("strace"); })
        .find(|candidate| { return candidate.is_file(); });
}

/// Whether `strace` can be started.
pub(crate) fn strace_available() -> bool {
    return strace_program().is_some_and(|program| {
        return Command::new(program).arg("--version").output().is_ok();
    });
}
