//! What:
//!     Recursive-descent grammar with single-atom operands for `&` and `|`.
//! Why:
//!      This file is the Rust module that groups the grammar implementation,
//!  so the
//!          compiler gives those items one namespace and sibling modules can import that name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module grammar: see exported functions and types below.
//! ```

/// What:
///     Imports the byte-set helpers for `.` and literal bytes.
/// Why:
///      The code below uses `dot_set`,
///  `singleton` directly;
///  importing from `crate/charset`
///          keeps each call site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { dot_set, singleton } from "crate/charset";
/// ```
use crate::charset::{dot_set, singleton};

/// What:
///     Imports the node algebra and constructors.
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
///     Imports the set-algebra and class constructors.
/// Why:
///      The code below uses `alt`,
///  `class`,
///  `comp`,
///  `concat`,
///  `inter`,
///  `optional` directly;
///          importing from `crate/ast/smart` keeps each call site focused on the matcher logic
///          instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import {
///   alt,
///   class,
///   comp,
///   concat,
///   inter,
///   optional,
/// } from "crate/ast/smart";
/// ```
use crate::ast::smart::{alt, class, comp, concat, inter, optional};

/// What:
///     Imports the error type.
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
///     Imports the atom-position escape parser.
/// Why:
///      The code below uses `parse_escape_atom` directly;
///  importing from
///          `crate/parse/escape` keeps each call site focused on the matcher logic instead of
///          the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parse_escape_atom } from "crate/parse/escape";
/// ```
use crate::parse::escape::parse_escape_atom;

/// What:
///     Imports the character-class parser.
/// Why:
///      The code below uses `parse_class` directly;
///  importing from `crate/parse/class` keeps
///          each call site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parse_class } from "crate/parse/class";
/// ```
use crate::parse::class::parse_class;

/// What:
///     Imports the repetition parser.
/// Why:
///      The code below uses `parse_repeat` directly;
///  importing from `crate/parse/repeat`
///          keeps each call site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parse_repeat } from "crate/parse/repeat";
/// ```
use crate::parse::repeat::parse_repeat;

/// Parses a full expression at one nesting level,
///  up to `close` or end.
///
/// What:
///  reads a concatenation,
///  then,
///  if `&` or `|` follows,
///  switches to the
/// set-algebra form where every operand must be exactly one atom.
///  Why:
///  this is
/// the chosen grammar with no operator precedence;
///  operators never mix with
/// concatenation at one level without explicit `(?:...)` grouping.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parse_setexpr(cur: Cursor, close: number | null): Node {
///   // Rust body below is the implementation.
/// }
/// ```
pub fn parse_setexpr(cur: &mut Cursor, close: Option<u8>) -> Result<Node, CompileError> {
    let units = parse_concat_units(cur, close)?;
    cur.skip_ignorable();
    // What: peek for a set-algebra operator. Why: its presence changes how the
    // already-parsed left side must be interpreted (single atom, not concat).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    match cur.peek() {
        Some(op @ (b'|' | b'&')) => return parse_set_algebra(cur, close, units, op),
        _ => return Ok(concat(units)),
    }
}

/// Parses a chain of single-atom operands joined by one operator.
///
/// What:
///  requires the left side to be a single atom,
///  then reads each further
/// operand as a single atom,
///  rejecting any mix of `&` and `|`.
///  Why:
///  enforces the
/// fully-wrapped-operand rule that removes precedence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parse_set_algebra(cur: Cursor, close: number | null, left: Node[], op: number): Node {
///   // Rust body below is the implementation.
/// }
/// ```
fn parse_set_algebra(
    cur: &mut Cursor,
    close: Option<u8>,
    left: Vec<Node>,
    op: u8,
) -> Result<Node, CompileError> {
    let mut operands = vec![single_atom(left, cur.pos())?];
    loop {
        cur.skip_ignorable();
        match cur.peek() {
            Some(c) if c == op => {
                cur.bump();
                let next = parse_concat_units(cur, close)?;
                operands.push(single_atom(next, cur.pos())?);
            }
            // What: the other operator at this level is illegal. Why: mixing `&`
            // and `|` needs an explicit `(?:...)` wrapper.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // // Same step as the Rust statement below, written with ordinary TS objects/functions.
            // ```
            Some(b'|') | Some(b'&') => {
                return Err(CompileError::Syntax {
                    pos: cur.pos(),
                    message: "cannot mix '&' and '|' at one level; wrap one in (?:...)".to_string(),
                });
            }
            _ => break,
        }
    }
    // What: build the matching node. Why: `op` selects union or intersection.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    if op == b'|' {
        return Ok(alt(operands))
    } else {
        return Ok(inter(operands))
    }
}

