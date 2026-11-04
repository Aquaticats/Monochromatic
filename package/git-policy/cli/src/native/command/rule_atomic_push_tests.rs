//! What:
//!  Every `atomic-push.unit.test.ts` case,
//!  then the spellings the incumbent missed.
//! Why:
//!  Injecting `--atomic` over an explicit `--no-at` would override the caller's choice.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(atomicPush(['push', 'origin', 'main'])).toEqual(['push', '--atomic', 'origin', 'main']);
//! ```

/// The decision under test,
///  its result type and the shared argument builder.
use super::atomic_push;
use crate::command_options::{OptionError, OptionErrorKind};
use crate::command_test_support::os_arguments;
use crate::rule_argument_rewrite::ArgumentRewrite;

/// Decide with no wrapper flags.
fn decide(values: &[&str]) -> Result<ArgumentRewrite, OptionError> {
    return atomic_push(os_arguments(values).as_slice(), &[]);
}

/// The expected rewritten list.
fn rewritten(values: &[&str]) -> Result<ArgumentRewrite, OptionError> {
    return Ok(ArgumentRewrite::Rewritten(os_arguments(values)));
}

/// Ported:
///  "passes non-push commands through unchanged".
#[test]
fn passes_non_push_commands_through_unchanged() {
    for values in [
        vec!["status", "--short"],
        vec![],
        vec!["--version", "push"],
        vec!["-C", "push", "status"],
        vec!["--bogus", "push"],
        vec!["-c"],
    ] {
        assert_eq!(
            decide(values.as_slice()),
            Ok(ArgumentRewrite::Unchanged),
            "{values:?}"
        );
    }
}

/// Ported:
///  "injects --atomic immediately after push" and "preserves global options before
/// push".
#[test]
fn injects_atomic_immediately_after_push() {
    assert_eq!(
        decide(&["push", "origin", "main"]),
        rewritten(&["push", "--atomic", "origin", "main"])
    );
    assert_eq!(
        decide(&["-C", "/fixture/repo", "push", "origin", "main"]),
        rewritten(&["-C", "/fixture/repo", "push", "--atomic", "origin", "main"])
    );
    assert_eq!(decide(&["push"]), rewritten(&["push", "--atomic"]));
}

/// Ported:
///  "skips injection when --atomic is already present" and "... --no-atomic ...";
/// divergence:
///  the abbreviations Git accepts are choices too.
#[test]
fn skips_injection_when_the_caller_chose() {
    for values in [
        vec!["push", "--atomic", "origin", "main"],
        vec!["push", "--no-atomic", "origin", "main"],
        vec!["push", "origin", "--at"],
        vec!["push", "--no-at", "origin"],
    ] {
        assert_eq!(
            decide(values.as_slice()),
            Ok(ArgumentRewrite::Unchanged),
            "{values:?}"
        );
    }
}

/// Divergence:
///  `--atomic` as an option value or after `--` is not a choice.
#[test]
fn injects_when_atomic_is_only_a_value_or_a_refspec() {
    assert_eq!(
        decide(&["push", "-o", "--atomic", "origin"]),
        rewritten(&["push", "--atomic", "-o", "--atomic", "origin"])
    );
    assert_eq!(
        decide(&["push", "origin", "--", "--atomic"]),
        rewritten(&["push", "--atomic", "origin", "--", "--atomic"])
    );
}

/// A push Git refuses is reported;
///  other wrapper flags are tolerated and kept.
#[test]
fn reports_refusals_and_tolerates_wrapper_flags() {
    assert_eq!(
        decide(&["push", "--unknown"]),
        Err(OptionError {
            kind: OptionErrorKind::UnknownOption,
            token: 0
        })
    );
    assert_eq!(
        atomic_push(
            os_arguments(&["push", "--cli-git-keep-going", "origin"]).as_slice(),
            &[b"--cli-git-keep-going"]
        ),
        rewritten(&["push", "--atomic", "--cli-git-keep-going", "origin"])
    );
}
