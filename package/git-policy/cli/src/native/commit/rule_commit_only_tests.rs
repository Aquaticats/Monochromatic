//! What:
//!  Every `commit-only.unit.test.ts` case,
//!  ported to the pure decision.
//! Why:
//!  The TypeScript test injected checker stubs;
//!  here a decision that needs no repository
//!      fact simply is not a `Needs...` variant,
//!  and one that does is resolved explicitly.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(await commitOnlyStateless(['commit', '-m', 'message', 'file.ts'])).toEqual([...]);
//! ```

/// The decision under test,
///  the fact types and the shared argument builder.
use super::{
    CommitOnlyDecision, CommitOnlyViolationCode, decide_commit_only, resolve_index_state,
    resolve_sequencer_state,
};
use crate::command_test_support::os_arguments;
use crate::rule_commit_index::IndexVsHead;
use crate::rule_commit_sequencer::SequencerState;
use std::ffi::OsString;

/// Decide from arguments alone;
///  the TypeScript "stateless" rule with forbidden checkers.
fn decide(values: &[&str]) -> CommitOnlyDecision {
    return decide_commit_only(os_arguments(values).as_slice(), &[]).expect("Git accepts");
}

/// The expected rewritten list.
fn rewritten(values: &[&str]) -> CommitOnlyDecision {
    return CommitOnlyDecision::Rewritten(os_arguments(values));
}

/// Decide,
///  require the index question,
///  and answer it.
fn decide_with_index(values: &[&str], state: IndexVsHead) -> CommitOnlyDecision {
    let arguments: Vec<OsString> = os_arguments(values);
    let first: CommitOnlyDecision =
        decide_commit_only(arguments.as_slice(), &[]).expect("Git accepts");
    if let CommitOnlyDecision::NeedsIndexState(pending) = &first {
        return resolve_index_state(arguments.as_slice(), pending, state);
    }
    panic!("{values:?} was decided without the index state: {first:?}");
}

/// Decide,
///  require the sequencer question,
///  and answer it.
fn decide_with_sequencer(values: &[&str], state: SequencerState) -> CommitOnlyDecision {
    let first: CommitOnlyDecision = decide(values);
    assert_eq!(first, CommitOnlyDecision::NeedsSequencerState, "{values:?}");
    return resolve_sequencer_state(state);
}

/// The code and message of a rejection.
fn rejection(decision: CommitOnlyDecision) -> (CommitOnlyViolationCode, String) {
    if let CommitOnlyDecision::Rejected(violation) = &decision {
        return (violation.code, violation.message.clone());
    }
    panic!("expected a rejection, got {decision:?}");
}

/// Ported:
///  "passes non-commit commands through unchanged",
///  "injects -o immediately after
/// commit when pathspec is present" and "preserves global options before commit".
#[test]
fn injects_only_after_commit_and_nowhere_else() {
    assert_eq!(
        decide(&["status", "--short"]),
        CommitOnlyDecision::Unchanged
    );
    assert_eq!(
        decide(&["commit", "-m", "message", "file.ts"]),
        rewritten(&["commit", "-o", "-m", "message", "file.ts"])
    );
    assert_eq!(
        decide(&["-C", "/fixture/repo", "commit", "-m", "message", "file.ts"]),
        rewritten(&[
            "-C",
            "/fixture/repo",
            "commit",
            "-o",
            "-m",
            "message",
            "file.ts"
        ])
    );
}

/// Ported:
///  "strips escape hatch and skips validation".
#[test]
fn strips_the_escape_hatch_and_skips_validation() {
    assert_eq!(
        decide(&["commit", "--no-enforce-only", "-am", "message"]),
        rewritten(&["commit", "-am", "message"])
    );
}

/// Ported:
///  "skips injection when explicit -o is present",
///  "... clustered -o ..." and
/// "allows --no-only as explicit opt-out without pathspec".
#[test]
fn skips_injection_for_an_explicit_only_choice() {
    for values in [
        vec!["commit", "-o", "-m", "message", "file.ts"],
        vec!["commit", "-om", "message", "file.ts"],
        vec!["commit", "--no-only", "-m", "message"],
    ] {
        assert_eq!(
            decide(values.as_slice()),
            CommitOnlyDecision::Unchanged,
            "{values:?}"
        );
    }
}

