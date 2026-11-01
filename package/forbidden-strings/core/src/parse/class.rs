//! What:
//!     Character-class `[...]` parsing.
//! Why:
//!      This file is the Rust module that groups the class implementation,
//!  so the
//!          compiler gives those items one namespace and sibling modules can import that name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module class: see exported functions and types below.
//! ```

/// What:
///     Imports the byte-set accumulator and node constructor.
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
///     Imports the node algebra produced by a class.
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
///     Imports the class smart constructor (collapses an empty set to `Fail`).
/// Why:
///      The code below uses `class` directly;
///  importing from `crate/ast/smart` keeps each
///          call site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { class } from "crate/ast/smart";
/// ```
use crate::ast::smart::class;

/// What:
///     Imports the error type for malformed classes.
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
///     Imports the cursor.
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

/// What:
///     Imports the escape parser and its result kind.
/// Why:
///      The code below uses `EscapeResult`,
///  `parse_escape` directly;
///  importing from
///          `crate/parse/escape` keeps each call site focused on the matcher logic instead of
///          the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { EscapeResult, parse_escape } from "crate/parse/escape";
/// ```
use crate::parse::escape::{EscapeResult, parse_escape};

/// One element inside a character class.
///
/// What:
///  either a concrete byte (a possible range endpoint) or a shorthand set.
/// Why:
///  ranges are only meaningful between two bytes,
///  so the parser must know
/// which kind it just read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ClassAtom =
///   | { kind: "variant" };
/// ```
enum ClassAtom {
    /// What:
    ///     A single byte that may begin or end a range.
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
        ///     Byte value,
        ///  usable as a range endpoint.
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
    ///     A shorthand set that cannot participate in a range.
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
        ///     Shorthand set,
        ///  never a range endpoint.
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
}

/// Parses a `[...]` (or `[^...]`) class into a class node.
///
/// What:
///  reads members and ranges until the closing `]`,
///  with no verbose-mode
/// skipping (whitespace and `#` are literal inside a class),
///  then negates the
/// accumulated set when the class opened with `^`.
///  Why:
///  byte classes are the
/// engine's main alphabet primitive and need their own grammar distinct from the
/// whitespace-insensitive outer one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parse_class(cur: Cursor): Node {
///   // Rust body below is the implementation.
/// }
/// ```
pub fn parse_class(cur: &mut Cursor) -> Result<Node, CompileError> {
    let pos = cur.pos();
    cur.bump();
    // What: an immediate `^` negates the class. Why: `[^...]` is set complement.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let negated = if cur.peek() == Some(b'^') {
        cur.bump();
        true
    } else {
        false
    };
    let mut set = ByteSet::empty();
    // What: `first` lets a leading `]` be a literal member rather than the close.
    // Why: matches the conventional class grammar.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let mut first = true;
    loop {
        let p = cur.peek().ok_or(CompileError::Syntax {
            pos,
            message: "unterminated character class".to_string(),
        })?;
        if p == b']' && !first {
            cur.bump();
            break;
        }
        read_class_element(cur, pos, &mut set)?;
        first = false;
    }
    // What: complement the bitmap when the class was negated. Why: `[^...]`
    // matches every byte not listed, including the newline (byte-level).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let final_set = if negated { set.negate() } else { set };
    return Ok(class(final_set))
}

/// Reads one element (member or range) and folds it into `set`.
///
/// What:
///  parses an atom,
///  then,
///  when it is a byte followed by a `-` and a real
/// endpoint,
///  reads a range;
///  otherwise inserts the single byte or unions the
/// shorthand.
///  Why:
///  keeps the loop in `parse_class` short and the range logic in
/// one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function read_class_element(cur: Cursor, pos: number, set: ByteSet): void {
///   // Rust body below is the implementation.
/// }
/// ```
fn read_class_element(cur: &mut Cursor, pos: usize, set: &mut ByteSet) -> Result<(), CompileError> {
    // What: classify the next element. Why: ranges only apply to byte atoms.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let lo = parse_class_atom(cur)?;
    let lo_b = match lo {
        ClassAtom::Set(s) => {
            set.union_with(&s);
            return Ok(());
        }
        ClassAtom::Byte(b) => b,
    };
    // What: a `-` that is not the class close starts a range. Why: `a-z` form;
    // a trailing `-` (before `]`) is a literal handled by the else branch.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let is_range = cur.peek() == Some(b'-') && cur.peek_at(1).is_some() && cur.peek_at(1) != Some(b']');
    if is_range {
        cur.bump();
        match parse_class_atom(cur)? {
            ClassAtom::Byte(hi_b) => {
                if hi_b < lo_b {
                    return Err(CompileError::Syntax {
                        pos,
                        message: "character-class range is out of order".to_string(),
                    });
                }
                set.insert_range(lo_b, hi_b);
            }
            ClassAtom::Set(_) => {
                return Err(CompileError::Syntax {
                    pos,
                    message: "a shorthand cannot be a range endpoint".to_string(),
                });
            }
        }
    } else {
        set.insert(lo_b);
    }
    return Ok(())
}

/// Parses one class atom:
///  an escape or a literal byte.
///
/// What:
///  dispatches a backslash to the shared escape parser,
///  otherwise consumes
/// one literal byte.
///  Why:
///  classes admit the same escapes as atoms except `\b`,
/// which the escape parser rejects when `in_class` is set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parse_class_atom(cur: Cursor): ClassAtom {
///   // Rust body below is the implementation.
/// }
/// ```
fn parse_class_atom(cur: &mut Cursor) -> Result<ClassAtom, CompileError> {
    let p = cur.peek().ok_or(CompileError::Syntax {
        pos: cur.pos(),
        message: "unterminated character class".to_string(),
    })?;
    if p == b'\\' {
        // What: reuse the escape grammar with class rules. Why: `\d`/`\t`/escaped
        // metacharacters all work the same inside a class.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same step as the Rust statement below, written with ordinary TS objects/functions.
        // ```
        match parse_escape(cur, true)? {
            EscapeResult::Byte(b) => return Ok(ClassAtom::Byte(b)),
            EscapeResult::Set(s) => return Ok(ClassAtom::Set(s)),
            EscapeResult::Boundary => return Err(CompileError::Syntax {
                pos: cur.pos(),
                message: "\\b is not valid in a character class".to_string(),
            }),
        }
    } else {
        cur.bump();
        return Ok(ClassAtom::Byte(p))
    }
}
