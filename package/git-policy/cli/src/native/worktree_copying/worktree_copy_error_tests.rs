//! Diagnostics of the worktree-copy failure type.

use super::*;

/// A message without a cause is printed unchanged.
#[test]
fn message_without_cause_is_unchanged() {
    let failure: WorktreeCopyError = failure_with_cause("cli-git: could not inspect.", None);
    assert_eq!(
        failure,
        WorktreeCopyError(String::from("cli-git: could not inspect."))
    );
}

/// A cause follows the message after the incumbent's separator.
#[test]
fn cause_follows_message_with_separator() {
    let cause: std::io::Error = std::io::Error::other("disk gone");
    let failure: WorktreeCopyError =
        failure_with_cause("cli-git: could not inspect.", Some(&cause));
    assert_eq!(failure.0, "cli-git: could not inspect. Cause: disk gone");
}

/// A raw failure keeps its own text, and an owned message is never worded again.
#[test]
fn raw_failures_are_worded_only_once() {
    let cause: std::io::Error = std::io::Error::other("boom");
    let raw: WorktreeCopyError = raw_failure(&cause);
    assert_eq!(raw.0, "boom");
    assert!(is_raw_failure(&raw));
    let worded: WorktreeCopyError = wrap_raw(raw, "cli-git: could not inspect.");
    assert_eq!(worded.0, "cli-git: could not inspect. Cause: boom");
    assert_eq!(wrap_raw(worded.clone(), "cli-git: other."), worded);
}

/// The lifecycle words a raw failure with the generic sentence, and leaves an owned message as it is.
#[test]
fn lifecycle_uses_generic_sentence_for_raw_failures_only() {
    let raw: WorktreeCopyError = raw_failure(&std::io::Error::other("boom"));
    assert_eq!(
        lifecycle_failure(raw).0,
        format!("{AUTOMATIC_FAILURE_MESSAGE} Cause: boom")
    );
    let owned: WorktreeCopyError = WorktreeCopyError(String::from("cli-git: owned."));
    assert_eq!(lifecycle_failure(owned.clone()), owned);
}
