//! What:
//!  The guard that every finding code and every engine failure code carries a hint,
//! and the wording of the hints.
//! Why:
//!  A new code added without a hint would ship a block that leaves the next move to guesswork;
//! the exhaustive match
//!  makes adding a variant a compile error here until the new code is listed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // for (const code of allCodes) expect(hintFor(code) ?? selfHinted(code)).toBeDefined();
//! ```

use super::{
    CommitCorrection,
    SELF_HINTED,
    finding_hint,
    manual_push_failure_hint,
};
use crate::dependent_version_policy::{
    DEPENDENT_VERSION_STALE_CODE,
    DEPENDENT_VERSION_UNSUPPORTED_CODE,
};
/// The hint of one engine-failure code,
/// which the missing-content cause answers with.
use crate::invocation_config::engine_failure_hint;
use crate::manual_push_cause::ManualPushCause;
use crate::policy_final_newline::FINAL_NEWLINE_CODE;
use crate::policy_forbidden_strings::FORBIDDEN_STRING_CODE;
use crate::policy_markdown::{
    MARKDOWN_AUTOFIX_CODE,
    MARKDOWN_VIOLATION_CODE,
};
use crate::policy_root_context::ROOT_CONTEXT_CODE;
use crate::rule_add_explicit::BULK_ADD_CODE;

/// Every finding code of every policy has a hint,
/// or its message carries its own and is listed as self-hinted,
///  whatever a commit does with the bytes it names.
#[test]
fn every_finding_code_has_a_hint_or_its_own() {
    let hinted: [(&str, &str); 14] = [
        ("add-explicit", BULK_ADD_CODE),
        ("final-newline", FINAL_NEWLINE_CODE),
        ("markdown/autofix", MARKDOWN_AUTOFIX_CODE),
        ("markdown/autofix", MARKDOWN_VIOLATION_CODE),
        ("mono/forbidden-root-context", ROOT_CONTEXT_CODE),
        ("mono/dependent-version-bump", DEPENDENT_VERSION_STALE_CODE),
        (
            "mono/dependent-version-bump",
            DEPENDENT_VERSION_UNSUPPORTED_CODE,
        ),
        ("mono/catalog-specifiers", "unsupported-manifest-format"),
        ("mono/catalog-specifiers", "invalid-manifest"),
        ("mono/catalog-specifiers", "malformed-section"),
        ("mono/catalog-specifiers", "duplicate-key"),
        ("mono/catalog-specifiers", "non-string-specifier"),
        ("mono/catalog-specifiers", "disallowed-specifier"),
        ("security/forbidden-strings", FORBIDDEN_STRING_CODE),
    ];
    for (policy, code) in hinted {
        for fix_available in [false, true] {
            // What: every answer a finding can carry about a commit, so a new case cannot print nothing.
            // Why:  the hint is chosen from both facts, and a case without words would leave the person guessing.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // for (const correction of [undefined, 'corrects', 'corrects-unless-the-selection-is-read-only', 'nothing-corrects-what-the-index-holds', 'nothing-corrects-what-a-commit-holds']) { /* ... */ }
            // ```
            for commit_correction in [
                None,
                Some(CommitCorrection::Corrects),
                Some(CommitCorrection::CorrectsUnlessTheSelectionIsReadOnly),
                Some(CommitCorrection::NothingCorrectsWhatTheIndexHolds),
                Some(CommitCorrection::NothingCorrectsWhatACommitHolds),
            ] {
                let hint: &str = finding_hint(policy, code, fix_available, commit_correction)
                    .unwrap_or_else(|| {
                        panic!(
                            "{policy}/{code} (fix {fix_available}, commit {commit_correction:?}) has no hint"
                        )
                    });
                assert!(hint.starts_with("Hint: "), "{policy}/{code}: {hint}");
            }
        }
    }
    for (policy, code) in SELF_HINTED {
        assert_eq!(
            finding_hint(policy, code, false, None),
            None,
            "{policy}/{code} would get two hints"
        );
    }
    assert_eq!(finding_hint("final-newline", "unknown", false, None), None);
}

