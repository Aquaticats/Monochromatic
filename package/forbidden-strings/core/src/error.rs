//! What:
//!     Compile-time error type for the forbidden-regex engine.
//! Why:
//!      This file is the Rust module that groups the error implementation,
//!  so the
//!          compiler gives those items one namespace and sibling modules can import that name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module error: see exported functions and types below.
//! ```

/// What:
///     Imports the formatter pieces used to render a human-readable error message.
/// Why:
///      The code below uses `fmt` directly;
///  importing from `std` keeps each call site
///          focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { fmt } from "std";
/// ```
use std::fmt;

/// Reasons a pattern (or a serialized DFA) is rejected before it can match.
///
/// What:
///  every failure path in parsing,
///  the empty-match guard,
///  the DFA state
/// cap,
///  and (de)serialization funnels into one of these variants.
/// Why:
///  callers (the scanner,
///  tests) get a single typed error to match on,
///  and
/// `compile`/`new` never panic on bad input;
///  they return one of these instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CompileError =
///   | { kind: "variant" };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileError {
    /// A syntax or unsupported-construct rejection at a byte offset.
    ///
    /// What:
    ///  holds the offset into the pattern and a message naming what was
    /// wrong (unsupported operator,
    ///  bad escape,
    ///  unbalanced bracket,
    ///  operand not
    /// a single atom,
    ///  mixed operators,
    ///  stacked quantifier,
    ///  bad repetition).
    /// Why:
    ///  one variant covers every parse rejection so the surface stays small
    /// while still pointing the rule author at the exact spot.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// // Same step as the Rust statement below, written with ordinary TS objects/functions.
    /// ```
    Syntax {
        /// What:
        ///     Byte offset into the pattern where the problem was detected.
        /// Why:
        ///      `pos` stores byte offset into the pattern where the problem was detected,
        ///  so
        ///          matcher code reads that precomputed state by name instead of recomputing or
        ///          passing it separately.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// pos: number;
        /// ```
        pos: usize,
        /// What:
        ///     Human-readable description of what was rejected.
        /// Why:
        ///      `message` stores human-readable description of what was rejected,
        ///  so matcher
        ///          code reads that precomputed state by name instead of recomputing or passing
        ///          it separately.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// message: string;
        /// ```
        message: String,
    },
    /// The pattern can match the empty string,
    ///  so under unanchored search it
    /// would match every input.
    ///
    /// What:
    ///  raised when the parsed root is nullable in some realizable anchor
    /// context.
    ///  Why:
    ///  such a rule is a footgun (flags every line);
    ///  rejecting it
    /// at compile time is safer than silently matching everything.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// // Same step as the Rust statement below, written with ordinary TS objects/functions.
    /// ```
    EmptyMatchable,
    /// Determinization produced more states than the configured cap.
    ///
    /// What:
    ///  complement and intersection can blow up the state count on
    /// pathological patterns.
    ///  Why:
    ///  a hard cap turns that into a clean error
    /// instead of unbounded memory use.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// // Same step as the Rust statement below, written with ordinary TS objects/functions.
    /// ```
    StateCap {
        /// What:
        ///     State-count limit that was exceeded.
        /// Why:
        ///      `limit` stores state-count limit that was exceeded,
        ///  so matcher code reads
        ///          that precomputed state by name instead of recomputing or passing it
        ///          separately.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// limit: number;
        /// ```
        limit: usize,
    },
    /// Encoding a compiled automaton to bytes failed.
    ///
    /// What:
    ///  wraps a bincode serialization failure as a string.
    ///  Why:
    ///  keeps the
    /// public error free of a bincode type in its signature.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// // Same step as the Rust statement below, written with ordinary TS objects/functions.
    /// ```
    Serialize {
        /// What:
        ///     Underlying codec failure rendered as text.
        /// Why:
        ///      `message` stores underlying codec failure rendered as text,
        ///  so matcher code
        ///          reads that precomputed state by name instead of recomputing or passing it
        ///          separately.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// message: string;
        /// ```
        message: String,
    },
    /// A deserialized automaton failed structural validation.
    ///
    /// What:
    ///  raised by `from_bytes` when decoded indices are out of bounds or
    /// lengths are inconsistent.
    ///  Why:
    ///  a hostile or corrupt blob must be rejected
    /// before it is ever executed,
    ///  so the match loop can never read out of
    /// bounds.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// // Same step as the Rust statement below, written with ordinary TS objects/functions.
    /// ```
    Invalid {
        /// What:
        ///     Description of which invariant the decoded automaton violated.
        /// Why:
        ///      `message` stores description of which invariant the decoded automaton
        ///          violated,
        ///  so matcher code reads that precomputed state by name instead of
        ///          recomputing or passing it separately.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// message: string;
        /// ```
        message: String,
    },
}

/// What:
///     Renders a `CompileError` for end users and logs.
/// Why:
///      The program attaches these functions to the named Rust type so callers can use
///          method syntax.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Methods are written inside a class or as functions that take the value.
/// ```
impl fmt::Display for CompileError {
    /// Writes a one-line description of the error.
    ///
    /// What:
    ///  matches each variant to a sentence.
    ///  Why:
    ///  `Display` is what the
    /// scanner surfaces and what `Error` builds on.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function fmt(f: fmt.Formatter<'_>): fmt.Result {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // What: branch per variant; no `match` value is discarded.
        // Why: each variant carries different fields to interpolate.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same step as the Rust statement below, written with ordinary TS objects/functions.
        // ```
        match self {
            CompileError::Syntax { pos, message } => {
                return write!(f, "syntax error at byte {pos}: {message}")
            }
            CompileError::EmptyMatchable => {
                return write!(f, "pattern can match the empty string, which would match every input")
            }
            CompileError::StateCap { limit } => {
                return write!(f, "pattern exceeded the DFA state cap of {limit}")
            }
            CompileError::Serialize { message } => {
                return write!(f, "failed to serialize automaton: {message}")
            }
            CompileError::Invalid { message } => {
                return write!(f, "invalid serialized automaton: {message}")
            }
        }
    }
}

/// What:
///     Lets `CompileError` participate in the standard error ecosystem.
/// Why:
///      The program attaches these functions to the named Rust type so callers can use
///          method syntax.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Methods are written inside a class or as functions that take the value.
/// ```
impl std::error::Error for CompileError {}

/// What:
///     Unit tests for error rendering,
///  in a sidecar (max-lines exempt).
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./tests";
/// ```
#[cfg(test)]
#[path = "error_tests.rs"]
mod tests;
