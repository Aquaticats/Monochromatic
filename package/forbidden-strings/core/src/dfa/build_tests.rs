// What:  unit tests for the DFA build's residual-size guard.
// Why:     This file groups the build test cases so behavior changes fail near the code path
//          they protect.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("build", () => {
//   // test cases below
// });
// ```

use super::{build_dfa_within, residual_too_large};
use crate::ast::node::Node;
use crate::charset::singleton;
use crate::error::CompileError;

// What:    A node with `count` leaf children (count + 1 sub-nodes total).
// Why:     The program needs this named step so callers can reuse the behavior without copying
//          its body.
//
// In TS you'd write (pseudocode):
// ```ts
// function wide_concat(count: number): Node {
//   // Rust body below is the implementation.
// }
// ```
fn wide_concat(count: usize) -> Node {
    return Node::Concat((0..count).map(|_| return Node::Class(singleton(b'a'))).collect())
}

#[test]
fn small_residuals_are_under_the_cap() {
    assert!(!residual_too_large(&Node::Class(singleton(b'a'))));
    assert!(!residual_too_large(&Node::Empty));
    assert!(!residual_too_large(&wide_concat(100)));
}

#[test]
fn an_oversized_residual_is_flagged() {
    assert!(residual_too_large(&wide_concat(5_000)));
}

#[test]
fn the_guard_descends_into_every_recursive_arm() {
    // What:    Each recursive node kind must carry its child's size up, or a giant residual
    //          hidden inside it would slip past the guard.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let big = wide_concat(5_000);
    assert!(residual_too_large(&Node::Alt(vec![big.clone()])));
    assert!(residual_too_large(&Node::Inter(vec![big.clone()])));
    assert!(residual_too_large(&Node::Comp(Box::new(big.clone()))));
    assert!(residual_too_large(&Node::Repeat { node: Box::new(big), min: 1, max: 2 }));
}

#[test]
fn build_bails_with_state_cap_on_an_oversized_root() {
    // What:    A root residual past the cap aborts the build as StateCap (the caller then
    //          falls back to the counting back-end instead of exhausting memory).
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let result = build_dfa_within(wide_concat(5_000), 10_000);
    assert!(matches!(result, Err(CompileError::StateCap { .. })));
}

#[test]
fn build_succeeds_on_a_normal_pattern() {
    // What:    A normal pattern builds well under the cap.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let dfa = build_dfa_within(crate::parse::parse("AKIA[A-Z2-7]{4}").expect("parses"), 10_000);
    assert!(dfa.is_ok());
}

#[test]
fn the_determinization_state_cap_is_an_inclusive_ceiling() {
    // What:    Build once with a generous cap to learn the exact state count, then rebuild
    //          with the cap set to exactly that count. The check is `states.len() > cap`, so
    //          reaching the cap is allowed (Ok); a cap one below must abort as StateCap. This
    //          pins the boundary against off-by-one mutants (`>` becoming `==` or `>=`).
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let node = crate::parse::parse("[a-z]{3}").expect("parses");
    let exact = build_dfa_within(node.clone(), 10_000).expect("builds").num_states as usize;
    assert!(exact >= 2, "the pattern should produce several states, got {exact}");
    assert!(build_dfa_within(node.clone(), exact).is_ok(), "a cap equal to the count is allowed");
    assert!(
        matches!(build_dfa_within(node, exact - 1), Err(CompileError::StateCap { .. })),
        "a cap one below the state count must abort as StateCap",
    );
}
