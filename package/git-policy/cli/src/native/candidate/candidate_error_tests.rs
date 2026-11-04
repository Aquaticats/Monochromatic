//! What:
//!  Controls for the candidate error value.
//! Why:
//!  The cause is what callers branch on and the message is the whole diagnostic,
//!  so
//!      constructing and printing the error must carry both unchanged.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(String(new CandidateError('git-failed', 'x'))).toBe('x');
//! ```

/// Import the error under test.
use super::{CandidateError, CandidateFailure};

/// Printing yields exactly the stored message,
///  and the cause is kept beside it.
#[test]
fn display_prints_exactly_the_message() {
    for message in ["git diff-index failed", "two\nlines \"quoted\"", ""] {
        let error: CandidateError = CandidateError::new(CandidateFailure::GitFailed, message);
        assert_eq!(error.failure, CandidateFailure::GitFailed);
        assert_eq!(error.message, message);
        // `.to_string()` goes through the `Display` implementation under test.
        assert_eq!(error.to_string(), message);
        assert_eq!(format!("[{error}]"), format!("[{message}]"));
    }
}

/// Errors compare by cause and message,
///  and report no underlying cause.
#[test]
fn errors_compare_by_cause_and_message() {
    let stale: CandidateError = CandidateError::new(CandidateFailure::StaleCandidate, "a");
    assert_eq!(
        stale,
        CandidateError::new(CandidateFailure::StaleCandidate, "a")
    );
    assert_ne!(
        stale,
        CandidateError::new(CandidateFailure::StaleCandidate, "b")
    );
    assert_ne!(
        stale,
        CandidateError::new(CandidateFailure::ReaderEnded, "a")
    );
    // `&dyn Error` views the value through the standard error interface only.
    let general: &dyn std::error::Error = &stale;
    assert!(general.source().is_none());
    assert_eq!(general.to_string(), "a");
}
