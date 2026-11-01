//! What:
//!     The element leaf type and its decode-time validation.
//! Why:
//!      This file is the Rust module that groups the element implementation,
//!  so the
//!          compiler gives those items one namespace and sibling modules can import that name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module element: see exported functions and types below.
//! ```

/// What:
///     Imports the serde derives so an element can be persisted.
/// Why:
///      The code below uses `Deserialize`,
///  `Serialize` directly;
///  importing from `serde`
///          keeps each call site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Deserialize, Serialize } from "serde";
/// ```
use serde::{Deserialize, Serialize};

/// What:
///     Imports the byte-set leaf type carried by class and counted elements.
/// Why:
///      The code below uses `ByteSet` directly;
///  importing from `crate/charset` keeps each
///          call site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ByteSet } from "crate/charset";
/// ```
use crate::charset::ByteSet;

/// What:
///     Imports the error type for validating a decoded element.
/// Why:
///      The code below uses `CompileError` directly;
///  importing from `crate/error` keeps each
///          call site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CompileError } from "crate/error";
/// ```
use crate::error::CompileError;

/// Largest repetition bound a decoded element may carry.
///
/// What:
///  an upper limit on a `Counted` element's `max`,
///  checked on decode.
///  Why:
///  the
/// counter-set can hold up to `max` distinct values,
///  so a hostile serialized bound
/// could force unbounded memory;
///  this caps it well above the parser's own limit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MAX_DECODED_COUNT: number = 1 << 16;
/// ```
pub(crate) const MAX_DECODED_COUNT: usize = 1 << 16;

/// One position of a counting NFA:
///  a byte class,
///  a counted class,
///  or an anchor.
///
/// What:
///  the byte-consuming positions (`Class`,
///  `Counted`) and the zero-width
/// anchors;
///  branching between positions lives in the NFA's follow sets,
///  not here.
/// Why:
///  keeping a bounded repetition as one `Counted` position is what moves the
/// count out of the state space and into a runtime counter-set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Element =
///   | { kind: "variant" };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Element {
    /// What:
    ///     Matches exactly one byte drawn from the set.
    /// Why:
    ///      The surrounding function uses this step to keep the matcher behavior correct
    ///          at this point.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// // Same step as the Rust statement below, written with ordinary TS objects/functions.
    /// ```
    Class(
        /// What:
        ///     Byte set the single matched byte must belong to.
        /// Why:
        ///      The surrounding function uses this step to keep the matcher behavior
        ///          correct at this point.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// // Same step as the Rust statement below, written with ordinary TS objects/functions.
        /// ```
        ByteSet,
    ),
    /// What:
    ///     Matches between `min` and `max` bytes from the set,
    ///  counted at runtime.
    /// Why:
    ///      The surrounding function uses this step to keep the matcher behavior correct
    ///          at this point.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// // Same step as the Rust statement below, written with ordinary TS objects/functions.
    /// ```
    Counted {
        /// What:
        ///     Byte set each repetition must match.
        /// Why:
        ///      `set` stores byte set each repetition must match,
        ///  so matcher code reads that
        ///          precomputed state by name instead of recomputing or passing it separately.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// set: ByteSet;
        /// ```
        set: ByteSet,
        /// What:
        ///     Fewest repetitions that satisfy the element.
        /// Why:
        ///      `min` stores fewest repetitions that satisfy the element,
        ///  so matcher code
        ///          reads that precomputed state by name instead of recomputing or passing it
        ///          separately.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// min: number;
        /// ```
        min: usize,
        /// What:
        ///     Most repetitions the element admits.
        /// Why:
        ///      `max` stores most repetitions the element admits,
        ///  so matcher code reads that
        ///          precomputed state by name instead of recomputing or passing it separately.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// max: number;
        /// ```
        max: usize,
    },
    /// What:
    ///     Zero-width `^`:
    ///  passable only at a line start.
    /// Why:
    ///      The surrounding function uses this step to keep the matcher behavior correct
    ///          at this point.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// // Same step as the Rust statement below, written with ordinary TS objects/functions.
    /// ```
    LineStart,
    /// What:
    ///     Zero-width `$`:
    ///  passable only at a line end.
    /// Why:
    ///      The surrounding function uses this step to keep the matcher behavior correct
    ///          at this point.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// // Same step as the Rust statement below, written with ordinary TS objects/functions.
    /// ```
    LineEnd,
    /// What:
    ///     Zero-width `\b`:
    ///  passable only where word-ness changes.
    /// Why:
    ///      The surrounding function uses this step to keep the matcher behavior correct
    ///          at this point.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// // Same step as the Rust statement below, written with ordinary TS objects/functions.
    /// ```
    WordBoundary,
}

/// Validates one decoded element.
///
/// What:
///  only `Counted` carries decode-time risk;
///  class and anchor elements are
/// always well formed.
///  Why:
///  a serialized bound may be hostile,
///  and the simulation
/// allocates a counter-set sized by `max`,
///  so it must be proven small first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function validate_element(element: Element): void {
///   // Rust body below is the implementation.
/// }
/// ```
pub(crate) fn validate_element(element: &Element) -> Result<(), CompileError> {
    if let Element::Counted { set, min, max } = element {
        let invalid = |message: &str| return CompileError::Invalid {
            message: message.to_string(),
        };
        if set.is_empty() {
            return Err(invalid("counted element has an empty set"));
        }
        if min > max {
            return Err(invalid("counted element has min greater than max"));
        }
        if *max > MAX_DECODED_COUNT {
            return Err(invalid("counted element bound exceeds the decode cap"));
        }
    }
    return Ok(())
}

/// What:
///     Unit tests for element decode validation,
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
#[path = "element_tests.rs"]
mod tests;
