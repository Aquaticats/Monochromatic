//! What:
//!  The guard that every finding code and every engine failure code carries a hint, and the wording of the hints.
//! Why:
//!  A new code added without a hint would ship a block that leaves the next move to guesswork; the exhaustive match
//!  makes adding a variant a compile error here until the new code is listed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // for (const code of allCodes) expect(hintFor(code) ?? selfHinted(code)).toBeDefined();
//! ```

use super::{SELF_HINTED, WARN_UNSAFE_HINT, engine_failure_hint, finding_hint, manual_push_failure_hint, push_hint_field};
use crate::manual_push_cause::ManualPushCause;
use crate::dependent_version_policy::{DEPENDENT_VERSION_STALE_CODE, DEPENDENT_VERSION_UNSUPPORTED_CODE};
use crate::diagnostics::{EngineFailureCode, LEGACY_CONFIG_IGNORED_CODE, engine_failure_code_name};
use crate::policy_final_newline::FINAL_NEWLINE_CODE;
use crate::policy_forbidden_strings::FORBIDDEN_STRING_CODE;
use crate::policy_markdown::{MARKDOWN_AUTOFIX_CODE, MARKDOWN_VIOLATION_CODE};
use crate::policy_root_context::ROOT_CONTEXT_CODE;
use crate::rule_add_explicit::BULK_ADD_CODE;
use crate::traditional_hook_warning::TRADITIONAL_HOOKS_UNSUPPORTED_CODE;

/// Every engine failure code, with an exhaustive match so a new variant stops compilation here.
fn every_engine_failure() -> Vec<EngineFailureCode> {
    let all = vec![
        EngineFailureCode::ConfigInvalid,
        EngineFailureCode::CoreIncomplete,
        EngineFailureCode::ContentUnavailable,
        EngineFailureCode::PolicyIncomplete,
        EngineFailureCode::PatchInvalid,
        EngineFailureCode::PatchConflict,
        EngineFailureCode::FixCycle,
        EngineFailureCode::FixPassLimit,
        EngineFailureCode::TransactionFailed,
    ];
    for code in &all {
        match code {
            EngineFailureCode::ConfigInvalid
            | EngineFailureCode::CoreIncomplete
            | EngineFailureCode::ContentUnavailable
            | EngineFailureCode::PolicyIncomplete
            | EngineFailureCode::PatchInvalid
            | EngineFailureCode::PatchConflict
            | EngineFailureCode::FixCycle
            | EngineFailureCode::FixPassLimit
            | EngineFailureCode::TransactionFailed => {}
        }
    }
    return all;
}

/// Every engine failure code has a hint that opens with the common marker and offers a next step.
#[test]
fn every_engine_failure_code_has_a_hint() {
    for code in every_engine_failure() {
        let name: &str = engine_failure_code_name(code);
        let hint: &str = engine_failure_hint(name).unwrap_or_else(|| panic!("{name} has no hint"));
        assert!(hint.starts_with("Hint: "), "{name}: {hint}");
        assert!(hint.len() > 80, "{name} is too thin to help: {hint}");
    }
    assert!(engine_failure_hint(LEGACY_CONFIG_IGNORED_CODE).is_some());
    let traditional: &str =
        engine_failure_hint(TRADITIONAL_HOOKS_UNSUPPORTED_CODE).expect("the traditional-hooks warning has a hint");
    assert!(traditional.starts_with("Hint: ") && traditional.contains("git monochromatic-git-policy-cli doctor"), "{traditional}");
    assert_eq!(engine_failure_hint("not-a-code"), None);
}

/// Every finding code of every policy has a hint, or its message carries its own and is listed as self-hinted.
#[test]
fn every_finding_code_has_a_hint_or_its_own() {
    let hinted: [(&str, &str); 14] = [
        ("add-explicit", BULK_ADD_CODE),
        ("final-newline", FINAL_NEWLINE_CODE),
        ("markdown/autofix", MARKDOWN_AUTOFIX_CODE),
        ("markdown/autofix", MARKDOWN_VIOLATION_CODE),
        ("mono/forbidden-root-context", ROOT_CONTEXT_CODE),
        ("mono/dependent-version-bump", DEPENDENT_VERSION_STALE_CODE),
        ("mono/dependent-version-bump", DEPENDENT_VERSION_UNSUPPORTED_CODE),
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
            let hint: &str = finding_hint(policy, code, fix_available)
                .unwrap_or_else(|| panic!("{policy}/{code} (fix {fix_available}) has no hint"));
            assert!(hint.starts_with("Hint: "), "{policy}/{code}: {hint}");
        }
    }
    for (policy, code) in SELF_HINTED {
        assert_eq!(finding_hint(policy, code, false), None, "{policy}/{code} would get two hints");
    }
    assert_eq!(finding_hint("final-newline", "unknown", false), None);
}

/// The final-newline hint differs by whether a correction exists, and names the direct fix only when it does.
#[test]
fn the_final_newline_hint_follows_the_availability_of_a_fix() {
    let with: &str = finding_hint("final-newline", FINAL_NEWLINE_CODE, true).expect("hint");
    let without: &str = finding_hint("final-newline", FINAL_NEWLINE_CODE, false).expect("hint");
    assert!(with.contains("git monochromatic-git-policy-cli fix"));
    assert!(!without.contains("git monochromatic-git-policy-cli fix"));
}

/// The hint field is appended as escaped JSON, and nothing is appended without a hint.
#[test]
fn the_hint_field_is_escaped_json_and_absent_without_a_hint() {
    let mut line: String = String::from("{\"message\":\"m\"");
    push_hint_field(&mut line, None);
    assert_eq!(line, "{\"message\":\"m\"");
    push_hint_field(&mut line, Some("Hint: a \"quoted\" word\nnext"));
    assert_eq!(line, "{\"message\":\"m\",\"hint\":\"Hint: a \\\"quoted\\\" word\\nnext\"");
    assert!(WARN_UNSAFE_HINT.starts_with("Hint: if you"));
}

/// Every cause of a manual-push discovery failure is listed here, so a new cause cannot be added without a decision about its hint.
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

/// Each cause of a manual-push discovery failure has its own hint, except the one whose message says what to do, and only
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
                assert_ne!(text, by_code, "{cause:?} must not claim that content is missing");
                assert!(!seen.contains(&text), "{cause:?} repeats another cause's hint: {text}");
                seen.push(text);
            }
        }
    }
}

/// The remedy each hint names is the one for its cause: run again, check the network and credentials, run Git's own
///  dry run, report the defect with the Git version, push a commit.
#[test]
fn each_hint_names_the_remedy_of_its_cause() {
    let hint = |cause: ManualPushCause| -> &'static str { return manual_push_failure_hint(cause).expect("a hint"); };
    assert!(hint(ManualPushCause::RemoteMoved).contains("again"));
    assert!(hint(ManualPushCause::RemoteUnreachable).contains("credentials") && hint(ManualPushCause::RemoteUnreachable).contains("ls-remote"));
    assert!(hint(ManualPushCause::GitNotStarted).contains("installation"));
    assert!(hint(ManualPushCause::GitSilent).contains("--dry-run"));
    assert!(hint(ManualPushCause::UnexpectedOutput).contains("git --version"));
    assert!(hint(ManualPushCause::UnsupportedObject).contains("commit"));
}
