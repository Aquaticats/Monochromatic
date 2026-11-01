// What:  unit tests for position-dependent nullability across every node variant and
//        the anchor contexts.
// Why:     This file groups the nullable test cases so behavior changes fail near the code path
//          they protect.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("nullable", () => {
//   // test cases below
// });
// ```

use super::nullable;
use crate::ast::node::Node;
use crate::charset::singleton;
use crate::context::Ctx;

// What:    A context with all four boundary flags off (mid-input, between non-word bytes).
// Why:     The program needs this named step so callers can reuse the behavior without copying
//          its body.
//
// In TS you'd write (pseudocode):
// ```ts
// function off(): Ctx {
//   // Rust body below is the implementation.
// }
// ```
fn off() -> Ctx {
    return Ctx { line_start: false, line_end: false, word_before: false, word_after: false }
}

fn lit(b: u8) -> Node {
    return Node::Class(singleton(b))
}

#[test]
fn constants_and_class() {
    assert!(nullable(&Node::Empty, off()));
    assert!(nullable(&Node::Top, off()));
    assert!(!nullable(&Node::Fail, off()));
    assert!(!nullable(&lit(b'a'), off()));
}

#[test]
fn concat_needs_all_nullable() {
    assert!(nullable(&Node::Concat(vec![Node::Empty, Node::Top]), off()));
    assert!(!nullable(&Node::Concat(vec![Node::Empty, lit(b'a')]), off()));
}

#[test]
fn alt_needs_one_nullable() {
    assert!(nullable(&Node::Alt(vec![lit(b'a'), Node::Empty]), off()));
    assert!(!nullable(&Node::Alt(vec![lit(b'a'), lit(b'b')]), off()));
}

#[test]
fn inter_needs_all_nullable() {
    assert!(nullable(&Node::Inter(vec![Node::Empty, Node::Top]), off()));
    assert!(!nullable(&Node::Inter(vec![Node::Empty, lit(b'a')]), off()));
}

#[test]
fn comp_flips_child() {
    assert!(nullable(&Node::Comp(Box::new(lit(b'a'))), off()));
    assert!(!nullable(&Node::Comp(Box::new(Node::Empty)), off()));
}

#[test]
fn repeat_nullable_when_min_zero_or_body_nullable() {
    assert!(nullable(&Node::Repeat { node: Box::new(lit(b'a')), min: 0, max: 3 }, off()));
    assert!(!nullable(&Node::Repeat { node: Box::new(lit(b'a')), min: 2, max: 3 }, off()));
    assert!(nullable(&Node::Repeat { node: Box::new(Node::Empty), min: 1, max: 3 }, off()));
}

#[test]
fn line_anchors_follow_context() {
    let at_start = Ctx { line_start: true, ..off() };
    let at_end = Ctx { line_end: true, ..off() };
    assert!(nullable(&Node::LineStart, at_start));
    assert!(!nullable(&Node::LineStart, off()));
    assert!(nullable(&Node::LineEnd, at_end));
    assert!(!nullable(&Node::LineEnd, off()));
}

#[test]
fn word_boundary_is_a_word_ness_change() {
    let entering = Ctx { word_before: false, word_after: true, ..off() };
    let inside = Ctx { word_before: true, word_after: true, ..off() };
    assert!(nullable(&Node::WordBoundary, entering));
    assert!(!nullable(&Node::WordBoundary, inside));
    assert!(!nullable(&Node::WordBoundary, off()));
}