/// Ported:
///  "does not mistake message text for -a flag",
///  "treats dash-leading tokens after --
/// as pathspecs" and "treats lone dash before -- as pathspec".
#[test]
fn reads_values_and_pathspecs_by_position() {
    for (values, expected) in [
        (
            vec!["commit", "-m", "-a", "file.ts"],
            vec!["commit", "-o", "-m", "-a", "file.ts"],
        ),
        (
            vec!["commit", "-m", "message", "--", "-dash-file"],
            vec!["commit", "-o", "-m", "message", "--", "-dash-file"],
        ),
        (
            vec!["commit", "-m", "message", "-"],
            vec!["commit", "-o", "-m", "message", "-"],
        ),
    ] {
        assert_eq!(
            decide(values.as_slice()),
            rewritten(expected.as_slice()),
            "{values:?}"
        );
    }
}

/// Ported:
///  "rejects pathless --amend when index differs from HEAD" and
/// "rejects pathless --allow-empty when index differs from HEAD".
#[test]
fn rejects_a_pathless_amend_or_empty_commit_over_a_dirty_index() {
    let (amend_code, amend_message): (CommitOnlyViolationCode, String) = rejection(
        decide_with_index(&["commit", "--amend", "--no-edit"], IndexVsHead::Differs),
    );
    assert_eq!(amend_code, CommitOnlyViolationCode::StagedChangesIgnored);
    assert!(amend_message.contains(
        "git commit --amend without pathspecs would silently ignore your staged changes"
    ));
    assert!(amend_message.contains("--no-only"));
    let (_, empty_message): (CommitOnlyViolationCode, String) = rejection(decide_with_index(
        &["commit", "--allow-empty", "-m", "message"],
        IndexVsHead::Differs,
    ));
    assert!(empty_message.contains(
        "git commit --allow-empty without pathspecs would silently ignore your staged changes"
    ));
}

/// Ported:
///  "injects -o for pathless --amend when index state is undeterminable" and the
/// clean-index `--amend`/`--allow-empty` rows of "allows pathless ...".
#[test]
fn injects_for_a_pathless_amend_over_a_clean_or_unknown_index() {
    for (values, state, expected) in [
        (
            vec!["commit", "--amend", "-m", "message"],
            IndexVsHead::Unknown,
            vec!["commit", "-o", "--amend", "-m", "message"],
        ),
        (
            vec!["commit", "--amend", "-m", "message"],
            IndexVsHead::Matches,
            vec!["commit", "-o", "--amend", "-m", "message"],
        ),
        (
            vec!["commit", "--allow-empty", "-m", "message"],
            IndexVsHead::Matches,
            vec!["commit", "-o", "--allow-empty", "-m", "message"],
        ),
    ] {
        assert_eq!(
            decide_with_index(values.as_slice(), state),
            rewritten(expected.as_slice()),
            "{values:?}"
        );
    }
}

/// Ported:
///  the four "skips index check ..." cases and the two pathspec-file rows of
/// "allows pathless ...".
#[test]
fn skips_the_index_check_when_paths_or_an_only_choice_are_given() {
    for line in [
        "commit --amend --no-edit file.ts",
        "commit --amend --pathspec-from-file paths.txt -m message",
        "commit --pathspec-from-file paths.txt -m message",
        "commit --pathspec-from-file=paths.txt -m message",
    ] {
        // Each expected list is the input with `-o` right after `commit`.
        let values: Vec<&str> = line.split(' ').collect();
        let mut expected: Vec<&str> = vec!["commit", "-o"];
        expected.extend_from_slice(&values[1..]);
        assert_eq!(
            decide(values.as_slice()),
            rewritten(expected.as_slice()),
            "{line}"
        );
    }
    for values in [
        vec!["commit", "--amend", "--no-only", "-m", "message"],
        vec!["commit", "-o", "--amend", "-m", "message"],
    ] {
        assert_eq!(
            decide(values.as_slice()),
            CommitOnlyDecision::Unchanged,
            "{values:?}"
        );
    }
}

