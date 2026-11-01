//! What:
//!      The crate's public failure types:
//!  a byte-offset parse failure,
//!  a number-token failure,
//!           a missing document address,
//!  and a wrong-shape address or target.
//! Why:
//!       Each failure answers a different caller question (where is the malformed byte,
//!  which
//!           literal is not a JSON number,
//!  which address does not exist,
//!  which target has the wrong
//!           shape),
//!  so collapsing them into one string would force callers to parse messages.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module error: class JsoncParseError extends Error; class JsoncPathNotFoundError extends Error;
//! ```

/// What:
///      Import the address type so a not-found failure can carry the address it was given.
/// Why:
///       A caller debugging a failed edit needs the exact path back,
///  not only a message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { JsoncPathSegment } from './path';
/// ```
use crate::path::JsoncPathSegment;

/// What:
///      A JSONC source rejection with the byte offset where scanning stopped.
/// Why:
///       Editors point at a position in the original document,
///  and a UTF-8 byte offset is the
///           only position this crate can report without re-scanning for UTF-16 units.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class JsoncParseError extends Error { offset: number }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JsoncParseError {
    /// Zero-based UTF-8 byte offset in the rejected source.
    pub offset: usize,
    /// Operation-focused explanation naming the malformed input,
    ///  never a moral judgement.
    pub message: String,
}

/// What:
///      Make `JsoncParseError` printable as a message.
/// Why:
///       Callers log and format failures directly,
///  and `message (at offset N)` is the shape the
///           maintained TypeScript package prints,
///  so shared fixtures can compare text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class JsoncParseError extends Error {}
/// ```
impl std::fmt::Display for JsoncParseError {
    /// What:
    ///      Render the failure the way the maintained TypeScript package renders it.
    /// Why:
    ///       Shared fixtures compare messages across both implementations,
    ///  so the shape is part
    ///           of the supported behavior.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toString(): string { return `${this.message} (at offset ${this.offset})`; }
    /// ```
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(formatter, "{} (at offset {})", self.message, self.offset);
    }
}

/// What:
///      Register `JsoncParseError` as a standard-library error type.
/// Why:
///       The marker implementation lets `?`,
///  `Box<dyn Error>` and error chains carry this
///           failure without a custom wrapper.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS needs no marker: extending Error is enough.
/// ```
impl std::error::Error for JsoncParseError {}

/// What:
///      Why one number token cannot become an exact mathematical identity.
/// Why:
///       A JSON number literal has several independent grammar parts,
///  and a caller fixing a
///           document needs to know which part failed rather than receiving one generic message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type JsoncNumberError = { kind: 'empty' | 'leading-zero' | ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JsoncNumberError {
    /// The token held no bytes at all.
    Empty,
    /// A leading plus sign,
    ///  which JSON does not admit.
    LeadingPlus,
    /// An integer part with a leading zero,
    ///  such as `01`.
    LeadingZero,
    /// No integer digit where the grammar requires one.
    MissingIntegerDigit,
    /// A decimal point with no following digit.
    MissingFractionDigit,
    /// An exponent marker with no following digit.
    MissingExponentDigit,
    /// A trailing byte that no JSON number part admits.
    UnexpectedSuffix,
}

/// What:
///      Make `JsoncNumberError` printable as a message.
/// Why:
///       Callers log and format failures directly,
///  and `the offending grammar part` is the shape the
///           maintained TypeScript package prints,
///  so shared fixtures can compare text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class JsoncNumberError extends Error {}
/// ```
impl std::fmt::Display for JsoncNumberError {
    /// What:
    ///      Name the offending grammar part in plain operational terms.
    /// Why:
    ///       The message is user-facing diagnostics,
    ///  so it describes the input rather than the
    ///           code that rejected it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toString(): string { return this.kind; }
    /// ```
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // What:     A `Record`-style lookup would need owned strings, so this match maps each
        //           variant to its static text.
        // Why:      Every variant must have a message; a catch-all arm would hide a new one.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const text = messagesByKind[this.kind];
        // ```
        let text = match self {
            JsoncNumberError::Empty => "number token is empty",
            JsoncNumberError::LeadingPlus => "number token starts with +",
            JsoncNumberError::LeadingZero => "number token has a leading zero",
            JsoncNumberError::MissingIntegerDigit => "number token has no integer digit",
            JsoncNumberError::MissingFractionDigit => "number token has no fraction digit",
            JsoncNumberError::MissingExponentDigit => "number token has no exponent digit",
            JsoncNumberError::UnexpectedSuffix => "number token has an unexpected trailing byte",
        };
        return formatter.write_str(text);
    }
}

/// What:
///      Register `JsoncNumberError` as a standard-library error type.
/// Why:
///       The marker implementation lets `?`,
///  `Box<dyn Error>` and error chains carry this
///           failure without a custom wrapper.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS needs no marker: extending Error is enough.
/// ```
impl std::error::Error for JsoncNumberError {}

