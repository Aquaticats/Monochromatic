//! What:
//!  The words a protocol line that cannot be read is refused with.
//! Why:
//!  Both readers of a protocol line answer a damaged one with the same sentence,
//!  so a person reads one sentence for one cause,
//!  and neither reader spells that sentence itself.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const errorText = (error: Error) => error.message;
//! ```
pub(crate) fn error_text(error: serde_json::Error) -> String {
    return error.to_string();
}