/// Ported:
///  the four "skips injection when ... include flag ..." cases.
#[test]
fn skips_injection_in_include_mode() {
    for values in [
        vec!["commit", "-i", "-m", "message", "file.ts"],
        vec!["commit", "--include", "-m", "message", "file.ts"],
        vec!["commit", "--inc", "-m", "message", "file.ts"],
        vec!["commit", "-im", "message", "file.ts"],
    ] {
        assert_eq!(
            decide(values.as_slice()),
            CommitOnlyDecision::Unchanged,
            "{values:?}"
        );
    }
}

/// Ported:
///  "passes pathless commit through unchanged during sequencer conclusion",
/// "still rejects -a during sequencer conclusion",
///  "pathless rejection names --no-only as
/// an explicit choice",
///  and the two pathless rows of "rejects ...".
#[test]
fn asks_for_the_sequencer_state_only_for_a_pathless_commit() {
    assert_eq!(
        decide_with_sequencer(
            &["commit", "-m", "merge main into feature"],
            SequencerState::InProgress
        ),
        CommitOnlyDecision::Unchanged
    );
    for values in [
        vec!["commit", "-m", "message"],
        vec![
            "commit",
            "--author",
            "Author <author@example.invalid>",
            "-m",
            "message",
        ],
    ] {
        let (code, message): (CommitOnlyViolationCode, String) = rejection(decide_with_sequencer(
            values.as_slice(),
            SequencerState::NotInProgress,
        ));
        assert_eq!(
            code,
            CommitOnlyViolationCode::PathspecRequired,
            "{values:?}"
        );
        assert!(
            message.contains("requires an explicit pathspec"),
            "{values:?}"
        );
        assert!(message.contains("--no-only"), "{values:?}");
    }
}

/// Ported:
///  the three `-a` rows of "rejects ..." and "still rejects -a during sequencer
/// conclusion":
///  the rejection needs no repository fact.
#[test]
fn rejects_the_all_flag_from_arguments_alone() {
    for values in [
        vec!["commit", "-a", "-m", "message"],
        vec!["commit", "--all", "-m", "message"],
        vec!["commit", "-am", "message"],
    ] {
        let (code, message): (CommitOnlyViolationCode, String) =
            rejection(decide(values.as_slice()));
        assert_eq!(code, CommitOnlyViolationCode::AllFlag, "{values:?}");
        assert!(message.contains("rejects -a/--all"), "{values:?}");
    }
}

/// Ported:
///  the five "detects pathspec after no-value flag ..." cases and the four
/// "detects pathspec after value option ..." cases.
#[test]
fn detects_a_pathspec_after_flags_and_value_options() {
    for values in [
        vec!["commit", "-q", "file.ts", "-F", "message.txt"],
        vec!["commit", "-v", "file.ts", "-m", "message"],
        vec!["commit", "-n", "file.ts", "-m", "message"],
        vec!["commit", "--no-verify", "file.ts", "-m", "message"],
        vec!["commit", "--dry-run", "file.ts", "-m", "message"],
        vec![
            "commit",
            "--author",
            "Author <author@example.invalid>",
            "file.ts",
            "-m",
            "message",
        ],
        vec!["commit", "--cleanup", "strip", "file.ts", "-m", "message"],
        vec![
            "commit",
            "--trailer",
            "Reviewed-by: A <a@example.invalid>",
            "file.ts",
            "-m",
            "m",
        ],
        vec!["commit", "--fixup", "HEAD", "file.ts"],
    ] {
        let mut expected: Vec<&str> = vec!["commit", "-o"];
        expected.extend_from_slice(&values[1..]);
        assert_eq!(
            decide(values.as_slice()),
            rewritten(expected.as_slice()),
            "{values:?}"
        );
    }
}