/// Returns the sole node of `units`,
///  or errors if it is not exactly one atom.
///
/// What:
///  an operand of `&`/`|` must be a single atom.
///  Why:
///  a bare concatenation
/// (or empty) operand is rejected so the author wraps it in `(?:...)`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function single_atom(units: Node[], pos: number): Node {
///   // Rust body below is the implementation.
/// }
/// ```
fn single_atom(units: Vec<Node>, pos: usize) -> Result<Node, CompileError> {
    if units.len() == 1 {
        return Ok(units.into_iter().next().unwrap())
    } else {
        return Err(CompileError::Syntax {
            pos,
            message: "operand of '&' or '|' must be a single atom; wrap it in (?:...)".to_string(),
        })
    }
}

/// Reads zero or more postfix atoms forming a concatenation.
///
/// What:
///  stops at end,
///  the closing delimiter,
///  or a set-algebra operator.
///  Why:
/// the caller decides whether the run is a plain concatenation or one operand of
/// an operator chain.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parse_concat_units(cur: Cursor, close: number | null): Node[] {
///   // Rust body below is the implementation.
/// }
/// ```
fn parse_concat_units(cur: &mut Cursor, close: Option<u8>) -> Result<Vec<Node>, CompileError> {
    let mut units = Vec::new();
    loop {
        cur.skip_ignorable();
        match cur.peek() {
            None => break,
            Some(c) if Some(c) == close => break,
            Some(b'|') | Some(b'&') => break,
            Some(_) => units.push(parse_postfix(cur)?),
        }
    }
    return Ok(units)
}

/// Parses one atom and an optional single quantifier.
///
/// What:
///  applies `?`,
///  `{n}`,
///  or `{n,m}`,
///  rejects `*`/`+`,
///  and forbids a stacked
/// second quantifier.
///  Why:
///  one quantifier per atom keeps the grammar simple and
/// the language finite.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parse_postfix(cur: Cursor): Node {
///   // Rust body below is the implementation.
/// }
/// ```
fn parse_postfix(cur: &mut Cursor) -> Result<Node, CompileError> {
    let atom = parse_atom(cur)?;
    cur.skip_ignorable();
    // What: dispatch on the quantifier byte. Why: each maps to a distinct shape.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let quantified = match cur.peek() {
        Some(b'?') => {
            cur.bump();
            optional(atom)
        }
        Some(b'{') => parse_repeat(cur, atom)?,
        Some(b'*') => {
            return Err(CompileError::Syntax {
                pos: cur.pos(),
                message: "'*' is unsupported; use {0,n}".to_string(),
            });
        }
        Some(b'+') => {
            return Err(CompileError::Syntax {
                pos: cur.pos(),
                message: "'+' is unsupported; use {1,n}".to_string(),
            });
        }
        _ => return Ok(atom),
    };
    reject_stacked(cur)?;
    return Ok(quantified)
}

/// Rejects a second quantifier directly following the first.
///
/// What:
///  after one quantifier,
///  another `?`/`{`/`*`/`+` is an error.
///  Why:
///  stacked
/// quantifiers are ambiguous and unsupported.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reject_stacked(cur: Cursor): void {
///   // Rust body below is the implementation.
/// }
/// ```
fn reject_stacked(cur: &mut Cursor) -> Result<(), CompileError> {
    cur.skip_ignorable();
    match cur.peek() {
        Some(b'?') | Some(b'{') | Some(b'*') | Some(b'+') => return Err(CompileError::Syntax {
            pos: cur.pos(),
            message: "stacked quantifiers are unsupported; wrap in (?:...)".to_string(),
        }),
        _ => return Ok(()),
    }
}

