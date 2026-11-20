//! What:
//!  Controls of the command line's statuses:
//!  the help and version words answer zero,
//!  every unreadable invocation and every unbuilt mode answers the refusal status,
//!  and no refusal reaches zero.
//! Why:
//!  A boot unit's `ExecStart` and `ExecStopPost` read this status,
//!  so a mode that refuses while answering zero would report a reconciled boot that never happened
//!  (`handler_mode_dispatch`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(run(['--help'])).toBe(0); expect(run([])).toBe(2);
//! ```

use super::{EXIT_DONE, EXIT_REFUSED, run};
use std::ffi::OsString;

/// The arguments of one invocation,
/// as the kernel hands them over.
fn arguments(words: &[&str]) -> Vec<OsString> {
    return words.iter().map(|word| return OsString::from(*word)).collect();
}

/// The two statuses are zero and the wrapper's own refusal status.
#[test]
fn the_two_statuses_are_zero_and_the_refusal_status() {
    assert_eq!(EXIT_DONE, 0);
    assert_eq!(EXIT_REFUSED, 2);
}

/// The words that ask what this build is answer zero.
#[test]
fn the_help_and_version_words_answer_zero() {
    assert_eq!(run(arguments(&["--help"]).as_slice()), EXIT_DONE);
    assert_eq!(run(arguments(&["--version"]).as_slice()), EXIT_DONE);
}

/// Every unbuilt mode refuses,
/// and the default invocation is one of them.
#[test]
fn every_unbuilt_mode_answers_the_refusal_status() {
    assert_eq!(run(arguments(&[]).as_slice()), EXIT_REFUSED);
    assert_eq!(run(arguments(&["boot-reconcile"]).as_slice()), EXIT_REFUSED);
    assert_eq!(run(arguments(&["shutdown-unmount"]).as_slice()), EXIT_REFUSED);
}

/// An unreadable invocation is a refusal,
/// never a mode taken by default.
#[test]
fn an_unreadable_invocation_answers_the_refusal_status() {
    assert_eq!(run(arguments(&["sweep"]).as_slice()), EXIT_REFUSED);
    assert_eq!(run(arguments(&["boot-reconcile", "--force"]).as_slice()), EXIT_REFUSED);
    assert_eq!(run(arguments(&["adopt-worktree-parent"]).as_slice()), EXIT_REFUSED);
    assert_eq!(run(arguments(&["adopt-worktree-parent", "--checkout"]).as_slice()), EXIT_REFUSED);
}
