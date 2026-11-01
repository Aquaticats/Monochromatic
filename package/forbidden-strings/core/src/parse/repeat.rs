//! What:
//!     Bounded repetition `{n}` and `{n,m}`,
//!  kept as a `Repeat` node.
//! Why:
//!      This file is the Rust module that groups the repeat implementation,
//!  so the
//!          compiler gives those items one namespace and sibling modules can import that name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module repeat: see exported functions and types below.
//! ```

/// What:
///     Imports the node algebra being repeated.
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
///     Imports the smart constructor that builds the `Repeat` node.
/// Why:
///      The code below uses `repeat` directly;
///  importing from `crate/ast/smart` keeps each
///          call site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { repeat } from "crate/ast/smart";
/// ```
use crate::ast::smart::repeat;

/// What:
///     Imports the error type for malformed or oversized repetitions.
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

/// Largest repetition count accepted,
///  bounding the counter range.
///
/// What:
///  an upper limit on `n` and `m`.
///  Why:
///  the count becomes a runtime counter
/// register,
///  so `{1000000}` would demand a huge counter domain;
///  capping turns that
/// into a clean error instead of an unbounded counter.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const REPEAT_CAP: number = 1024;
/// ```
const REPEAT_CAP: usize = 1024;

/// Parses a `{n}` or `{n,m}` quantifier and applies it to `atom`.
///
/// What:
///  reads the counts after `{`,
///  rejects unbounded `{n,}` and `n > m`,
///  caps
/// the magnitude,
///  and builds a `Repeat` node.
///  Why:
///  the count stays symbolic on the
/// node (later a counter register) rather than being unrolled into states.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parse_repeat(cur: Cursor, atom: Node): Node {
///   // Rust body below is the implementation.
/// }
/// ```
pub fn parse_repeat(cur: &mut Cursor, atom: Node) -> Result<Node, CompileError> {
    let pos = cur.pos();
    cur.bump();
    let n = parse_number(cur, pos)?;
    // What: branch on what follows the first number. Why: `}` is exact, `,`
    // begins a range, anything else is malformed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    match cur.peek() {
        Some(b'}') => {
            cur.bump();
            check_cap(n, pos)?;
            return Ok(repeat(atom, n, n))
        }
        Some(b',') => {
            cur.bump();
            return parse_repeat_upper(cur, atom, n, pos)
        }
        _ => return Err(CompileError::Syntax {
            pos,
            message: "malformed repetition; expected '}' or ','".to_string(),
        }),
    }
}

/// Parses the upper bound of a `{n,m}` after the comma.
///
/// What:
///  rejects the unbounded `{n,}` form,
///  reads `m`,
///  checks ordering and the
/// cap,
///  and builds the range.
///  Why:
///  keeps `parse_repeat` short and isolates the
/// range-specific validation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parse_repeat_upper(cur: Cursor, atom: Node, n: number, pos: number): Node {
///   // Rust body below is the implementation.
/// }
/// ```
fn parse_repeat_upper(cur: &mut Cursor, atom: Node, n: usize, pos: usize) -> Result<Node, CompileError> {
    // What: a `}` here means `{n,}`, which is unbounded and unsupported.
    // Why: the engine only admits finite bounded repetition.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    if cur.peek() == Some(b'}') {
        return Err(CompileError::Syntax {
            pos,
            message: "unbounded repetition {n,} is unsupported; use {n,m}".to_string(),
        });
    }
    let m = parse_number(cur, pos)?;
    if cur.peek() != Some(b'}') {
        return Err(CompileError::Syntax {
            pos,
            message: "malformed repetition; expected '}'".to_string(),
        });
    }
    cur.bump();
    if m < n {
        return Err(CompileError::Syntax {
            pos,
            message: "repetition {n,m} has n greater than m".to_string(),
        });
    }
    check_cap(m, pos)?;
    return Ok(repeat(atom, n, m))
}

/// Reads a run of ASCII digits as a `usize`,
///  failing on overflow or none.
///
/// What:
///  folds digits left to right with checked arithmetic.
///  Why:
///  a missing or
/// overflowing count is a syntax error,
///  not a panic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parse_number(cur: Cursor, pos: number): number {
///   // Rust body below is the implementation.
/// }
/// ```
fn parse_number(cur: &mut Cursor, pos: usize) -> Result<usize, CompileError> {
    let mut value: usize = 0;
    let mut any = false;
    // What: consume while the next byte is a digit. Why: bounded linear scan of
    // the count; not recursion over the input.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    while let Some(c) = cur.peek() {
        if !c.is_ascii_digit() {
            break;
        }
        any = true;
        value = value
            .checked_mul(10)
            .and_then(|v| return v.checked_add((c - b'0') as usize))
            .ok_or(CompileError::Syntax {
                pos,
                message: "repetition count is too large".to_string(),
            })?;
        cur.bump();
    }
    if !any {
        return Err(CompileError::Syntax {
            pos,
            message: "expected a number inside {...}".to_string(),
        });
    }
    return Ok(value)
}

/// Rejects a count above the cap.
///
/// What:
///  compares against `REPEAT_CAP`.
///  Why:
///  bounds desugared expansion size.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function check_cap(count: number, pos: number): void {
///   // Rust body below is the implementation.
/// }
/// ```
fn check_cap(count: usize, pos: usize) -> Result<(), CompileError> {
    if count > REPEAT_CAP {
        return Err(CompileError::Syntax {
            pos,
            message: format!("repetition count {count} exceeds the cap of {REPEAT_CAP}"),
        })
    } else {
        return Ok(())
    }
}
