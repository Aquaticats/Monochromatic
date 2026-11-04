//! What:
//!  Commit-only decisions the incumbent test did not cover:
//!  readings where Git 2.56.0
//!       differs from the incumbent parser,
//!  `--fixup` suboptions,
//!  global-option outcomes,
//!       wrapper flags and Git's own refusals.
//! Why:
//!  Each is a branch of the decision;
//!  real-Git controls for the Git behavior these
//!      rely on are in `command_commit_git_tests.rs` and `rule_commit_only_git_tests.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(await commitOnly(['commit', '-qa', '-m', 'x'])).rejects.toThrow('rejects -a/--all');
//! ```

/// The decision under test,
///  the fact types and the shared argument builders.
use crate::command_options::OptionErrorKind;
use crate::command_test_support::os_arguments;
use crate::rule_commit_index::IndexVsHead;
use crate::rule_commit_only::{
    CommitOnlyDecision, CommitOnlyViolationCode, PendingInjection, decide_commit_only,
    resolve_index_state, violation_code_text,
};
use std::ffi::OsString;

/// Decide from arguments alone with no other wrapper flags.
fn decide(values: &[&str]) -> CommitOnlyDecision {
    return decide_commit_only(os_arguments(values).as_slice(), &[]).expect("Git accepts");
}

/// The expected rewritten list.
fn rewritten(values: &[&str]) -> CommitOnlyDecision {
    return CommitOnlyDecision::Rewritten(os_arguments(values));
}

/// Whether a decision is the all-flag rejection.
fn is_all_flag_rejection(decision: &CommitOnlyDecision) -> bool {
    if let CommitOnlyDecision::Rejected(violation) = decision {
        return violation.code == CommitOnlyViolationCode::AllFlag;
    }
    return false;
}

/// Divergence:
///  `-a` is found in any cluster,
///  and a later `--no-all` cancels it.
#[test]
fn judges_the_final_all_state_in_any_spelling() {
    for values in [
        vec!["commit", "-qa", "-m", "x"],
        vec!["commit", "-vam", "x"],
        vec!["commit", "--all", "file.ts"],
        vec!["commit", "--no-all", "-a", "file.ts"],
    ] {
        assert!(
            is_all_flag_rejection(&decide(values.as_slice())),
            "{values:?}"
        );
    }
    assert_eq!(
        decide(&["commit", "-a", "--no-all", "-m", "x", "file.ts"]),
        rewritten(&["commit", "-o", "-a", "--no-all", "-m", "x", "file.ts"])
    );
    // `--no-only` is the explicit whole-index choice, with or without `-a`.
    assert_eq!(
        decide(&["commit", "--no-only", "-a", "-m", "x"]),
        CommitOnlyDecision::Unchanged
    );
    // A later `--only` withdraws the opt-out, so `-a` is judged again.
    assert!(is_all_flag_rejection(&decide(&[
        "commit",
        "--no-only",
        "--only",
        "-a"
    ])));
}

/// Divergence:
///  only option-position hatch tokens are removed;
///  the incumbent removed every
/// equal token,
///  including a message value and a path after `--`.
#[test]
fn removes_the_escape_hatch_by_position() {
    assert_eq!(
        decide(&[
            "-C",
            "/r",
            "commit",
            "--no-enforce-only",
            "-m",
            "--no-enforce-only",
            "--no-enforce-only",
            "--",
            "--no-enforce-only"
        ]),
        rewritten(&[
            "-C",
            "/r",
            "commit",
            "-m",
            "--no-enforce-only",
            "--",
            "--no-enforce-only"
        ])
    );
    // In value or path position the token is not a hatch, so enforcement applies.
    assert_eq!(
        decide(&["commit", "-m", "--no-enforce-only", "file.ts"]),
        rewritten(&["commit", "-o", "-m", "--no-enforce-only", "file.ts"])
    );
    assert_eq!(
        decide(&["commit", "-m", "x", "--", "--no-enforce-only"]),
        rewritten(&["commit", "-o", "-m", "x", "--", "--no-enforce-only"])
    );
}

