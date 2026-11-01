//! What:
//!     Pattern parsing:
//!  text to a normalized node,
//!  plus the empty-match guard.
//! Why:
//!      This file is the Rust module that groups the parse implementation,
//!  so the
//!          compiler gives those items one namespace and sibling modules can import that name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module parse: see exported functions and types below.
//! ```

/// What:
///     The verbose-mode-aware byte cursor.
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./cursor";
/// ```
mod cursor;

/// What:
///     Backslash-escape parsing.
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./escape";
/// ```
mod escape;

/// What:
///     Character-class parsing.
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./class";
/// ```
mod class;

/// What:
///     Bounded-repetition desugaring.
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./repeat";
/// ```
mod repeat;

/// What:
///     The recursive-descent grammar.
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./grammar";
/// ```
mod grammar;

/// What:
///     Imports the node algebra produced by parsing.
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
///     Imports the boundary context used by the empty-match guard.
/// Why:
///      The code below uses `Ctx` directly;
///  importing from `crate/context` keeps each call
///          site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Ctx } from "crate/context";
/// ```
use crate::context::Ctx;

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
///     Imports nullability for the empty-match guard.
/// Why:
///      The code below uses `nullable` directly;
///  importing from `crate/nullable` keeps each
///          call site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { nullable } from "crate/nullable";
/// ```
use crate::nullable::nullable;

/// What:
///     Imports the cursor.
/// Why:
///      The code below uses `Cursor` directly;
///  importing from `cursor` keeps each call site
///          focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Cursor } from "cursor";
/// ```
use cursor::Cursor;

/// What:
///     Imports the grammar entry point.
/// Why:
///      The code below uses `parse_setexpr` directly;
///  importing from `grammar` keeps each
///          call site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parse_setexpr } from "grammar";
/// ```
use grammar::parse_setexpr;

/// Parses a pattern into a normalized,
///  non-empty-matchable node.
///
/// What:
///  runs the grammar over the whole pattern,
///  requires all input to be
/// consumed,
///  and rejects a pattern that can match the empty string.
///  Why:
///  this is
/// the single front door from pattern text to the node the DFA builder consumes,
/// and the empty-match guard prevents a rule that would flag every line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parse(pattern: string): Node {
///   // Rust body below is the implementation.
/// }
/// ```
pub fn parse(pattern: &str) -> Result<Node, CompileError> {
    let mut cur = Cursor::new(pattern.as_bytes());
    let node = parse_setexpr(&mut cur, None)?;
    cur.skip_ignorable();
    // What: any leftover bytes mean the grammar stopped early. Why: a stray `)`
    // or operator should be reported rather than silently ignored.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    if !cur.eof() {
        return Err(CompileError::Syntax {
            pos: cur.pos(),
            message: "unexpected trailing input".to_string(),
        });
    }
    if can_match_empty(&node) {
        return Err(CompileError::EmptyMatchable);
    }
    return Ok(node)
}

/// Reports whether the node can match the empty string at some real boundary.
///
/// What:
///  checks nullability across the realizable anchor contexts (a line start
/// cannot have a word byte before it,
///  a line end cannot have one after it).
///  Why:
/// under unanchored search an empty match means the rule matches every input,
///  so
/// such patterns are rejected at compile time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function can_match_empty(node: Node): boolean {
///   // Rust body below is the implementation.
/// }
/// ```
fn can_match_empty(node: &Node) -> bool {
    // What: enumerate the four context booleans, skipping impossible combos.
    // Why: a precise realizable-context check avoids false rejections while still
    // catching every genuinely empty-matchable pattern.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    for line_start in [false, true] {
        for line_end in [false, true] {
            for word_before in [false, true] {
                for word_after in [false, true] {
                    // What: drop unrealizable combinations. Why: a line boundary
                    // forces the adjacent byte to be a non-word newline or edge.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
                    // ```
                    if (line_start && word_before) || (line_end && word_after) {
                        continue;
                    }
                    let ctx = Ctx {
                        line_start,
                        line_end,
                        word_before,
                        word_after,
                    };
                    if nullable(node, ctx) {
                        return true;
                    }
                }
            }
        }
    }
    return false
}

/// What:
///     Unit tests for the parser,
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
#[path = "parse_tests.rs"]
mod tests;
