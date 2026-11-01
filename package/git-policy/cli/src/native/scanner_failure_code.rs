//! What:
//!  Which engine failure code each failure of the candidate and scanner layers carries.
//! Why:
//!  An engine failure names its cause.
//!  `content-unavailable` means candidate bytes or a
//!      repository fact could not be read;
//!  `policy-incomplete` means the policy's own
//!      machinery failed.
//!  Deciding the code here,
//!  beside the failures,
//!  makes every policy
//!      that reads candidates or scans them report the same code for the same cause.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const code = scanRunFailureCode(error); // 'content-unavailable' | 'policy-incomplete'
//! ```

/// Import the candidate layer's closed list of causes.
use super::candidate_error::CandidateFailure;
/// Import the engine's failure codes;
///  this module only chooses among them.
use super::diagnostics::EngineFailureCode;
/// Import the adapter's closed list of causes.
use super::scanner_adapter::ScannerFailure;
/// Import the failure of a whole scan pass.
use super::scanner_run::ScanRunError;
/// The scanner library's per-match result,
///  which also carries its fail-closed notices.
use forbidden_strings::ScanFinding;

/// What:
///  The code of one candidate-layer failure.
///  `match` lists every cause by name and
///       has no catch-all arm;
///  `A | B` in an arm means "either of these".
/// Why:
///   Every cause but one is something Git could not list or hand over,
///  or a private
///       index a prediction needed could not be prepared,
///  so the content was unavailable.
///  A stale candidate is different:
///  its content can be read through
///       a fresh version,
///  and what failed is the calling pass,
///  which kept a candidate
///       across an invalidation.
///  Without a catch-all arm,
///  a cause added later does not
///       compile until someone chooses its code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function candidateFailureCode(failure: CandidateFailure): EngineFailureCode;
/// ```
pub fn candidate_failure_code(failure: CandidateFailure) -> EngineFailureCode {
    match failure {
        CandidateFailure::GitNotStarted
        | CandidateFailure::GitFailed
        | CandidateFailure::ListingMalformed
        | CandidateFailure::UnsupportedMode
        | CandidateFailure::UnsupportedStatus
        | CandidateFailure::UnmergedPath
        | CandidateFailure::ReaderEnded
        | CandidateFailure::ReplyMalformed
        | CandidateFailure::ReplyTruncated
        | CandidateFailure::ReplyMismatched
        | CandidateFailure::ObjectMissing
        | CandidateFailure::ObjectKindUnexpected
        | CandidateFailure::PrivateIndexUnavailable => {
            return EngineFailureCode::ContentUnavailable;
        }
        CandidateFailure::StaleCandidate => return EngineFailureCode::PolicyIncomplete,
    }
}

/// What:
///  The code of one adapter failure.
/// Why:
///   Rules that cannot be loaded,
///  and a pathname that was read but cannot be handed
///       to the scanner on this platform,
///  are both failures of the policy's machinery;
///       no repository content was unreadable.
///  The `match` names both causes so a cause
///       added later does not compile until someone chooses its code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scannerFailureCode(failure: ScannerFailure): EngineFailureCode;
/// ```
pub fn scanner_failure_code(failure: ScannerFailure) -> EngineFailureCode {
    match failure {
        ScannerFailure::RulesNotLoaded | ScannerFailure::PathnameUnrepresentable => {
            return EngineFailureCode::PolicyIncomplete;
        }
    }
}

/// What:
///  The code of a failed scan pass.
///  `&ScanRunError` borrows the failure read-only.
/// Why:
///   A pass fails in the candidate layer or in the adapter,
///  and each already has its
///       code;
///  the pass reports the code of the failure it wraps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scanRunFailureCode(error: ScanRunError): EngineFailureCode;
/// ```
pub fn scan_run_failure_code(error: &ScanRunError) -> EngineFailureCode {
    match error {
        ScanRunError::Candidate(candidate) => return candidate_failure_code(candidate.failure),
        ScanRunError::Scanner(scanner) => return scanner_failure_code(scanner.failure),
    }
}

/// What:
///  The code of a finding that is not a violation,
///  or nothing for a violation.
///       `Option<EngineFailureCode>` is "a code or nothing";
///  `{ .. }` ignores a variant's fields.
/// Why:
///   The scanner reports two things among its findings that are not matches:
///  a matcher
///       that failed,
///  and a pathname with a line break that it could not inspect.
///  Neither
///       says the candidate is clean and neither names a rule,
///  so each must end the pass
///       as an engine failure instead of being printed as a rule violation.
///  Both arrive
///       after the candidate was read,
///  so the cause is the policy's machinery.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findingFailureCode(finding: ScanFinding): EngineFailureCode | undefined;
/// ```
pub fn finding_failure_code(finding: &ScanFinding) -> Option<EngineFailureCode> {
    match finding {
        // `None` is the "absent" variant: a content or name match is a violation, not a failure.
        ScanFinding::Content { .. } | ScanFinding::Name { .. } => return None,
        // `Some(...)` is the "present" variant.
        ScanFinding::EngineError | ScanFinding::PathnameLineBreak => {
            return Some(EngineFailureCode::PolicyIncomplete);
        }
    }
}

/// One control per mapping,
///  over every variant.
#[cfg(test)]
#[path = "scanner_failure_code_tests.rs"]
mod tests;
