//! What:
//!  One difference between the module's declaration and the expectation:
//! the step it
//!  is reported as,
//! and the field path with the expected and actual values.
//! Why:
//!  The layout walk and the request walk both stop at their first difference
//! and both
//!  answer the same value,
//! so the step and the detail live in one place and neither walk
//!  imports the other.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Mismatch = { step: Step; detail: string };
//! ```

/// The step one difference is reported as.
use super::refusal::Step;

/// What:
///  One difference between the module and the expectation.
/// Why:
///   The walk stops at the first one,
/// so a difference carries only its step and its
///  detail;
/// the file it is attributed to is the caller's,
/// which read that file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Mismatch = { step: Step; detail: string };
/// ```
pub struct Mismatch {
    /// The step the difference is reported as.
    pub step: Step,
    /// The field path with the expected and actual values.
    pub detail: String,
}
