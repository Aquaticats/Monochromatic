// What:  unit tests for the smart constructors that keep nodes canonical.
// Why:     This file groups the smart test cases so behavior changes fail near the code path
//          they protect.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("smart", () => {
//   // test cases below
// });
// ```

use super::{alt, class, comp, concat, inter, optional, repeat};
use crate::ast::node::Node;
use crate::charset::singleton;

// What:    A distinct one-byte class leaf, for building composite nodes.
// Why:     The program needs this named step so callers can reuse the behavior without copying
//          its body.
//
// In TS you'd write (pseudocode):
// ```ts
// function lit(b: number): Node {
//   // Rust body below is the implementation.
// }
// ```
fn lit(b: u8) -> Node {
    return Node::Class(singleton(b))
}

#[test]
fn concat_drops_empty_and_collapses_arity() {
    assert_eq!(concat(vec![]), Node::Empty);
    assert_eq!(concat(vec![Node::Empty, Node::Empty]), Node::Empty);
    assert_eq!(concat(vec![lit(b'a')]), lit(b'a'));
    assert_eq!(concat(vec![Node::Empty, lit(b'a'), Node::Empty]), lit(b'a'));
    assert_eq!(concat(vec![lit(b'a'), lit(b'b')]), Node::Concat(vec![lit(b'a'), lit(b'b')]));
}

#[test]
fn concat_absorbs_fail() {
    assert_eq!(concat(vec![lit(b'a'), Node::Fail, lit(b'b')]), Node::Fail);
}

#[test]
fn concat_flattens_nested() {
    let nested = concat(vec![concat(vec![lit(b'a'), lit(b'b')]), lit(b'c')]);
    assert_eq!(nested, Node::Concat(vec![lit(b'a'), lit(b'b'), lit(b'c')]));
}

#[test]
fn alt_is_commutative_idempotent_and_sorted() {
    // What:    Order-independent and deduped: {a,b} from either order, and {a,a} == a.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert_eq!(alt(vec![lit(b'b'), lit(b'a')]), alt(vec![lit(b'a'), lit(b'b')]));
    assert_eq!(alt(vec![lit(b'a'), lit(b'a')]), lit(b'a'));
}

#[test]
fn alt_identity_and_absorbing() {
    assert_eq!(alt(vec![]), Node::Fail);
    assert_eq!(alt(vec![Node::Fail, lit(b'a')]), lit(b'a'));
    assert_eq!(alt(vec![lit(b'a'), Node::Top]), Node::Top);
}

#[test]
fn alt_flattens_nested() {
    let nested = alt(vec![alt(vec![lit(b'a'), lit(b'b')]), lit(b'c')]);
    assert_eq!(nested, alt(vec![lit(b'a'), lit(b'b'), lit(b'c')]));
    assert!(matches!(nested, Node::Alt(ref v) if v.len() == 3));
}

#[test]
fn inter_identity_and_absorbing() {
    assert_eq!(inter(vec![]), Node::Top);
    assert_eq!(inter(vec![Node::Top, lit(b'a')]), lit(b'a'));
    assert_eq!(inter(vec![lit(b'a'), Node::Fail]), Node::Fail);
}

#[test]
fn inter_is_commutative_idempotent_and_flattened() {
    assert_eq!(inter(vec![lit(b'b'), lit(b'a')]), inter(vec![lit(b'a'), lit(b'b')]));
    assert_eq!(inter(vec![lit(b'a'), lit(b'a')]), lit(b'a'));
    let nested = inter(vec![inter(vec![lit(b'a'), lit(b'b')]), lit(b'c')]);
    assert!(matches!(nested, Node::Inter(ref v) if v.len() == 3));
}

#[test]
fn comp_collapses_double_negation_and_constants() {
    assert_eq!(comp(Node::Fail), Node::Top);
    assert_eq!(comp(Node::Top), Node::Fail);
    assert_eq!(comp(comp(lit(b'a'))), lit(b'a'));
    assert_eq!(comp(lit(b'a')), Node::Comp(Box::new(lit(b'a'))));
}

#[test]
fn repeat_collapses_trivial_arities() {
    assert_eq!(repeat(lit(b'a'), 0, 0), Node::Empty);
    assert_eq!(repeat(lit(b'a'), 1, 1), lit(b'a'));
    assert_eq!(
        repeat(lit(b'a'), 2, 5),
        Node::Repeat { node: Box::new(lit(b'a')), min: 2, max: 5 },
    );
}

#[test]
fn repeat_handles_fail_body() {
    assert_eq!(repeat(Node::Fail, 0, 3), Node::Empty);
    assert_eq!(repeat(Node::Fail, 2, 3), Node::Fail);
}

#[test]
fn optional_is_zero_to_one() {
    assert_eq!(optional(lit(b'a')), Node::Repeat { node: Box::new(lit(b'a')), min: 0, max: 1 });
}

#[test]
fn class_collapses_empty_set_to_fail() {
    assert_eq!(class(crate::charset::ByteSet::empty()), Node::Fail);
    assert_eq!(class(singleton(b'z')), Node::Class(singleton(b'z')));
}
