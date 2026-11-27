//! What:
//!  What one candidate's scan becomes:
//!  the outcome of a scan that could not complete,
//!  the finding of each match,
//!  and the record of a candidate whose content a parent commit's tree already holds.
//! Why:
//!  A matcher failure and an uninspectable pathname are reported inside an otherwise
//!       successful scan,
//!  so they are read before any match is reported;
//!  and the words of a match are the installed
//!       wrapper's,
//!  which never show the matched text.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const outcome = incompleteScan(scan) ?? matchesOf(scan);
//! ```

/// Import the candidates whose content a parent commit's tree already holds.
use crate::candidate_known_content::KnownContent;
/// Import the debug diagnostics on standard error.
use crate::diagnostic_log::debug;
/// Import the engine-failure codes.
use crate::diagnostics::EngineFailureCode;
/// Import the event form of a pathname.
use crate::event_path::EventPath;
/// Import the finding and outcome types of a check.
use crate::policy_engine::{
    PolicyFinding,
    PolicyOutcome,
};
/// Import the code of each scan failure.
use crate::scanner_failure_code::finding_failure_code;
/// Import the scanner library's per-candidate result and per-match finding.
use forbidden_strings::{
    CandidateScan,
    ScanFinding,
};

/// The policy-local code of a match.
pub const FORBIDDEN_STRING_CODE: &str = "forbidden-string";

/// The engine-failure message of a scan a matcher could not complete.
pub const ENGINE_ERROR_MESSAGE: &str = "The forbidden-strings scanner reported a matcher failure while scanning a candidate, so the scan is incomplete.";

/// The engine-failure message of a pathname the line-based scanner cannot inspect.
pub const LINE_BREAK_MESSAGE: &str = "The forbidden-strings scanner cannot inspect a candidate pathname that contains a line break, so the scan is incomplete.";

/// What:
///  The outcome of a check that could not finish.
///  `&str` is borrowed message text.
/// Why:
///   Every failure of this policy names its cause through its code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const failed = (code, message): PolicyOutcome => ({ kind: 'failed', code, message });
/// ```
pub(crate) fn failed(
    code: EngineFailureCode,
    message: &str,
) -> PolicyOutcome {
    return PolicyOutcome::Failed {
        code,
        message: String::from(message),
        // `None`: a scan failure never names a pathname, which may hold a secret.
        path: None,
    };
}

/// What:
///  The failure a scan result carries,
///  or nothing when its findings are all matches.
///       `&CandidateScan` borrows one candidate's result.
/// Why:
///   The scanner reports a failed matcher and an uninspectable pathname inside an
///       otherwise successful result;
///  either makes the whole scan incomplete,
///  so it is
///       found before any match is reported.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function incompleteScan(scan: CandidateScan): PolicyOutcome | undefined;
/// ```
pub(crate) fn incomplete_scan(scan: &CandidateScan) -> Option<PolicyOutcome> {
    for finding in &scan.findings {
        // `if let Some(code) = ...` runs only for a finding that is a failure.
        if let Some(code) = finding_failure_code(finding) {
            let message: &str = if *finding == ScanFinding::PathnameLineBreak {
                LINE_BREAK_MESSAGE
            } else {
                ENGINE_ERROR_MESSAGE
            };
            return Some(failed(code, message));
        }
    }
    return None;
}

/// What:
///  The finding for one match in one scan,
///  or nothing for a finding that is not a match.
/// Why:
///   The words,
///  the code and the masked path are the installed wrapper's,
///  so a match
///       is reported the same way by both wrappers and never shows the matched text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const toFinding = (hit, displayPath): PolicyFinding => ({ code: 'forbidden-string', message, path: displayPath });
/// ```
pub(crate) fn match_finding(
    finding: &ScanFinding,
    display_path: &str,
) -> Option<PolicyFinding> {
    let message: String = match finding {
        ScanFinding::Content {
            line,
            rule,
        } => {
            format!("Forbidden string matched at line {line} (rule {rule}).")
        }
        ScanFinding::Name {
            component,
            rule,
        } => {
            format!("Forbidden string matched in pathname segment {component} (rule {rule}).")
        }
        ScanFinding::EngineError | ScanFinding::PathnameLineBreak => return None,
    };
    return Some(PolicyFinding {
        code: FORBIDDEN_STRING_CODE,
        message,
        // The scanner's masked display is text only: its bytes would undo the masking.
        path: Some(EventPath::display(display_path)),
        location: None,
        fix_available: false,
        commit_correction: None,
    });
}

/// What:
///  Record each skipped candidate as a decision:
///  its masked pathname,
///  its blob and the parent commit that holds it.
///       `&[CandidateScan]` borrows the scan results,
///  whose identity is the candidate's position.
/// Why:
///   A skip changes what the policy reads,
///  so the inputs that justify it are kept for whoever checks a verdict later.
///       The pathname is the scanner's masked display,
///  never the raw pathname,
///  which may hold a secret.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function logKnownContent(scans: CandidateScan[], known: KnownContent): void;
/// ```
pub(crate) fn log_known_content(
    scans: &[CandidateScan],
    known: &KnownContent,
) {
    for scan in scans {
        if let Some(entry) = known.entry(scan.identity) {
            debug(
                "checkForbiddenStrings",
                entry.describe(scan.display_path.as_str()).as_str(),
            );
        }
    }
}
