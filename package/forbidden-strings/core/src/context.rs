//! What:
//!     Position context that resolves the zero-width assertions `^`,
//!  `$`,
//!  and `\b`.
//! Why:
//!      This file is the Rust module that groups the context implementation,
//!  so the
//!          compiler gives those items one namespace and sibling modules can import that name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module context: see exported functions and types below.
//! ```

/// The boundary context at one position between (or at the ends of) the input.
///
/// What:
///  four booleans describing the gap where a derivative is taken or
/// nullability is checked.
///  `line_start`/`line_end` answer `^`/`$`;
/// `word_before`/`word_after` are the word-ness of the adjacent bytes and answer
/// `\b`.
///  Why:
///  anchors consume no byte,
///  so their meaning depends entirely on this
/// context rather than on the node alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Ctx = {
///   // fields documented in Rust above
/// };
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ctx {
    /// What:
    ///     True at the start of the input or just after a newline.
    /// Why:
    ///      `line_start` stores true at the start of the input or just after a newline,
    ///  so
    ///          matcher code reads that precomputed state by name instead of recomputing or
    ///          passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// line_start: boolean;
    /// ```
    pub line_start: bool,
    /// What:
    ///     True at the end of the input or just before a newline.
    /// Why:
    ///      `line_end` stores true at the end of the input or just before a newline,
    ///  so
    ///          matcher code reads that precomputed state by name instead of recomputing or
    ///          passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// line_end: boolean;
    /// ```
    pub line_end: bool,
    /// What:
    ///     True when the byte immediately before this position is a word byte.
    /// Why:
    ///      `word_before` stores true when the byte immediately before this position is a
    ///          word byte,
    ///  so matcher code reads that precomputed state by name instead of
    ///          recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// word_before: boolean;
    /// ```
    pub word_before: bool,
    /// What:
    ///     True when the byte immediately after this position is a word byte.
    /// Why:
    ///      `word_after` stores true when the byte immediately after this position is a word
    ///          byte,
    ///  so matcher code reads that precomputed state by name instead of
    ///          recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// word_after: boolean;
    /// ```
    pub word_after: bool,
}
