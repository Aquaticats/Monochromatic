//! Controls for the guarded-child order: the exit code is returned after one release, and a release failure is reported.

use super::*;
use std::sync::Mutex;

/// The signal guard is one per process, so the controls that install it run one at a time.
static GUARD_TURN: Mutex<()> = Mutex::new(());

/// Counts how often the release step ran, and fails when asked to.
struct Releases {
    /// Number of release calls.
    count: usize,
    /// Whether release fails.
    fails: bool,
}

impl ReleaseSteps for Releases {
    fn release(&mut self) -> Result<(), String> {
        self.count += 1;
        if self.fails {
            return Err(String::from("lock refused"));
        }
        return Ok(());
    }
}

/// The guarded child's exit code is returned, and release runs exactly once after the child ended.
#[cfg(unix)]
#[test]
fn the_guarded_child_exit_code_is_returned_after_one_release() {
    let _turn = GUARD_TURN.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut releases: Releases = Releases { count: 0, fails: false };
    let mut command: Command = Command::new("/bin/sh");
    command.args(["-c", "exit 3"]);
    assert_eq!(run_guarded_child(&mut command, &mut releases), Ok(3));
    assert_eq!(releases.count, 1);
}

/// A failed release is reported, not hidden behind the child's success.
#[cfg(unix)]
#[test]
fn a_failed_release_is_reported_after_the_child_ends() {
    let _turn = GUARD_TURN.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut releases: Releases = Releases { count: 0, fails: true };
    let mut command: Command = Command::new("/bin/sh");
    command.args(["-c", "exit 0"]);
    let outcome: Result<i32, String> = run_guarded_child(&mut command, &mut releases);
    assert!(matches!(&outcome, Err(message) if message.contains("releasing after the child failed")), "{outcome:?}");
    assert_eq!(releases.count, 1);
}