/// Every place of the final-newline finding says what changes the bytes there,
///  and each says a step that works where it is printed.
#[test]
fn the_final_newline_hint_says_what_corrects_the_file() {
    let here: &str = finding_hint(
        "final-newline",
        FINAL_NEWLINE_CODE,
        true,
        Some(CommitCorrection::Corrects),
    )
    .expect("hint");
    let by_the_commit_that_takes_the_file: &str = finding_hint(
        "final-newline",
        FINAL_NEWLINE_CODE,
        false,
        Some(CommitCorrection::Corrects),
    )
    .expect("hint");
    let unless_the_selection_is_read_only: &str = finding_hint(
        "final-newline",
        FINAL_NEWLINE_CODE,
        false,
        Some(CommitCorrection::CorrectsUnlessTheSelectionIsReadOnly),
    )
    .expect("hint");
    let when_the_index_holds_them: &str = finding_hint(
        "final-newline",
        FINAL_NEWLINE_CODE,
        false,
        Some(CommitCorrection::NothingCorrectsWhatTheIndexHolds),
    )
    .expect("hint");
    let when_a_commit_holds_them: &str = finding_hint(
        "final-newline",
        FINAL_NEWLINE_CODE,
        false,
        Some(CommitCorrection::NothingCorrectsWhatACommitHolds),
    )
    .expect("hint");
    // What: the three places the correction is still ahead of the person name the fix.
    // Why:  those three cases share the promise that a command corrects the bytes, and two add the
    //       condition under which it does.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const hint of [here, byTheFileTaker, unlessReadOnly]) expect(hint).toContain('git monochromatic-git-policy-cli fix');
    // ```
    for hint in [
        here,
        by_the_commit_that_takes_the_file,
        unless_the_selection_is_read_only,
    ] {
        assert!(
            hint.contains("git monochromatic-git-policy-cli fix -- <path>"),
            "{hint}"
        );
        assert!(!hint.contains("edit it"), "{hint}");
    }
    assert!(here.contains("a commit corrects the ending"), "{here}");
    assert!(
        by_the_commit_that_takes_the_file.contains("a commit that takes it corrects the ending"),
        "{by_the_commit_that_takes_the_file}"
    );
    // What: an `add` leads with the fix and the staging after it, because an `add` may be refused.
    // Why:  a person whose add was refused can stage nothing until the file is corrected, and an index that
    //       already holds the bytes keeps them until it is staged once more (issue 6627's read, Low 3).
    assert!(
        unless_the_selection_is_read_only.starts_with(
            "Hint: to correct the ending now, run `git monochromatic-git-policy-cli fix -- <path>`"
        ),
        "{unless_the_selection_is_read_only}"
    );
    assert!(
        unless_the_selection_is_read_only
            .contains("then stage the file again with `git add -- <path>`"),
        "{unless_the_selection_is_read_only}"
    );
    assert!(
        unless_the_selection_is_read_only
            .contains("a commit whose selection is read-only (`-p`, `-i` or `--include`) records"),
        "{unless_the_selection_is_read_only}"
    );
    // What: a lifecycle whose bytes nothing corrects tells the person to edit the file, and names the step after it.
    // Why:  an `add` was told to edit a file the next commit corrected, which is issue 6627, and a push was told
    //       to stage bytes it had already recorded.
    assert!(
        when_the_index_holds_them
            .contains("edit it so it ends with exactly one newline, then stage it again"),
        "{when_the_index_holds_them}"
    );
    assert!(
        !when_the_index_holds_them.contains("corrects the ending"),
        "{when_the_index_holds_them}"
    );
    assert!(
        when_a_commit_holds_them.contains(
            "edit the file so it ends with exactly one newline, then commit the correction"
        ),
        "{when_a_commit_holds_them}"
    );
    assert!(
        !when_a_commit_holds_them.contains("corrects the ending"),
        "{when_a_commit_holds_them}"
    );
    assert!(
        !when_a_commit_holds_them.contains("stage it again"),
        "{when_a_commit_holds_them}"
    );
    // What: the five answers are five different sentences, so nothing printed reads as another place's promise.
    // Why:  one text reused would tell a person the wrong thing at one of the lifecycles.
    let distinct: [&str; 5] = [
        here,
        by_the_commit_that_takes_the_file,
        unless_the_selection_is_read_only,
        when_the_index_holds_them,
        when_a_commit_holds_them,
    ];
    for (position, hint) in distinct.iter().enumerate() {
        for other in distinct.iter().skip(position + 1) {
            assert_ne!(hint, other);
        }
    }
    assert_eq!(
        when_the_index_holds_them,
        finding_hint("final-newline", FINAL_NEWLINE_CODE, false, None).expect("hint")
    );
}

