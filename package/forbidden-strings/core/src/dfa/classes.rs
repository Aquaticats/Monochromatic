//! What:
//!     Byte-class equivalence:
//!  collapse the 256 bytes into transition-equivalent groups.
//! Why:
//!      This file is the Rust module that groups the classes implementation,
//!  so the
//!          compiler gives those items one namespace and sibling modules can import that name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module classes: see exported functions and types below.
//! ```

/// What:
///     Imports the hash map used to deduplicate byte signatures.
/// Why:
///      The code below uses `HashMap` directly;
///  importing from `std/collections` keeps each
///          call site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { HashMap } from "std/collections";
/// ```
use std::collections::HashMap;

/// What:
///     Imports the byte-set leaf type and the word predicate.
/// Why:
///      The code below uses `ByteSet`,
///  `is_word_byte` directly;
///  importing from
///          `crate/charset` keeps each call site focused on the matcher logic instead of the
///          full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ByteSet, is_word_byte } from "crate/charset";
/// ```
use crate::charset::{ByteSet, is_word_byte};

/// What:
///     Imports the node algebra walked to gather classes.
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

/// The byte-class partition derived from a pattern.
///
/// What:
///  how many classes there are,
///  a byte-to-class map,
///  a representative byte
/// per class (for computing transitions),
///  and per-class word/newline flags.
///  Why:
/// transitions and acceptance depend on a byte only through its class,
///  so the
/// transition table can be `num_states * nclasses` wide instead of `* 256`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Classes = {
///   // fields documented in Rust above
/// };
/// ```
pub struct Classes {
    /// What:
    ///     Number of distinct classes.
    /// Why:
    ///      `nclasses` stores number of distinct classes,
    ///  so matcher code reads that
    ///          precomputed state by name instead of recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// nclasses: number;
    /// ```
    pub nclasses: usize,
    /// What:
    ///     Length-256 map from a byte to its class id.
    /// Why:
    ///      `class_map` stores length-256 map from a byte to its class id,
    ///  so matcher code
    ///          reads that precomputed state by name instead of recomputing or passing it
    ///          separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// class_map: number[];
    /// ```
    pub class_map: Vec<u8>,
    /// What:
    ///     One representative byte per class id.
    /// Why:
    ///      `reps` stores one representative byte per class id,
    ///  so matcher code reads that
    ///          precomputed state by name instead of recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// reps: number[];
    /// ```
    pub reps: Vec<u8>,
    /// What:
    ///     Per-class word-byte flag.
    /// Why:
    ///      `class_word` stores per-class word-byte flag,
    ///  so matcher code reads that
    ///          precomputed state by name instead of recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// class_word: boolean[];
    /// ```
    pub class_word: Vec<bool>,
    /// What:
    ///     Per-class newline flag.
    /// Why:
    ///      `class_newline` stores per-class newline flag,
    ///  so matcher code reads that
    ///          precomputed state by name instead of recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// class_newline: boolean[];
    /// ```
    pub class_newline: Vec<bool>,
}

/// Computes the byte-class partition for a node.
///
/// What:
///  gathers every distinct `Class` byte set in the node,
///  then groups bytes
/// whose `(is_word, is_newline, membership-in-each-set)` signature is identical.
/// Why:
///  two bytes with the same signature drive identical derivatives and the
/// same boundary context,
///  so they are interchangeable in the DFA.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function compute_classes(root: Node): Classes {
///   // Rust body below is the implementation.
/// }
/// ```
pub fn compute_classes(root: &Node) -> Classes {
    // What: collect the distinct leaf sets. Why: set membership is what makes two
    // bytes behave differently under a `Class` derivative.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let mut sets: Vec<ByteSet> = Vec::new();
    collect_sets(root, &mut sets);

    // What: assign a class id to each distinct byte signature.
    // Why: the signature captures every byte-dependent behavior.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let mut signature_to_id: HashMap<Vec<bool>, u8> = HashMap::new();
    let mut class_map: Vec<u8> = vec![0; 256];
    let mut reps: Vec<u8> = Vec::new();
    let mut class_word: Vec<bool> = Vec::new();
    let mut class_newline: Vec<bool> = Vec::new();
    for byte in 0u16..256 {
        let b = byte as u8;
        let signature = byte_signature(b, &sets);
        // What: reuse an existing id or mint a new one. Why: the first byte of a
        // signature becomes that class's representative.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same step as the Rust statement below, written with ordinary TS objects/functions.
        // ```
        let id = match signature_to_id.get(&signature) {
            Some(&id) => id,
            None => {
                let id = reps.len() as u8;
                signature_to_id.insert(signature, id);
                reps.push(b);
                class_word.push(is_word_byte(b));
                class_newline.push(b == b'\n');
                id
            }
        };
        class_map[b as usize] = id;
    }

    return Classes {
        nclasses: reps.len(),
        class_map,
        reps,
        class_word,
        class_newline,
    }
}

/// Builds the equivalence signature of one byte.
///
/// What:
///  word-ness,
///  newline-ness,
///  then membership in each collected set.
///  Why:
/// bytes sharing this vector produce identical transitions everywhere.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function byte_signature(b: number, sets: ByteSet[]): boolean[] {
///   // Rust body below is the implementation.
/// }
/// ```
fn byte_signature(b: u8, sets: &[ByteSet]) -> Vec<bool> {
    // What: start with the two context bits, then one bit per set.
    // Why: order is fixed so equal signatures hash and compare equal.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let mut signature = Vec::with_capacity(2 + sets.len());
    signature.push(is_word_byte(b));
    signature.push(b == b'\n');
    for set in sets {
        signature.push(set.contains(b));
    }
    return signature
}

/// Walks a node collecting its distinct `Class` byte sets.
///
/// What:
///  a structural recursion over the node tree,
///  adding each unseen set.
///  Why:
/// only `Class` nodes constrain bytes;
///  the constants and anchors do not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function collect_sets(node: Node, out: ByteSet[]): void {
///   // Rust body below is the implementation.
/// }
/// ```
fn collect_sets(node: &Node, out: &mut Vec<ByteSet>) {
    // What: recurse into children; record sets at the leaves. Why: bounded walk
    // of the AST, never of input bytes.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    match node {
        Node::Class(set) => {
            if !out.contains(set) {
                out.push(*set);
            }
        }
        Node::Concat(parts) | Node::Alt(parts) | Node::Inter(parts) => {
            for part in parts {
                collect_sets(part, out);
            }
        }
        Node::Comp(inner) => collect_sets(inner, out),
        Node::Repeat { node: body, .. } => collect_sets(body, out),
        Node::Empty | Node::Fail | Node::Top | Node::LineStart | Node::LineEnd | Node::WordBoundary => {}
    }
}
