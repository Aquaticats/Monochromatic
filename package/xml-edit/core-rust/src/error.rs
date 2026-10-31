//! What:
//!  The crate's public failure types: a positioned parse rejection and an edit rejection.
//! Why:
//!  Parsing answers "where is the malformed byte", while editing answers "which request cannot be honored",
//!  so each gets its own type instead of one message string callers would have to parse.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class XmlParseError extends Error { offset: number; line: number; column: number }
//! class XmlEditError extends Error {}
//! ```

/// What:
///  A rejected XML source with the byte offset, line and column where scanning stopped.
/// Why:
///  A person fixing a settings file needs a position in the original text, and the line and column are
///  computed once, when the error is built, so the happy path never pays for them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class XmlParseError extends Error { offset: number; line: number; column: number }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct XmlParseError {
    /// Zero-based UTF-8 byte offset into the rejected source.
    pub offset: usize,
    /// One-based line; CRLF, LF and a bare CR each end one line.
    pub line: usize,
    /// One-based column counted in Unicode scalar values from the line start.
    pub column: usize,
    /// Operation-focused explanation that names the malformed input.
    pub message: String,
}

impl XmlParseError {
    /// What:
    ///  Build a positioned rejection for `offset` within `source`.
    /// Why:
    ///  Every scanner and parser rejection funnels through here, so line and column are derived in one place.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function at(source: string, offset: number, message: string): XmlParseError;
    /// ```
    pub fn at(source: &str, offset: usize, message: &str) -> XmlParseError {
        // What: `usize` is the unsigned integer wide enough to index any byte (siblings `u32`, `u64`).
        // Why: Slicing and `char_indices` offsets are `usize`, so line and column counters match them.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // let line = 1; let column = 1;
        // ```
        let mut line: usize = 1;
        let mut column: usize = 1;
        // What: `previous_was_cr` remembers that the last scalar was a carriage return.
        // Why: CRLF must count as one line break, so an LF right after a CR is not a second line.
        let mut previous_was_cr: bool = false;
        // What: `source.char_indices()` yields `(byte offset, char)` pairs; `char` is one Unicode scalar value.
        // Why: Walking scalars (not bytes) keeps columns meaningful for multi-byte text.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // for (const [index, character] of iterateWithByteOffsets(source)) { ... }
        // ```
        for (index, character) in source.char_indices() {
            if index >= offset {
                break;
            }
            if character == '\n' {
                if !previous_was_cr {
                    line += 1;
                }
                column = 1;
                previous_was_cr = false;
            } else if character == '\r' {
                line += 1;
                column = 1;
                previous_was_cr = true;
            } else {
                column += 1;
                previous_was_cr = false;
            }
        }
        // What: `XmlParseError { .. }` builds the struct; `message.to_string()` copies the borrowed `&str`
        //  into an owned `String` (sibling `.clone()` duplicates an already-owned value).
        // Why: The error outlives the borrowed message literal, so it must own its text.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { offset, line, column, message };
        // ```
        return XmlParseError { offset, line, column, message: message.to_string() };
    }
}

/// What:
///  Print a parse rejection as `message (line L, column C, byte offset N)`.
/// Why:
///  Callers log failures directly, and the position belongs in the same sentence as the reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// toString(): string { return `${message} (line ${line}, column ${column}, byte offset ${offset})`; }
/// ```
impl std::fmt::Display for XmlParseError {
    /// What: Render the rejection with its position. Why: One shape for every diagnostic.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toString(): string { return `${this.message} (line ...)`; }
    /// ```
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            formatter,
            "{} (line {}, column {}, byte offset {})",
            self.message, self.line, self.column, self.offset
        );
    }
}

/// What: Mark the parse rejection as a standard Rust error. Why: Callers can box it or use `?` with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class XmlParseError extends Error {}
/// ```
impl std::error::Error for XmlParseError {}

/// What:
///  Why an edit request was refused.
/// Why:
///  Callers react differently to a stale element handle, a missing attribute, text XML cannot carry,
///  and an edit that would produce invalid XML, so each is its own variant.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type XmlEditError =
///   | { kind: 'unknownElement' } | { kind: 'attributeNotFound'; name: string }
///   | { kind: 'invalidName'; name: string } | { kind: 'invalidCharacter'; context: string }
///   | { kind: 'duplicateAttribute'; name: string } | { kind: 'mixedContent'; element: string }
///   | { kind: 'cannotRemoveRoot' } | { kind: 'rejectedResult'; error: XmlParseError };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum XmlEditError {
    /// The element handle does not belong to the document it was used with.
    UnknownElement,
    /// The named attribute is not on the element.
    AttributeNotFound {
        /// Attribute name that was requested.
        name: String,
    },
    /// The element or attribute name is not an XML Name.
    InvalidName {
        /// Name that was rejected.
        name: String,
    },
    /// The attribute is listed twice in one new element.
    DuplicateAttribute {
        /// Attribute name that repeats.
        name: String,
    },
    /// The text holds a character XML 1.0 cannot represent, even as a reference.
    InvalidCharacter {
        /// Where the text was going, such as `attribute value` or `text content`.
        context: String,
        /// The unrepresentable character.
        character: char,
    },
    /// The element mixes text with child elements, so one rendering would be a guess.
    MixedContent {
        /// Name of the element involved.
        element: String,
    },
    /// The document root cannot be removed.
    CannotRemoveRoot,
    /// The edit produced XML this crate rejects; the original document is unchanged.
    RejectedResult(XmlParseError),
}

/// What: Print an edit refusal as one sentence. Why: Callers surface it to a person without matching variants.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// toString(): string { /* switch on kind */ }
/// ```
impl std::fmt::Display for XmlEditError {
    /// What: Render the refusal. Why: Each variant names the input that caused it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toString(): string { /* switch on kind */ }
    /// ```
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // What: `match self { Variant => ..., }` picks the arm whose variant `self` holds, like a `switch` on a
        //  tagged union; the compiler rejects a match that forgets a variant.
        // Why: Each variant needs its own sentence, and exhaustiveness keeps new variants from going unprinted.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // switch (this.kind) { case 'unknownElement': return '...'; /* one case per kind */ }
        // ```
        match self {
            XmlEditError::UnknownElement => {
                return write!(formatter, "the element handle does not belong to this document");
            }
            XmlEditError::AttributeNotFound { name } => {
                return write!(formatter, "the element has no attribute named {name:?}");
            }
            XmlEditError::InvalidName { name } => {
                return write!(formatter, "{name:?} is not a valid XML name");
            }
            XmlEditError::DuplicateAttribute { name } => {
                return write!(formatter, "the new element lists attribute {name:?} more than once");
            }
            XmlEditError::InvalidCharacter { context, character } => {
                return write!(
                    formatter,
                    "U+{:04X} cannot be written in {context} because XML 1.0 has no representation for it",
                    u32::from(*character)
                );
            }
            XmlEditError::MixedContent { element } => {
                return write!(formatter, "element {element:?} mixes text with child elements, so the edit is ambiguous");
            }
            XmlEditError::CannotRemoveRoot => {
                return write!(formatter, "the root element cannot be removed because the document would have no root");
            }
            XmlEditError::RejectedResult(error) => {
                return write!(formatter, "the edit would produce invalid XML: {error}");
            }
        }
    }
}

/// What: Mark the edit refusal as a standard Rust error. Why: Callers can box it or use `?` with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class XmlEditError extends Error {}
/// ```
impl std::error::Error for XmlEditError {}
