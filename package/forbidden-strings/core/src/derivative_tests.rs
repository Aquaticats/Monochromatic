// What:  unit tests for Brzozowski byte derivatives, per node variant plus a few
//        end-to-end derive-a-string checks.
// Why:     This file groups the derivative test cases so behavior changes fail near the code
//          path they protect.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("derivative", () => {
//   // test cases below
// });
// ```

use super::derivative;
use crate::ast::node::Node;
use crate::charset::singleton;
use crate::context::Ctx;
use crate::nullable::nullable;

fn off() -> Ctx {
    return Ctx { line_start: false, line_end: false, word_before: false, word_after: false }
}

fn lit(b: u8) -> Node {
    return Node::Class(singleton(b))
}

// What:    Derives `input` byte by byte (interior context) and reports final nullability.
// Why:     The program needs this named step so callers can reuse the behavior without copying
//          its body.
//
// In TS you'd write (pseudocode):
// ```ts
// function matches_interior(node: Node, input: Uint8Array): boolean {
//   // Rust body below is the implementation.
// }
// ```
fn matches_interior(node: &Node, input: &[u8]) -> bool {
    let mut residual = node.clone();
    for &b in input {
        residual = derivative(&residual, b, off());
    }
    return nullable(&residual, off())
}

#[test]
fn constants_and_anchors_derive_to_fail_or_top() {
    assert_eq!(derivative(&Node::Empty, b'a', off()), Node::Fail);
    assert_eq!(derivative(&Node::Fail, b'a', off()), Node::Fail);
    assert_eq!(derivative(&Node::Top, b'a', off()), Node::Top);
    assert_eq!(derivative(&Node::LineStart, b'a', off()), Node::Fail);
    assert_eq!(derivative(&Node::LineEnd, b'a', off()), Node::Fail);
    assert_eq!(derivative(&Node::WordBoundary, b'a', off()), Node::Fail);
}

#[test]
fn class_consumes_only_members() {
    assert_eq!(derivative(&lit(b'a'), b'a', off()), Node::Empty);
    assert_eq!(derivative(&lit(b'a'), b'b', off()), Node::Fail);
}

#[test]
fn concat_uses_the_nullable_prefix_rule() {
    let ab = Node::Concat(vec![lit(b'a'), lit(b'b')]);
    // What:    D_a(ab) = b ; D_b(ab) = Fail.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert_eq!(derivative(&ab, b'a', off()), lit(b'b'));
    assert_eq!(derivative(&ab, b'b', off()), Node::Fail);
}

#[test]
fn alt_distributes_over_the_derivative() {
    let alt = Node::Alt(vec![lit(b'a'), lit(b'b')]);
    assert_eq!(derivative(&alt, b'a', off()), Node::Empty);
    assert_eq!(derivative(&alt, b'b', off()), Node::Empty);
    assert_eq!(derivative(&alt, b'c', off()), Node::Fail);
}

#[test]
fn comp_negates_the_derivative() {
    // What:    D_a(~a) = ~(D_a a) = ~Empty.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert_eq!(derivative(&Node::Comp(Box::new(lit(b'a'))), b'a', off()), Node::Comp(Box::new(Node::Empty)));
}

#[test]
fn derive_a_string_decides_membership() {
    let abc = Node::Concat(vec![lit(b'a'), lit(b'b'), lit(b'c')]);
    assert!(matches_interior(&abc, b"abc"));
    assert!(!matches_interior(&abc, b"ab"));
    assert!(!matches_interior(&abc, b"abcd"));
    assert!(!matches_interior(&abc, b"axc"));
}

#[test]
fn repeat_matches_the_allowed_counts() {
    // What:    a{2,3} interior: 2 or 3 a's match, 1 or 4 do not.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let rep = Node::Repeat { node: Box::new(lit(b'a')), min: 2, max: 3 };
    assert!(!matches_interior(&rep, b"a"));
    assert!(matches_interior(&rep, b"aa"));
    assert!(matches_interior(&rep, b"aaa"));
    assert!(!matches_interior(&rep, b"aaaa"));
}

#[test]
fn intersection_requires_both_sides() {
    // What:    [a-c] & [b-d] interior matches exactly 'b' or 'c'.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let mut left = crate::charset::ByteSet::empty();
    left.insert_range(b'a', b'c');
    let mut right = crate::charset::ByteSet::empty();
    right.insert_range(b'b', b'd');
    let inter = Node::Inter(vec![Node::Class(left), Node::Class(right)]);
    assert!(matches_interior(&inter, b"b"));
    assert!(matches_interior(&inter, b"c"));
    assert!(!matches_interior(&inter, b"a"));
    assert!(!matches_interior(&inter, b"d"));
}