/// Parses a single atom.
///
/// What:
///  groups,
///  complements,
///  classes,
///  `.`,
///  anchors,
///  escapes,
///  or a literal byte;
/// every other metacharacter in atom position is an error.
///  Why:
///  atoms are the
/// leaves the rest of the grammar combines.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parse_atom(cur: Cursor): Node {
///   // Rust body below is the implementation.
/// }
/// ```
fn parse_atom(cur: &mut Cursor) -> Result<Node, CompileError> {
    cur.skip_ignorable();
    let pos = cur.pos();
    let b = cur.peek().ok_or(CompileError::Syntax {
        pos,
        message: "expected an expression".to_string(),
    })?;
    match b {
        b'(' => return parse_group(cur),
        b'~' => return parse_complement(cur),
        b'[' => return parse_class(cur),
        b'.' => {
            cur.bump();
            return Ok(class(dot_set()))
        }
        b'^' => {
            cur.bump();
            return Ok(Node::LineStart)
        }
        b'$' => {
            cur.bump();
            return Ok(Node::LineEnd)
        }
        b'\\' => return parse_escape_atom(cur),
        b')' | b']' | b'}' => return Err(CompileError::Syntax {
            pos,
            message: format!("unmatched '{}'", b as char),
        }),
        b'{' => return Err(CompileError::Syntax {
            pos,
            message: "'{' quantifier without a preceding atom".to_string(),
        }),
        b'*' | b'+' => return Err(CompileError::Syntax {
            pos,
            message: format!("'{}' is unsupported", b as char),
        }),
        b'|' | b'&' => return Err(CompileError::Syntax {
            pos,
            message: format!("'{}' has no left operand", b as char),
        }),
        _ => {
            cur.bump();
            return Ok(class(singleton(b)))
        }
    }
}

/// Parses a `(?:...)` non-capturing group.
///
/// What:
///  requires the `(?:` prefix and rejects capturing groups,
///  lookaround,
///  and
/// inline flags.
///  Why:
///  only non-capturing grouping is supported.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parse_group(cur: Cursor): Node {
///   // Rust body below is the implementation.
/// }
/// ```
fn parse_group(cur: &mut Cursor) -> Result<Node, CompileError> {
    let pos = cur.pos();
    cur.bump();
    // What: only `(?:` is accepted after `(`. Why: `(` alone is a capture and
    // `(?` followed by anything but `:` is lookaround or an inline flag.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    if cur.peek() != Some(b'?') {
        return Err(CompileError::Syntax {
            pos,
            message: "capturing groups are unsupported; use (?:...)".to_string(),
        });
    }
    cur.bump();
    if cur.peek() != Some(b':') {
        return Err(CompileError::Syntax {
            pos,
            message: "only (?:...) groups are supported (no lookaround or inline flags)".to_string(),
        });
    }
    cur.bump();
    let inner = parse_setexpr(cur, Some(b')'))?;
    expect_close(cur, pos)?;
    return Ok(inner)
}

/// Parses a `~(...)` complement.
///
/// What:
///  requires `~(` then a full subexpression then `)`.
///  Why:
///  complement is
/// always explicitly parenthesized per the engine's grammar.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parse_complement(cur: Cursor): Node {
///   // Rust body below is the implementation.
/// }
/// ```
fn parse_complement(cur: &mut Cursor) -> Result<Node, CompileError> {
    let pos = cur.pos();
    cur.bump();
    cur.skip_ignorable();
    if cur.peek() != Some(b'(') {
        return Err(CompileError::Syntax {
            pos,
            message: "'~' must be followed by '(' as in ~(...)".to_string(),
        });
    }
    cur.bump();
    let inner = parse_setexpr(cur, Some(b')'))?;
    expect_close(cur, pos)?;
    return Ok(comp(inner))
}

/// Consumes the closing `)` of a group or complement.
///
/// What:
///  skips verbose noise then requires `)`.
///  Why:
///  a missing close is an
/// unbalanced-parenthesis error reported at the opener.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function expect_close(cur: Cursor, open_pos: number): void {
///   // Rust body below is the implementation.
/// }
/// ```
fn expect_close(cur: &mut Cursor, open_pos: usize) -> Result<(), CompileError> {
    cur.skip_ignorable();
    if cur.peek() == Some(b')') {
        cur.bump();
        return Ok(())
    } else {
        return Err(CompileError::Syntax {
            pos: open_pos,
            message: "missing closing ')'".to_string(),
        })
    }
}
