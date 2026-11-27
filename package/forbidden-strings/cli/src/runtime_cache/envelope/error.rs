//! What:
//!  The one rejection type the cache envelope can report,
//!  and the reason word each variant becomes in the cache warning.
//! Why:
//!  A rejection reaches a warning protocol whose reason values are closed,
//!  so the mapping lives with the type rather than in the decoder;
//!  `invalid_envelope` is the `map_err` target that names a failure whose
//!  own error is never disclosed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // type EnvelopeError = 'incompatible' | 'source-mismatch' | 'invalid'
//! ```

/// Imports the closed warning-reason values a rejection maps to.
use crate::runtime_cache::warning::CacheWarningReason;

/// Redacted envelope rejection category.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EnvelopeError {
    /// Schema,
    ///  scanner version,
    ///  or platform identity differs.
    Incompatible,
    /// Embedded source digest disagrees with current source.
    SourceMismatch,
    /// Framing,
    ///  names,
    ///  counts,
    ///  or engine bytes are malformed.
    Invalid,
}

/// Converts envelope rejection into closed warning protocol reason.
impl EnvelopeError {
    /// Returns corresponding scanner cache-warning reason.
    pub(crate) fn warning_reason(self) -> CacheWarningReason {
        if self == EnvelopeError::Incompatible {
            return CacheWarningReason::Incompatible;
        }
        if self == EnvelopeError::SourceMismatch {
            return CacheWarningReason::SourceMismatch;
        }
        return CacheWarningReason::Invalid;
    }
}

/// Turns any lower-level failure into the redacted invalid-envelope rejection,
///  so a `map_err` names this function instead of a closure.
pub(super) fn invalid_envelope<Cause>(_cause: Cause) -> EnvelopeError {
    return EnvelopeError::Invalid;
}
