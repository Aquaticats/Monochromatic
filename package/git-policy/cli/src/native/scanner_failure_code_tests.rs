//! What:
//!  Controls for the failure-code mapping,
//!  over every variant of every mapped type.
//! Why:
//!  The code decides how an engine failure is reported.
//!  A cause mapped to the wrong
//!      code still fails closed,
//!  so only a control that names each cause can notice it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(candidateFailureCode('stale-candidate')).toBe('policy-incomplete');
//! ```

/// Import the mappings under test.
use super::{
    candidate_failure_code, finding_failure_code, scan_run_failure_code, scanner_failure_code,
};
/// Import the failures being mapped and the codes they map to.
use crate::candidate_error::{CandidateError, CandidateFailure};
use crate::diagnostics::{EngineFailureCode, engine_failure_code_name};
use crate::scanner_adapter::{ScannerError, ScannerFailure};
use crate::scanner_run::ScanRunError;
use forbidden_strings::ScanFinding;

/// Every candidate-layer cause that means "Git could not list or hand over content",
/// including a private index that could not be prepared for a prediction.
const UNREADABLE: [CandidateFailure; 13] = [
    CandidateFailure::GitNotStarted,
    CandidateFailure::GitFailed,
    CandidateFailure::ListingMalformed,
    CandidateFailure::UnsupportedMode,
    CandidateFailure::UnsupportedStatus,
    CandidateFailure::UnmergedPath,
    CandidateFailure::ReaderEnded,
    CandidateFailure::ReplyMalformed,
    CandidateFailure::ReplyTruncated,
    CandidateFailure::ReplyMismatched,
    CandidateFailure::ObjectMissing,
    CandidateFailure::ObjectKindUnexpected,
    CandidateFailure::PrivateIndexUnavailable,
];

/// The two codes print as the names the decision uses.
#[test]
fn the_two_codes_have_the_decided_names() {
    assert_eq!(
        engine_failure_code_name(EngineFailureCode::ContentUnavailable),
        "content-unavailable"
    );
    assert_eq!(
        engine_failure_code_name(EngineFailureCode::PolicyIncomplete),
        "policy-incomplete"
    );
}

/// Each cause of unreadable content is `content-unavailable`;
///  a stale candidate alone is `policy-incomplete`.
#[test]
fn every_candidate_failure_has_its_code() {
    for failure in UNREADABLE {
        assert_eq!(
            candidate_failure_code(failure),
            EngineFailureCode::ContentUnavailable,
            "{failure:?}"
        );
    }
    assert_eq!(
        candidate_failure_code(CandidateFailure::StaleCandidate),
        EngineFailureCode::PolicyIncomplete
    );
}

/// Both adapter causes are `policy-incomplete`.
#[test]
fn every_scanner_failure_is_policy_incomplete() {
    for failure in [
        ScannerFailure::RulesNotLoaded,
        ScannerFailure::PathnameUnrepresentable,
    ] {
        assert_eq!(
            scanner_failure_code(failure),
            EngineFailureCode::PolicyIncomplete,
            "{failure:?}"
        );
    }
}

/// A failed pass carries the code of the failure it wraps,
///  whichever layer failed.
#[test]
fn scan_run_errors_carry_the_code_of_the_failure_they_wrap() {
    for failure in UNREADABLE {
        let unreadable: ScanRunError = ScanRunError::Candidate(CandidateError::new(failure, "x"));
        assert_eq!(
            scan_run_failure_code(&unreadable),
            EngineFailureCode::ContentUnavailable,
            "{failure:?}"
        );
    }
    let stale: ScanRunError =
        ScanRunError::Candidate(CandidateError::new(CandidateFailure::StaleCandidate, "x"));
    assert_eq!(
        scan_run_failure_code(&stale),
        EngineFailureCode::PolicyIncomplete
    );
    for failure in [
        ScannerFailure::RulesNotLoaded,
        ScannerFailure::PathnameUnrepresentable,
    ] {
        let scanner: ScanRunError = ScanRunError::Scanner(ScannerError {
            failure,
            message: String::from("x"),
        });
        assert_eq!(
            scan_run_failure_code(&scanner),
            EngineFailureCode::PolicyIncomplete,
            "{failure:?}"
        );
    }
}

/// Matches are violations with no code;
///  the scanner's two fail-closed notices are `policy-incomplete`.
#[test]
fn only_findings_that_are_not_matches_carry_a_code() {
    let content: ScanFinding = ScanFinding::Content {
        line: 3,
        rule: String::from("0"),
    };
    let name: ScanFinding = ScanFinding::Name {
        component: 1,
        rule: String::from("0"),
    };
    assert_eq!(finding_failure_code(&content), None);
    assert_eq!(finding_failure_code(&name), None);
    assert_eq!(
        finding_failure_code(&ScanFinding::EngineError),
        Some(EngineFailureCode::PolicyIncomplete)
    );
    assert_eq!(
        finding_failure_code(&ScanFinding::PathnameLineBreak),
        Some(EngineFailureCode::PolicyIncomplete)
    );
}
