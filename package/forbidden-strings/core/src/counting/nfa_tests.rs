// What:  unit tests for counting-NFA decode validation.
// Why:     This file groups the nfa test cases so behavior changes fail near the code path they
//          protect.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("nfa", () => {
//   // test cases below
// });
// ```

use super::CountingNfa;
use crate::counting::build_nfa;
use crate::parse::parse;

fn good() -> CountingNfa {
    return build_nfa(&parse("AKIA[A-Z2-7]{4}").expect("parses")).expect("expressible as an NFA")
}

#[test]
fn validate_accepts_a_built_nfa() {
    assert!(good().validate().is_ok());
}

#[test]
fn validate_rejects_no_positions() {
    let mut nfa = good();
    nfa.elements.clear();
    assert!(nfa.validate().is_err());
}

#[test]
fn validate_rejects_a_follow_length_mismatch() {
    let mut nfa = good();
    nfa.follow.push(Vec::new());
    assert!(nfa.validate().is_err());
}

#[test]
fn validate_rejects_an_out_of_range_target() {
    let mut nfa = good();
    let beyond_accept = nfa.elements.len() as u32 + 5;
    nfa.start.push(beyond_accept);
    assert!(nfa.validate().is_err());
}
