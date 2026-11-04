//! Controls for the no-change message.

use super::no_change_message;

/// Without conflicts the message says the copies were reconciled.
#[test]
fn a_clean_reconciliation_says_so() {
    let message: String = no_change_message(&[]);
    assert!(message.starts_with("No commit created: normalization removed every selected change; matching worktree and index copies were reconciled."), "{message}");
    assert!(message.contains(" Hint: if you were committing only whitespace or final-newline edits"), "{message}");
}

/// Conflicted worktree copies are named as a JSON list, as the incumbent writes them.
#[test]
fn kept_worktree_copies_are_named() {
    let message: String = no_change_message(&[b"a.txt".to_vec(), b"b \"c\".txt".to_vec()]);
    assert!(message.starts_with("No commit created: normalization removed every selected change; index copies were reconciled, but worktree copies changed and were kept: [\"a.txt\",\"b \\\"c\\\".txt\"]. Compare them with HEAD before retrying."), "{message}");
    assert!(message.contains(" Hint: if you were committing only whitespace or final-newline edits"), "{message}");
}

/// A name that is not UTF-8 shows its readable text in the message.
#[test]
fn a_kept_copy_that_is_not_utf8_shows_replacement_characters() {
    let message: String = no_change_message(&[b"caf\xe9.txt".to_vec()]);
    assert!(message.starts_with("No commit created: normalization removed every selected change; index copies were reconciled, but worktree copies changed and were kept: [\"caf\u{fffd}.txt\"]. Compare them with HEAD before retrying."), "{message}");
    assert!(message.contains(" Hint: if you were committing only whitespace or final-newline edits"), "{message}");
}