/// `--fixup=reword:` owns `--only` itself;
///  `--fixup=amend:` is a pathless-allowed mode;
/// a plain `--fixup` still needs a pathspec source.
#[test]
fn follows_git_for_fixup_suboptions() {
    for values in [
        vec!["commit", "--fixup=reword:HEAD"],
        vec!["commit", "--fixup", "reword:HEAD", "-m", "x"],
        vec!["commit", "--fixup=squash:HEAD"],
    ] {
        assert_eq!(
            decide(values.as_slice()),
            CommitOnlyDecision::Unchanged,
            "{values:?}"
        );
    }
    let amend: Vec<OsString> = os_arguments(&["commit", "--fixup=amend:HEAD"]);
    let first: CommitOnlyDecision = decide_commit_only(amend.as_slice(), &[]).expect("accepted");
    let pending: &PendingInjection = if let CommitOnlyDecision::NeedsIndexState(found) = &first {
        found
    } else {
        panic!("expected the index question, got {first:?}");
    };
    assert_eq!(pending.flag_text, "--fixup=amend:<commit>");
    assert_eq!(
        resolve_index_state(amend.as_slice(), pending, IndexVsHead::Matches),
        rewritten(&["commit", "-o", "--fixup=amend:HEAD"])
    );
    let dirty: CommitOnlyDecision =
        resolve_index_state(amend.as_slice(), pending, IndexVsHead::Differs);
    if let CommitOnlyDecision::Rejected(violation) = &dirty {
        assert!(
            violation
                .message
                .contains("git commit --fixup=amend:<commit> without")
        );
    } else {
        panic!("expected a rejection, got {dirty:?}");
    }
    assert_eq!(
        decide(&["commit", "--fixup=HEAD"]),
        CommitOnlyDecision::NeedsSequencerState
    );
    // Both pathless-allowed flags are echoed together.
    let both: CommitOnlyDecision = decide(&["commit", "--amend", "--allow-empty"]);
    if let CommitOnlyDecision::NeedsIndexState(pending_both) = &both {
        assert_eq!(pending_both.flag_text, "--amend --allow-empty");
        assert_eq!(pending_both.command_index, 0);
    } else {
        panic!("expected the index question, got {both:?}");
    }
}

/// Interactive and patch selection are pathspec sources and Git owns their only semantics.
#[test]
fn leaves_interactive_and_patch_selection_to_git() {
    for values in [
        vec!["commit", "--interactive"],
        vec!["commit", "-p", "-m", "x"],
        vec!["commit", "--patch", "file.ts"],
    ] {
        assert_eq!(
            decide(values.as_slice()),
            CommitOnlyDecision::Unchanged,
            "{values:?}"
        );
    }
    // A negated selection is not a source.
    assert_eq!(
        decide(&["commit", "-p", "--no-patch", "-m", "x"]),
        CommitOnlyDecision::NeedsSequencerState
    );
}

/// Global forms that run no subcommand,
///  and `commit` in a value position,
///  are untouched.
#[test]
fn acts_only_on_a_real_commit_subcommand() {
    for values in [
        vec![],
        vec!["--version", "commit"],
        vec!["-C"],
        vec!["--bogus", "commit", "-m", "x"],
        vec!["-C", "commit", "status"],
        vec!["-c", "alias.x=commit", "x", "-m", "m"],
    ] {
        assert_eq!(
            decide(values.as_slice()),
            CommitOnlyDecision::Unchanged,
            "{values:?}"
        );
    }
}

/// A command line Git itself refuses is reported,
///  never decided from guessed facts.
#[test]
fn reports_what_git_refuses() {
    for (values, kind) in [
        (
            vec!["commit", "--unknown", "file.ts"],
            OptionErrorKind::UnknownOption,
        ),
        (
            vec!["commit", "file.ts", "-m"],
            OptionErrorKind::MissingValue,
        ),
        (
            vec!["commit", "-all"],
            OptionErrorKind::SingleDashLongOption,
        ),
        (vec!["commit", "--a"], OptionErrorKind::AmbiguousOption),
        (vec!["commit", "-h"], OptionErrorKind::HelpRequested),
    ] {
        assert_eq!(
            decide_commit_only(os_arguments(values.as_slice()).as_slice(), &[])
                .expect_err("refused")
                .kind,
            kind,
            "{values:?}"
        );
    }
}

/// Other wrapper flags are tolerated and kept;
///  the caller removes them by position.
#[test]
fn tolerates_other_wrapper_flags() {
    let arguments: Vec<OsString> =
        os_arguments(&["commit", "--cli-git-keep-going", "-m", "x", "file.ts"]);
    assert_eq!(
        decide_commit_only(arguments.as_slice(), &[b"--cli-git-keep-going"]),
        Ok(rewritten(&[
            "commit",
            "-o",
            "--cli-git-keep-going",
            "-m",
            "x",
            "file.ts"
        ]))
    );
    assert!(decide_commit_only(arguments.as_slice(), &[]).is_err());
}

/// Event code spellings are stable.
#[test]
fn spells_violation_codes() {
    assert_eq!(
        violation_code_text(CommitOnlyViolationCode::AllFlag),
        "all-flag"
    );
    assert_eq!(
        violation_code_text(CommitOnlyViolationCode::PathspecRequired),
        "pathspec-required"
    );
    assert_eq!(
        violation_code_text(CommitOnlyViolationCode::StagedChangesIgnored),
        "staged-changes-ignored"
    );
}

/// Bytes that are not UTF-8 survive the rewrite unchanged.
#[cfg(unix)]
#[test]
fn keeps_non_utf8_bytes_when_rewriting() {
    use crate::command_test_support::byte_argument;
    let arguments: Vec<OsString> = vec![
        OsString::from("commit"),
        byte_argument(b"-m\xff"),
        byte_argument(b"dir/\xfe.txt"),
    ];
    let expected: Vec<OsString> = vec![
        OsString::from("commit"),
        OsString::from("-o"),
        byte_argument(b"-m\xff"),
        byte_argument(b"dir/\xfe.txt"),
    ];
    assert_eq!(
        decide_commit_only(arguments.as_slice(), &[]),
        Ok(CommitOnlyDecision::Rewritten(expected))
    );
}
