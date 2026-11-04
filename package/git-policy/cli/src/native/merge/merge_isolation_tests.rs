//! Controls for the decisions of the isolated merge that need no repository.
//! The merge itself is checked against real Git through the built executable, in `binary_merge_isolation_tests.rs`.

use super::*;

/// Git's boolean spellings, in any letter case; anything else is no answer.
#[test]
fn boolean_settings_read_as_git_reads_them() {
    for text in ["true", "TRUE", "Yes", "on", "1"] {
        assert_eq!(parse_maybe_bool(text), Some(true), "{text}");
    }
    for text in ["false", "No", "OFF", "0", ""] {
        assert_eq!(parse_maybe_bool(text), Some(false), "{text}");
    }
    for text in ["maybe", "2", "only", "ture"] {
        assert_eq!(parse_maybe_bool(text), None, "{text}");
    }
}

/// `GIT_MERGE_AUTOEDIT` wins over the terminal test; with it unset, a shared terminal opens the editor.
#[test]
fn the_editor_follows_the_environment_then_the_terminal() {
    assert_eq!(editor_would_open(Some("no"), true), Some(false));
    assert_eq!(editor_would_open(Some("yes"), false), Some(true));
    assert_eq!(editor_would_open(None, true), Some(true));
    assert_eq!(editor_would_open(None, false), Some(false));
    assert_eq!(editor_would_open(Some("sometimes"), false), None);
}

/// The markers of other operations in progress are the ones Git itself reports.
#[test]
fn the_operations_in_progress_are_the_ones_git_checks() {
    for name in ["MERGE_HEAD", "CHERRY_PICK_HEAD", "REVERT_HEAD", "rebase-merge", "rebase-apply", "sequencer"] {
        assert!(IN_PROGRESS_MARKERS.contains(&name), "{name}");
    }
}
