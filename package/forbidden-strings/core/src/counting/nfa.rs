//! What:
//!     The serializable counting NFA:
//!  positions,
//!  follow sets,
//!  and start set.
//! Why:
//!      This file is the Rust module that groups the nfa implementation,
//!  so the
//!          compiler gives those items one namespace and sibling modules can import that name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module nfa: see exported functions and types below.
//! ```

/// What:
///     Imports the serde derives so a counting NFA can be persisted.
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
///     Imports the position kind and its decode-time check.
/// Why:
///      The code below uses `Element`,
///  `validate_element` directly;
///  importing from
///          `crate/counting/element` keeps each call site focused on the matcher logic instead
///          of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Element, validate_element } from "crate/counting/element";
/// ```
use crate::counting::element::{Element, validate_element};

/// What:
///     Imports the error type for validating a decoded NFA.
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

/// A counting NFA over byte-class positions with runtime counter-sets.
///
/// What:
///  `elements[p]` is the position kind,
///  `follow[p]` its successor positions
/// (an id equal to `elements.len()` is the virtual accept),
///  and `start` the
/// initially active positions.
///  Why:
///  branching lives in the follow and start sets,
/// so alternation costs only edges while each `{n,m}` stays one counted position;
/// the whole structure is linear in the pattern,
///  never in any repetition bound.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CountingNfa = {
///   // fields documented in Rust above
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountingNfa {
    /// What:
    ///     Position kinds,
    ///  indexed by position id.
    /// Why:
    ///      `elements` stores position kinds,
    ///  indexed by position id,
    ///  so matcher code reads
    ///          that precomputed state by name instead of recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// elements: Element[];
    /// ```
    pub elements: Vec<Element>,
    /// What:
    ///     Successor ids per position;
    ///  the id `elements.len()` is the accept sink.
    /// Why:
    ///      `follow` stores successor ids per position;
    ///  the id `elements.len()` is the
    ///          accept sink,
    ///  so matcher code reads that precomputed state by name instead of
    ///          recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// follow: number[][];
    /// ```
    pub follow: Vec<Vec<u32>>,
    /// What:
    ///     Positions active before any input is read.
    /// Why:
    ///      `start` stores positions active before any input is read,
    ///  so matcher code reads
    ///          that precomputed state by name instead of recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// start: number[];
    /// ```
    pub start: Vec<u32>,
}

/// What:
///     Matching and decode validation for a counting NFA.
/// Why:
///      The program attaches these functions to the named Rust type so callers can use
///          method syntax.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Methods are written inside a class or as functions that take the value.
/// ```
impl CountingNfa {
    /// Reports whether the NFA matches some substring of `line`.
    ///
    /// What:
    ///  defers to the counting-set search.
    ///  Why:
    ///  the boolean answer for one
    /// linear or alternation pattern;
    ///  the prefilter that fast-rejects most lines
    /// lives one level up in `Engine`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function is_match(line: Uint8Array): boolean {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    ///
    /// @example
    /// ```ignore
    /// let nfa = build_nfa(&parse("(?:(?:AKIA)|(?:ASIA))[A-Z2-7]{4}").unwrap()).unwrap();
    /// assert!(nfa.is_match(b"xxAKIAB2C7"));
    /// ```
    pub fn is_match(&self, line: &[u8]) -> bool {
        return crate::counting::run::run(self, line)
    }

    /// Checks that a decoded NFA is safe to run against untrusted input.
    ///
    /// What:
    ///  requires positions,
    ///  validates each,
    ///  and bounds every follow and start
    /// id by the accept index.
    ///  Why:
    ///  a serialized NFA may be hostile and is executed
    /// on attacker-influenced input,
    ///  so its indices must be proven in range and its
    /// counter bounds proven small before it runs.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function validate(): void {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn validate(&self) -> Result<(), CompileError> {
        if self.elements.is_empty() {
            return Err(CompileError::Invalid {
                message: "counting nfa has no positions".to_string(),
            });
        }
        if self.follow.len() != self.elements.len() {
            return Err(CompileError::Invalid {
                message: "counting nfa follow length mismatch".to_string(),
            });
        }
        for element in &self.elements {
            validate_element(element)?;
        }
        let accept = self.elements.len() as u32;
        for &id in self.follow.iter().flatten().chain(&self.start) {
            if id > accept {
                return Err(CompileError::Invalid {
                    message: "counting nfa target out of range".to_string(),
                });
            }
        }
        return Ok(())
    }
}

/// What:
///     Unit tests for counting-NFA decode validation,
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
#[path = "nfa_tests.rs"]
mod tests;
