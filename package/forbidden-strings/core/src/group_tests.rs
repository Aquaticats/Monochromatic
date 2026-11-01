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

use super::group_seedless;
use crate::parse::parse;

fn node(pattern: &str) -> crate::ast::node::Node {
    return parse(pattern).expect("test pattern parses")
}

#[test]
fn empty_input_yields_no_groups() {
    assert!(group_seedless(vec![]).is_empty());
}

#[test]
fn a_group_matches_any_of_its_members_as_a_substring() {
    let engines = group_seedless(vec![node("[0-9]{4}"), node("[a-z]{4}")]);
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
    let engines = group_seedless(vec![node("[A-Z2-7]{16}")]);
    assert!(!engines.is_empty());
    let sixteen = b"ABCDEFGH23456777";
    assert!(engines.iter().any(|engine| return engine.is_match(sixteen)));
    assert!(!engines.iter().any(|engine| return engine.is_match(b"ABCDEFGH")));
}
