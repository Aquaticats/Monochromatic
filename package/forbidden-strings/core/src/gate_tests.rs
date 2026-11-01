// What:  unit tests for the SetGate: which rules a line is a candidate for, the hit
//        position handed to the per-rule check, and the seedless-rule skip.
// Why:     This file groups the gate test cases so behavior changes fail near the code path they
//          protect.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("gate", () => {
//   // test cases below
// });
// ```

use super::SetGate;

// What:    Builds a three-rule gate: rule 0 on "AKIA", rule 1 on "ghp_", rule 2 literal-free.
// Why:     The program needs this named step so callers can reuse the behavior without copying
//          its body.
//
// In TS you'd write (pseudocode):
// ```ts
// function three_rule_gate(): SetGate {
//   // Rust body below is the implementation.
// }
// ```
fn three_rule_gate() -> SetGate {
    let seeds = vec![
        Some(vec![b"AKIA".to_vec()]),
        Some(vec![b"ghp_".to_vec()]),
        None,
    ];
    return SetGate::build(&seeds)
}

#[test]
fn flags_only_rules_whose_literal_is_present() {
    let gate = three_rule_gate();
    let mut seen = Vec::new();
    gate.for_each_candidate(b"xx AKIA yy", |rule, _pos| seen.push(rule));
    assert_eq!(seen, vec![0]);

    seen.clear();
    gate.for_each_candidate(b"a ghp_ b", |rule, _pos| seen.push(rule));
    assert_eq!(seen, vec![1]);
}

#[test]
fn rejects_lines_without_any_seed() {
    let gate = three_rule_gate();
    assert!(!gate.any_candidate(b"nothing relevant here", |_rule, _pos| return true));
    assert!(!gate.prefilter_present(b"nothing relevant here"));
    assert!(gate.prefilter_present(b"has AKIA"));
}

#[test]
fn seedless_rule_is_never_a_gate_candidate() {
    let gate = three_rule_gate();
    let mut seen = Vec::new();
    // What:    Rule 2 is literal-free; even on a line it cannot be flagged by the gate.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    gate.for_each_candidate(b"AKIA ghp_", |rule, _pos| seen.push(rule));
    assert!(!seen.contains(&2));
    assert!(seen.contains(&0));
    assert!(seen.contains(&1));
}

#[test]
fn reports_the_hit_position() {
    let gate = three_rule_gate();
    let mut positions = Vec::new();
    gate.for_each_candidate(b"zzAKIA", |rule, pos| {
        if rule == 0 {
            positions.push(pos);
        }
    });
    assert_eq!(positions, vec![2]);
}

#[test]
fn any_candidate_short_circuits_on_the_first_check_hit() {
    let gate = three_rule_gate();
    assert!(gate.any_candidate(b"has AKIA", |rule, _pos| return rule == 0));
    // What:    A check that never accepts means no candidate "matches" even when a seed is
    //          present.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert!(!gate.any_candidate(b"has AKIA", |_rule, _pos| return false));
}

#[test]
fn an_all_seedless_gate_flags_nothing() {
    let gate = SetGate::build(&[None, None]);
    assert!(!gate.any_candidate(b"anything at all", |_rule, _pos| return true));
    assert!(!gate.prefilter_present(b"anything at all"));
}