/// What:
///      A document address that does not name an existing key or element.
/// Why:
///       Reading and deleting must distinguish "absent" from "wrong shape",
///  so an absent
///           address is its own failure type.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class JsoncPathNotFoundError extends Error { path: JsoncPath }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JsoncPathNotFoundError {
    /// The address that named nothing in the document.
    pub path: Vec<JsoncPathSegment>,
}

/// What:
///      Make `JsoncPathNotFoundError` printable as a message.
/// Why:
///       Callers log and format failures directly,
///  and `no JSONC node at path ...` is the shape the
///           maintained TypeScript package prints,
///  so shared fixtures can compare text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class JsoncPathNotFoundError extends Error {}
/// ```
impl std::fmt::Display for JsoncPathNotFoundError {
    /// What:
    ///      Render the missing address with debug formatting for its segments.
    /// Why:
    ///       A mixed key and index address is ambiguous in plain text,
    ///  and this message is for
    ///           diagnostics rather than document output.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toString(): string { return `no JSONC node at path ${JSON.stringify(this.path)}`; }
    /// ```
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(formatter, "no JSONC node at path {:?}", self.path);
    }
}

/// What:
///      Register `JsoncPathNotFoundError` as a standard-library error type.
/// Why:
///       The marker implementation lets `?`,
///  `Box<dyn Error>` and error chains carry this
///           failure without a custom wrapper.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS needs no marker: extending Error is enough.
/// ```
impl std::error::Error for JsoncPathNotFoundError {}

/// What:
///      An address or target whose shape does not fit the requested operation.
/// Why:
///       Indexing an object with a number,
///  or a key with a string,
///  is a caller mistake that must
///           not be reinterpreted as an absent address.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class JsoncTypeError extends Error {}
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JsoncTypeError {
    /// Operation-focused explanation naming the mismatched input.
    pub message: String,
}

/// What:
///      Make `JsoncTypeError` printable as a message.
/// Why:
///       Callers log and format failures directly,
///  and `the mismatch message` is the shape the
///           maintained TypeScript package prints,
///  so shared fixtures can compare text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class JsoncTypeError extends Error {}
/// ```
impl std::fmt::Display for JsoncTypeError {
    /// What:
    ///      Pass the explanation through unchanged.
    /// Why:
    ///       The constructing site already knows the operation and the offending value.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toString(): string { return this.message; }
    /// ```
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return formatter.write_str(&self.message);
    }
}

/// What:
///      Register `JsoncTypeError` as a standard-library error type.
/// Why:
///       The marker implementation lets `?`,
///  `Box<dyn Error>` and error chains carry this
///           failure without a custom wrapper.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS needs no marker: extending Error is enough.
/// ```
impl std::error::Error for JsoncTypeError {}

/// What:
///      Any failure an edit,
///  read or comment operation can return.
/// Why:
///       A caller addressing a document can hit a missing address or a wrong-shaped target,
///  and
///           both are ordinary outcomes rather than panics,
///  so one enum keeps signatures short.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type JsoncEditError = JsoncPathNotFoundError | JsoncTypeError;
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JsoncEditError {
    /// The addressed key or element does not exist in the document.
    ///
    /// What:
    ///      holds the not-found failure with the address that named nothing.
    /// Why:
    ///       callers can report or recover from an absent address without string matching.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// { kind: 'path-not-found', error: JsoncPathNotFoundError }
    /// ```
    PathNotFound {
        /// What:
        ///     The underlying not-found failure.
        /// Why:
        ///      `error` stores the address,
        ///  so a caller can echo exactly what was requested.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// error: JsoncPathNotFoundError;
        /// ```
        error: JsoncPathNotFoundError,
    },
    /// The addressed value or segment has the wrong shape for the requested operation.
    ///
    /// What:
    ///      holds the shape failure.
    /// Why:
    ///       indexing an object with a number is a caller mistake,
    ///  not an absent address.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// { kind: 'type', error: JsoncTypeError }
    /// ```
    Type {
        /// What:
        ///     The underlying shape failure.
        /// Why:
        ///      `error` stores the explanation naming the mismatched input.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// error: JsoncTypeError;
        /// ```
        error: JsoncTypeError,
    },
}

/// What:
///      Print whichever underlying failure this edit error carries.
/// Why:
///       A caller logging the error should see the same text the underlying type produces.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// toString(): string { return this.error.message; }
/// ```
impl std::fmt::Display for JsoncEditError {
    /// What:
    ///      Delegate to the wrapped failure's own rendering.
    /// Why:
    ///       The wrapper adds no information,
    ///  so duplicating the text would risk divergence.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toString(): string { return String(this.error); }
    /// ```
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            JsoncEditError::PathNotFound { error } => write!(formatter, "{error}"),
            JsoncEditError::Type { error } => write!(formatter, "{error}"),
        };
    }
}

/// What:
///      Register the edit failure as a standard-library error type.
/// Why:
///       `?` and error chains then carry it without a custom wrapper.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS needs no marker: extending Error is enough.
/// ```
impl std::error::Error for JsoncEditError {}
