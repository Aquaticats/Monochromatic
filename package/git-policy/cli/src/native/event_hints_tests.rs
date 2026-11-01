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

use super::{SELF_HINTED, WARN_UNSAFE_HINT, engine_failure_hint, finding_hint, push_hint_field};
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
        EngineFailureCode::IndexLockUnprovenOwner,
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
            | EngineFailureCode::TransactionFailed
            | EngineFailureCode::IndexLockUnprovenOwner => {}
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
    assert!(traditional.starts_with("Hint: ") && traditional.contains("git cli-git doctor"), "{traditional}");
    assert_eq!(engine_failure_hint("not-a-code"), None);
}

/// Every finding code of every policy has a hint, or its message carries its own and is listed as self-hinted.
#[test]
fn every_finding_code_has_a_hint_or_its_own() {
    let hinted: [(&str, &str); 8] = [
        ("add-explicit", BULK_ADD_CODE),
        ("final-newline", FINAL_NEWLINE_CODE),
        ("markdown/autofix", MARKDOWN_AUTOFIX_CODE),
        ("markdown/autofix", MARKDOWN_VIOLATION_CODE),
        ("mono/forbidden-root-context", ROOT_CONTEXT_CODE),
        ("mono/dependent-version-bump", DEPENDENT_VERSION_STALE_CODE),
        ("mono/dependent-version-bump", DEPENDENT_VERSION_UNSUPPORTED_CODE),
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
    assert!(with.contains("git cli-git fix"));
    assert!(!without.contains("git cli-git fix"));
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
