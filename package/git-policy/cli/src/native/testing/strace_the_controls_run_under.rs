//! What:
//!  Finding `strace`,
//! which the budget,
//! fault,
//! child-bound and stop-lock controls run a wrapped command under,
//! and the one rule that decides whether a control which needs it may run.
//! Why:
//!  A traced command runs with the fixture's restricted `PATH`,
//! which lacks the directory of `strace`,
//! so a bare name would
//!  fail to start there even where the tool is installed.
//! A run without the tracer cannot make the cut,
//! the fault or the count those controls are
//!  about,
//! so a control there fails with the reason instead of passing while it checks nothing.
//! It prints one line naming itself and
//!  passes only when the run states that the absence is known (`ABSENT_ACKNOWLEDGED_VARIABLE`),
//! the way the btrfs and ZFS
//!  controls state a missing fixture root
//! (`src/native/binary/binary_support_btrfs_controls.rs`,
//! `src/native/binary/binary_support_zfs_controls.rs`).
//! Nothing in this repository sets that
//!  acknowledgement:
//! the gate image installs `strace` (`native-test-base.Containerfile`) and
//!  this host installs it too,
//! so a run that loses the tracer is red rather than green with every traced control unchecked.
//!  One copy serves both crates that need it:
//! the unit tests reach it as `crate::testing::strace_the_controls_run_under`,
//! and the binary controls' own crate includes this file by path
//!  (`src/native/binary/binary_tests.rs`),
//! as it includes `real_git_lfs_the_run_requires.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const tracer = path.split(':').map((folder) => join(folder, 'strace')).find(isFile); // absent: the run's acknowledgement decides
//! ```

use std::path::PathBuf;
use std::process::Command;

/// The environment variable that acknowledges "this run has no `strace`".
/// An empty value states nothing,
/// as the btrfs and ZFS acknowledgements read their own.
/// Nothing here sets it:
/// the container gate image installs `strace`
/// (`native-test-base.Containerfile`),
/// and so does every host these tasks run on,
/// so an image or a machine that lost the tracer fails the controls that need it
/// instead of passing them unchecked.
pub(crate) const ABSENT_ACKNOWLEDGED_VARIABLE: &str = "CLI_GIT_STRACE_ABSENT_ACKNOWLEDGED";

/// The absolute path of `strace` on the test process's own `PATH`,
/// or nothing when it is not installed.
///  A traced command runs with the fixture's restricted `PATH`,
/// which lacks the directory of `strace`,
/// so a bare name
///  would fail to start there even where the tool is installed.
/// A control that runs a command under it first asks
///  [`strace_available_or_acknowledged`],
/// so this read is only reached in a run that has the tracer.
pub(crate) fn strace_program() -> Option<PathBuf> {
    let search = std::env::var_os("PATH")?;
    return std::env::split_paths(&search)
        .map(|directory| {
            return directory.join("strace");
        })
        .find(|candidate| {
            return candidate.is_file();
        });
}

/// What:
///  The tracer a control can start,
/// or nothing when no directory of `PATH` holds one or it cannot be started.
/// Why:
///   A file named `strace` that cannot be run is no tracer,
/// so the answer is the program that answers `--version`,
/// never the file's presence alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const tracer = found?.startsWith(/* spawnSync(found, ['--version']).status === 0 */);
/// ```
fn runnable_strace() -> Option<PathBuf> {
    return strace_program().filter(|program| {
        return Command::new(program).arg("--version").output().is_ok();
    });
}

/// What:
///  Whether this run states that the tracer's absence is known.
/// Why:
///   The statement is the variable's non-empty value,
/// as the btrfs and ZFS acknowledgements read theirs,
/// so an empty value names nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const acknowledged = (process.env.CLI_GIT_STRACE_ABSENT_ACKNOWLEDGED ?? '') !== '';
/// ```
fn the_run_acknowledges_the_absence() -> bool {
    return std::env::var_os(ABSENT_ACKNOWLEDGED_VARIABLE).is_some_and(|value| {
        return !value.is_empty();
    });
}

/// What:
///  Whether the control named may run its command under `strace`,
/// or `false` after one line naming the control when this run states the absence is known.
/// Why:
///   A control whose cut,
/// fault or count cannot be made must not pass as coverage it does not have,
/// so a run that
///  never said it has no tracer fails the control with both ways forward.
/// A run that stated it gets the line that says
///  which controls did not run,
/// and
///  `--success-output=final` brings that line of a passing control into the log,
/// as it does the btrfs one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (!straceAvailableOrAcknowledged('the control')) return;
/// ```
pub(crate) fn strace_available_or_acknowledged(control: &str) -> bool {
    return decided(
        control,
        runnable_strace(),
        the_run_acknowledges_the_absence(),
    )
    .is_some();
}

/// What:
///  The same decision,
/// with the tracer found and the run's statement given rather than read.
/// Why:
///   A control cannot change this process's environment beside the other controls of its binary,
/// so the arms that prove
///  this rule run it with the absence and the statement a host without the tracer has
/// (`testing/strace_the_controls_run_under_tests.rs`).
/// The environment itself is proved where a child process can set it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const decided = (control, found, acknowledged) => found ?? (acknowledged ? noteSkipped(control) : fail(control));
/// ```
pub(crate) fn decided(
    control: &str,
    found: Option<PathBuf>,
    acknowledged: bool,
) -> Option<PathBuf> {
    if let Some(tracer) = found {
        return Some(tracer);
    }
    assert!(
        acknowledged,
        "the control {control} needs strace, which is on no directory of PATH or cannot be started; install strace, or set \
         {ABSENT_ACKNOWLEDGED_VARIABLE}=1 to state that this run has none, and then the control names itself and passes. The \
         container gate image installs strace, so this is an image or a machine that lost it."
    );
    eprintln!(
        "STRACE CONTROL NOT RUN: {control} (no strace on PATH, and {ABSENT_ACKNOWLEDGED_VARIABLE} states that this run has \
         none; the container gate image installs strace)"
    );
    return None;
}
