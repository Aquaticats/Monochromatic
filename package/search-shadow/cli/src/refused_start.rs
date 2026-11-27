//! What:
//! What a refused start gives its callers:
//! the one line `search-shadow` prints,
//! and the error a caller that reports a start it could not make hands on.
//! Why:
//! A person sees only this line,
//! and a refusal comes in two shapes:
//! the system's own answer for a program that could not be opened,
//! and the sentence naming the shadow a program is or leads to.
//! Both name the program once and say what was found,
//! so the words live in one place that a test can read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function describeRefusedStart(program: string, cause: string): string
//! export function intoError(refusal: VetRefusal): Error
//! ```

/// Imports the error type a caller that reports a failed start hands on.
use std::io::Error;
/// Imports the borrowed path type.
use std::path::Path;

/// Imports the two shapes a refusal comes in.
use crate::shadow_identity::VetRefusal;

/// What:
/// `pub fn describe_refused_start(program: &Path, cause: &str) -> String` builds the line the shadow prints
/// for a program it did not start.
/// Why:
/// The start check (`vet`) and a start the system refused both end in this line,
/// so a reader of one message reads them all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function describeRefusedStart(program: string, cause: string): string {
///   return `search-shadow: could not start ${program}: ${cause}`;
/// }
/// ```
pub fn describe_refused_start(
    program: &Path,
    cause: &str,
) -> String {
    return format!(
        "search-shadow: could not start {}: {cause}",
        program.display()
    );
}

/// What:
/// `pub fn into_error(refusal: VetRefusal) -> Error` turns the start check's refusal into the error a caller reports.
/// Why:
/// The two shapes stay apart on the way:
/// a program that could not be opened keeps the system's error and its kind
/// (a program that is gone answers `NotFound`),
/// and a shadow's sentence travels as an error whose kind claims nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function intoError(refusal: VetRefusal): Error
/// ```
pub fn into_error(refusal: VetRefusal) -> Error {
    return match refusal {
        // What:     `VetRefusal::NotOpened { error }` is the system's own answer, kind and text together.
        // Why:      Handed on as it is, so the caller reads what the system said.
        VetRefusal::NotOpened {
            error,
        } => error,
        // What:     `VetRefusal::LeadsToShadow { sentence }` is the sentence naming the shadow the file is or leads to.
        // Why:      `Error::other` carries the sentence and says nothing about a cause it cannot know.
        VetRefusal::LeadsToShadow {
            sentence,
        } => Error::other(sentence),
    };
}
