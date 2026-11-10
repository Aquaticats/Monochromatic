// What:  unit tests for grouping literal-free rules into union DFAs.
// Why:     This file groups the group test cases so behavior changes fail near the code path
//          they protect.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("group", () => {
//   // test cases below
// });
// ```

use super::{GROUP_DFA_CAP, group_seedless};
use crate::ast::node::Node;
use crate::ast::smart::concat;
use crate::build::build_engine;
use crate::dfa::build_dfa_within;
use crate::engine::Engine;
use crate::error::CompileError;
use crate::parse::parse;

fn node(pattern: &str) -> Node {
    return parse(pattern).expect("test pattern parses")
}

/// Groups test nodes the way a ruleset does,
///  giving every rule its own engine as the fallback.
///
/// What:
///  builds each node's per-rule engine and passes them as the group fallbacks.
///  Why:
///  the
/// production caller pairs the seedless nodes with those same engines,
///  so the tests
/// exercise the real coverage contract.
fn grouped_with_own_engines(nodes: &[Node]) -> Vec<Engine> {
    let per_rule: Vec<Engine> = nodes
        .iter()
        .map(|node| return build_engine(node.clone()).expect("a test rule builds its own engine"))
        .collect();
    let fallbacks: Vec<&Engine> = per_rule.iter().collect();
    return group_seedless(nodes, &fallbacks)
}

#[test]
fn empty_input_yields_no_groups() {
    assert!(group_seedless(&[], &[]).is_empty());
}

#[test]
fn a_group_matches_any_of_its_members_as_a_substring() {
    let engines = grouped_with_own_engines(&[node("[0-9]{4}"), node("[a-z]{4}")]);
    assert!(!engines.is_empty());
    let any = |line: &[u8]| return engines.iter().any(|engine| return engine.is_match(line));
    // What:    a 4-digit run.
    // Why:     The nearby assertion or value needs this note so the test records the exact
    //          behavior being pinned.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same assertion or value, with the important expectation named above.
    // ```
    assert!(any(b"xx 1234 xx"));
    // What:    a 4-letter run.
    // Why:     The nearby assertion or value needs this note so the test records the exact
    //          behavior being pinned.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same assertion or value, with the important expectation named above.
    // ```
    assert!(any(b"xx abcd xx"));
    // What:    neither run reaches length 4.
    // Why:     The nearby assertion or value needs this note so the test records the exact
    //          behavior being pinned.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same assertion or value, with the important expectation named above.
    // ```
    assert!(!any(b"xx 12 ab xx"));
}

#[test]
fn a_single_member_groups_and_matches() {
    let engines = grouped_with_own_engines(&[node("[A-Z2-7]{16}")]);
    assert!(!engines.is_empty());
    let sixteen = b"ABCDEFGH23456777";
    assert!(engines.iter().any(|engine| return engine.is_match(sixteen)));
    assert!(!engines.iter().any(|engine| return engine.is_match(b"ABCDEFGH")));
}

#[test]
fn a_group_above_the_state_cap_still_covers_its_rule() {
    // What:    A literal-free rule whose own search DFA needs more states than one union group
    //          may hold is still matched by the returned engines, because the group state cap
    //          is a throughput budget rather than a coverage limit. `[0-9].{11}` (a digit
    //          followed by eleven bytes) determinizes past the cap, so the group build fails
    //          and the rule must fall back to the engine it built for itself.
    // Why:     Dropping the group silently removed the rule from every boolean path while
    //          `matches` still reported it, the false-negative direction for a deny-rule
    //          scanner.
    let nodes = vec![node("[0-9].{11}")];
    let wrapped = concat(vec![Node::Top, nodes[0].clone()]);
    assert!(
        matches!(build_dfa_within(wrapped, GROUP_DFA_CAP), Err(CompileError::StateCap { .. })),
        "the pattern must exceed the group state cap for this case to cover the fallback",
    );
    let engines = grouped_with_own_engines(&nodes);
    assert_eq!(engines.len(), 1, "an over-cap rule keeps one engine of its own");
    assert!(
        engines.iter().any(|engine| return engine.is_match(b"id 123456789012 ok")),
        "the over-cap rule must still match the line it matched before",
    );
}
