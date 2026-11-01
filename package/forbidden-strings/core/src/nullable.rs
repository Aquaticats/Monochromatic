//! What:
//!     Position-dependent nullability:
//!  does a node match the empty string here?
//! Why:
//!      This file is the Rust module that groups the nullable implementation,
//!  so the
//!          compiler gives those items one namespace and sibling modules can import that name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module nullable: see exported functions and types below.
//! ```

/// What:
///     Imports the node algebra being tested.
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
///     Imports the boundary context that resolves anchors.
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

/// Reports whether `node` accepts the empty string at the given boundary.
///
/// What:
///  the standard Brzozowski nullability function,
///  extended so the anchors
/// read `ctx`.
///  `Empty`/`Top` are nullable;
///  `Fail`/`Class` are not;
///  `Concat` and
/// `Inter` need all children nullable;
///  `Alt` needs one;
///  `Comp` flips its child;
/// `Repeat` is nullable when zero copies are allowed or the body itself is
/// nullable;
///  the anchors consult the context.
///  Why:
///  a string is accepted exactly
/// when the residual after consuming it is nullable,
///  so this is the acceptance
/// test the matcher and DFA builder both rely on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function nullable(node: Node, ctx: Ctx): boolean {
///   // Rust body below is the implementation.
/// }
/// ```
pub fn nullable(node: &Node, ctx: Ctx) -> bool {
    // What: one arm per variant; structural recursion over the node tree (a
    // bounded walk, never over flat input). Why: nullability is defined by case.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    match node {
        Node::Empty | Node::Top => return true,
        Node::Fail | Node::Class(_) => return false,
        Node::Concat(parts) | Node::Inter(parts) => return parts.iter().all(|p| return nullable(p, ctx)),
        Node::Alt(parts) => return parts.iter().any(|p| return nullable(p, ctx)),
        Node::Comp(inner) => return !nullable(inner, ctx),
        Node::Repeat { node: body, min, .. } => return *min == 0 || nullable(body, ctx),
        Node::LineStart => return ctx.line_start,
        Node::LineEnd => return ctx.line_end,
        Node::WordBoundary => return ctx.word_before != ctx.word_after,
    }
}

/// What:
///     Unit tests for nullability,
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
#[path = "nullable_tests.rs"]
mod tests;
