//! What:
//!  A Git step of a commit that shows no sign of work for its window is stopped and started again, and the commit lands.
//! Why:
//!  The bound used to be an elapsed time, so a step that stalled for a moment under load ended the commit
//!  (the owner's rule is "Commits should never fail" and "Stop only on no progress").
//!  Each control starts a real wrapped commit under `strace` with a delay on the return of one `execve`
//!  (the start of one of the wrapper's own children) and a window shorter than that delay, so the step the wrapper watches
//!  shows no sign of work for longer than its window; the invariants are those of the fault tables (`binary_fault_support.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await injectAndExpectLanded({ call: 'execve', delayMs: windowMs + margin, env: { CLI_GIT_CHILD_WAIT_SECONDS: window } });
//! ```

use super::commit::remove_settled;
use super::fault::{Effect, Injection, Ran, When, run_injected_with_environment, staged_repository, verify_recovery};
use super::strace::strace_available;
use super::support::{Fixture, fixture};
use git_policy_cli::child_class::CHILD_WAIT_VARIABLE;
use std::path::PathBuf;

/// The window the control sets, in seconds: the smallest the variable accepts.
const WINDOW_SECONDS: u64 = 1;

/// How long the start of the delayed child is held beyond the window, in seconds.
const MARGIN_SECONDS: u64 = 1;

/// Which `execve` of the traced set is delayed: the wrapper's own start is the first, so the second is the first child
/// the wrapper starts and watches.
const DELAYED_CALL: usize = 2;

/// The commit with the window set and the notice interval at its smallest, so a notice precedes a stop.
fn windowed_commit(fixture: &Fixture, repo: &std::path::Path, label: &str) -> Ran {
    let fault: Injection = Injection {
        syscall: "execve",
        effect: Effect::SlowReturn((WINDOW_SECONDS + MARGIN_SECONDS) * 1_000_000),
        when: When::Nth(DELAYED_CALL),
        follow_forks: true,
    };
    let window: String = WINDOW_SECONDS.to_string();
    // The notice interval equals the window, so the grace, which never passes the interval, is at most the window.
    return run_injected_with_environment(
        fixture,
        repo,
        label,
        &fault,
        &[(CHILD_WAIT_VARIABLE, window.as_str()), ("CLI_GIT_WAIT_NOTICE_SECONDS", window.as_str())],
    );
}

/// A step that is held past its window does not end the commit: it is stopped and started again, the commit lands,
///  and the repository passes the checks every fault control makes.
#[test]
fn a_git_step_held_past_its_window_is_started_again_and_the_commit_lands() {
    if !strace_available() {
        eprintln!("skipped: strace is not installed, so no start was delayed");
        return;
    }
    let fixture: Fixture = fixture("child-window");
    let repo: PathBuf = staged_repository(&fixture, "repo");
    let ran: Ran = windowed_commit(&fixture, repo.as_path(), "child-window");
    let text: String = ran.text();
    assert!(ran.injected(), "no start was delayed, so nothing was tested: {text}");
    assert_eq!(ran.code(), 0, "a step held past its window ended the commit: {text}");
    assert_eq!(verify_recovery(&fixture, repo.as_path(), "child-window", text.as_str()), Ok(()));
    remove_settled(&fixture);
}