/// Every cause of a manual-push discovery failure is listed here,
/// so a new cause cannot be added without a decision about its hint.
fn every_manual_push_cause() -> Vec<ManualPushCause> {
    let mut all: Vec<ManualPushCause> = Vec::new();
    for cause in [
        ManualPushCause::ContentMissing,
        ManualPushCause::RemoteMoved,
        ManualPushCause::RemoteUnreachable,
        ManualPushCause::GitNotStarted,
        ManualPushCause::GitSilent,
        ManualPushCause::UnexpectedOutput,
        ManualPushCause::UnsupportedObject,
        ManualPushCause::SelfExplained,
    ] {
        // An exhaustive match: a new variant fails to compile until it is listed above.
        match cause {
            ManualPushCause::ContentMissing
            | ManualPushCause::RemoteMoved
            | ManualPushCause::RemoteUnreachable
            | ManualPushCause::GitNotStarted
            | ManualPushCause::GitSilent
            | ManualPushCause::UnexpectedOutput
            | ManualPushCause::UnsupportedObject
            | ManualPushCause::SelfExplained => all.push(cause),
        }
    }
    return all;
}

/// Each cause of a manual-push discovery failure has its own hint,
/// except the one whose message says what to do,
/// and only
///  the missing-content cause keeps the hint of the `content-unavailable` code.
#[test]
fn each_manual_push_cause_has_its_own_hint() {
    let by_code: &str = engine_failure_hint("content-unavailable").expect("the code's hint");
    let mut seen: Vec<&str> = Vec::new();
    for cause in every_manual_push_cause() {
        let hint: Option<&str> = manual_push_failure_hint(cause);
        match cause {
            ManualPushCause::SelfExplained => assert_eq!(hint, None, "{cause:?}"),
            ManualPushCause::ContentMissing => assert_eq!(hint, Some(by_code)),
            ManualPushCause::RemoteMoved
            | ManualPushCause::RemoteUnreachable
            | ManualPushCause::GitNotStarted
            | ManualPushCause::GitSilent
            | ManualPushCause::UnexpectedOutput
            | ManualPushCause::UnsupportedObject => {
                let text: &str = hint.unwrap_or_else(|| panic!("{cause:?} has no hint"));
                assert!(text.starts_with("Hint: "), "{cause:?}: {text}");
                assert_ne!(
                    text, by_code,
                    "{cause:?} must not claim that content is missing"
                );
                assert!(
                    !seen.contains(&text),
                    "{cause:?} repeats another cause's hint: {text}"
                );
                seen.push(text);
            }
        }
    }
}

/// The remedy each hint names is the one for its cause:
/// run again,
/// check the network and credentials,
/// run Git's own
///  dry run,
/// report the defect with the Git version,
/// push a commit.
#[test]
fn each_hint_names_the_remedy_of_its_cause() {
    let hint = |cause: ManualPushCause| -> &'static str {
        return manual_push_failure_hint(cause).expect("a hint");
    };
    assert!(hint(ManualPushCause::RemoteMoved).contains("again"));
    assert!(
        hint(ManualPushCause::RemoteUnreachable).contains("credentials")
            && hint(ManualPushCause::RemoteUnreachable).contains("ls-remote")
    );
    assert!(hint(ManualPushCause::GitNotStarted).contains("installation"));
    assert!(hint(ManualPushCause::GitSilent).contains("--dry-run"));
    assert!(hint(ManualPushCause::UnexpectedOutput).contains("git --version"));
    assert!(hint(ManualPushCause::UnsupportedObject).contains("commit"));
}
