//! What:
//!     Backslash escapes,
//!  shared by atom parsing and character-class parsing.
//! Why:
//!      This file is the Rust module that groups the escape implementation,
//!  so the
//!          compiler gives those items one namespace and sibling modules can import that name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module escape: see exported functions and types below.
//! ```

/// What:
///     Imports the byte-set type for shorthand escapes.
/// Why:
///      The code below uses `ByteSet`,
///  `digit_set`,
///  `singleton`,
///  `space_set`,
///  `word_set`
///          directly;
///  importing from `crate/charset` keeps each call site focused on the matcher
///          logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import {
///   ByteSet,
///   digit_set,
///   singleton,
///   space_set,
///   word_set,
/// } from "crate/charset";
/// ```
use crate::charset::{ByteSet, digit_set, singleton, space_set, word_set};

/// What:
///     Imports the node algebra produced by an atom-position escape.
/// Why:
///      The code below uses `Node` directly;
///  importing from `crate/ast/node` keeps each call
///          site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Node } from "crate/ast/node";
/// ```
use crate::ast::node::Node;

/// What:
///     Imports the error type for unsupported escapes.
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

/// What:
///     Imports the cursor read by the escape parser.
/// Why:
///      The code below uses `Cursor` directly;
///  importing from `crate/parse/cursor` keeps
///          each call site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Cursor } from "crate/parse/cursor";
/// ```
use crate::parse::cursor::Cursor;

/// What a backslash escape denotes.
///
/// What:
///  either one literal byte,
///  a shorthand byte set,
///  or the `\b` word-boundary
/// assertion.
///  Why:
///  the same escape grammar is reused inside a class (where only
/// `Byte`/`Set` are legal) and in atom position (where `Boundary` is also legal).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type EscapeResult =
///   | { kind: "variant" };
/// ```
pub enum EscapeResult {
    /// What:
    ///     A single literal byte (`\t`,
    ///  `\#`,
    ///  an escaped metacharacter,
    ///  escaped
    ///          whitespace).
    /// Why:
    ///      The surrounding function uses this step to keep the matcher behavior correct
    ///          at this point.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// // Same step as the Rust statement below, written with ordinary TS objects/functions.
    /// ```
    Byte(
        /// What:
        ///     Literal byte value the escape denotes.
        /// Why:
        ///      The surrounding function uses this step to keep the matcher behavior
        ///          correct at this point.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// // Same step as the Rust statement below, written with ordinary TS objects/functions.
        /// ```
        u8,
    ),
    /// What:
    ///     A shorthand byte set (`\d \w \s` and their negations).
    /// Why:
    ///      The surrounding function uses this step to keep the matcher behavior correct
    ///          at this point.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// // Same step as the Rust statement below, written with ordinary TS objects/functions.
    /// ```
    Set(
        /// What:
        ///     Byte set the shorthand expands to.
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
    ///     The `\b` word-boundary assertion.
    /// Why:
    ///      The surrounding function uses this step to keep the matcher behavior correct
    ///          at this point.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// // Same step as the Rust statement below, written with ordinary TS objects/functions.
    /// ```
    Boundary,
}

/// Parses one backslash escape,
///  rejecting anything outside the supported set.
///
/// What:
///  consumes the backslash and its following byte and classifies it;
///  `\b`
/// is a boundary only outside a class and an error inside one.
///  Why:
///  the engine's
/// escape vocabulary is deliberately small,
///  and unknown escapes must fail loudly
/// rather than silently become literals.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parse_escape(cur: Cursor, in_class: boolean): EscapeResult {
///   // Rust body below is the implementation.
/// }
/// ```
pub fn parse_escape(cur: &mut Cursor, in_class: bool) -> Result<EscapeResult, CompileError> {
    let pos = cur.pos();
    cur.bump();
    // What: read the escaped byte; a dangling backslash is an error.
    // Why: `\` must always introduce a concrete escape.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let e = cur.bump().ok_or(CompileError::Syntax {
        pos,
        message: "trailing backslash".to_string(),
    })?;
    match e {
        b't' => return Ok(EscapeResult::Byte(b'\t')),
        b'b' => {
            // What: `\b` is the boundary assertion, illegal inside `[...]`.
            // Why: inside a class `\b` would mean backspace, which is unsupported.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // // Same step as the Rust statement below, written with ordinary TS objects/functions.
            // ```
            if in_class {
                return Err(CompileError::Syntax {
                    pos,
                    message: "\\b (word boundary) is not allowed inside a character class".to_string(),
                })
            } else {
                return Ok(EscapeResult::Boundary)
            }
        }
        b'd' => return Ok(EscapeResult::Set(digit_set())),
        b'D' => return Ok(EscapeResult::Set(digit_set().negate())),
        b'w' => return Ok(EscapeResult::Set(word_set())),
        b'W' => return Ok(EscapeResult::Set(word_set().negate())),
        b's' => return Ok(EscapeResult::Set(space_set())),
        b'S' => return Ok(EscapeResult::Set(space_set().negate())),
        b'.' | b'[' | b']' | b'(' | b')' | b'{' | b'}' | b'?' | b'|' | b'&' | b'~' | b'^'
        | b'$' | b'\\' | b'#' | b'-' | b'/' | b'*' | b'+' => return Ok(EscapeResult::Byte(e)),
        // What: a backslash before whitespace yields that literal whitespace byte.
        // Why: in always-on verbose mode this is how a literal space is written.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same step as the Rust statement below, written with ordinary TS objects/functions.
        // ```
        byte if byte.is_ascii_whitespace() => return Ok(EscapeResult::Byte(byte)),
        _ => return Err(CompileError::Syntax {
            pos,
            message: format!("unsupported escape \\{}", e as char),
        }),
    }
}

/// Parses an escape in atom position and lifts it to a node.
///
/// What:
///  `Byte`/`Set` become one-byte classes;
///  `Boundary` becomes the
/// `WordBoundary` node.
///  Why:
///  atom parsing wants a `Node`,
///  not the intermediate
/// escape classification.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parse_escape_atom(cur: Cursor): Node {
///   // Rust body below is the implementation.
/// }
/// ```
pub fn parse_escape_atom(cur: &mut Cursor) -> Result<Node, CompileError> {
    // What: dispatch on the escape kind. Why: each kind maps to a distinct node.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    match parse_escape(cur, false)? {
        EscapeResult::Byte(b) => return Ok(crate::ast::smart::class(singleton(b))),
        EscapeResult::Set(set) => return Ok(crate::ast::smart::class(set)),
        EscapeResult::Boundary => return Ok(Node::WordBoundary),
    }
}
