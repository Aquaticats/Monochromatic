//! The single error type the fallible source methods return.
//!
//! Platform adapters wrap their own decoder errors (the desktop and Android crates
//! each have a `PlayerError`) into this crate-owned type at the `TruePeakSource`
//! boundary,
//!  so the shared core never names a platform error.
//!  The repo throws typed
//! errors rather than returning codes or `null`,
//!  so this is a small custom enum that
//! implements `Display` and `std::error::Error`.

/// What:
///      `use std::fmt;`.
///  The formatting module,
///  for `fmt::Formatter`/`fmt::Result`
///           used by the `Display` impl.
///  Sibling:
///  importing `std::fmt::Display`
///           directly;
///  we import the module so both the trait path and the helper
///           types read consistently.
/// Why:
///       `Display` renders a human-readable message for logs and `Error`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // no import needed; TS uses string interpolation directly
/// ```
use std::fmt;

/// What:
///      `pub enum TruePeakError { ... }`.
///  A sum type (tagged union) naming each
///           way a decoded-audio source can fail.
///  `#[derive(Debug, Clone,
///           PartialEq, Eq)]` auto-generates debug printing,
///  value cloning,
///  and
///           equality (handy for asserting an exact error in tests).
///  Sibling shape:
///           a struct with an error-kind field;
///  an enum keeps each cause distinct.
/// Why:
///       Callers get one typed error to match on,
///  and the source methods never
///           panic on a decode or seek failure;
///  they return one of these instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TruePeakError =
///   | { kind: "decode"; message: string }
///   | { kind: "seek"; message: string };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TruePeakError {
    /// What:
    ///      `Decode { message: String }`.
    ///  A variant carrying an owned `String`
    ///           describing a decode failure.
    ///  `String` (sibling `&str`) because the
    ///           message is built from a platform error and must outlive that error.
    /// Why:
    ///       `next_chunk` failed to produce the next block of samples.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// { kind: "decode"; message: string }
    /// ```
    Decode {
        /// What:
        ///      `message: String`.
        ///  Owned human-readable text from the adapter.
        /// Why:
        ///       Name the underlying decoder failure for logs and tests.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// message: string;
        /// ```
        message: String,
    },
    /// What:
    ///      `Seek { message: String }`.
    ///  A variant carrying an owned `String`
    ///           describing a seek failure.
    ///  `String` (sibling `&str`) for the same
    ///           outlive reason as `Decode`.
    /// Why:
    ///       `seek_to_frame` could not land at the requested frame.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// { kind: "seek"; message: string }
    /// ```
    Seek {
        /// What:
        ///      `message: String`.
        ///  Owned human-readable text from the adapter.
        /// Why:
        ///       Name the underlying seek failure for logs and tests.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// message: string;
        /// ```
        message: String,
    },
}

/// What:
///      `impl fmt::Display for TruePeakError { ... }`.
///  Renders the error as a
///           one-line human message.
/// Why:
///       `Display` is what logs surface and what the `Error` impl below builds on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // toString() on the error object
/// ```
impl fmt::Display for TruePeakError {
    /// What:
    ///      `fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result`.
    ///           Write a sentence for each variant into the formatter.
    ///  `&self` borrows
    ///           the error read-only;
    ///  `fmt::Result` is `Result<(), fmt::Error>`.
    /// Why:
    ///       Each variant interpolates its own message.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toString(): string { ... }
    /// ```
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // What:     `match self { ... }`. Branch on the variant; no arm is discarded.
        //           `Self::Decode { message }` binds the inner string by reference.
        // Why:      Each variant carries a different message to render.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // switch (this.kind) { case "decode": ...; case "seek": ...; }
        // ```
        match self {
            // What:     `Self::Decode { message } => write!(...)`. `write!` formats into
            //           the formatter, returning `fmt::Result`. Tail of this arm.
            // Why:      Surface the decode cause.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return `true-peak decode failed: ${message}`;
            // ```
            Self::Decode { message } => return write!(formatter, "true-peak decode failed: {message}"),
            // What:     `Self::Seek { message } => write!(...)`. Same shape for seeks.
            // Why:      Surface the seek cause.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return `true-peak seek failed: ${message}`;
            // ```
            Self::Seek { message } => return write!(formatter, "true-peak seek failed: {message}"),
        }
    }
}

/// What:
///      `impl std::error::Error for TruePeakError {}`.
///  An empty impl opting the
///           type into the standard error trait (default methods suffice).
/// Why:
///       Lets `TruePeakError` participate in `?` propagation and generic std
///           error reporting the way any standard error does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class TruePeakError extends Error {}
/// ```
impl std::error::Error for TruePeakError {}

/// What:
///      `#[cfg(test)] #[path = "error_tests.rs"] mod tests;`.
///  Test-only submodule
///           in the sibling file `error_tests.rs`,
///  gated to test builds.
/// Why:
///       Keep this file to production code;
///  sibling `*_tests.rs` is max-lines exempt.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // error.unit.test.ts
/// ```
#[cfg(test)]
#[path = "error_tests.rs"]
mod tests;
