//! What:
//!     Brzozowski byte derivatives over the node algebra.
//! Why:
//!      This file is the Rust module that groups the derivative implementation,
//!  so the
//!          compiler gives those items one namespace and sibling modules can import that name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module derivative: see exported functions and types below.
//! ```

/// What:
///     Imports the node algebra being differentiated.
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
///     Imports the smart constructors that keep results canonical.
/// Why:
///      The code below uses `alt`,
///  `comp`,
///  `concat`,
///  `inter`,
///  `repeat` directly;
///  importing
///          from `crate/ast/smart` keeps each call site focused on the matcher logic instead of
///          the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import {
///   alt,
///   comp,
///   concat,
///   inter,
///   repeat,
/// } from "crate/ast/smart";
/// ```
use crate::ast::smart::{alt, comp, concat, inter, repeat};

/// What:
///     Imports the boundary context used when a sequence steps past a nullable anchor.
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
///     Imports nullability,
///  needed by the concatenation rule.
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

/// Returns the derivative of `node` with respect to one `byte` at `ctx`.
///
/// What:
///  `D_b(r)` is the regex matching the remaining input after a leading `b`.
/// Constants and anchors derive to `Fail` or `Top`;
///  a class derives to `Empty`
/// when it contains `b`;
///  the boolean operators distribute;
///  concatenation uses
/// the nullable-prefix rule below.
///  Why:
///  feeding the input one byte at a time
/// through this function and checking nullability at the end decides membership,
/// and the boolean cases are what give the engine `&` and `~`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function derivative(node: Node, byte: number, ctx: Ctx): Node {
///   // Rust body below is the implementation.
/// }
/// ```
pub fn derivative(node: &Node, byte: u8, ctx: Ctx) -> Node {
    // What: one arm per variant; recursion is structural over child nodes.
    // Why: the derivative is defined inductively on the regex shape.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    match node {
        Node::Empty | Node::Fail => return Node::Fail,
        Node::Top => return Node::Top,
        Node::LineStart | Node::LineEnd | Node::WordBoundary => return Node::Fail,
        Node::Class(set) => {
            // What: a class consumes `b` iff `b` is a member. Why: after
            // consuming the matched byte nothing remains, hence `Empty`.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // // Same step as the Rust statement below, written with ordinary TS objects/functions.
            // ```
            if set.contains(byte) {
                return Node::Empty
            } else {
                return Node::Fail
            }
        }
        Node::Alt(parts) => return alt(parts.iter().map(|p| return derivative(p, byte, ctx)).collect()),
        Node::Inter(parts) => return inter(parts.iter().map(|p| return derivative(p, byte, ctx)).collect()),
        Node::Comp(inner) => return comp(derivative(inner, byte, ctx)),
        Node::Concat(parts) => return derivative_concat(parts, byte, ctx),
        Node::Repeat { node: body, min, max } => return derivative_repeat(body, *min, *max, byte, ctx),
    }
}

/// Interim countdown derivative of a bounded repetition.
///
/// What:
///  consuming a byte spends one copy of the body,
///  leaving `D_b(R)` followed
/// by the decremented repetition `R{min-1, max-1}`;
///  when the body is nullable that
/// first copy may instead match empty,
///  so the byte may start a later copy,
///  adding
/// `D_b(R{min-1, max-1})`.
///  Why:
///  this keeps the eager derivative path correct while
/// the counting back-end (which carries the count in a register,
///  not in states) is
/// built;
///  it still blows up under search,
///  so it is a stepping stone,
///  not the goal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function derivative_repeat(node: Node, min: number, max: number, byte: number, ctx: Ctx): Node {
///   // Rust body below is the implementation.
/// }
/// ```
fn derivative_repeat(node: &Node, min: usize, max: usize, byte: u8, ctx: Ctx) -> Node {
    // What: an exhausted repetition (no copies left) matches only the empty
    // string, whose derivative is `Fail`. Why: nothing can be consumed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    if max == 0 {
        return Node::Fail;
    }
    // What: the repetition with one fewer mandatory and one fewer allowed copy.
    // Why: spending a copy on this byte leaves exactly this tail to match.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let dec = repeat(node.clone(), min.saturating_sub(1), max - 1);
    // What: the body advances by `b`, then the decremented repetition follows.
    // Why: the common case where this byte belongs to the current copy.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let main = concat(vec![derivative(node, byte, ctx), dec.clone()]);
    if nullable(node, ctx) {
        // What: the current copy matched empty, so `b` opens a later copy.
        // Why: a nullable body lets the byte skip into the decremented tail.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same step as the Rust statement below, written with ordinary TS objects/functions.
        // ```
        return alt(vec![main, derivative(&dec, byte, ctx)])
    } else {
        return main
    }
}

/// Derivative of a concatenation,
///  summing over its nullable prefix.
///
/// What:
///  for `x0 x1 ...`,
///  the derivative is `D(x0)·x1...` plus,
///  when `x0` is
/// nullable here,
///  the derivative of the tail,
///  and so on through every leading
/// nullable factor (so zero-width anchors at the front are stepped over).
///  Why:
/// the empty string a nullable factor matches lets the next factor also start on
/// this same byte,
///  which is exactly how anchors inside a sequence take effect.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function derivative_concat(parts: Node[], byte: number, ctx: Ctx): Node {
///   // Rust body below is the implementation.
/// }
/// ```
fn derivative_concat(parts: &[Node], byte: u8, ctx: Ctx) -> Node {
    // What: accumulate one alternative per leading nullable factor; an index
    // loop is used because each step needs the tail slice `parts[i + 1..]`.
    // Why: lookahead into the remaining factors is the allowed counter-loop case.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let mut branches: Vec<Node> = Vec::new();
    for i in 0..parts.len() {
        // What: D(parts[i]) followed by the untouched tail parts[i+1..].
        // Why: this is the contribution of starting the byte inside factor i.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same step as the Rust statement below, written with ordinary TS objects/functions.
        // ```
        let mut seq: Vec<Node> = Vec::with_capacity(parts.len() - i);
        seq.push(derivative(&parts[i], byte, ctx));
        seq.extend_from_slice(&parts[i + 1..]);
        branches.push(concat(seq));
        // What: only continue past factor i if it can match empty here.
        // Why: a non-nullable factor must consume the byte, so later factors
        // cannot also begin on it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same step as the Rust statement below, written with ordinary TS objects/functions.
        // ```
        if !nullable(&parts[i], ctx) {
            break;
        }
    }
    return alt(branches)
}

/// What:
///     Unit tests for byte derivatives,
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
#[path = "derivative_tests.rs"]
mod tests;
