//! What:
//!     Smart constructors that keep nodes canonical so derivative states stay finite.
//! Why:
//!      This file is the Rust module that groups the smart implementation,
//!  so the
//!          compiler gives those items one namespace and sibling modules can import that name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module smart: see exported functions and types below.
//! ```

/// What:
///     Imports the byte-set leaf type for the `class` constructor.
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
///     Imports the node algebra these constructors build.
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

/// Builds a sequence,
///  dropping `Empty`,
///  absorbing `Fail`,
///  and flattening.
///
/// What:
///  concatenation is associative with `Empty` as identity and `Fail` as a
/// zero.
///  Why:
///  normalizing here means equal languages produce equal nodes,
///  which
/// bounds the number of derivative states.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function concat(parts: Node[]): Node {
///   // Rust body below is the implementation.
/// }
/// ```
pub fn concat(parts: Vec<Node>) -> Node {
    // What: flatten one level of nested `Concat` and drop `Empty` children.
    // Why: associativity and identity, applied as the parts are collected.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let mut flat: Vec<Node> = Vec::new();
    for part in parts {
        // What: a `Fail` anywhere makes the whole sequence unmatchable.
        // Why: `Fail` is the multiplicative zero of concatenation.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same step as the Rust statement below, written with ordinary TS objects/functions.
        // ```
        if part == Node::Fail {
            return Node::Fail;
        }
        match part {
            Node::Empty => {}
            Node::Concat(inner) => flat.extend(inner),
            other => flat.push(other),
        }
    }
    // What: collapse the degenerate arities. Why: a 0- or 1-element sequence is
    // not a `Concat`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    if flat.is_empty() {
        return Node::Empty
    } else if flat.len() == 1 {
        return flat.into_iter().next().unwrap()
    } else {
        return Node::Concat(flat)
    }
}

/// Builds an alternation with ACI normalization (flatten,
///  drop `Fail`,
///  sort,
/// dedup),
///  absorbing `Top`.
///
/// What:
///  union is associative,
///  commutative,
///  and idempotent,
///  with `Fail` as
/// identity and `Top` as the absorbing element.
///  Why:
///  sorting plus dedup gives a
/// canonical form so two equal unions are the same node.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function alt(parts: Node[]): Node {
///   // Rust body below is the implementation.
/// }
/// ```
pub fn alt(parts: Vec<Node>) -> Node {
    // What: flatten nested `Alt`, drop `Fail`, and short-circuit on `Top`.
    // Why: union identities applied while collecting.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let mut flat: Vec<Node> = Vec::new();
    for part in parts {
        if part == Node::Top {
            return Node::Top;
        }
        match part {
            Node::Fail => {}
            Node::Alt(inner) => flat.extend(inner),
            other => flat.push(other),
        }
    }
    // What: sort then dedup for the commutative-idempotent canonical form.
    // Why: `Ord`/`Eq` on `Node` make this a stable canonicalization.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    flat.sort();
    flat.dedup();
    if flat.is_empty() {
        return Node::Fail
    } else if flat.len() == 1 {
        return flat.into_iter().next().unwrap()
    } else {
        return Node::Alt(flat)
    }
}

/// Builds an intersection with ACI normalization,
///  absorbing `Fail` and dropping
/// `Top`.
///
/// What:
///  intersection is associative,
///  commutative,
///  idempotent,
///  with `Top` as
/// identity and `Fail` as the absorbing element.
///  Why:
///  canonical form bounds
/// state growth from `&` and complement.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function inter(parts: Node[]): Node {
///   // Rust body below is the implementation.
/// }
/// ```
pub fn inter(parts: Vec<Node>) -> Node {
    // What: flatten nested `Inter`, drop `Top`, short-circuit on `Fail`.
    // Why: intersection identities applied while collecting.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let mut flat: Vec<Node> = Vec::new();
    for part in parts {
        if part == Node::Fail {
            return Node::Fail;
        }
        match part {
            Node::Top => {}
            Node::Inter(inner) => flat.extend(inner),
            other => flat.push(other),
        }
    }
    flat.sort();
    flat.dedup();
    // What: an empty intersection is the universe. Why: `Top` is the identity,
    // so intersecting nothing constrains nothing.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    if flat.is_empty() {
        return Node::Top
    } else if flat.len() == 1 {
        return flat.into_iter().next().unwrap()
    } else {
        return Node::Inter(flat)
    }
}

/// Builds a complement,
///  collapsing double negation and the constants.
///
/// What:
///  `~~x = x`,
///  `~Fail = Top`,
///  `~Top = Fail`.
///  Why:
///  keeps complement nodes
/// from stacking and resolves the constant cases immediately.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function comp(inner: Node): Node {
///   // Rust body below is the implementation.
/// }
/// ```
pub fn comp(inner: Node) -> Node {
    match inner {
        Node::Comp(x) => return *x,
        Node::Fail => return Node::Top,
        Node::Top => return Node::Fail,
        other => return Node::Comp(Box::new(other)),
    }
}

/// Builds a bounded repetition,
///  collapsing the trivial arities.
///
/// What:
///  `max == 0` matches nothing-repeated (the empty string);
///  a `Fail` body is
/// `Empty` when zero reps are allowed else `Fail`;
///  exactly-one becomes the body.
/// Why:
///  keeps the count in a `Repeat` node (later a counter register) rather than
/// unrolling it into states,
///  and removes degenerate nodes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function repeat(node: Node, min: number, max: number): Node {
///   // Rust body below is the implementation.
/// }
/// ```
pub fn repeat(node: Node, min: usize, max: usize) -> Node {
    if max == 0 {
        return Node::Empty;
    }
    if node == Node::Fail {
        return if min == 0 { Node::Empty } else { Node::Fail };
    }
    if min == 1 && max == 1 {
        return node;
    }
    return Node::Repeat {
        node: Box::new(node),
        min,
        max,
    }
}

/// Builds `x?` as `x{0,1}`.
///
/// What:
///  optional is repetition between zero and one.
///  Why:
///  a single spelling for
/// the optional tail used by `?` and by `{n,m}` upper bounds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function optional(node: Node): Node {
///   // Rust body below is the implementation.
/// }
/// ```
pub fn optional(node: Node) -> Node {
    return repeat(node, 0, 1)
}

/// Builds a one-byte class node,
///  collapsing an empty set to `Fail`.
///
/// What:
///  a class matching no byte matches nothing.
///  Why:
///  keeps `Fail` as the sole
/// representation of the empty language so nullability and derivatives stay
/// simple.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function class(set: ByteSet): Node {
///   // Rust body below is the implementation.
/// }
/// ```
pub fn class(set: ByteSet) -> Node {
    if set.is_empty() {
        return Node::Fail
    } else {
        return Node::Class(set)
    }
}

/// What:
///     Unit tests for the smart constructors,
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
#[path = "smart_tests.rs"]
mod tests;
